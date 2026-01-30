# Docker Compose Setup

Create docker-compose.yml for the Spanner emulator.

## Requirements

1. Create `docker-compose.yml` with Spanner emulator service:
   - Image: `gcr.io/cloud-spanner-emulator/emulator`
   - Ports: 9010 (gRPC), 9020 (REST)
   - Container name for easy reference
2. Verify emulator starts correctly with `docker-compose up`

## Acceptance Criteria

- [ ] docker-compose.yml created
- [ ] Spanner emulator starts with `docker-compose up`
- [ ] Ports 9010 and 9020 are accessible
- [ ] Emulator responds to health checks
