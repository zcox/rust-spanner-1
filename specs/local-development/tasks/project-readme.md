# Project README

Write comprehensive project documentation.

## Requirements

Create `README.md` at project root with:

1. **Project Title and Description**
   - rust-spanner-kv
   - Brief description of what it does

2. **Prerequisites**
   - Rust (latest stable)
   - Docker and Docker Compose

3. **Quick Start**
   - Clone repo
   - Copy .env.example to .env
   - Start emulator: `docker-compose up -d`
   - Run service: `cargo run` (auto-creates instance/database/table on first run)
   - Verify with curl

4. **API Reference**
   - POST /kv/:id - Store document
   - GET /kv/:id - Retrieve document
   - GET /health - Health check

5. **Configuration Reference**
   - Table of all environment variables

6. **Example Usage**
   - Complete curl/jq examples for all operations

## Acceptance Criteria

- [x] README.md created at project root
- [x] All sections included
- [x] Examples are accurate and work
- [x] Prerequisites are complete
- [x] Quick start is easy to follow
