# PUT Endpoint

Implement PUT /kv/:id for storing JSON documents.

## Requirements

1. Route: `PUT /kv/:id`
2. Extract UUID from path parameter
3. Validate UUID format, return 400 if invalid
4. Parse JSON request body
5. Call upsert function with ID and data
6. Return 200 with `{ "id": "..." }` on success
7. Return appropriate error responses on failure

## Acceptance Criteria

- [x] Endpoint accepts PUT requests at /kv/:id
- [x] UUID validation works correctly
- [x] Invalid UUID returns 400 with descriptive error
- [x] Valid JSON body is stored
- [x] Invalid JSON returns 400
- [x] Successful store returns 200 with ID
- [x] Database errors return 500
- [x] `cargo build` succeeds
