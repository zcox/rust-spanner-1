# rust-spanner-kv

A simple JSON key-value store web service built with Rust/Axum and backed by Google Cloud Spanner. The service automatically creates instances, databases, and tables on first run when using the emulator, making local development effortless.

## Prerequisites

- **Rust** (latest stable version)
- **Docker** and **Docker Compose**

## Quick Start

```bash
# Clone the repository
git clone <repository-url>
cd rust-spanner-1

# Copy environment configuration
cp .env.example .env

# Start the Spanner emulator
docker-compose up -d

# Run the service (automatically creates instance/database/table on first run)
cargo run
```

The service will start on `http://localhost:3000` by default.

## API Reference

### Store Document
```
PUT /kv/:id
```
Stores a JSON document with the specified ID.

### Retrieve Document
```
GET /kv/:id
```
Retrieves a JSON document by ID.

### Health Check
```
GET /health
```
Returns the health status of the service.

## OpenAPI Documentation

The service provides interactive API documentation via Swagger UI:

```bash
# Start the service
cargo run

# Open Swagger UI in your browser
open http://localhost:3000/swagger-ui
```

The raw OpenAPI specification is available at:
```
GET /api-doc/openapi.json
```

## Configuration Reference

All configuration is managed through environment variables. Copy `.env.example` to `.env` and modify as needed.

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `SPANNER_EMULATOR_HOST` | Spanner emulator connection (set for local dev, unset for production) | `localhost:9010` | No |
| `SPANNER_PROJECT` | Google Cloud project ID | `test-project` | Yes |
| `SPANNER_INSTANCE` | Spanner instance name | `test-instance` | Yes |
| `SPANNER_DATABASE` | Spanner database name | `test-database` | Yes |
| `SERVICE_PORT` | HTTP server port | `3000` | Yes |
| `SERVICE_HOST` | HTTP server bind address | `0.0.0.0` | Yes |

## Example Usage

### Store a JSON Document

```bash
curl -X PUT http://localhost:3000/kv/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{"name": "test", "value": 42}' | jq
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Retrieve a JSON Document

```bash
curl http://localhost:3000/kv/550e8400-e29b-41d4-a716-446655440000 | jq
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "data": {
    "name": "test",
    "value": 42
  }
}
```

### Health Check

```bash
curl http://localhost:3000/health | jq
```

**Response:**
```json
{
  "status": "healthy"
}
```

## Local Development Notes

When running locally with the Spanner emulator:

- **Automatic Provisioning**: The service automatically creates the Spanner instance, database, and table on first startup. No manual setup required.
- **Emulator Configuration**: Setting `SPANNER_EMULATOR_HOST` tells the service to connect to the local emulator instead of production Spanner.
- **Data Persistence**: Data in the emulator is ephemeral and will be lost when the container is stopped.

## Development Workflow

```bash
# Start the emulator
docker-compose up -d

# Run the service
cargo run

# In another terminal, test the API
curl -X PUT http://localhost:3000/kv/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{"hello": "world"}' | jq

# Stop the emulator when done
docker-compose down
```
