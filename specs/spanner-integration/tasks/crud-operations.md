# CRUD Operations

Implement upsert and read operations for the key-value store.

## Requirements

### Upsert Operation
1. Function signature: `async fn upsert(client, id: Uuid, data: serde_json::Value) -> Result<()>`
2. Insert or replace row with given ID
3. Set `created_at` and `updated_at` to commit timestamp
4. Handle Spanner errors with descriptive messages

### Read Operation
1. Function signature: `async fn read(client, id: Uuid) -> Result<Option<serde_json::Value>>`
2. Query row by ID
3. Return `Some(data)` if found, `None` if not found
4. Handle Spanner errors with descriptive messages

### Implementation Notes
- Use Spanner mutations for upsert (InsertOrUpdate)
- Use Spanner read or SQL query for read
- Ensure proper JSON serialization/deserialization

## Acceptance Criteria

- [x] Upsert function implemented
- [x] Read function implemented
- [x] Functions handle errors gracefully
- [x] JSON data round-trips correctly
- [x] `cargo build` succeeds
- [x] Unit tests pass (if applicable)
