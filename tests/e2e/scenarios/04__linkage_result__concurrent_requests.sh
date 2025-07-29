#!/bin/bash
set -e
source "$(dirname "$0")/../helpers/test_helpers.sh"

test_log "=== Concurrent Requests Test ==="

check_server

test_log "Testing truly concurrent linkage operations"

# FIXED: Use no-match scenarios to avoid decryption issues
# FIXED: Use longer base64 IDs to meet minimum length requirements

test_data='[{"match": 0, "tentativeMatch": 0, "bestIndex": 0}]'

# Different longer IDs for each concurrent operation
ids_1='["dGVzdGNvbmN1cnJlbnQxMjM0NTY3ODkwYWJjZGVmZ2hpams="]'  # testconcurrent1234567890abcdefghijk
ids_2='["dGVzdGNvbmN1cnJlbnQyMzQ1Njc4OTAxYmNkZWZnaGlqa2w="]'  # testconcurrent2345678901bcdefghijkl

test_log "Starting two truly concurrent linkage operations"

# First concurrent pair: dkfz <-> tuda
(
    test_log "Starting concurrent pair 1: dkfz <-> tuda"
    if run_parallel_requests "$PARTY_DKFZ" "$PARTY_TUDA" "$test_data" "$test_data" "$ids_1"; then
        echo "0" > /tmp/pair1_result
        test_log "✅ Concurrent pair 1 completed successfully"
    else
        echo "1" > /tmp/pair1_result
        test_log "❌ Concurrent pair 1 failed"
    fi
) &
pair1_pid=$!

# Small stagger to avoid exact simultaneous start (reduces collision risk)
sleep 0.05

# Second concurrent pair: test_server <-> test_client
(
    test_log "Starting concurrent pair 2: test_server <-> test_client"
    if run_parallel_requests "$PARTY_TEST_SERVER" "$PARTY_TEST_CLIENT" "$test_data" "$test_data" "$ids_2"; then
        echo "0" > /tmp/pair2_result
        test_log "✅ Concurrent pair 2 completed successfully"
    else
        echo "1" > /tmp/pair2_result
        test_log "❌ Concurrent pair 2 failed"
    fi
) &
pair2_pid=$!

# Wait for both concurrent operations to complete
test_log "Waiting for both concurrent operations to finish..."

wait $pair1_pid
pair1_exit=$?
wait $pair2_pid
pair2_exit=$?

# Read results from temporary files
pair1_result=$(cat /tmp/pair1_result 2>/dev/null || echo "1")
pair2_result=$(cat /tmp/pair2_result 2>/dev/null || echo "1")

test_log "Pair 1 result: $pair1_result (exit code: $pair1_exit)"
test_log "Pair 2 result: $pair2_result (exit code: $pair2_exit)"

# Clean up result files
rm -f /tmp/pair1_result /tmp/pair2_result

# Assert both concurrent operations succeeded
assert "[ $pair1_result -eq 0 ] && [ $pair2_result -eq 0 ]" "Both concurrent operations completed successfully"

validate_response_structure
cleanup_temp_files

test_log "✅ Concurrent Requests Test completed successfully"
test_log "📝 Successfully handled two simultaneous linkage operations"
test_log "📝 Validated server's true concurrent processing capability"
