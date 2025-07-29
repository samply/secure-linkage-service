#!/bin/bash
set -e
source "$(dirname "$0")/../helpers/test_helpers.sh"

test_log "=== Basic Server-Client Flow Test ==="

check_server

# Test data - Using no-match scenario to avoid decryption issues
# When there are no matches (all bits are 0), the system generates random linkage IDs
# This avoids the need for pre-encrypted server IDs

test_log "ℹ️  Using no-match scenario to test server-first flow without decryption complexity"

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

# Use longer base64 IDs to meet minimum length requirements
ids_data='[
    "dGVzdGRhdGExMjM0NTY3ODkwYWJjZGVmZ2hpams=",
    "dGVzdGRhdGEyMzQ1Njc4OTAxYmNkZWZnaGlqa2w=",
    "dGVzdGRhdGEzNDU2Nzg5MDEyY2RlZmdoaWprbG0="
]'

test_log "🚀 Testing server-client coordination with no-match scenario"

run_parallel_requests "$PARTY_DKFZ" "$PARTY_TUDA" "$server_result_data" "$client_result_data" "$ids_data"

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

test_log "✅ Basic Server-Client Flow Test completed successfully"
test_log "📝 Note: This test uses no-match scenario to avoid pre-encrypted ID requirements"
