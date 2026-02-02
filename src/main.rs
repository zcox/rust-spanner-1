mod api_doc;
mod config;
mod error;
mod handlers;
mod models;
mod routes;
mod spanner;
mod state;

use api_doc::ApiDoc;
use axum::{routing::get, routing::put, Router};
use config::Config;
use handlers::{get_handler, health_handler, list_handler, put_handler};
use spanner::SpannerClient;
use state::AppState;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    tracing::info!("rust-spanner-kv starting");

    let config = Config::from_env()?;
    config.log_startup();

    let spanner_client = SpannerClient::from_config(&config).await?;

    // Create shared application state
    let state = AppState {
        spanner_client,
        config: Arc::new(config.clone()),
    };

    // Build the router
    let app = Router::new()
        .route(routes::HEALTH, get(health_handler))
        .route(routes::KV_LIST, get(list_handler))
        .route(routes::KV_ITEM, put(put_handler).get(get_handler))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    // Create the server address
    let addr = format!("{}:{}", state.config.service_host, state.config.service_port);
    tracing::info!("Starting server on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
