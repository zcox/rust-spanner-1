#!/bin/bash

echo "/implement-next-task" | claude -p --output-format stream-json --verbose --dangerously-skip-permissions | ./parse-claude --compact | ./show-turn-timing.sh
