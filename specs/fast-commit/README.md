# Fast Commit Script

A lightweight shell script that performs git commits in under 1 second by avoiding LLM roundtrips entirely.

## Problem

The current commit workflow at the end of skills like `implement-next-task` takes 10+ seconds due to:
- Spawning a Bash subagent
- Multiple sequential git commands (status, diff, log)
- LLM-generated commit messages requiring additional inference

This creates friction and slows down the development feedback loop.

## Solution

A simple shell script that:
1. Accepts the commit message as an argument
2. Stages all changes and commits in a single operation
3. Returns minimal output

Skills and users call it directly via Bash - no skill wrapper needed.

## Usage

```bash
./scripts/commit.sh "Your commit message here"
```

## Behavior

### Process

1. Stage all changes: `git add -A`
2. Commit with message and Co-Authored-By trailer
3. Output the short commit hash on success
4. Exit with error on failure (pre-commit hooks, nothing to commit, etc.)

### Output

On success: Just the short commit hash (e.g., `a1b2c3d`)

On failure: Git's error message, non-zero exit code

## Non-Goals

- Generating commit messages (caller provides the message)
- Validating changes exist before committing (let git handle it)
- Handling pre-commit hook failures (fail fast, caller retries)

## Performance Target

Total execution time: **under 1 second**

## File Location

```
scripts/commit.sh
```

## Tasks

See [tasks/README.md](./tasks/README.md) for implementation plan.
