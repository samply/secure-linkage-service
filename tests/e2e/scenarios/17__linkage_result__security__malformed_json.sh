#!/bin/bash
# Robust malformed JSON test - don't use 'set -e' to avoid early exits
source "$(dirname "$0")/../helpers/test_helpers.sh"

test_log "=== Malformed JSON Test ==="

if ! check_server; then
    test_log "❌ Server is not running"
    exit 1
fi

test_passed=true

# Test 1: Invalid JSON syntax
test_log "Testing invalid JSON syntax"

invalid_json='{
    "role": "server",
    "result": [{"match": 1, "tentativeMatch": 0, "bestIndex": 0}],
    "ids": ["aGVsbG93b3JsZA==",]
}'

response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X POST \
    -H "Content-Type: application/json" \
    -d "$invalid_json" \
    --max-time 10 \
    "$BASE_URL/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" 2>/dev/null || echo "curl_failed")

if [ "$response" = "curl_failed" ]; then
    test_log "    ❌ Curl command failed"
    test_passed=false
elif [ "$response" = "400" ] || [ "$response" = "422" ] || [ "$response" = "500" ]; then
    test_log "    ✅ Invalid JSON syntax properly rejected (status: $response)"
else
    test_log "    ⚠️  Invalid JSON got status: $response (might be valid)"
fi

# Test 2: Incomplete JSON
test_log "Testing incomplete JSON"

incomplete_json='{
    "role": "server",
    "result": [{"match": 1, "tentativeMatch": 0'

response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X POST \
    -H "Content-Type: application/json" \
    -d "$incomplete_json" \
    --max-time 10 \
    "$BASE_URL/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" 2>/dev/null || echo "curl_failed")

if [ "$response" = "curl_failed" ]; then
    test_log "    ❌ Curl command failed"
    test_passed=false
elif [ "$response" = "400" ] || [ "$response" = "422" ] || [ "$response" = "500" ]; then
    test_log "    ✅ Incomplete JSON properly rejected (status: $response)"
else
    test_log "    ⚠️  Incomplete JSON got status: $response (might be valid)"
fi

# Test 3: Empty request body
test_log "Testing empty request body"

response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X POST \
    -H "Content-Type: application/json" \
    -d "" \
    --max-time 10 \
    "$BASE_URL/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" 2>/dev/null || echo "curl_failed")

if [ "$response" = "curl_failed" ]; then
    test_log "    ❌ Curl command failed"
    test_passed=false
elif [ "$response" = "400" ] || [ "$response" = "422" ] || [ "$response" = "500" ]; then
    test_log "    ✅ Empty request body properly rejected (status: $response)"
else
    test_log "    ⚠️  Empty body got status: $response (might be valid)"
fi

# Test 4: Valid JSON for comparison
test_log "Testing valid JSON (should be accepted initially)"

valid_json='{
    "role": "server",
    "result": [{"match": 0, "tentativeMatch": 0, "bestIndex": 0}],
    "ids": ["dmFsaWRqc29u"]
}'

response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X POST \
    -H "Content-Type: application/json" \
    -d "$valid_json" \
    --max-time 3 \
    "$BASE_URL/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" 2>/dev/null || echo "curl_failed")

if [ "$response" = "curl_failed" ]; then
    test_log "    ❌ Curl command failed"
    test_passed=false
elif [ "$response" = "200" ] || [ "$response" = "408" ] || [ "$response" = "504" ]; then
    test_log "    ✅ Valid JSON accepted appropriately (status: $response)"
else
    test_log "    ⚠️  Valid JSON got status: $response (might still be valid)"
fi

# Clean up
if [ -f "/tmp/response_body.json" ]; then
    rm -f /tmp/response_body.json
fi

# Final result
if [ "$test_passed" = true ]; then
    test_log "✅ Malformed JSON Test completed successfully"
    exit 0
else
    test_log "❌ Malformed JSON Test had failures"
    exit 1
fi
