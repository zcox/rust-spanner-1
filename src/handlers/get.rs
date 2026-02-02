use crate::error::{ApiError, ErrorResponse};
use crate::models::GetResponse;
use crate::routes;
use crate::state::AppState;
use axum::{extract::State, extract::Path, http::StatusCode, Json};
use uuid::Uuid;

/// GET /kv/:id handler - Retrieve a JSON document
#[utoipa::path(
    get,
    path = routes::KV_ITEM,
    params(
        ("id" = String, Path, description = "UUID key for the document")
    ),
    responses(
        (status = 200, description = "Document found", body = GetResponse),
        (status = 400, description = "Invalid UUID format", body = ErrorResponse),
        (status = 404, description = "Key not found", body = ErrorResponse),
        (status = 500, description = "Database error", body = ErrorResponse)
    ),
    tag = "kv"
)]
pub async fn get_handler(
    State(state): State<AppState>,
    Path(id_str): Path<String>,
) -> Result<(StatusCode, Json<GetResponse>), ApiError> {
    // Parse and validate UUID
    let id = Uuid::parse_str(&id_str).map_err(|_| ApiError::InvalidUuid(id_str.clone()))?;

    // Retrieve the document
    match state.spanner_client.read(id).await? {
        Some(data) => {
            tracing::info!("Successfully retrieved document with id: {}", id);
            Ok((
                StatusCode::OK,
                Json(GetResponse {
                    id: id.to_string(),
                    data,
                }),
            ))
        }
        None => {
            tracing::info!("Document not found with id: {}", id);
            Err(ApiError::KeyNotFound(id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::spanner::SpannerClient;
    use axum::{body::Body, http::Request, routing::put, Router};
    use std::sync::Arc;
    use tower::ServiceExt;

    // PUT handler needed for tests
    use crate::handlers::put::put_handler;

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
            .route(crate::routes::KV_ITEM, put(put_handler).get(get_handler))
            .with_state(state)
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
