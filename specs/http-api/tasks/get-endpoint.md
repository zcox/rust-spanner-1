# GET Endpoint

Implement GET /kv/{id} for retrieving JSON documents.

## Requirements

1. Route: `GET /kv/{id}` (Note: Axum 0.8 uses `{id}` syntax instead of `:id`)
2. Extract UUID from path parameter
3. Validate UUID format, return 400 if invalid
4. Call read function with ID
5. Return 200 with `{ "id": "...", "data": ... }` if found
6. Return 404 if key doesn't exist
7. Return appropriate error responses on failure

## Acceptance Criteria

- [x] Endpoint accepts GET requests at /kv/{id}
- [x] UUID validation works correctly
- [x] Invalid UUID returns 400 with descriptive error
- [x] Existing key returns 200 with data
- [x] Non-existent key returns 404 with descriptive error
- [x] Database errors return 500
- [x] `cargo build` succeeds
