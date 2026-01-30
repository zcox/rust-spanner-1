# Error Handling

Implement consistent error response handling across all endpoints.

## Requirements

1. Create a custom error type that can be converted to Axum responses
2. Error responses always have format: `{ "error": "message" }`
3. Map error types to appropriate HTTP status codes:
   - UUID parse errors -> 400
   - JSON parse errors -> 400
   - Key not found -> 404
   - Database errors -> 500
4. Error messages should be descriptive and helpful for debugging

## Acceptance Criteria

- [x] Custom error type defined
- [x] Error type implements IntoResponse
- [x] All error responses use consistent JSON format
- [x] Error messages are descriptive
- [x] HTTP status codes are correct
- [x] `cargo build` succeeds
