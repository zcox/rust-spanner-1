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

- [ ] Endpoint accepts PUT requests at /kv/:id
- [ ] UUID validation works correctly
- [ ] Invalid UUID returns 400 with descriptive error
- [ ] Valid JSON body is stored
- [ ] Invalid JSON returns 400
- [ ] Successful store returns 200 with ID
- [ ] Database errors return 500
- [ ] `cargo build` succeeds
