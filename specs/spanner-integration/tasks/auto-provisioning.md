# Auto-Provisioning

Automatically create Spanner instance, database, and table on startup.

## Requirements

1. Create instance admin and database admin clients
2. On startup, perform auto-provisioning:
   - Check if configured instance exists
   - If not, create instance using admin API
   - Check if configured database exists in that instance
   - If not, create database using admin API
   - Check if `kv_store` table exists in that database
   - If not, execute CREATE TABLE DDL
3. Log each provisioning step (created vs already existed)
4. Handle errors gracefully with descriptive messages
5. Table schema:
   ```sql
   CREATE TABLE kv_store (
       id STRING(36) NOT NULL,
       data JSON NOT NULL,
       created_at TIMESTAMP NOT NULL OPTIONS (allow_commit_timestamp=true),
       updated_at TIMESTAMP NOT NULL OPTIONS (allow_commit_timestamp=true),
   ) PRIMARY KEY (id)
   ```

## Implementation Notes
- For emulator, instance config can be simple (e.g., "emulator-config")
- Database creation should specify Google Standard SQL dialect
- Use admin APIs, not gcloud commands
- This enables zero-setup local development

## Acceptance Criteria

- [x] Instance existence check works
- [x] Instance is created if it doesn't exist
- [x] Database existence check works
- [x] Database is created if it doesn't exist
- [x] Table existence check works
- [x] Table is created if it doesn't exist
- [x] Existing resources are not modified
- [x] All errors produce descriptive messages
- [x] Startup logs indicate provisioning status for each resource
- [x] `cargo build` succeeds
- [x] Works with emulator (docker-compose)
