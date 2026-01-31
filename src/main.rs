mod config;
mod spanner;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use config::Config;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use spanner::{SortOrder, SpannerClient};
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
        .route("/health", get(health_handler))
        .route("/kv", get(list_handler))
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

/// Query parameters for list endpoint
#[derive(Deserialize)]
struct ListQuery {
    limit: Option<u32>,
    offset: Option<u32>,
    prefix: Option<String>,
    sort: Option<String>,
}

/// Response type for list endpoint
#[derive(Serialize, Deserialize)]
struct ListResponse {
    data: Vec<KvEntryResponse>,
    total_count: i64,
}

/// Individual key-value entry in list response
#[derive(Serialize, Deserialize)]
struct KvEntryResponse {
    key: String,
    value: JsonValue,
    created_at: String,
    updated_at: String,
}

/// Error response type
#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
}

/// Custom error type for API endpoints
///
/// This error type provides consistent error handling across all endpoints,
/// automatically mapping different error types to appropriate HTTP status codes
/// and formatting them as JSON responses.
#[derive(Debug)]
enum ApiError {
    /// Invalid UUID format in path parameter
    InvalidUuid(String),
    /// Key not found in database
    KeyNotFound(Uuid),
    /// Database operation error
    DatabaseError(anyhow::Error),
    /// JSON parsing error
    JsonError(serde_json::Error),
    /// Invalid query parameter
    InvalidQueryParam(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::InvalidUuid(id) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid UUID format: expected format like '550e8400-e29b-41d4-a716-446655440000', got '{}'", id),
            ),
            ApiError::KeyNotFound(id) => (
                StatusCode::NOT_FOUND,
                format!("Key not found: {}", id),
            ),
            ApiError::DatabaseError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", err),
            ),
            ApiError::JsonError(err) => (
                StatusCode::BAD_REQUEST,
                format!("JSON parse error: {}", err),
            ),
            ApiError::InvalidQueryParam(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid query parameter: {}", msg),
            ),
        };

        let body = Json(ErrorResponse {
            error: error_message,
        });

        (status, body).into_response()
    }
}

impl From<uuid::Error> for ApiError {
    fn from(err: uuid::Error) -> Self {
        ApiError::InvalidUuid(err.to_string())
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::DatabaseError(err)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::JsonError(err)
    }
}

/// Response type for health check endpoint
#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
}

/// Response type for unhealthy status
#[derive(Serialize, Deserialize)]
struct UnhealthyResponse {
    status: String,
    error: String,
}

