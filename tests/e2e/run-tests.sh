#!/bin/bash -e

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
    echo "✅ Linkage Result (Client Role) Test: passed!"
else
    echo "🆘 Linkage Result (Client Role) Test: failed. Status code: $RESPONSE_CLIENT_ROLE"
    exit 1
fi
