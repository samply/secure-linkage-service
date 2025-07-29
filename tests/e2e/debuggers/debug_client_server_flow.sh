#!/bin/bash
# Debug script to understand the client-server flow issue

set -e

echo "🔍 Debugging Client-Server Flow"
echo "=============================="

# Check if server is running
if ! curl -s -o /dev/null --max-time 1 http://localhost:5000 2>/dev/null; then
    echo "❌ Server is not running. Please start the server first."
    exit 1
fi

echo "✅ Server is running"

# Test data
client_result_data='[
    {"match": 0, "tentativeMatch": 0, "bestIndex": 0},
    {"match": 0, "tentativeMatch": 0, "bestIndex": 1},
    {"match": 0, "tentativeMatch": 0, "bestIndex": 2}
]'

server_result_data='[
    {"match": 0, "tentativeMatch": 0, "bestIndex": 0},
    {"match": 0, "tentativeMatch": 0, "bestIndex": 1},
    {"match": 0, "tentativeMatch": 0, "bestIndex": 2}
]'

ids_data='[
    "dGVzdGRhdGE0NTY3ODkwMTIzZGVmZ2hpamtsbW4=",
    "dGVzdGRhdGE1Njc4OTAxMjM0ZWZnaGlqa2xtbm8=",
    "dGVzdGRhdGE2Nzg5MDEyMzQ1ZmdoaWprbG1ub3A="
]'

echo ""
echo "🎯 Understanding the Request Matching Logic"
echo "=========================================="
echo "Based on your server code, the matching works like this:"
echo "- Client request from A→B looks for server request from B→A"
echo "- Server request from B→A looks for client request from A→B"
echo ""

echo "📋 Current test setup:"
echo "- Client request: tuda → dkfz"
echo "- Server request: dkfz → tuda"
echo "- Expected match: client(tuda→dkfz) should find server(dkfz→tuda)"
echo ""

echo "🧪 Test 1: Server request first (should work)"
echo "============================================"

# Start server request first
echo "Starting server request: dkfz → tuda"
(
    response=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "{\"role\": \"server\", \"result\": $server_result_data, \"ids\": $ids_data}" \
        --max-time 25 \
        "http://localhost:5000/linkageResult/dkfz/tuda")

    status_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)

    echo "Server response status: $status_code"
    echo "Server response: $(echo "$body" | jq -c '.' 2>/dev/null || echo "$body")"
) &
server_pid=$!

# Wait a moment
sleep 2

# Start client request
echo "Starting client request: tuda → dkfz"
response=$(curl -s -w "\n%{http_code}" \
    -X POST \
    -H "Content-Type: application/json" \
    -d "{\"role\": \"client\", \"result\": $client_result_data}" \
    --max-time 25 \
    "http://localhost:5000/linkageResult/tuda/dkfz")

status_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | head -n -1)

echo "Client response status: $status_code"
echo "Client response: $(echo "$body" | jq -c '.' 2>/dev/null || echo "$body")"

# Wait for server request
wait $server_pid

if [ "$status_code" = "200" ]; then
    echo "✅ Test 1 PASSED: Server-first approach works"
else
    echo "❌ Test 1 FAILED: Both approaches have issues"
fi

sleep 3

echo ""
echo "🧪 Test 2: Client request first with longer timeout"
echo "================================================="

# Start client request first with longer timeout
echo "Starting client request: tuda → dkfz (with 30s timeout)"
(
    response=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "{\"role\": \"client\", \"result\": $client_result_data}" \
        --max-time 35 \
        "http://localhost:5000/linkageResult/tuda/dkfz")

    status_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)

    echo "Client response status: $status_code"
    echo "Client response: $(echo "$body" | jq -c '.' 2>/dev/null || echo "$body")"
) &
client_pid=$!

# Wait briefly, then start server request
sleep 1
echo "Starting server request: dkfz → tuda (after 1s delay)"
response=$(curl -s -w "\n%{http_code}" \
    -X POST \
    -H "Content-Type: application/json" \
    -d "{\"role\": \"server\", \"result\": $server_result_data, \"ids\": $ids_data}" \
    --max-time 30 \
    "http://localhost:5000/linkageResult/dkfz/tuda")

server_status=$(echo "$response" | tail -n1)
server_body=$(echo "$response" | head -n -1)

echo "Server response status: $server_status"
echo "Server response: $(echo "$server_body" | jq -c '.' 2>/dev/null || echo "$server_body")"

# Wait for client request
wait $client_pid
client_exit_code=$?

if [ $client_exit_code -eq 0 ]; then
    echo "✅ Test 2 PASSED: Client-first approach works with shorter delay"
else
    echo "❌ Test 2 FAILED: Client-first still has timeout issues"
fi

echo ""
echo "💡 Diagnosis Results:"
echo "===================="
echo "If Test 1 passed but Test 2 failed:"
echo "  → The server timeout is shorter than expected"
echo "  → Need to reduce delay or increase server timeout"
echo ""
echo "If both tests failed:"
echo "  → Check request matching logic in server code"
echo "  → Verify party names match exactly"
echo ""
echo "🔧 Recommended fix for 02_basic_client_server_flow.sh:"
echo "- Reduce sleep delay from 0.5s to 0.1s"
echo "- Or use a different approach that starts server first"
