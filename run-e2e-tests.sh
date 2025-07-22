#!/bin/bash -e

# Start the server in the background
source ./start-server.sh

# Run the test scenarios
source ./tests/e2e/run-tests.sh

# No need to manually terminate the server as it will be handled by the trap
