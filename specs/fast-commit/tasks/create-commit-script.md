# Create Commit Script

Write a shell script at `scripts/commit.sh` that performs fast git commits.

## Requirements

1. Accept commit message as first argument
2. Fail with usage message if no argument provided
3. Stage all changes: `git add -A`
4. Commit with:
   - The provided message
   - Co-Authored-By trailer: `Co-Authored-By: Claude <noreply@anthropic.com>`
5. On success: output only the short commit hash
6. On failure: let git's error propagate, exit non-zero
7. Make the script executable

## Script Location

```
scripts/commit.sh
```

## Example Usage

```bash
$ ./scripts/commit.sh "Add user authentication"
a1b2c3d
```

## Acceptance Criteria

- [x] Script exists at scripts/commit.sh
- [x] Script is executable
- [x] Commit message is required (fails gracefully if missing)
- [x] All changes are staged before commit
- [x] Co-Authored-By line is included in every commit
- [x] Only the short hash is output on success
- [x] Execution completes in under 1 second (no network calls, no LLM)
