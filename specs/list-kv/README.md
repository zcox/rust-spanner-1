# List Key-Value Pairs

A GET endpoint to retrieve all key-value pairs from the store with optional pagination, filtering, and sorting.

## Overview

Adds a `GET /kv` endpoint that returns an array of all stored key-value pairs. When no parameters are provided, returns all pairs sorted by key ascending. Supports optional pagination, key prefix filtering, and sorting.

## Requirements

### Endpoint

`GET /kv` - List all key-value pairs

### Response Format

Returns an array of key-value objects with metadata:

```json
{
  "data": [
    {
      "key": "uuid-1",
      "value": { "any": "json" },
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    },
    {
      "key": "uuid-2",
      "value": { "other": "data" },
      "created_at": "2024-01-14T09:00:00Z",
      "updated_at": "2024-01-15T11:00:00Z"
    }
  ],
  "total_count": 42
}
```

### Query Parameters

All parameters are optional:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | none (all) | Maximum number of results to return |
| `offset` | integer | 0 | Number of results to skip |
| `prefix` | string | none | Filter keys starting with this value |
| `sort` | string | `key_asc` | Sort order (see options below) |

### Sort Options

- `key_asc` - Sort by key alphabetically ascending (default)
- `key_desc` - Sort by key alphabetically descending
- `created_asc` - Sort by created_at timestamp ascending (oldest first)
- `created_desc` - Sort by created_at timestamp descending (newest first)
- `updated_asc` - Sort by updated_at timestamp ascending
- `updated_desc` - Sort by updated_at timestamp descending (most recently updated first)

### Default Behavior

When no query parameters are provided:
- Returns all key-value pairs
- Sorted by key ascending
- Includes total count in response

### Error Responses

- `400 Bad Request` - Invalid query parameters (negative limit, invalid sort value)
- `500 Internal Server Error` - Database error

## Tasks

See [tasks/README.md](./tasks/README.md) for implementation tasks.
