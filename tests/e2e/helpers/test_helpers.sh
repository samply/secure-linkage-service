#!/bin/bash
# Common helper functions for E2E tests

# Server configuration
SERVER_HOST="localhost"
SERVER_PORT="5000"
BASE_URL="http://$SERVER_HOST:$SERVER_PORT"
REQUEST_TIMEOUT=30

# Test data
VALID_IDS=('aGVsbG93b3JsZA==' 'ZGF0YXNjaWVuY2U=' 'cHl0aG9uaXNmdW4=' 'bWFjaGluZWxlYXJuaW5n' 'YXJ0aWZpY2lhbGludGVsbGlnZW5jZQ==')

# Known test parties from the server setup
PARTY_DKFZ="dkfz"
PARTY_TUDA="tuda"
PARTY_TEST_CLIENT="test_client"
PARTY_TEST_SERVER="test_server"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging function
test_log() {
    echo "[$(date '+%H:%M:%S')] $1"
}

# Check if server is running
check_server() {
    local max_attempts=30
    local attempt=0

    test_log "Checking if server is running on $BASE_URL"

    while ! curl -s -o /dev/null --max-time 1 "$BASE_URL" 2>/dev/null; do
        attempt=$((attempt + 1))
        if [ $attempt -ge $max_attempts ]; then
            test_log "❌ Server is not responding after $max_attempts attempts"
            return 1
        fi
        sleep 1
    done

    test_log "✅ Server is running"
    return 0
}

# Generate test data for a given size
generate_result_data() {
    local size=$1
    local include_matches=${2:-true}
    local result="["

    for ((i=0; i<size; i++)); do
        if [ "$include_matches" = true ] && [ $((i % 3)) -eq 0 ]; then
            # Include some matches
            result+="{\"match\": 1, \"tentativeMatch\": 0, \"bestIndex\": $i}"
        elif [ "$include_matches" = true ] && [ $((i % 5)) -eq 0 ]; then
            # Include some tentative matches
            result+="{\"match\": 0, \"tentativeMatch\": 1, \"bestIndex\": $i}"
        else
            # No match
            result+="{\"match\": 0, \"tentativeMatch\": 0, \"bestIndex\": $i}"
        fi

        if [ $i -lt $((size - 1)) ]; then
            result+=","
        fi
    done

    result+="]"
    echo "$result"
}

# Generate IDs array for server requests
generate_ids_array() {
    local size=$1
    local result="["

    for ((i=0; i<size; i++)); do
        local id_index=$((i % ${#VALID_IDS[@]}))
        result+="\"${VALID_IDS[$id_index]}\""

        if [ $i -lt $((size - 1)) ]; then
            result+=","
        fi
    done

    result+="]"
    echo "$result"
}

# Make HTTP request and return status code
make_request() {
    local method=$1
    local endpoint=$2
    local data=$3
    local expected_status=${4:-200}
    local timeout=${5:-$REQUEST_TIMEOUT}

    local response
    response=$(curl -s -w "%{http_code}" -o /tmp/response_body.json \
        -X "$method" \
        -H "Content-Type: application/json" \
        -d "$data" \
        --max-time "$timeout" \
        "$BASE_URL$endpoint" 2>/dev/null)

    local status_code="$response"

    if [ "$status_code" = "$expected_status" ]; then
        return 0
    else
        test_log "❌ Expected status $expected_status, got $status_code"
        test_log "Response body: $(cat /tmp/response_body.json 2>/dev/null || echo 'No response body')"
        return 1
    fi
}

# Make server role request
make_server_request() {
    local source_party=$1
    local dest_party=$2
    local result_data=$3
    local ids_data=$4
    local expected_status=${5:-200}

    local payload="{
        \"role\": \"server\",
        \"result\": $result_data,
        \"ids\": $ids_data
    }"

    make_request "POST" "/linkageResult/$source_party/$dest_party" "$payload" "$expected_status"
}

# Make client role request
make_client_request() {
    local source_party=$1
    local dest_party=$2
    local result_data=$3
    local expected_status=${4:-200}

    local payload="{
        \"role\": \"client\",
        \"result\": $result_data
    }"

    make_request "POST" "/linkageResult/$source_party/$dest_party" "$payload" "$expected_status"
}

# Run requests in parallel and check both succeed
run_parallel_requests() {
    local server_party=$1
    local client_party=$2
    local server_result_data=$3
    local client_result_data=$4
    local ids_data=$5

    test_log "Running parallel server and client requests"

    # Start server request in background
    (make_server_request "$server_party" "$client_party" "$server_result_data" "$ids_data") &
    local server_pid=$!

    # Small delay to ensure server request is processed first
    sleep 0.5

    # Start client request
    (make_client_request "$client_party" "$server_party" "$client_result_data") &
    local client_pid=$!

    # Wait for both to complete
    local server_result=0
    local client_result=0

    wait $server_pid || server_result=$?
    wait $client_pid || client_result=$?

    if [ $server_result -eq 0 ] && [ $client_result -eq 0 ]; then
        test_log "✅ Both requests completed successfully"
        return 0
    else
        test_log "❌ One or both requests failed (server: $server_result, client: $client_result)"
        return 1
    fi
}

# Validate JSON response structure
validate_response_structure() {
    local response_file="/tmp/response_body.json"

    if [ ! -f "$response_file" ]; then
        test_log "❌ No response file found"
        return 1
    fi

    # Check if response is valid JSON
    if ! jq empty "$response_file" 2>/dev/null; then
        test_log "❌ Response is not valid JSON"
        return 1
    fi

    # Check required fields
    local status
    status=$(jq -r '.status // empty' "$response_file")

    if [ -z "$status" ]; then
        test_log "❌ Missing 'status' field in response"
        return 1
    fi

    test_log "✅ Response structure is valid (status: $status)"
    return 0
}

# Clean up temporary files
cleanup_temp_files() {
    rm -f /tmp/response_body.json
    rm -f /tmp/test_*.json
}

# Assert test condition
assert() {
    local condition="$1"
    local message="$2"

    if eval "$condition"; then
        test_log "✅ Assertion passed: $message"
        return 0
    else
        test_log "❌ Assertion failed: $message"
        return 1
    fi
}

# Generate large dataset for stress testing
generate_large_dataset() {
    local size=$1
    local result_file="/tmp/large_result_data.json"
    local ids_file="/tmp/large_ids_data.json"

    test_log "Generating large dataset with $size records"

    echo "[" > "$result_file"
    echo "[" > "$ids_file"

    for ((i=0; i<size; i++)); do
        # Generate result data
        echo "{\"match\": $((RANDOM % 2)), \"tentativeMatch\": $((RANDOM % 2)), \"bestIndex\": $i}" >> "$result_file"

        # Generate ID
        local id_index=$((i % ${#VALID_IDS[@]}))
        echo "\"${VALID_IDS[$id_index]}\"" >> "$ids_file"

        if [ $i -lt $((size - 1)) ]; then
            echo "," >> "$result_file"
            echo "," >> "$ids_file"
        fi
    done

    echo "]" >> "$result_file"
    echo "]" >> "$ids_file"

    test_log "✅ Large dataset generated"
    echo "$result_file $ids_file"
}
