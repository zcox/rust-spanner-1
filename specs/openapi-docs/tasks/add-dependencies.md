# Add utoipa Dependencies

Add the required utoipa crates to the project.

## Changes

Add to `Cargo.toml`:

```toml
utoipa = { version = "5", features = ["axum_extras", "uuid"] }
utoipa-swagger-ui = { version = "9", features = ["axum"] }
```

## Features Required

- `axum_extras` - Integration with Axum extractors (Path, Query, Json, State)
- `uuid` - Support for UUID types in schemas
- `axum` (on swagger-ui) - Axum router integration

## Verification

- `cargo build` should succeed after adding dependencies
