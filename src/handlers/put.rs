use crate::error::{ApiError, ErrorResponse};
use crate::models::PutResponse;
use crate::state::AppState;
use axum::{extract::State, extract::Path, http::StatusCode, Json};
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// PUT /kv/:id handler - Store a JSON document
#[utoipa::path(
    put,
    path = "/kv/{id}",
    params(
        ("id" = String, Path, description = "UUID key for the document")
    ),
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Document stored successfully", body = PutResponse),
        (status = 400, description = "Invalid UUID format or invalid JSON", body = ErrorResponse),
        (status = 500, description = "Database error", body = ErrorResponse)
    ),
    tag = "kv"
)]
pub async fn put_handler(
    State(state): State<AppState>,
    Path(id_str): Path<String>,
    Json(data): Json<JsonValue>,
) -> Result<(StatusCode, Json<PutResponse>), ApiError> {
    // Parse and validate UUID
    let id = Uuid::parse_str(&id_str).map_err(|_| ApiError::InvalidUuid(id_str.clone()))?;

    // Store the document
    state.spanner_client.upsert(id, data).await?;

    tracing::info!("Successfully stored document with id: {}", id);
    Ok((
        StatusCode::OK,
        Json(PutResponse {
            id: id.to_string(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::spanner::SpannerClient;
    use axum::{body::Body, http::Request, routing::put, Router};
    use std::sync::Arc;
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
            .route("/kv/{id}", put(put_handler))
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
}
