# Add List Query to Spanner Layer

Add a function to the Spanner module that queries all key-value pairs with support for filtering, sorting, and pagination.

## Requirements

### Function Signature

Add a `list_all` function (or similar) to the Spanner client that:
- Accepts optional parameters for limit, offset, prefix filter, and sort order
- Returns a vector of key-value records with timestamps
- Returns the total count of matching records (for pagination UI)

### Query Building

Build the SQL query dynamically based on parameters:
- Base query selects `id`, `data`, `created_at`, `updated_at` from `kv_store`
- Apply `WHERE id LIKE @prefix || '%'` if prefix is provided
- Apply `ORDER BY` clause based on sort parameter
- Apply `LIMIT` and `OFFSET` if provided

### Count Query

Run a separate count query to get `total_count`:
- `SELECT COUNT(*) FROM kv_store` (with same WHERE clause if prefix filter)

### Return Type

Define a struct for the list result:
```rust
struct KvEntry {
    key: String,
    value: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

struct ListResult {
    entries: Vec<KvEntry>,
    total_count: i64,
}
```

## Acceptance Criteria

- [x] Function handles all sort options correctly
- [x] Prefix filtering works with SQL LIKE pattern
- [x] Pagination with limit/offset works correctly
- [x] Total count reflects filtered count when prefix is used
- [x] Empty result returns empty vec with count 0
