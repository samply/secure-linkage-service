#!/bin/bash -e

# Initialize the server and redirect output to server.log
./tests/e2e/server.sh > ../server.log 2>&1 &

# Capture the server PID to terminate it later
SERVER_PID=$!

# Function to terminate the server and its children
terminate_server() {
  echo "Terminating server process group: $SERVER_PID"
  # Kill entire process group to ensure all children are terminated
  kill -- -$SERVER_PID 2>/dev/null || true
  wait $SERVER_PID 2>/dev/null || true
}

# Ensure the server process is killed on script exit
trap terminate_server EXIT

# Wait for server to start using port check
echo "Waiting for server to start on port 5000..."
MAX_ATTEMPTS=30
ATTEMPT=0
while ! curl -s -o /dev/null --max-time 1 http://localhost:5000; do
    ATTEMPT=$((ATTEMPT+1))
    if [ $ATTEMPT -ge $MAX_ATTEMPTS ]; then
        echo "Server did not start within $MAX_ATTEMPTS seconds. Aborting."
        echo "Server logs:"
        cat ../server.log
        exit 1
    fi
    sleep 1
done
echo "Server started after $ATTEMPT seconds"
