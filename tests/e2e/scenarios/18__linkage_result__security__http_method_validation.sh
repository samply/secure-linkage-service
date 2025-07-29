#!/bin/bash
set -e
source "$(dirname "$0")/../helpers/test_helpers.sh"

test_log "=== HTTP Method Validation Test ==="

check_server

valid_payload='{
    "role": "server",
    "result": [{"match": 1, "tentativeMatch": 0, "bestIndex": 0}],
    "ids": ["aGVsbG93b3JsZA=="]
}'

# Test 1: GET method (should not be allowed)
test_log "Testing GET method"
response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X GET \
    "$BASE_URL/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" 2>/dev/null)

assert "[ \"$response\" = \"405\" ] || [ \"$response\" = \"404\" ]" "GET method rejected with 405 or 404"

# Test 2: PUT method (should not be allowed)
test_log "Testing PUT method"
response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X PUT \
    -H "Content-Type: application/json" \
    -d "$valid_payload" \
    "$BASE_URL/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" 2>/dev/null)

assert "[ \"$response\" = \"405\" ]" "PUT method rejected with 405"

# Test 3: DELETE method (should not be allowed)
test_log "Testing DELETE method"
response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X DELETE \
    "$BASE_URL/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" 2>/dev/null)

assert "[ \"$response\" = \"405\" ]" "DELETE method rejected with 405"

# Test 4: PATCH method (should not be allowed)
test_log "Testing PATCH method"
response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X PATCH \
    -H "Content-Type: application/json" \
    -d "$valid_payload" \
    "$BASE_URL/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" 2>/dev/null)

assert "[ \"$response\" = \"405\" ]" "PATCH method rejected with 405"

# Test 5: HEAD method
test_log "Testing HEAD method"
response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X HEAD \
    "$BASE_URL/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" 2>/dev/null)

assert "[ \"$response\" = \"405\" ] || [ \"$response\" = \"404\" ]" "HEAD method rejected with 405 or 404"

# Test 6: OPTIONS method (might be allowed for CORS)
test_log "Testing OPTIONS method"
response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X OPTIONS \
    "$BASE_URL/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" 2>/dev/null)

# OPTIONS might be allowed for CORS, so we accept 200, 204, or 405
assert "[ \"$response\" = \"200\" ] || [ \"$response\" = \"204\" ] || [ \"$response\" = \"405\" ]" "OPTIONS method handled appropriately"

# Test 7: Custom method (should not be allowed)
test_log "Testing custom method"
response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
    -X CUSTOM \
    -H "Content-Type: application/json" \
    -d "$valid_payload" \
    "$BASE_URL/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" 2>/dev/null)

assert "[ \"$response\" = \"405\" ] || [ \"$response\" = \"400\" ]" "Custom method rejected"

cleanup_temp_files
test_log "✅ HTTP Method Validation Test completed successfully"