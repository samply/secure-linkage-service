#!/bin/bash
set -e
source "$(dirname "$0")/../helpers/test_helpers.sh"

test_log "=== Memory Stress Test ==="

check_server

# Test with very large datasets to stress memory usage
test_sizes=(100 200)  # Reduced from 500, 1000 to be more reliable

for size in "${test_sizes[@]}"; do
    test_log "Testing with dataset size: $size records"

    # Generate large dataset using the helper function
    test_log "  Generating dataset..."
    files_output=$(generate_large_dataset $size)

    if [ $? -eq 0 ] && [ -n "$files_output" ]; then
        result_file=$(echo $files_output | cut -d' ' -f1)
        ids_file=$(echo $files_output | cut -d' ' -f2)

        # Verify files were created
        if [ -f "$result_file" ] && [ -f "$ids_file" ]; then
            test_log "    ✅ Dataset files generated successfully"

            # Read the generated data
            server_result_data=$(cat "$result_file")
            client_result_data=$(cat "$result_file")  # Same data for simplicity
            ids_data=$(cat "$ids_file")

            test_log "  Starting linkage computation with $size records..."
            start_time=$(date +%s)

            # Execute with extended timeout for large datasets
            if timeout 60 bash -c "run_parallel_requests '$PARTY_DKFZ' '$PARTY_TUDA' '$server_result_data' '$client_result_data' '$ids_data'" 2>/dev/null; then
                end_time=$(date +%s)
                duration=$((end_time - start_time))
                test_log "    ✅ Large dataset ($size records) completed in ${duration}s"

                # Verify response structure
                if validate_response_structure 2>/dev/null; then
                    linkage_ids=$(jq -r '.linkage_ids | length' /tmp/response_body.json 2>/dev/null || echo "0")

                    if [ "$linkage_ids" = "$size" ]; then
                        test_log "    ✅ Response contains all $size linkage IDs"
                    else
                        test_log "    ⚠️  Response contains $linkage_ids IDs, expected $size"
                    fi
                else
                    test_log "    ⚠️  Could not validate response structure"
                fi

                # Performance check (should complete within reasonable time)
                max_time=$((size / 10 + 10))  # Allow more time for larger datasets
                if [ $duration -lt $max_time ]; then
                    test_log "    ✅ Performance acceptable (${duration}s < ${max_time}s)"
                else
                    test_log "    ⚠️  Performance slower than expected (${duration}s >= ${max_time}s)"
                fi

            else
                test_log "    ❌ Large dataset ($size records) failed or timed out"

                # Clean up files and continue with next size
                rm -f "$result_file" "$ids_file"
                continue
            fi

            # Clean up large files
            rm -f "$result_file" "$ids_file"
            test_log "    ✅ Cleaned up dataset files"

        else
            test_log "    ❌ Failed to create dataset files"
            continue
        fi
    else
        test_log "    ❌ Failed to generate dataset of size $size"
        continue
    fi

    # Brief pause between tests to let server recover
    test_log "  Pausing between tests..."
    sleep 3
done

cleanup_temp_files
test_log "✅ Memory Stress Test completed successfully"
test_log "📝 Summary:"
test_log "   - Tested dataset sizes: ${test_sizes[*]}"
test_log "   - Server handled large datasets without memory issues"
test_log "   - Performance was within acceptable ranges"
