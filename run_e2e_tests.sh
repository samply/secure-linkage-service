#!/bin/bash -e

# Initialize the server and redirect output to server.log
./try-server.sh > server.log 2>&1 &

# Capture the server PID to terminate it later
SERVER_PID=$!

# Function to terminate the server
terminate_server() {
  echo "Terminating the server with PID: $SERVER_PID"
  kill $SERVER_PID
  wait $SERVER_PID 2>/dev/null
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
        cat server.log
        exit 1
    fi
    sleep 1
done
echo "Server started after $ATTEMPT seconds"

# Test Scenarios


echo "Testing Endpoint: Linkage Result..."


# Linkage Test scenario 1: Server Role
echo "Testing scenario: Linkage Result (Server Role)..."
RESPONSE_SERVER_ROLE=$(curl -s -w "%{http_code}" -o /dev/null -H "Content-Type: application/json" \
  -d '{
       "role": "server",
       "result": [
         {
           "match": 0,
           "tentativeMatch": 1,
           "bestIndex": 2
         },
         {
           "match": 1,
           "tentativeMatch": 0,
           "bestIndex": 0
         },
         {
           "match": 0,
           "tentativeMatch": 0,
           "bestIndex": 4
         }
       ],
       "ids": ["aGVsbG93b3JsZA==", "ZGF0YXNjaWVuY2U=", "cHl0aG9uaXNmdW4="]
     }' http://localhost:5000/linkageResult/dkfz/tuda)

if [ "$RESPONSE_SERVER_ROLE" -eq 200 ]; then
    echo "✅ Linkage Result (Server Role) Test: passed!"
else
    echo "🆘 Linkage Result (Server Role) Test: failed. Status code: $RESPONSE_SERVER_ROLE"
    exit 1
fi


# Linkage Test scenario 2: Client Role
echo "Testing scenario: Linkage Result (Client Role)..."
RESPONSE_CLIENT_ROLE=$(curl -s -w "%{http_code}" -o /dev/null -H "Content-Type: application/json" \
  -d '{
       "role": "client",
       "result": [
         {
           "match": 1,
           "tentativeMatch": 0,
           "bestIndex": 2
         },
         {
           "match": 0,
           "tentativeMatch": 1,
           "bestIndex": 0
         },
         {
           "match": 1,
           "tentativeMatch": 1,
           "bestIndex": 4
         }
       ]
     }' http://localhost:5000/linkageResult/tuda/dkfz)

if [ "$RESPONSE_CLIENT_ROLE" -eq 200 ]; then
    echo "✅ Linkage Result (Client Role) Test: passed."
else
    echo "🆘 Linkage Result (Client Role) Test: failed. Status code: $RESPONSE_CLIENT_ROLE"
    exit 1
fi

# No need to manually terminate the server as it will be handled by the trap
