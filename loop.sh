#!/bin/bash

# Check if argument is provided
if [ $# -eq 0 ]; then
    echo "Error: Please provide the number of times to run step.sh"
    echo "Usage: $0 <number>"
    exit 1
fi

# Check if argument is a positive integer
if ! [[ "$1" =~ ^[0-9]+$ ]]; then
    echo "Error: Argument must be a positive integer"
    echo "Usage: $0 <number>"
    exit 1
fi

count=$1

# Check if step.sh exists
if [ ! -f "step.sh" ]; then
    echo "Error: step.sh not found in current directory"
    exit 1
fi

# Loop and call step.sh
echo "Running step.sh $count times..."
for ((i=1; i<=count; i++)); do
    echo ""
    echo "=== Iteration $i of $count ==="
    ./step.sh
    if [ $? -ne 0 ]; then
        echo "Error: step.sh failed on iteration $i"
        exit 1
    fi
done

echo ""
echo "=== Completed all $count iterations successfully ==="
