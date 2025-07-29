#!/bin/bash
# Script to run a single test scenario for debugging

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <test_number>"
    echo "Example: $0 01  # runs 01_basic_server_client_flow.sh"
    exit 1
fi

TEST_NUMBER="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Find the test script
TEST_SCRIPT=$(find "$SCRIPT_DIR/scenarios" -name "${TEST_NUMBER}_*.sh" | head -1)

if [ -z "$TEST_SCRIPT" ]; then
    echo "❌ Test script not found for number: $TEST_NUMBER"
    echo "Available tests:"
    ls "$SCRIPT_DIR/scenarios/"
    exit 1
fi

echo "Running single test: $(basename "$TEST_SCRIPT")"

# Check if server is running
if ! curl -s -o /dev/null --max-time 1 http://localhost:5000 2>/dev/null; then
    echo "❌ Server is not running. Please start the server first."
    echo "Run: ./start-server.sh"
    exit 1
fi

# Run the specific test
bash "$TEST_SCRIPT"

echo "✅ Single test completed: $(basename "$TEST_SCRIPT")"
