#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Setting up E2E test environment..."

# Create necessary directories
mkdir -p "$SCRIPT_DIR/results"
mkdir -p "$SCRIPT_DIR/scenarios"
mkdir -p "$SCRIPT_DIR/helpers"

# Make sure all scripts are executable
find "$SCRIPT_DIR" -name "*.sh" -exec chmod +x {} \;

# Create a health check endpoint test
cat > "$SCRIPT_DIR/test_server_health.sh" << 'EOF'
#!/bin/bash
set -e

SERVER_URL="http://localhost:5000"
MAX_ATTEMPTS=10
ATTEMPT=0

echo "Testing server health..."

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    if curl -s -f "$SERVER_URL" >/dev/null 2>&1; then
        echo "Attempt $ATTEMPT/$MAX_ATTEMPTS - Server not ready yet..."
    sleep 2
done

echo "❌ Server health check failed after $MAX_ATTEMPTS attempts"
exit 1
EOF

chmod +x "$SCRIPT_DIR/test_server_health.sh"

echo "✅ E2E test environment setup completed"✅ Server is healthy and responding"
        exit 0
    fi

    ATTEMPT=$((ATTEMPT + 1))
    echo "
