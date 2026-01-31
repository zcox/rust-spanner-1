# Add Schema Derives

Add `ToSchema` derives to all request/response structs for OpenAPI schema generation.

## Structs to Update

Add `#[derive(utoipa::ToSchema)]` to:

1. **PutResponse** - Response for PUT operations
2. **GetResponse** - Response for GET single key
3. **ListQuery** - Query parameters for list endpoint
4. **ListResponse** - Response for list endpoint
5. **KvEntryResponse** - Individual entry in list response
6. **ErrorResponse** - Standard error response format
7. **HealthResponse** - Health check response
8. **UnhealthyResponse** - Unhealthy status response

## Notes

- The `JsonValue` (serde_json::Value) type is handled automatically by utoipa
- Add descriptions using `#[schema(description = "...")]` where helpful
- UUIDs displayed as strings in the schema

## Verification

- `cargo build` should succeed after adding derives
