#!/bin/bash

# Stop the rust-spanner-kv server if it's running

PID=$(lsof -ti:3000 2>/dev/null)

if [ -z "$PID" ]; then
    echo "Server is not running on port 3000"
    exit 0
fi

echo "Stopping server (PID: $PID)..."
kill -9 $PID 2>/dev/null

# Verify it stopped
sleep 0.5
if lsof -ti:3000 >/dev/null 2>&1; then
    echo "Warning: Server may still be running"
    exit 1
else
    echo "Server stopped successfully"
    exit 0
fi
