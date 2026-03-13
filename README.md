# rust-spanner-kv

A JSON key-value store web service built with Rust/Axum and backed by Google Cloud Spanner.

## Quick Start

```bash
cp .env.example .env
docker-compose up -d
cargo run
```

Service starts on `http://localhost:3000`.

## API

### `GET /health`
Returns service health status. Queries Spanner to verify connectivity.

```bash
curl http://localhost:3000/health
```
```json
{"status": "healthy"}
```

---

### `PUT /kv/{id}`
Store a JSON document. `id` must be a UUID. Creates or overwrites the entry.

```bash
curl -X PUT http://localhost:3000/kv/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{"name": "test", "value": 42}'
```
```json
{"id": "550e8400-e29b-41d4-a716-446655440000"}
```

---

### `GET /kv/{id}`
Retrieve a JSON document by UUID. Returns 404 if not found.

```bash
curl http://localhost:3000/kv/550e8400-e29b-41d4-a716-446655440000
```
```json
{"id": "550e8400-e29b-41d4-a716-446655440000", "data": {"name": "test", "value": 42}}
```

---

### `GET /kv`
List all entries. Supports pagination, prefix filtering, and sorting.

| Param | Description | Default |
|-------|-------------|---------|
| `limit` | Max results to return | (all) |
| `offset` | Results to skip | `0` |
| `prefix` | Filter keys by prefix | (none) |
| `sort` | `key_asc`, `key_desc`, `created_asc`, `created_desc`, `updated_asc`, `updated_desc` | `key_asc` |

```bash
curl "http://localhost:3000/kv?limit=10&sort=created_desc"
```
```json
{
  "data": [
    {
      "key": "550e8400-e29b-41d4-a716-446655440000",
      "value": {"name": "test", "value": 42},
      "created_at": "2026-03-13T16:32:26.485579+00:00",
      "updated_at": "2026-03-13T16:32:26.485579+00:00"
    }
  ],
  "total_count": 1
}
```

---

Interactive docs: `http://localhost:3000/swagger-ui`
OpenAPI spec: `GET /api-doc/openapi.json`

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `SPANNER_EMULATOR_HOST` | Emulator address (unset for production) | `localhost:9010` |
| `SPANNER_PROJECT` | Google Cloud project ID | `test-project` |
| `SPANNER_INSTANCE` | Spanner instance name | `test-instance` |
| `SPANNER_DATABASE` | Spanner database name | `test-database` |
| `SERVICE_PORT` | HTTP server port | `3000` |
| `SERVICE_HOST` | HTTP server bind address | `0.0.0.0` |

On first startup, the service automatically creates the Spanner instance, database, and table.
