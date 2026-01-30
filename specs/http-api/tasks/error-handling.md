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

- [ ] Custom error type defined
- [ ] Error type implements IntoResponse
- [ ] All error responses use consistent JSON format
- [ ] Error messages are descriptive
- [ ] HTTP status codes are correct
- [ ] `cargo build` succeeds
