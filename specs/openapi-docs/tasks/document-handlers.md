# Document Handlers

Add `#[utoipa::path]` attributes to all handler functions.

## Handlers to Document

### health_handler

```rust
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy", body = UnhealthyResponse)
    ),
    tag = "health"
)]
```

### put_handler

```rust
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
```

### get_handler

```rust
#[utoipa::path(
    get,
    path = "/kv/{id}",
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
```

### list_handler

```rust
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
```

## Verification

- `cargo build` should succeed after adding attributes
