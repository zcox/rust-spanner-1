# Config Struct and Loading

Implement the configuration struct and environment variable loading.

## Requirements

1. Create a `Config` struct with fields for all environment variables:
   - `spanner_emulator_host: Option<String>`
   - `spanner_project: String`
   - `spanner_instance: String`
   - `spanner_database: String`
   - `service_port: u16`
   - `service_host: String`
2. Implement a `Config::from_env()` function that:
   - Reads from environment variables
   - Applies defaults for optional variables
   - Returns descriptive errors for missing/invalid variables
3. Load config in main and log startup configuration
4. Create `.env.example` file documenting all variables

## Acceptance Criteria

- [x] Config struct defined with all fields
- [x] `Config::from_env()` reads environment variables
- [x] Missing required variables produce clear error messages
- [x] Invalid values (e.g., non-numeric port) produce clear error messages
- [x] `.env.example` documents all variables with example values
- [x] `cargo build` succeeds
- [x] Running without required vars shows descriptive error
