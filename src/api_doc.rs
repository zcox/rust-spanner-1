use utoipa::OpenApi;

use crate::error::{ErrorResponse, HealthResponse, UnhealthyResponse};
use crate::handlers;
use crate::models::{GetResponse, KvEntryResponse, ListResponse, PutResponse};

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "rust-spanner-kv API",
        version = "1.0.0",
        description = "A simple JSON key-value store backed by Google Cloud Spanner"
    ),
    paths(
        handlers::health::health_handler,
        handlers::put::put_handler,
        handlers::get::get_handler,
        handlers::list::list_handler
    ),
    components(
        schemas(
            PutResponse,
            GetResponse,
            ListResponse,
            KvEntryResponse,
            ErrorResponse,
            HealthResponse,
            UnhealthyResponse
        )
    ),
    tags(
        (name = "health", description = "Health check operations"),
        (name = "kv", description = "Key-value store operations")
    )
)]
pub struct ApiDoc;
