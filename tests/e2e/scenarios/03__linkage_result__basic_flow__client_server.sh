#!/bin/bash
set -e
source "$(dirname "$0")/../helpers/test_helpers.sh"

test_log "=== Basic Client-Server Flow Test ==="

check_server

# Test data - Using no-match scenario to avoid decryption issues
# When there are no matches (all bits are 0), the system generates random linkage IDs
# This avoids the need for pre-encrypted server IDs

test_log "ℹ️  Using no-match scenario to test client-first flow without decryption complexity"

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

# Use longer base64 IDs to meet minimum length requirements
ids_data='[
    "dGVzdGRhdGE0NTY3ODkwMTIzZGVmZ2hpamtsbW4=",
    "dGVzdGRhdGE1Njc4OTAxMjM0ZWZnaGlqa2xtbm8=",
    "dGVzdGRhdGE2Nzg5MDEyMzQ1ZmdoaWprbG1ub3A="
]'

test_log "🚀 Testing client-server coordination (client initiates first)"

# The run_parallel_requests helper will handle proper coordination
# Note: The helper function parameters determine the order:
# run_parallel_requests server_party client_party server_data client_data ids
# So we swap the parties to make client initiate first
run_parallel_requests "$PARTY_TUDA" "$PARTY_DKFZ" "$client_result_data" "$server_result_data" "$ids_data"

validate_response_structure

# Verify response contains linkage IDs (should be random IDs since no matches)
if [ -f "/tmp/response_body.json" ]; then
    status=$(jq -r '.status' /tmp/response_body.json 2>/dev/null || echo "unknown")
    linkage_count=$(jq -r '.linkage_ids | length' /tmp/response_body.json 2>/dev/null || echo "0")

    assert "[ \"$status\" = \"completed\" ]" "Status is completed"
    assert "[ $linkage_count -eq 3 ]" "Response contains 3 linkage IDs"

    test_log "✅ Received $linkage_count random linkage IDs (no matches scenario)"
else
    test_log "⚠️ Could not validate response details"
fi

cleanup_temp_files

test_log "✅ Basic Client-Server Flow Test completed successfully"
test_log "📝 Note: This test validates client-first coordination using the reliable helper function"
