mod config;
mod spanner;

use axum::{routing::get, Router};
use config::Config;
use spanner::SpannerClient;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

/// Shared application state
#[derive(Clone)]
struct AppState {
    spanner_client: SpannerClient,
    config: Arc<Config>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    // Build the router with placeholder routes
    // (actual endpoints will be implemented in subsequent tasks)
    let app = Router::new()
        .route("/health", get(health_handler))
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

/// Placeholder health check handler
/// (actual implementation will be done in a subsequent task)
async fn health_handler() -> &'static str {
    "OK"
}
