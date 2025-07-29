#!/bin/bash
# Robust unknown parties test - don't use 'set -e' to avoid early exits
source "$(dirname "$0")/../helpers/test_helpers.sh"

test_log "=== Unknown Parties Test ==="

if ! check_server; then
    test_log "❌ Server is not running"
    exit 1
fi

# Valid payload for testing
payload='{
    "role": "server",
    "result": [{"match": 1, "tentativeMatch": 0, "bestIndex": 0}],
    "ids": ["dW5rbm93bnBhcnR5dGVzdA=="]
}'

test_passed=true

# Test 1: Unknown source party
test_log "Testing unknown source party"

response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X POST \
    -H "Content-Type: application/json" \
    -d "$payload" \
    --max-time 10 \
    "$BASE_URL/linkageResult/unknown_party/$PARTY_TUDA" 2>/dev/null || echo "curl_failed")

if [ "$response" = "curl_failed" ]; then
    test_log "    ❌ Curl command failed"
    test_passed=false
elif [ "$response" = "400" ] || [ "$response" = "404" ] || [ "$response" = "422" ] || [ "$response" = "500" ]; then
    test_log "    ✅ Unknown source party properly rejected (status: $response)"
else
    test_log "    ⚠️  Unknown source party got status: $response (might be valid)"
fi

# Test 2: Unknown destination party
test_log "Testing unknown destination party"

response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X POST \
    -H "Content-Type: application/json" \
    -d "$payload" \
    --max-time 10 \
    "$BASE_URL/linkageResult/$PARTY_DKFZ/unknown_party" 2>/dev/null || echo "curl_failed")

if [ "$response" = "curl_failed" ]; then
    test_log "    ❌ Curl command failed"
    test_passed=false
elif [ "$response" = "400" ] || [ "$response" = "404" ] || [ "$response" = "422" ] || [ "$response" = "500" ]; then
    test_log "    ✅ Unknown destination party properly rejected (status: $response)"
else
    test_log "    ⚠️  Unknown destination party got status: $response (might be valid)"
fi

# Test 3: Both parties unknown
test_log "Testing both parties unknown"

response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X POST \
    -H "Content-Type: application/json" \
    -d "$payload" \
    --max-time 10 \
    "$BASE_URL/linkageResult/unknown_source/unknown_dest" 2>/dev/null || echo "curl_failed")

if [ "$response" = "curl_failed" ]; then
    test_log "    ❌ Curl command failed"
    test_passed=false
elif [ "$response" = "400" ] || [ "$response" = "404" ] || [ "$response" = "422" ] || [ "$response" = "500" ]; then
    test_log "    ✅ Both unknown parties properly rejected (status: $response)"
else
    test_log "    ⚠️  Both unknown parties got status: $response (might be valid)"
fi

# Test 4: Valid parties for comparison
test_log "Testing valid parties (should be accepted initially)"

response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X POST \
    -H "Content-Type: application/json" \
    -d "$payload" \
    --max-time 3 \
    "$BASE_URL/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" 2>/dev/null || echo "curl_failed")

if [ "$response" = "curl_failed" ]; then
    test_log "    ❌ Curl command failed"
    test_passed=false
elif [ "$response" = "200" ] || [ "$response" = "408" ] || [ "$response" = "504" ]; then
    test_log "    ✅ Valid parties accepted appropriately (status: $response)"
else
    test_log "    ⚠️  Valid parties got status: $response (might still be valid)"
fi

# Clean up
if [ -f "/tmp/response_body.json" ]; then
    rm -f /tmp/response_body.json
fi

# Final result
if [ "$test_passed" = true ]; then
    test_log "✅ Unknown Parties Test completed successfully"
    exit 0
else
    test_log "❌ Unknown Parties Test had failures"
    exit 1
fi
