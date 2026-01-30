# Local Development

Docker Compose setup with Spanner emulator and usage documentation.

## Overview

Provide everything needed to run the service locally with the Spanner emulator in Docker. The Rust service handles all instance/database/table creation automatically on startup via the emulator's admin APIs.

## Requirements

### Docker Compose

Create a `docker-compose.yml` that runs:
1. Cloud Spanner Emulator
   - Use official `gcr.io/cloud-spanner-emulator/emulator` image
   - Expose gRPC port (9010) and REST port (9020)
   - No additional configuration needed - the Rust service creates instances/databases programmatically

### Environment Configuration

Create `.env.example` and `.env` files for local development:
```
# Spanner Emulator (when set, connects to emulator instead of production)
SPANNER_EMULATOR_HOST=localhost:9010

# Spanner Configuration
SPANNER_PROJECT=test-project
SPANNER_INSTANCE=test-instance
SPANNER_DATABASE=test-database

# Service Configuration
SERVICE_PORT=3000
SERVICE_HOST=0.0.0.0
```

Add `.env` to `.gitignore` to prevent committing local configuration.

### Documentation

Create a README.md at project root with:
1. Project description
2. Prerequisites (Rust latest stable, Docker and Docker Compose)
3. Quick start instructions:
   ```bash
   # Start emulator
   docker-compose up -d

   # Copy environment config
   cp .env.example .env

   # Run service (auto-creates instance/database/table on first run)
   cargo run
   ```
4. API usage examples with curl/jq
5. Configuration reference
6. Note that instance/database/table are created automatically

### End-to-End Verification

Document how to verify everything works:
```bash
# Store a document
curl -X PUT http://localhost:3000/kv/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{"name": "test", "value": 42}' | jq

# Retrieve it
curl http://localhost:3000/kv/550e8400-e29b-41d4-a716-446655440000 | jq

# Health check
curl http://localhost:3000/health | jq
```

## Tasks

See [tasks/README.md](./tasks/README.md) for implementation tasks.
