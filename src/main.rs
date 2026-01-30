mod config;
mod spanner;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use config::Config;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use spanner::SpannerClient;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

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

    // Build the router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/kv/{id}", put(put_handler).get(get_handler))
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

/// Response type for successful PUT operations
#[derive(Serialize, Deserialize)]
struct PutResponse {
    id: String,
}

/// Response type for successful GET operations
#[derive(Serialize, Deserialize)]
struct GetResponse {
    id: String,
    data: JsonValue,
}

/// Error response type
#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
}

/// PUT /kv/:id handler - Store a JSON document
async fn put_handler(
    State(state): State<AppState>,
    Path(id_str): Path<String>,
    Json(data): Json<JsonValue>,
) -> Result<(StatusCode, Json<PutResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Parse and validate UUID
    let id = match Uuid::parse_str(&id_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            tracing::warn!("Invalid UUID format: {}", id_str);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid UUID format: expected format like '550e8400-e29b-41d4-a716-446655440000'".to_string(),
                }),
            ));
        }
    };

    // Store the document
    match state.spanner_client.upsert(id, data).await {
        Ok(_) => {
            tracing::info!("Successfully stored document with id: {}", id);
            Ok((
                StatusCode::OK,
                Json(PutResponse {
                    id: id.to_string(),
                }),
            ))
        }
        Err(e) => {
            tracing::error!("Database error while storing document: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            ))
        }
    }
}

/// GET /kv/:id handler - Retrieve a JSON document
async fn get_handler(
    State(state): State<AppState>,
    Path(id_str): Path<String>,
) -> Result<(StatusCode, Json<GetResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Parse and validate UUID
    let id = match Uuid::parse_str(&id_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            tracing::warn!("Invalid UUID format: {}", id_str);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid UUID format: expected format like '550e8400-e29b-41d4-a716-446655440000'".to_string(),
                }),
            ));
        }
    };

    // Retrieve the document
    match state.spanner_client.read(id).await {
        Ok(Some(data)) => {
            tracing::info!("Successfully retrieved document with id: {}", id);
            Ok((
                StatusCode::OK,
                Json(GetResponse {
                    id: id.to_string(),
                    data,
                }),
            ))
        }
        Ok(None) => {
            tracing::info!("Document not found with id: {}", id);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Key not found: {}", id),
                }),
            ))
        }
        Err(e) => {
            tracing::error!("Database error while retrieving document: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            ))
        }
    }
}

/// Placeholder health check handler
/// (actual implementation will be done in a subsequent task)
async fn health_handler() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    async fn setup_test_app() -> Router {
        // Set up config with emulator
        unsafe {
            std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        }

        let config = Config {
            spanner_emulator_host: Some("localhost:9010".to_string()),
            spanner_project: "test-project".to_string(),
            spanner_instance: "put-endpoint-test".to_string(),
            spanner_database: "put-endpoint-test-db".to_string(),
            service_port: 3000,
            service_host: "0.0.0.0".to_string(),
        };

        let spanner_client = SpannerClient::from_config(&config)
            .await
            .expect("Failed to create Spanner client");

        let state = AppState {
            spanner_client,
            config: Arc::new(config),
        };

        Router::new()
            .route("/kv/{id}", put(put_handler).get(get_handler))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_put_endpoint_success() {
        let app = setup_test_app().await;

        let test_id = Uuid::new_v4();
        let test_data = serde_json::json!({
            "name": "test",
            "value": 42
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/kv/{}", test_id))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&test_data).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: PutResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(response_json.id, test_id.to_string());

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_put_endpoint_invalid_uuid() {
        let app = setup_test_app().await;

        let test_data = serde_json::json!({
            "name": "test"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/kv/not-a-uuid")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&test_data).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_response: ErrorResponse = serde_json::from_slice(&body).unwrap();
        assert!(error_response.error.contains("Invalid UUID format"));

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_put_endpoint_complex_json() {
        let app = setup_test_app().await;

        let test_id = Uuid::new_v4();
        let test_data = serde_json::json!({
            "string": "hello",
            "number": 123,
            "boolean": true,
            "null": null,
            "array": [1, 2, 3],
            "nested": {
                "key": "value"
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/kv/{}", test_id))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&test_data).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_put_endpoint_invalid_json() {
        let app = setup_test_app().await;

        let test_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/kv/{}", test_id))
                    .header("content-type", "application/json")
                    .body(Body::from("{invalid json}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Axum's Json extractor returns 400 for invalid JSON
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_get_endpoint_success() {
        let app = setup_test_app().await;

        let test_id = Uuid::new_v4();
        let test_data = serde_json::json!({
            "name": "test document",
            "value": 42
        });

        // First, PUT the data
        let put_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/kv/{}", test_id))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&test_data).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(put_response.status(), StatusCode::OK);

        // Now, GET the data
        let get_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/kv/{}", test_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(get_response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(get_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: GetResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(response_json.id, test_id.to_string());
        assert_eq!(response_json.data, test_data);

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_get_endpoint_not_found() {
        let app = setup_test_app().await;

        // Try to GET a non-existent key
        let non_existent_id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/kv/{}", non_existent_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_response: ErrorResponse = serde_json::from_slice(&body).unwrap();
        assert!(error_response.error.contains("Key not found"));
        assert!(error_response.error.contains(&non_existent_id.to_string()));

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_get_endpoint_invalid_uuid() {
        let app = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv/not-a-uuid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_response: ErrorResponse = serde_json::from_slice(&body).unwrap();
        assert!(error_response.error.contains("Invalid UUID format"));

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_get_endpoint_complex_json() {
        let app = setup_test_app().await;

        let test_id = Uuid::new_v4();
        let test_data = serde_json::json!({
            "string": "hello",
            "number": 123,
            "boolean": true,
            "null": null,
            "array": [1, 2, 3],
            "nested": {
                "key": "value"
            }
        });

        // First, PUT the data
        let put_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/kv/{}", test_id))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&test_data).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(put_response.status(), StatusCode::OK);

        // Now, GET the data
        let get_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/kv/{}", test_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(get_response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(get_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: GetResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(response_json.data, test_data);

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }
}
