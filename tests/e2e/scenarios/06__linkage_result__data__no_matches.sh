#!/bin/bash
set -e
source "$(dirname "$0")/../helpers/test_helpers.sh"

test_log "=== No Matches Test ==="

check_server

# Test with no matches at all (all bits are 0)
server_result_data='[
    {"match": 0, "tentativeMatch": 0, "bestIndex": 0},
    {"match": 0, "tentativeMatch": 0, "bestIndex": 1},
    {"match": 0, "tentativeMatch": 0, "bestIndex": 2}
]'

client_result_data='[
    {"match": 0, "tentativeMatch": 0, "bestIndex": 0},
    {"match": 0, "tentativeMatch": 0, "bestIndex": 1},
    {"match": 0, "tentativeMatch": 0, "bestIndex": 2}
]'

ids_data='["aGVsbG93b3JsZA==", "ZGF0YXNjaWVuY2U=", "cHl0aG9uaXNmdW4="]'

run_parallel_requests "$PARTY_DKFZ" "$PARTY_TUDA" "$server_result_data" "$client_result_data" "$ids_data"

validate_response_structure

# Verify response still contains linkage IDs (should be random IDs)
status=$(jq -r '.status' /tmp/response_body.json)
linkage_ids=$(jq -r '.linkage_ids | length' /tmp/response_body.json)

assert "[ \"$status\" = \"completed\" ]" "Status is completed"
assert "[ $linkage_ids -eq 3 ]" "Response contains 3 linkage IDs"

cleanup_temp_files
test_log "✅ No Matches Test completed successfully"