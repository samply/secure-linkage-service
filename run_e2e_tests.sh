#!/bin/bash -e

# Initialize the server using the try-server.sh script
./try-server.sh &

# Capture the server PID to terminate it later
SERVER_PID=$!

# Function to terminate the server
#!/bin/bash -e

# Initialize the server using the try-server.sh script
./try-server.sh &

# Capture the server PID to terminate it later
SERVER_PID=$!

# Function to terminate the server
#!/bin/bash -e

# Initialize the server using the try-server.sh script
./try-server.sh &

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

# Wait for the server to start
sleep 10

# Test scenario 1: Server Role
echo "Running test scenario 1: Server Role"
RESPONSE_1=$(curl -s -w "%{http_code}" -o /dev/null -H "Content-Type: application/json" \
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

# Assertion: Check if the response status code is 200
if [ "$RESPONSE_1" -eq 200 ]; then
    echo "✅ Test scenario 1 passed."
else
    echo "🆘 Test scenario 1 failed. Status code: $RESPONSE_1"
    exit 1
fi

# Test scenario 2: Client Role
echo "Running test scenario 2: Client Role"
RESPONSE_2=$(curl -s -w "%{http_code}" -o /dev/null -H "Content-Type: application/json" \
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

# Assertion: Check if the response status code is 200
if [ "$RESPONSE_2" -eq 200 ]; then
    echo "✅ Test scenario 2 passed."
else
    echo "🆘 Test scenario 2 failed. Status code: $RESPONSE_2"
    exit 1
fi

# No need to manually terminate the server as it will be handled by the trap
