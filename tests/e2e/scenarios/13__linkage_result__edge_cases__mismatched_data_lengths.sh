#!/bin/bash
set -e
source "$(dirname "$0")/../helpers/test_helpers.sh"

test_log "=== Mismatched Data Lengths Test ==="

check_server

# Test 1: Server data - more results than IDs
test_log "Testing more results than IDs"
server_result_data='[
    {"match": 1, "tentativeMatch": 0, "bestIndex": 0},
    {"match": 0, "tentativeMatch": 1, "bestIndex": 1},
    {"match": 1, "tentativeMatch": 0, "bestIndex": 2}
]'
client_result_data='[
    {"match": 0, "tentativeMatch": 1, "bestIndex": 0},
    {"match": 1, "tentativeMatch": 0, "bestIndex": 1},
    {"match": 0, "tentativeMatch": 1, "bestIndex": 2}
]'
ids_data='["aGVsbG93b3JsZA==", "ZGF0YXNjaWVuY2U="]'  # Only 2 IDs for 3 results

# Start server request
(make_server_request "$PARTY_DKFZ" "$PARTY_TUDA" "$server_result_data" "$ids_data") &
server_pid=$!

sleep 0.5

# Start client request - this should trigger the mismatch error
(make_client_request "$PARTY_TUDA" "$PARTY_DKFZ" "$client_result_data" 500) &
client_pid=$!

wait $server_pid
wait $client_pid

# Test 2: Client and server different result lengths
test_log "Testing client-server result length mismatch"
server_result_data='[{"match": 1, "tentativeMatch": 0, "bestIndex": 0}]'
client_result_data='[
    {"match": 0, "tentativeMatch": 1, "bestIndex": 0},
    {"match": 1, "tentativeMatch": 0, "bestIndex": 1}
]'
ids_data='["aGVsbG93b3JsZA=="]'

# Start server request
(make_server_request "$PARTY_TEST_SERVER" "$PARTY_TEST_CLIENT" "$server_result_data" "$ids_data") &
server_pid=$!

sleep 0.5

# Client request with different length should cause error
(make_client_request "$PARTY_TEST_CLIENT" "$PARTY_TEST_SERVER" "$client_result_data" 500) &
client_pid=$!

wait $server_pid
wait $client_pid

cleanup_temp_files
test_log "✅ Mismatched Data Lengths Test completed successfully"