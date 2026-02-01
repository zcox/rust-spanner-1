use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Error response type
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

/// Response type for health check endpoint
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

/// Response type for unhealthy status
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct UnhealthyResponse {
    pub status: String,
    pub error: String,
}

/// Custom error type for API endpoints
///
/// This error type provides consistent error handling across all endpoints,
/// automatically mapping different error types to appropriate HTTP status codes
/// and formatting them as JSON responses.
#[derive(Debug)]
pub enum ApiError {
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
