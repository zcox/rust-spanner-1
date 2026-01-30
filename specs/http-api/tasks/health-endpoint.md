# Health Endpoint

Implement GET /health for health checks.

## Requirements

1. Route: `GET /health`
2. Perform a simple Spanner query to verify connectivity
3. Return 200 with `{ "status": "healthy" }` if successful
4. Return 503 with `{ "status": "unhealthy", "error": "..." }` if Spanner is unreachable

## Acceptance Criteria

- [ ] Endpoint accepts GET requests at /health
- [ ] Healthy database returns 200
- [ ] Unhealthy/unreachable database returns 503
- [ ] Response includes status field
- [ ] Error response includes error description
- [ ] `cargo build` succeeds
