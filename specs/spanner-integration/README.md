# Spanner Integration

Database connection, schema management, and CRUD operations for the key-value store.

## Overview

The service connects to Google Cloud Spanner (or the emulator) to store and retrieve JSON documents. The schema is automatically created on startup if it doesn't exist.

## Requirements

### Rust Version
- Use the latest stable Rust version
- All dependencies must use their latest available versions

### Database Schema

A single table for storing key-value pairs:

```sql
CREATE TABLE kv_store (
    id STRING(36) NOT NULL,
    data JSON NOT NULL,
    created_at TIMESTAMP NOT NULL OPTIONS (allow_commit_timestamp=true),
    updated_at TIMESTAMP NOT NULL OPTIONS (allow_commit_timestamp=true),
) PRIMARY KEY (id)
```

- `id`: UUID string (36 characters with hyphens)
- `data`: Native Spanner JSON column type
- `created_at`: Timestamp when record was first created
- `updated_at`: Timestamp of last update

### Auto-Provisioning

On startup, the service should automatically set up the complete Spanner environment:
1. Create Spanner admin clients (instance admin and database admin)
2. Check if the configured instance exists, create it if not
3. Check if the configured database exists in that instance, create it if not
4. Check if the `kv_store` table exists in that database, create it if not
5. If everything exists, continue (no schema versioning needed for this simple case)

This allows developers to start the emulator and run the service without any manual setup. The service uses Spanner's admin APIs to provision all required resources.

### CRUD Operations

#### Write (Upsert)
- Insert or update a JSON document at a given UUID key
- Set `created_at` on insert, update `updated_at` on every write
- Use commit timestamps for both fields

#### Read
- Retrieve JSON document by UUID key
- Return `None` if key doesn't exist

### Connection Management

- Create a Spanner client at startup using config
- The client should be shared across request handlers
- Handle connection errors gracefully with descriptive messages

## Tasks

See [tasks/README.md](./tasks/README.md) for implementation tasks.
