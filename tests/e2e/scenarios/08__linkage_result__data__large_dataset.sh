#!/bin/bash
set -e
source "$(dirname "$0")/../helpers/test_helpers.sh"

test_log "=== Large Dataset Test ==="

check_server

# Generate large dataset (100 records)
dataset_size=100
test_log "Generating dataset with $dataset_size records"

server_result_data=$(generate_result_data $dataset_size true)
client_result_data=$(generate_result_data $dataset_size true)
ids_data=$(generate_ids_array $dataset_size)

test_log "Starting large dataset linkage"

# Use longer timeout for large dataset
run_parallel_requests "$PARTY_DKFZ" "$PARTY_TUDA" "$server_result_data" "$client_result_data" "$ids_data"

validate_response_structure

# Verify all records processed
linkage_ids=$(jq -r '.linkage_ids | length' /tmp/response_body.json)
assert "[ $linkage_ids -eq $dataset_size ]" "Response contains $dataset_size linkage IDs"

cleanup_temp_files
test_log "✅ Large Dataset Test completed successfully"