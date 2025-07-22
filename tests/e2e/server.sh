#!/usr/bin/env bash

# Destroy the current virtual environment
rm -rf ./venv

# Create the virtual environment
python3 -m venv ./venv

# Activate the virtual environment
source venv/bin/activate

# Install the dependencies
pip install -r requirements.txt

# Start the server in development mode (without TLS) in the background
LS_NO_TLS=true python3 src/main.py &
SERVER_PYTHON_PID=$!

# Function to clean up processes
cleanup() {
    echo "Stopping server (PID: $SERVER_PYTHON_PID)"
    kill $SERVER_PYTHON_PID 2>/dev/null
    deactivate
}
trap cleanup EXIT

# Wait indefinitely until terminated
wait $SERVER_PYTHON_PID
