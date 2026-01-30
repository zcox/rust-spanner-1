# Axum Server Setup

Create the Axum application with shared state and routing.

## Requirements

1. Create Axum app with:
   - Router with routes for all endpoints
   - Shared state containing Spanner client and config
2. Configure server to bind to host:port from config
3. Set up tracing/logging middleware
4. Start server in main() after config and Spanner setup

## Acceptance Criteria

- [x] Axum app created with router
- [x] Shared state includes Spanner client
- [x] Server binds to configured host:port
- [x] Request logging enabled
- [x] `cargo build` succeeds
- [x] Server starts without errors (even if endpoints not implemented)
