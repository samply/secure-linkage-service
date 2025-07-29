#!/bin/bash
set -e
source "$(dirname "$0")/../helpers/test_helpers.sh"

test_log "=== Invalid Base64 IDs Test ==="

check_server

# Test 1: Invalid base64 string
test_log "Testing invalid base64 ID"
invalid_payload='{
    "role": "server",
    "result": [{"match": 1, "tentativeMatch": 0, "bestIndex": 0}],
    "ids": ["invalid_base64!@#"]
}'

make_request "POST" "/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" "$invalid_payload" 500

# Test 2: Empty string ID
test_log "Testing empty string ID"
invalid_payload='{
    "role": "server",
    "result": [{"match": 1, "tentativeMatch": 0, "bestIndex": 0}],
    "ids": [""]
}'

make_request "POST" "/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" "$invalid_payload" 500

# Test 3: Mixed valid and invalid IDs
test_log "Testing mixed valid and invalid IDs"
invalid_payload='{
    "role": "server",
    "result": [
        {"match": 1, "tentativeMatch": 0, "bestIndex": 0},
        {"match": 0, "tentativeMatch": 1, "bestIndex": 1}
    ],
    "ids": ["aGVsbG93b3JsZA==", "invalid!@#"]
}'

make_request "POST" "/linkageResult/$PARTY_DKFZ/$PARTY_TUDA" "$invalid_payload" 500

cleanup_temp_files
test_log "✅ Invalid Base64 IDs Test completed successfully"