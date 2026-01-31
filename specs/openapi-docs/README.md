# OpenAPI Documentation

Generate and serve OpenAPI documentation for the rust-spanner-kv REST API.

## Overview

The service exposes an OpenAPI 3.1 specification describing all available endpoints, request/response schemas, and error types. Interactive Swagger UI is provided for exploring and testing the API.

## Requirements

### Library

Use `utoipa` with the following crates:
- `utoipa` - Core OpenAPI derive macros
- `utoipa-swagger-ui` - Swagger UI integration for Axum

### Documented Endpoints

All existing endpoints must be documented:

| Endpoint | Description |
|----------|-------------|
| `GET /health` | Health check endpoint |
| `GET /kv` | List all key-value pairs with pagination/filtering |
| `GET /kv/{id}` | Retrieve a specific key-value pair |
| `PUT /kv/{id}` | Store (upsert) a key-value pair |

### Schema Documentation

Document all request/response types:
- `PutResponse` - Response for PUT operations
- `GetResponse` - Response for GET single key
- `ListResponse` - Response for list endpoint
- `KvEntryResponse` - Individual entry in list response
- `ListQuery` - Query parameters for list endpoint
- `ErrorResponse` - Standard error response format
- `HealthResponse` - Health check response
- `UnhealthyResponse` - Unhealthy status response

### Endpoints Added

| Path | Description |
|------|-------------|
| `/swagger-ui` | Interactive Swagger UI for exploring the API |
| `/api-doc/openapi.json` | Raw OpenAPI specification in JSON format |

### API Metadata

The OpenAPI spec should include:
- **Title**: rust-spanner-kv API
- **Version**: 1.0.0
- **Description**: A simple JSON key-value store backed by Google Cloud Spanner

### Implementation Notes

- Use `#[derive(ToSchema)]` on all request/response structs
- Use `#[utoipa::path(...)]` attributes on handler functions
- Create an `ApiDoc` struct with `#[derive(OpenApi)]` to collect all paths
- Mount Swagger UI and OpenAPI JSON endpoints in the Axum router

## Tasks

See [tasks/README.md](./tasks/README.md) for implementation tasks.
