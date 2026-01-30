# Configuration

Environment variable configuration for the rust-spanner-kv service.

## Overview

All service configuration is provided via environment variables following 12-factor app principles. The service reads configuration at startup and fails fast with descriptive errors if required variables are missing or invalid.

## Requirements

### Rust Version
- Use the latest stable Rust version
- All dependencies must use their latest available versions

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `SPANNER_EMULATOR_HOST` | No | None | Spanner emulator endpoint (e.g., `localhost:9010`). When set, the client connects to the emulator instead of production Spanner. |
| `SPANNER_PROJECT` | Yes | - | GCP project ID |
| `SPANNER_INSTANCE` | Yes | - | Spanner instance ID |
| `SPANNER_DATABASE` | Yes | - | Spanner database name |
| `SERVICE_PORT` | No | `3000` | HTTP port the service listens on |
| `SERVICE_HOST` | No | `0.0.0.0` | Host address to bind |

### Startup Behavior

1. Load all environment variables at startup
2. Validate required variables are present
3. Validate variable formats (e.g., port is a valid number)
4. If validation fails, exit with a descriptive error message explaining what's missing or invalid
5. Log configuration summary on successful startup (excluding sensitive values)

### Implementation Notes

- Use a config struct that is constructed once at startup
- Consider using a crate like `envy` or manual `std::env` parsing
- The config struct should be passed to components that need it (dependency injection)

## Tasks

See [tasks/README.md](./tasks/README.md) for implementation tasks.
