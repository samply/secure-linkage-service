#!/bin/bash
# Diagnostic script to test ID lengths and find the minimum required length

set -e

echo "🔍 Test ID Diagnostic"
echo "===================="

# Check current IDs
echo "📋 Current test IDs from your test:"
old_ids=("aGVsbG93b3JsZA==" "ZGF0YXNjaWVuY2U=" "cHl0aG9uaXNmdW4=")

for i in "${!old_ids[@]}"; do
    id="${old_ids[$i]}"
    decoded_length=$(echo "$id" | base64 -d 2>/dev/null | wc -c || echo "0")
    decoded_text=$(echo "$id" | base64 -d 2>/dev/null || echo "decode_failed")
    echo "  ID $((i+1)): $id"
    echo "    Decoded: '$decoded_text' ($decoded_length bytes)"
done

echo ""
echo "❌ Problem: These IDs are too short (10-12 bytes)"
echo "   The error suggests minimum 12 bytes (NONCE_LEN) + encrypted data"

echo ""
echo "✅ Solution: Use longer test IDs"
echo "=============================="

# Generate longer test IDs
new_ids=(
    "$(echo 'testdata1234567890abcdefghijklmnop' | base64 -w 0)"
    "$(echo 'testdata2345678901bcdefghijklmnopq' | base64 -w 0)"
    "$(echo 'testdata3456789012cdefghijklmnopqr' | base64 -w 0)"
)

echo "📝 New longer test IDs:"
for i in "${!new_ids[@]}"; do
    id="${new_ids[$i]}"
    decoded_length=$(echo "$id" | base64 -d 2>/dev/null | wc -c || echo "0")
    decoded_text=$(echo "$id" | base64 -d 2>/dev/null || echo "decode_failed")
    echo "  ID $((i+1)): $id"
    echo "    Decoded: '$decoded_text' ($decoded_length bytes)"
done

echo ""
echo "🔧 Quick Fix:"
echo "============="
echo "Replace the ids_data in your test with:"
echo ""
echo "ids_data='[\"${new_ids[0]}\", \"${new_ids[1]}\", \"${new_ids[2]}\"]'"

echo ""
echo "🎯 Testing with new IDs:"
echo "========================"

# Check if server is running
if ! curl -s -o /dev/null --max-time 1 http://localhost:5000 2>/dev/null; then
    echo "❌ Server is not running. Please start the server first."
    echo "Run: ./start-server.sh"
    exit 1
fi

# Test with the new longer IDs
test_request='{
    "role": "server",
    "result": [
        {"match": 0, "tentativeMatch": 1, "bestIndex": 2},
        {"match": 1, "tentativeMatch": 0, "bestIndex": 0},
        {"match": 0, "tentativeMatch": 0, "bestIndex": 4}
    ],
    "ids": ["'${new_ids[0]}'", "'${new_ids[1]}'", "'${new_ids[2]}'"]
}'

echo "Testing server request with new IDs..."
response=$(curl -s -w "\n%{http_code}" \
    -X POST \
    -H "Content-Type: application/json" \
    -d "$test_request" \
    "http://localhost:5000/linkageResult/dkfz/tuda")

status_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | head -n -1)

echo "Status: $status_code"
echo "Response: $(echo "$body" | jq '.' 2>/dev/null || echo "$body")"

if [ "$status_code" = "200" ]; then
    echo ""
    echo "✅ SUCCESS: New IDs work! The server accepted the request."
    echo ""
    echo "🎯 Next steps:"
    echo "1. Update your test file with the new IDs"
    echo "2. Or download the updated test_helpers.sh I created"
    echo "3. Run the test again"
else
    echo ""
    echo "❌ Still failing. Let's check the server logs:"
    echo "tail -f server.log"
    echo ""
    echo "The issue might be more complex than just ID length."
fi
