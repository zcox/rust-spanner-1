use crate::error::{ApiError, ErrorResponse};
use crate::models::{KvEntryResponse, ListQuery, ListResponse};
use crate::spanner::SortOrder;
use crate::state::AppState;
use axum::{extract::Query, extract::State, http::StatusCode, Json};

/// GET /kv handler - List all key-value pairs
///
/// Returns a paginated, filterable, and sortable list of all key-value pairs.
/// Query parameters:
/// - limit: Maximum number of results to return (optional)
/// - offset: Number of results to skip (optional, default: 0)
/// - prefix: Filter keys starting with this value (optional)
/// - sort: Sort order - one of: key_asc, key_desc, created_asc, created_desc, updated_asc, updated_desc (optional, default: key_asc)
#[utoipa::path(
    get,
    path = "/kv",
    params(
        ("limit" = Option<u32>, Query, description = "Maximum number of results to return"),
        ("offset" = Option<u32>, Query, description = "Number of results to skip"),
        ("prefix" = Option<String>, Query, description = "Filter keys starting with this value"),
        ("sort" = Option<String>, Query, description = "Sort order: key_asc, key_desc, created_asc, created_desc, updated_asc, updated_desc")
    ),
    responses(
        (status = 200, description = "List of key-value pairs", body = ListResponse),
        (status = 400, description = "Invalid query parameter", body = ErrorResponse),
        (status = 500, description = "Database error", body = ErrorResponse)
    ),
    tag = "kv"
)]
pub async fn list_handler(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::error::ErrorResponse;
    use crate::handlers::{get_handler, put_handler};
    use crate::models::GetResponse;
    use crate::spanner::SpannerClient;
    use axum::{body::Body, http::Request, routing::get, routing::put, Router};
    use std::sync::Arc;
    use tower::ServiceExt;
    use uuid::Uuid;

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
            .route("/kv", get(list_handler))
            .route("/kv/{id}", put(put_handler).get(get_handler))
            .with_state(state)
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

    // Integration tests for GET /kv endpoint - comprehensive coverage
    // These tests verify pagination, sorting, filtering, and error handling

    /// Helper function to create a fresh test database with known data
    async fn setup_list_test_app() -> (Router, Vec<Uuid>) {
        unsafe {
            std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        }

        let config = Config {
            spanner_emulator_host: Some("localhost:9010".to_string()),
            spanner_project: "test-project".to_string(),
            spanner_instance: "list-integration-test".to_string(),
            spanner_database: "list-integration-test-db".to_string(),
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
            .route("/kv", get(list_handler))
            .route("/kv/{id}", put(put_handler).get(get_handler))
            .with_state(state);

        // Insert test data
        let mut ids = Vec::new();
        let test_data = vec![
            serde_json::json!({"type": "fruit", "color": "red", "name": "apple"}),
            serde_json::json!({"type": "fruit", "color": "yellow", "name": "banana"}),
            serde_json::json!({"type": "vegetable", "color": "orange", "name": "carrot"}),
            serde_json::json!({"type": "fruit", "color": "brown", "name": "date"}),
        ];

        for data in test_data {
            let id = Uuid::new_v4();
            ids.push(id);

            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("PUT")
                        .uri(format!("/kv/{}", id))
                        .header("content-type", "application/json")
                        .body(Body::from(serde_json::to_string(&data).unwrap()))
                        .unwrap(),
                )
                .await
                .unwrap();

            // Small delay to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        (app, ids)
    }

    #[tokio::test]
    async fn test_list_integration_pagination_limit() {
        let (app, _ids) = setup_list_test_app().await;

        // Test limit=2
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv?limit=2")
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

        // Should return exactly 2 entries
        assert_eq!(response_json.data.len(), 2);
        // Total count should reflect all entries
        assert!(response_json.total_count >= 4);

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_integration_pagination_offset() {
        let (app, _ids) = setup_list_test_app().await;

        // First, get all entries to know what to expect
        let all_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let all_body = axum::body::to_bytes(all_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let all_json: ListResponse = serde_json::from_slice(&all_body).unwrap();

        if all_json.data.len() < 2 {
            panic!("Not enough test data");
        }

        // Now test offset=1
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv?offset=1")
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

        // Should skip first entry
        assert_eq!(response_json.data.len(), all_json.data.len() - 1);
        // First key should be the second key from all results
        assert_eq!(response_json.data[0].key, all_json.data[1].key);

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_integration_pagination_limit_and_offset() {
        let (app, _ids) = setup_list_test_app().await;

        // First, get all entries
        let all_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let all_body = axum::body::to_bytes(all_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let all_json: ListResponse = serde_json::from_slice(&all_body).unwrap();

        // Test limit=2&offset=1
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv?limit=2&offset=1")
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

        // Should return 2 entries starting from offset 1
        assert_eq!(response_json.data.len(), 2);
        assert_eq!(response_json.data[0].key, all_json.data[1].key);
        assert_eq!(response_json.data[1].key, all_json.data[2].key);
        // Total count should reflect all entries
        assert_eq!(response_json.total_count, all_json.total_count);

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_integration_sort_key_asc() {
        let (app, _ids) = setup_list_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv?sort=key_asc")
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

        // Verify keys are sorted alphabetically ascending
        for i in 0..response_json.data.len() - 1 {
            assert!(
                response_json.data[i].key <= response_json.data[i + 1].key,
                "Keys should be sorted ascending"
            );
        }

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_integration_sort_key_desc() {
        let (app, _ids) = setup_list_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv?sort=key_desc")
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

        // Verify keys are sorted alphabetically descending
        for i in 0..response_json.data.len() - 1 {
            assert!(
                response_json.data[i].key >= response_json.data[i + 1].key,
                "Keys should be sorted descending"
            );
        }

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_integration_sort_created_asc() {
        let (app, _ids) = setup_list_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv?sort=created_asc")
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

        // Verify timestamps are sorted ascending (oldest first)
        for i in 0..response_json.data.len() - 1 {
            let created1 = chrono::DateTime::parse_from_rfc3339(&response_json.data[i].created_at)
                .unwrap();
            let created2 =
                chrono::DateTime::parse_from_rfc3339(&response_json.data[i + 1].created_at)
                    .unwrap();
            assert!(
                created1 <= created2,
                "Timestamps should be sorted ascending (oldest first)"
            );
        }

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_integration_sort_created_desc() {
        let (app, _ids) = setup_list_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv?sort=created_desc")
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

        // Verify timestamps are sorted descending (newest first)
        for i in 0..response_json.data.len() - 1 {
            let created1 = chrono::DateTime::parse_from_rfc3339(&response_json.data[i].created_at)
                .unwrap();
            let created2 =
                chrono::DateTime::parse_from_rfc3339(&response_json.data[i + 1].created_at)
                    .unwrap();
            assert!(
                created1 >= created2,
                "Timestamps should be sorted descending (newest first)"
            );
        }

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_integration_sort_updated_asc() {
        let (app, _ids) = setup_list_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv?sort=updated_asc")
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

        // Verify updated timestamps are sorted ascending
        for i in 0..response_json.data.len() - 1 {
            let updated1 = chrono::DateTime::parse_from_rfc3339(&response_json.data[i].updated_at)
                .unwrap();
            let updated2 =
                chrono::DateTime::parse_from_rfc3339(&response_json.data[i + 1].updated_at)
                    .unwrap();
            assert!(
                updated1 <= updated2,
                "Updated timestamps should be sorted ascending"
            );
        }

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_integration_sort_updated_desc() {
        let (app, _ids) = setup_list_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv?sort=updated_desc")
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

        // Verify updated timestamps are sorted descending
        for i in 0..response_json.data.len() - 1 {
            let updated1 = chrono::DateTime::parse_from_rfc3339(&response_json.data[i].updated_at)
                .unwrap();
            let updated2 =
                chrono::DateTime::parse_from_rfc3339(&response_json.data[i + 1].updated_at)
                    .unwrap();
            assert!(
                updated1 >= updated2,
                "Updated timestamps should be sorted descending"
            );
        }

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_integration_prefix_filter() {
        let (app, _ids) = setup_list_test_app().await;

        // Filter by prefix - look for keys starting with specific UUID prefix
        // Since we're using deterministic UUIDs, we need to get the actual keys first
        let all_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let all_body = axum::body::to_bytes(all_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let all_json: ListResponse = serde_json::from_slice(&all_body).unwrap();

        if all_json.data.is_empty() {
            panic!("No test data found");
        }

        // Use a prefix that should match at least one key
        let prefix = &all_json.data[0].key[..8]; // First 8 characters

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/kv?prefix={}", prefix))
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

        // All returned keys should start with the prefix
        for entry in &response_json.data {
            assert!(
                entry.key.starts_with(prefix),
                "Key '{}' should start with prefix '{}'",
                entry.key,
                prefix
            );
        }

        // Total count should reflect filtered count
        assert_eq!(
            response_json.total_count,
            response_json.data.len() as i64
        );

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_integration_prefix_with_pagination() {
        let (app, _ids) = setup_list_test_app().await;

        // Get a prefix that matches multiple entries
        let all_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let all_body = axum::body::to_bytes(all_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let all_json: ListResponse = serde_json::from_slice(&all_body).unwrap();

        // Use a short prefix that should match multiple entries
        let prefix = &all_json.data[0].key[..4];

        // Test prefix with limit
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/kv?prefix={}&limit=1", prefix))
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

        // All keys should match prefix
        for entry in &response_json.data {
            assert!(entry.key.starts_with(prefix));
        }

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_integration_response_fields() {
        let (app, _ids) = setup_list_test_app().await;

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

        // Verify each entry has all required fields
        for entry in &response_json.data {
            // Key should be a valid UUID
            assert!(Uuid::parse_str(&entry.key).is_ok());

            // Value should be valid JSON
            assert!(entry.value.is_object() || entry.value.is_array());

            // Timestamps should be valid ISO 8601
            assert!(chrono::DateTime::parse_from_rfc3339(&entry.created_at).is_ok());
            assert!(chrono::DateTime::parse_from_rfc3339(&entry.updated_at).is_ok());
        }

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_integration_total_count_accuracy() {
        let (app, _ids) = setup_list_test_app().await;

        // Get all entries
        let all_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let all_body = axum::body::to_bytes(all_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let all_json: ListResponse = serde_json::from_slice(&all_body).unwrap();

        // Get with limit
        let limited_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv?limit=2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let limited_body = axum::body::to_bytes(limited_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let limited_json: ListResponse = serde_json::from_slice(&limited_body).unwrap();

        // Total count should be the same regardless of limit
        assert_eq!(all_json.total_count, limited_json.total_count);

        // But data length should be limited
        assert_eq!(limited_json.data.len(), 2);

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_integration_error_invalid_sort() {
        let (app, _ids) = setup_list_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/kv?sort=invalid_value")
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

        // Should include helpful error message
        assert!(error_response.error.contains("sort must be one of"));
        assert!(error_response.error.contains("invalid_value"));

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_list_integration_default_sort() {
        let (app, _ids) = setup_list_test_app().await;

        // Request without sort parameter should default to key_asc
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

        // Verify default sort is key ascending
        for i in 0..response_json.data.len() - 1 {
            assert!(
                response_json.data[i].key <= response_json.data[i + 1].key,
                "Default sort should be key ascending"
            );
        }

        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }
}
