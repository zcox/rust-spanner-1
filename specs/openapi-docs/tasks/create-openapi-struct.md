# Create OpenApi Struct and Mount Routes

Create the ApiDoc struct that collects all documented paths and mount Swagger UI routes.

## ApiDoc Struct

```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "rust-spanner-kv API",
        version = "1.0.0",
        description = "A simple JSON key-value store backed by Google Cloud Spanner"
    ),
    paths(
        health_handler,
        put_handler,
        get_handler,
        list_handler
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
struct ApiDoc;
```

## Mount Routes

Add to router in main():

```rust
use utoipa_swagger_ui::SwaggerUi;

let app = Router::new()
    .route("/health", get(health_handler))
    .route("/kv", get(list_handler))
    .route("/kv/{id}", put(put_handler).get(get_handler))
    .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
    .layer(TraceLayer::new_for_http())
    .with_state(state.clone());
```

## Verification

1. `cargo build` should succeed
2. `cargo run` and navigate to `http://localhost:3000/swagger-ui` - should show Swagger UI
3. `curl http://localhost:3000/api-doc/openapi.json` - should return OpenAPI JSON
