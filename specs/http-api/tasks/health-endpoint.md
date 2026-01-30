# Health Endpoint

Implement GET /health for health checks.

## Requirements

1. Route: `GET /health`
2. Perform a simple Spanner query to verify connectivity
3. Return 200 with `{ "status": "healthy" }` if successful
4. Return 503 with `{ "status": "unhealthy", "error": "..." }` if Spanner is unreachable

## Acceptance Criteria

- [x] Endpoint accepts GET requests at /health
- [x] Healthy database returns 200
- [x] Unhealthy/unreachable database returns 503
- [x] Response includes status field
- [x] Error response includes error description
- [x] `cargo build` succeeds