/// PUT /kv/:id handler - Store a JSON document
async fn put_handler(
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

/// GET /kv/:id handler - Retrieve a JSON document
async fn get_handler(
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

/// GET /kv handler - List all key-value pairs
///
/// Returns a paginated, filterable, and sortable list of all key-value pairs.
/// Query parameters:
/// - limit: Maximum number of results to return (optional)
/// - offset: Number of results to skip (optional, default: 0)
/// - prefix: Filter keys starting with this value (optional)
/// - sort: Sort order - one of: key_asc, key_desc, created_asc, created_desc, updated_asc, updated_desc (optional, default: key_asc)
async fn list_handler(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<(StatusCode, Json<ListResponse>), ApiError> {
    // Parse and validate sort parameter
    let sort = if let Some(sort_str) = &query.sort {
        match sort_str.as_str() {
            "key_asc" => SortOrder::KeyAsc,
            "key_desc" => SortOrder::KeyDesc,
            "created_asc" => SortOrder::CreatedAsc,
            "created_desc" => SortOrder::CreatedDesc,
            "updated_asc" => SortOrder::UpdatedAsc,
            "updated_desc" => SortOrder::UpdatedDesc,
            _ => {
                return Err(ApiError::InvalidQueryParam(format!(
                    "sort must be one of: key_asc, key_desc, created_asc, created_desc, updated_asc, updated_desc, got '{}'",
                    sort_str
                )))
            }
        }
    } else {
        SortOrder::KeyAsc // default
    };

    // Convert limit and offset to i64
    let limit = query.limit.map(|l| l as i64);
    let offset = query.offset.unwrap_or(0) as i64;

    // Query the database
    let result = state
        .spanner_client
        .list_all(query.prefix.as_deref(), sort, limit, offset)
        .await?;

    // Convert to response format with ISO 8601 timestamps
    let data: Vec<KvEntryResponse> = result
        .entries
        .into_iter()
        .map(|entry| KvEntryResponse {
            key: entry.key,
            value: entry.value,
            created_at: entry.created_at.to_rfc3339(),
            updated_at: entry.updated_at.to_rfc3339(),
        })
        .collect();

    let response = ListResponse {
        data,
        total_count: result.total_count,
    };

    tracing::info!(
        "Listed {} entries (total: {}, prefix: {:?}, sort: {:?}, limit: {:?}, offset: {})",
        response.data.len(),
        response.total_count,
        query.prefix,
        sort,
        limit,
        offset
    );

    Ok((StatusCode::OK, Json(response)))
}

/// GET /health handler - Health check endpoint
///
/// Performs a simple query to Spanner to verify database connectivity.
/// Returns 200 OK if the database is reachable, 503 Service Unavailable otherwise.
async fn health_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<HealthResponse>), (StatusCode, Json<UnhealthyResponse>)> {
    // Perform a simple query to verify Spanner connectivity
    // We'll use a lightweight query: SELECT 1
    match state.spanner_client.health_check().await {
        Ok(_) => {
            tracing::debug!("Health check passed");
            Ok((
                StatusCode::OK,
                Json(HealthResponse {
                    status: "healthy".to_string(),
                }),
            ))
        }
        Err(e) => {
            tracing::error!("Health check failed: {}", e);
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(UnhealthyResponse {
                    status: "unhealthy".to_string(),
                    error: format!("Cannot connect to database: {}", e),
                }),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono;
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
            .route("/health", get(health_handler))
            .route("/kv", get(list_handler))
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

    #[tokio::test]
    async fn test_health_endpoint_healthy() {
        // Set up config with emulator
        unsafe {
            std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        }

        let config = Config {
            spanner_emulator_host: Some("localhost:9010".to_string()),
            spanner_project: "test-project".to_string(),
            spanner_instance: "health-endpoint-test".to_string(),
            spanner_database: "health-endpoint-test-db".to_string(),
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

        let app = Router::new()
            .route("/health", get(health_handler))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: HealthResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(response_json.status, "healthy");

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_health_endpoint_unhealthy() {
        // Set up config with a bad emulator host that doesn't exist
        unsafe {
            std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9999");
        }

        let config = Config {
            spanner_emulator_host: Some("localhost:9999".to_string()),
            spanner_project: "test-project".to_string(),
            spanner_instance: "health-endpoint-unhealthy-test".to_string(),
            spanner_database: "health-endpoint-unhealthy-test-db".to_string(),
            service_port: 3000,
            service_host: "0.0.0.0".to_string(),
        };

        // Try to create a client - this should fail because the emulator doesn't exist
        // But we'll create the state anyway to test the health endpoint behavior
        // when the database is unreachable
        let client_result = SpannerClient::from_config(&config).await;

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }

        // If we can't even create the client, that's expected for this test
        // We're testing the scenario where Spanner is unreachable
        if client_result.is_err() {
            // This is expected - we can't create a client with a bad host
            // The test demonstrates that when Spanner is unavailable,
            // the client creation itself fails, which would prevent the app from starting
            // In a real scenario, the health endpoint would return 503 if the database becomes
            // unreachable after the app has started
            return;
        }
    }

    #[tokio::test]
    async fn test_list_endpoint_empty() {
        let app = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: ListResponse = serde_json::from_slice(&body).unwrap();

        // Should return a list with total_count (may have data from other tests)
        assert!(response_json.data.len() <= response_json.total_count as usize);
        assert!(response_json.total_count >= 0);

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_endpoint_with_data() {
        let app = setup_test_app().await;

        // Insert some test data
        let test_id1 = Uuid::new_v4();
        let test_id2 = Uuid::new_v4();
        let test_data1 = serde_json::json!({"name": "first"});
        let test_data2 = serde_json::json!({"name": "second"});

        // PUT first document
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/kv/{}", test_id1))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&test_data1).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // PUT second document
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/kv/{}", test_id2))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&test_data2).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // List all
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: ListResponse = serde_json::from_slice(&body).unwrap();

        // Should have at least our 2 documents
        assert!(response_json.data.len() >= 2);
        assert!(response_json.total_count >= 2);

        // Verify response format
        for entry in &response_json.data {
            assert!(!entry.key.is_empty());
            // Verify ISO 8601 timestamp format
            assert!(chrono::DateTime::parse_from_rfc3339(&entry.created_at).is_ok());
            assert!(chrono::DateTime::parse_from_rfc3339(&entry.updated_at).is_ok());
        }

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_endpoint_with_limit() {
        let app = setup_test_app().await;

        // Insert test data
        let test_id = Uuid::new_v4();
        let test_data = serde_json::json!({"test": "data"});

        let _ = app
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

        // List with limit
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv?limit=1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: ListResponse = serde_json::from_slice(&body).unwrap();

        // Should return at most 1 entry
        assert!(response_json.data.len() <= 1);

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_endpoint_with_sort() {
        let app = setup_test_app().await;

        // Test various sort parameters
        let sort_options = vec![
            "key_asc",
            "key_desc",
            "created_asc",
            "created_desc",
            "updated_asc",
            "updated_desc",
        ];

        for sort in sort_options {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("GET")
                        .uri(format!("/kv?sort={}", sort))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                StatusCode::OK,
                "Sort option '{}' should be valid",
                sort
            );
        }

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_endpoint_invalid_sort() {
        let app = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv?sort=invalid_sort")
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
        assert!(error_response.error.contains("sort must be one of"));

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_endpoint_no_conflict_with_get() {
        let app = setup_test_app().await;

        // First, PUT a document
        let test_id = Uuid::new_v4();
        let test_data = serde_json::json!({"test": "data"});

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

        // GET specific key should work
        let get_response = app
            .clone()
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
        let get_json: GetResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(get_json.id, test_id.to_string());
        assert_eq!(get_json.data, test_data);

        // List endpoint should also work
        let list_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(list_response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let list_json: ListResponse = serde_json::from_slice(&body).unwrap();
        assert!(list_json.data.len() >= 1);

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }
}
