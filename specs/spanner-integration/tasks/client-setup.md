# Spanner Client Setup

Create the Spanner client connection from configuration.

## Requirements

1. Create a Spanner client using the config values:
   - Project, instance, and database from config
   - Emulator host if configured
2. Wrap client in a shareable type (Arc) for use in Axum state
3. Implement proper error handling for connection failures
4. Log successful connection on startup

## Acceptance Criteria

- [x] Spanner client created from config
- [x] Client works with emulator when `SPANNER_EMULATOR_HOST` is set
- [x] Connection errors produce descriptive messages
- [x] Client is shareable across async handlers
- [x] `cargo build` succeeds
