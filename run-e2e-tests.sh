#!/bin/bash -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Start the server in the background
echo "Starting server..."
source "$SCRIPT_DIR/start-server.sh" &
SERVER_SCRIPT_PID=$!

# Function to cleanup
cleanup() {
    echo "Cleaning up test environment..."
    kill $SERVER_SCRIPT_PID 2>/dev/null || true
    wait $SERVER_SCRIPT_PID 2>/dev/null || true
}

# Ensure cleanup on exit
trap cleanup EXIT

# Wait a bit for server to be ready
sleep 3

# Run the test scenarios
echo "Running E2E tests..."
bash "$SCRIPT_DIR/tests/e2e/run-all-tests.sh"

echo "E2E tests completed"