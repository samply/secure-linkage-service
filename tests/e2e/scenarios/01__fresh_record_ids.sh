#!/bin/bash
set -e
source "$(dirname "$0")/../helpers/test_helpers.sh"

test_log "=== Fresh Record IDs Endpoint Test ==="

check_server

test_log "🧪 Testing Fresh Record ID generation for all known parties"

# Test 1: Generate default single ID for each party
test_log "📋 Test 1: Single Fresh Record ID generation"
for party in "$PARTY_DKFZ" "$PARTY_TUDA" "$PARTY_TEST_CLIENT" "$PARTY_TEST_SERVER"; do
    test_log "  Testing party: $party"

    response=$(curl -s -w "%{http_code}" -o /tmp/fresh_id_response.json \
        -X POST \
        -H "Content-Type: application/json" \
        --max-time 10 \
        "$BASE_URL/freshRecordIDs/$party" 2>/dev/null)

    status_code="$response"

    if [ "$status_code" = "200" ]; then
        if jq empty /tmp/fresh_id_response.json 2>/dev/null; then
            party_status=$(jq -r '.status' /tmp/fresh_id_response.json 2>/dev/null || echo "unknown")
            party_count=$(jq -r '.count' /tmp/fresh_id_response.json 2>/dev/null || echo "0")
            fresh_ids_count=$(jq -r '.fresh_record_ids | length' /tmp/fresh_id_response.json 2>/dev/null || echo "0")

            assert "[ \"$party_status\" = \"success\" ]" "Status is success for $party"
            assert "[ $party_count -eq 1 ]" "Count is 1 for $party"
            assert "[ $fresh_ids_count -eq 1 ]" "Response contains 1 fresh record ID for $party"

            # Verify the ID is a valid base64 string and reasonably long
            fresh_id=$(jq -r '.fresh_record_ids[0]' /tmp/fresh_id_response.json 2>/dev/null || echo "")
            if [ -n "$fresh_id" ] && [ ${#fresh_id} -gt 20 ]; then
                test_log "    ✅ Generated valid fresh record ID for $party (${#fresh_id} chars)"
            else
                test_log "    ❌ Invalid fresh record ID for $party: '$fresh_id'"
                exit 1
            fi
        else
            test_log "    ❌ Invalid JSON response for $party"
            exit 1
        fi
    else
        test_log "    ❌ HTTP error $status_code for $party"
        exit 1
    fi
done

# Test 2: Generate multiple IDs with query parameter
test_log "📋 Test 2: Multiple Fresh Record ID generation"
test_counts=(3 5 10 25)

for count in "${test_counts[@]}"; do
    test_log "  Testing count: $count"

    response=$(curl -s -w "%{http_code}" -o /tmp/fresh_id_response.json \
        -X POST \
        -H "Content-Type: application/json" \
        --max-time 15 \
        "$BASE_URL/freshRecordIDs/$PARTY_DKFZ?n=$count" 2>/dev/null)

    status_code="$response"

    if [ "$status_code" = "200" ]; then
        if jq empty /tmp/fresh_id_response.json 2>/dev/null; then
            response_count=$(jq -r '.count' /tmp/fresh_id_response.json 2>/dev/null || echo "0")
            fresh_ids_count=$(jq -r '.fresh_record_ids | length' /tmp/fresh_id_response.json 2>/dev/null || echo "0")

            assert "[ $response_count -eq $count ]" "Response count matches requested count ($count)"
            assert "[ $fresh_ids_count -eq $count ]" "Array contains $count fresh record IDs"

            # Verify all IDs are unique and valid
            ids_json=$(jq -r '.fresh_record_ids[]' /tmp/fresh_id_response.json 2>/dev/null)
            ids_array=()
            while IFS= read -r line; do
                ids_array+=("$line")
            done <<< "$ids_json"

            # Check uniqueness
            unique_count=$(printf '%s\n' "${ids_array[@]}" | sort -u | wc -l)
            assert "[ $unique_count -eq $count ]" "All $count IDs are unique"

            test_log "    ✅ Generated $count unique fresh record IDs"
        else
            test_log "    ❌ Invalid JSON response for count $count"
            exit 1
        fi
    else
        test_log "    ❌ HTTP error $status_code for count $count"
        exit 1
    fi
done

# Test 3: Boundary testing
test_log "📋 Test 3: Boundary and error condition testing"

# Test zero count (should fail)
test_log "  Testing zero count (should fail)"
response=$(curl -s -w "%{http_code}" -o /tmp/fresh_id_response.json \
    -X POST \
    -H "Content-Type: application/json" \
    --max-time 10 \
    "$BASE_URL/freshRecordIDs/$PARTY_DKFZ?n=0" 2>/dev/null)

status_code="$response"
assert "[ \"$status_code\" = \"400\" ]" "Zero count returns 400 Bad Request"

if jq empty /tmp/fresh_id_response.json 2>/dev/null; then
    error_status=$(jq -r '.status' /tmp/fresh_id_response.json 2>/dev/null || echo "unknown")
    assert "[ \"$error_status\" = \"error\" ]" "Error response has error status"
    test_log "    ✅ Zero count properly rejected"
fi

# Test excessive count (should fail)
test_log "  Testing excessive count (should fail)"
response=$(curl -s -w "%{http_code}" -o /tmp/fresh_id_response.json \
    -X POST \
    -H "Content-Type: application/json" \
    --max-time 10 \
    "$BASE_URL/freshRecordIDs/$PARTY_DKFZ?n=1001" 2>/dev/null)

status_code="$response"
assert "[ \"$status_code\" = \"400\" ]" "Excessive count returns 400 Bad Request"

if jq empty /tmp/fresh_id_response.json 2>/dev/null; then
    error_status=$(jq -r '.status' /tmp/fresh_id_response.json 2>/dev/null || echo "unknown")
    assert "[ \"$error_status\" = \"error\" ]" "Error response has error status"
    test_log "    ✅ Excessive count properly rejected"
fi

# Test unknown party (should fail)
test_log "  Testing unknown party (should fail)"
response=$(curl -s -w "%{http_code}" -o /tmp/fresh_id_response.json \
    -X POST \
    -H "Content-Type: application/json" \
    --max-time 10 \
    "$BASE_URL/freshRecordIDs/unknown_party?n=1" 2>/dev/null)

status_code="$response"
assert "[ \"$status_code\" = \"404\" ]" "Unknown party returns 404 Not Found"

if jq empty /tmp/fresh_id_response.json 2>/dev/null; then
    error_status=$(jq -r '.status' /tmp/fresh_id_response.json 2>/dev/null || echo "unknown")
    assert "[ \"$error_status\" = \"error\" ]" "Error response has error status"
    test_log "    ✅ Unknown party properly rejected"
fi

# Test 4: Performance test with maximum allowed count
test_log "📋 Test 4: Performance test with maximum count (1000)"
start_time=$(date +%s%N)

response=$(curl -s -w "%{http_code}" -o /tmp/fresh_id_response.json \
    -X POST \
    -H "Content-Type: application/json" \
    --max-time 30 \
    "$BASE_URL/freshRecordIDs/$PARTY_DKFZ?n=1000" 2>/dev/null)

end_time=$(date +%s%N)
duration_ms=$(( (end_time - start_time) / 1000000 ))

status_code="$response"

if [ "$status_code" = "200" ]; then
    if jq empty /tmp/fresh_id_response.json 2>/dev/null; then
        response_count=$(jq -r '.count' /tmp/fresh_id_response.json 2>/dev/null || echo "0")
        fresh_ids_count=$(jq -r '.fresh_record_ids | length' /tmp/fresh_id_response.json 2>/dev/null || echo "0")

        assert "[ $response_count -eq 1000 ]" "Response count is 1000"
        assert "[ $fresh_ids_count -eq 1000 ]" "Array contains 1000 fresh record IDs"

        test_log "    ✅ Generated 1000 fresh record IDs in ${duration_ms}ms"

        # Performance assertion (should complete within reasonable time)
        assert "[ $duration_ms -lt 10000 ]" "Generation completed within 10 seconds"
    else
        test_log "    ❌ Invalid JSON response for maximum count test"
        exit 1
    fi
else
    test_log "    ❌ HTTP error $status_code for maximum count test"
    exit 1
fi

# Test 5: HTTP method validation
test_log "📋 Test 5: HTTP method validation"

# Test GET method (should fail)
response=$(curl -s -w "%{http_code}" -o /tmp/fresh_id_response.json \
    -X GET \
    "$BASE_URL/freshRecordIDs/$PARTY_DKFZ" 2>/dev/null)

status_code="$response"
assert "[ \"$status_code\" = \"405\" ]" "GET method returns 405 Method Not Allowed"
test_log "    ✅ GET method properly rejected"

cleanup_temp_files

test_log "✅ Fresh Record IDs Endpoint Test completed successfully"
test_log "📊 Summary:"
test_log "   - Single record ID generation: ✅"
test_log "   - Multiple record ID generation: ✅"
test_log "   - Boundary condition handling: ✅"
test_log "   - Performance with 1000 record IDs: ✅"
test_log "   - HTTP method validation: ✅"
