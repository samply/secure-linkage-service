#!/bin/bash
# Script to validate test data and scenarios

set -e
source "$(dirname "$0")/helpers/test_helpers.sh"

echo "Validating test data and scenarios..."

# Test 1: Validate base64 test IDs
echo "Checking base64 test IDs..."
for id in "${VALID_IDS[@]}"; do
    if echo "$id" | base64 -d >/dev/null 2>&1; then
        echo "✅ Valid base64 ID: $id"
    else
        echo "❌ Invalid base64 ID: $id"
        exit 1
    fi
done

# Test 2: Check all test scenarios exist
echo "Checking test scenario files..."
for i in $(seq -f "%02g" 1 17); do
    test_file="scenarios/${i}_*.sh"
    if ls $test_file >/dev/null 2>&1; then
        script_name=$(ls $test_file | head -1)
        if [ -x "$script_name" ]; then
            echo "✅ Test scenario $i: $(basename "$script_name")"
        else
            echo "❌ Test scenario $i not executable: $(basename "$script_name")"
            exit 1
        fi
    else
        echo "❌ Missing test scenario: $i"
        exit 1
    fi
done

# Test 3: Validate helper functions
echo "Checking helper functions..."
if declare -f check_server >/dev/null 2>&1; then
    echo "✅ check_server function available"
else
    echo "❌ check_server function missing"
    exit 1
fi

if declare -f make_request >/dev/null 2>&1; then
    echo "✅ make_request function available"
else
    echo "❌ make_request function missing"
    exit 1
fi

# Test 4: Check test parties are properly defined
echo "Checking test parties..."
parties=("$PARTY_DKFZ" "$PARTY_TUDA" "$PARTY_TEST_CLIENT" "$PARTY_TEST_SERVER")
for party in "${parties[@]}"; do
    if [ -n "$party" ]; then
        echo "✅ Party defined: $party"
    else
        echo "❌ Party not defined"
        exit 1
    fi
done

echo "✅ All test data and scenarios validated successfully"
