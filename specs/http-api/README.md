# HTTP API

REST endpoints for storing and retrieving JSON data.

## Overview

The service exposes a simple REST API for key-value operations. All endpoints use JSON for request/response bodies and return descriptive error messages.

## Requirements

### Rust Version
- Use the latest stable Rust version
- All dependencies must use their latest available versions

### Endpoints

#### PUT /kv/{id}
Store a JSON document at the given key (upsert - create or replace).

**Note:** Path parameters use `{id}` syntax for Axum 0.8.

**Path Parameters:**
- `id` (UUID) - The key to store the document at

**Request Body:**
- Any valid JSON value

**Responses:**
- `200 OK` - Document stored successfully
  ```json
  { "id": "550e8400-e29b-41d4-a716-446655440000" }
  ```
- `400 Bad Request` - Invalid UUID format or invalid JSON
  ```json
  { "error": "Invalid UUID format: expected format like '550e8400-e29b-41d4-a716-446655440000'" }
  ```
- `500 Internal Server Error` - Database error
  ```json
  { "error": "Database error: connection failed" }
  ```

#### GET /kv/{id}
Retrieve a JSON document by key.

**Path Parameters:**
- `id` (UUID) - The key to retrieve

**Responses:**
- `200 OK` - Document found
  ```json
  { "id": "550e8400-e29b-41d4-a716-446655440000", "data": { ... } }
  ```
- `400 Bad Request` - Invalid UUID format
  ```json
  { "error": "Invalid UUID format: expected format like '550e8400-e29b-41d4-a716-446655440000'" }
  ```
- `404 Not Found` - Key doesn't exist
  ```json
  { "error": "Key not found: 550e8400-e29b-41d4-a716-446655440000" }
  ```
- `500 Internal Server Error` - Database error

#### GET /health
Health check endpoint.

**Responses:**
- `200 OK` - Service is healthy and can connect to Spanner
  ```json
  { "status": "healthy" }
  ```
- `503 Service Unavailable` - Cannot connect to Spanner
  ```json
  { "status": "unhealthy", "error": "Cannot connect to database" }
  ```

### Error Response Format

All error responses use a consistent format:
```json
{ "error": "Descriptive error message" }
```

### Content-Type

- All requests with bodies must have `Content-Type: application/json`
- All responses have `Content-Type: application/json`

## Tasks

See [tasks/README.md](./tasks/README.md) for implementation tasks.
