#!/usr/bin/env bash
set -euo pipefail

# Fast git commit script
# Usage: ./scripts/commit.sh "commit message"

if [ $# -eq 0 ]; then
    echo "Error: Commit message required" >&2
    echo "Usage: $0 \"commit message\"" >&2
    exit 1
fi

commit_message="$1"

# Stage all changes
git add -A

# Commit with message and Co-Authored-By trailer
git commit -m "$commit_message" -m "Co-Authored-By: Claude <noreply@anthropic.com>"

# Output only the short commit hash
git rev-parse --short HEAD
