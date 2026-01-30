# Initialize Rust Project

Create the Cargo project structure with latest Rust and dependencies.

## Requirements

1. Initialize a new Cargo binary project named `rust-spanner-kv`
2. Configure `Cargo.toml` with:
   - Latest Rust edition (2024 if available, otherwise 2021)
   - All dependencies at their latest versions
3. Required dependencies:
   - `axum` - HTTP framework
   - `tokio` - async runtime (full features)
   - `serde` + `serde_json` - JSON serialization
   - `google-cloud-spanner` (or equivalent gcloud-spanner crate)
   - `uuid` - UUID generation/validation
   - `tracing` + `tracing-subscriber` - logging
   - `anyhow` or `thiserror` - error handling
4. Create basic `src/main.rs` with async main function
5. Verify project compiles with `cargo build`

## Acceptance Criteria

- [x] `cargo build` succeeds
- [x] All dependencies are at their latest versions
- [x] Project structure follows Rust conventions
