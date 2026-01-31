# Add GET /kv Handler

Add the HTTP handler for the list endpoint with query parameter parsing and response formatting.

## Requirements

### Route Registration

Register `GET /kv` as a new route in the Axum router. This should not conflict with the existing `GET /kv/:key` route - Axum should route parameterless requests to the list handler.

### Query Parameter Parsing

Define a struct for query parameters using `serde::Deserialize`:

```rust
#[derive(Deserialize)]
struct ListQuery {
    limit: Option<u32>,
    offset: Option<u32>,
    prefix: Option<String>,
    sort: Option<String>,  // Validate against allowed values
}
```

Use Axum's `Query` extractor to parse parameters.

### Validation

- `limit` must be non-negative (u32 handles this)
- `offset` must be non-negative (u32 handles this)
- `sort` must be one of: `key_asc`, `key_desc`, `created_asc`, `created_desc`, `updated_asc`, `updated_desc`
- Return 400 Bad Request for invalid sort values

### Response Format

Return JSON matching the spec:

```rust
#[derive(Serialize)]
struct ListResponse {
    data: Vec<KvEntryResponse>,
    total_count: i64,
}

#[derive(Serialize)]
struct KvEntryResponse {
    key: String,
    value: serde_json::Value,
    created_at: String,  // ISO 8601 format
    updated_at: String,  // ISO 8601 format
}
```

### Error Handling

- Invalid query params: 400 with error message
- Database errors: 500 with generic error message

## Acceptance Criteria

- [x] Route correctly handles `GET /kv` without conflicting with `GET /kv/:key`
- [x] All query parameters parsed correctly
- [x] Invalid sort values return 400
- [x] Response format matches spec exactly
- [x] Timestamps formatted as ISO 8601
