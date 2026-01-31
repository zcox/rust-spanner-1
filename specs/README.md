# rust-spanner-kv Specifications

Design documentation for rust-spanner-kv, a simple JSON key-value store web service built with Rust/Axum and backed by Google Cloud Spanner.

## Current Status Notes

Project not yet started. All specs defined, ready to begin implementation with Local Development spec first (need emulator running to develop against).

## Build a JSON Key-Value Store Service

| Status | Spec | Purpose |
|--------|------|---------|
| ✅ | [Local Development](./local-development/README.md) | Docker Compose setup with Spanner emulator (no manual setup needed) |
| ✅ | [Configuration](./configuration/README.md) | Environment variable configuration for Spanner and service settings |
| ✅ | [Spanner Integration](./spanner-integration/README.md) | Database connection, auto-provisioning, and CRUD operations |
| ✅ | [HTTP API](./http-api/README.md) | REST endpoints for storing and retrieving JSON data |
| 🔄 | [List Key-Value Pairs](./list-kv/README.md) | GET endpoint to list all KV pairs with pagination/filtering |

## Developer Tooling

| Status | Spec | Purpose |
|--------|------|---------|
| ✅ | [Fast Commit](./fast-commit/README.md) | Shell script for sub-1-second git commits |
