# Environment Files

Create .env and .env.example files for local development.

## Requirements

1. Create `.env.example` with all variables documented:
   ```
   # Spanner Emulator (comment out for production)
   SPANNER_EMULATOR_HOST=localhost:9010

   # Spanner Configuration
   SPANNER_PROJECT=test-project
   SPANNER_INSTANCE=test-instance
   SPANNER_DATABASE=test-database

   # Service Configuration
   SERVICE_PORT=3000
   SERVICE_HOST=0.0.0.0
   ```
2. Create `.env` with working local values
3. Add `.env` to `.gitignore` (keep .env.example tracked)

## Acceptance Criteria

- [ ] .env.example created with all variables
- [ ] .env created with working local values
- [ ] .env is in .gitignore
- [ ] .env.example is tracked in git
