#!/bin/bash -e

SERVER_BINARY=secure-linkage-service
SERVER_PORT=5000
SERVER_HOST=localhost

# Function to terminate the server and its children
terminate_server() {
    if [ ! -z "$SERVER_PID" ]; then
        echo "Terminating server process group: $SERVER_PID"
        # Kill entire process group to ensure all children are terminated
        kill -- -$SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi
}

# Function to check if server is already running
is_server_running() {
    curl -s -o /dev/null --max-time 1 "http://$SERVER_HOST:$SERVER_PORT" 2>/dev/null
}

# Function to find existing server process
#!/bin/bash -e

SERVER_BINARY=secure-linkage-service
SERVER_PORT=5000
SERVER_HOST=localhost

# Global variables to track server state
STARTED_NEW_SERVER=false
SERVER_PID=""

# Function to terminate the server and its children
terminate_server() {
    if [ "$STARTED_NEW_SERVER" = true ] && [ ! -z "$SERVER_PID" ]; then
        echo "🛑 Terminating server started by this script (PID: $SERVER_PID)"
        # Kill entire process group to ensure all children are terminated
        kill -- -$SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
        echo "✅ Server terminated"
    elif [ "$STARTED_NEW_SERVER" = false ]; then
        echo "ℹ️  Leaving existing server running (not started by this script)"
    fi
}

# Set up cleanup trap
trap terminate_server EXIT

# Function to check if server is already running
is_server_running() {
    curl -s -o /dev/null --max-time 1 "http://$SERVER_HOST:$SERVER_PORT" 2>/dev/null
}

# Function to find existing server process
#!/bin/bash -e

SERVER_BINARY=secure-linkage-service
SERVER_PORT=5000
SERVER_HOST=localhost

# Global variables to track server state
STARTED_NEW_SERVER=false
SERVER_PID=""

# Function to terminate the server and its children
terminate_server() {
    if [ "$STARTED_NEW_SERVER" = true ] && [ ! -z "$SERVER_PID" ]; then
        echo ""
        echo "🛑 Terminating server started by this script (PID: $SERVER_PID)"

        # First, try graceful termination with SIGTERM
        if kill -0 $SERVER_PID 2>/dev/null; then
            echo "📤 Sending SIGTERM to server..."
            kill -TERM $SERVER_PID 2>/dev/null || true

            # Wait up to 5 seconds for graceful shutdown
            local count=0
            while kill -0 $SERVER_PID 2>/dev/null && [ $count -lt 5 ]; do
                sleep 1
                count=$((count + 1))
                echo "⏳ Waiting for graceful shutdown... (${count}s)"
            done

            # If still running, force kill
            if kill -0 $SERVER_PID 2>/dev/null; then
                echo "💀 Force killing server with SIGKILL..."
                kill -KILL $SERVER_PID 2>/dev/null || true
                sleep 1
            fi

            # Verify termination
            if kill -0 $SERVER_PID 2>/dev/null; then
                echo "❌ Failed to terminate server process"
            else
                echo "✅ Server terminated successfully"
            fi
        else
            echo "ℹ️  Server process already terminated"
        fi
    elif [ "$STARTED_NEW_SERVER" = false ]; then
        echo "ℹ️  Leaving existing server running (not started by this script)"
    fi
}

# Set up cleanup trap for multiple signals
trap terminate_server EXIT INT TERM

# Function to check if server is already running
is_server_running() {
    curl -s -o /dev/null --max-time 1 "http://$SERVER_HOST:$SERVER_PORT" 2>/dev/null
}

# Function to find existing server process
find_existing_server() {
    # Look for the server binary process
    pgrep -f "$SERVER_BINARY" 2>/dev/null | head -1
}

# Check if server is already running
if is_server_running; then
    existing_pid=$(find_existing_server)
    if [ ! -z "$existing_pid" ]; then
        echo "✅ Server is already running (PID: $existing_pid) on port $SERVER_PORT"
        echo "📍 Using existing server instance"

        # Set SERVER_PID for reference, but mark as not started by us
        SERVER_PID=$existing_pid
        STARTED_NEW_SERVER=false

        echo "ℹ️  Note: Server will continue running after this script exits"

        # Verify it's responding properly
        echo "🔍 Verifying server health..."
        if is_server_running; then
            echo "✅ Server is healthy and responding"
        else
            echo "⚠️  Server process exists but not responding properly"
        fi

        # Display server info and exit immediately for existing servers
        echo ""
        echo "📝 Existing server info:"
        echo "   PID: $SERVER_PID"
        echo "   URL: http://$SERVER_HOST:$SERVER_PORT"
        echo ""
        echo "💡 To view logs: tail -f server.log"
        echo "💡 To stop server: kill $SERVER_PID"

        exit 0
    else
        echo "⚠️  Port $SERVER_PORT is in use but couldn't identify the process"
        echo "🔍 Checking what's using the port..."
        lsof -i :$SERVER_PORT 2>/dev/null || netstat -tlnp 2>/dev/null | grep ":$SERVER_PORT " || echo "Could not identify the process"
        echo "📍 Assuming compatible server is running"
        STARTED_NEW_SERVER=false
        exit 0
    fi
fi

# Server is not running, so start it
echo "🚀 Starting new server instance..."
STARTED_NEW_SERVER=true

# Check if Rust binary exists, if not build it
if [ ! -f "target/release/$SERVER_BINARY" ] && [ ! -f "target/debug/$SERVER_BINARY" ]; then
    echo "🔨 Building Rust application..."
    cargo build --release
fi

# Determine which binary to use
if [ -f "target/release/$SERVER_BINARY" ]; then
    BINARY_PATH="target/release/$SERVER_BINARY"
    BUILD_TYPE="release"
else
    BINARY_PATH="target/debug/$SERVER_BINARY"
    BUILD_TYPE="debug"
fi

echo "📦 Using $BUILD_TYPE binary: $BINARY_PATH"

# Start the Rust server in the background
echo "⚡ Starting Rust secure linkage service..."
./"$BINARY_PATH" > server.log 2>&1 &

# Capture the server PID to terminate it later
SERVER_PID=$!
echo "🔢 Server started with PID: $SERVER_PID"

# Wait for server to start using port check
echo "⏳ Waiting for server to start on port $SERVER_PORT..."
MAX_ATTEMPTS=60
ATTEMPT=0

while ! is_server_running; do
    # Check if the process is still alive
    if ! kill -0 $SERVER_PID 2>/dev/null; then
        echo "💀 Server process died during startup"
        echo "📋 Server logs:"
        echo "----------------------------------------"
        cat server.log 2>/dev/null || echo "No log file found"
        echo "----------------------------------------"
        exit 1
    fi

    ATTEMPT=$((ATTEMPT+1))
    if [ $ATTEMPT -ge $MAX_ATTEMPTS ]; then
        echo "❌ Server did not start within $MAX_ATTEMPTS seconds. Aborting."
        echo "📋 Server logs:"
        echo "----------------------------------------"
        cat server.log 2>/dev/null || echo "No log file found"
        echo "----------------------------------------"

        echo "🛑 Terminating unresponsive server..."
        kill -TERM $SERVER_PID 2>/dev/null || true
        sleep 2
        kill -KILL $SERVER_PID 2>/dev/null || true

        exit 1
    fi

    # Show progress every 10 attempts
    if [ $((ATTEMPT % 10)) -eq 0 ]; then
        echo "⏳ Still waiting... (${ATTEMPT}s elapsed)"
    fi

    sleep 1
done

echo "✅ Server started successfully after $ATTEMPT seconds"
echo "🌐 Server is now accepting connections on http://$SERVER_HOST:$SERVER_PORT"

# Additional health check
echo "🔍 Performing health check..."
HTTP_STATUS=$(curl -s -w "%{http_code}" -o /dev/null --max-time 5 "http://$SERVER_HOST:$SERVER_PORT" 2>/dev/null || echo "000")
if [ "$HTTP_STATUS" != "000" ]; then
    echo "✅ Health check passed (HTTP $HTTP_STATUS)"
else
    echo "⚠️  Health check inconclusive, but server appears to be running"
fi

echo "🎉 Server setup complete!"
echo ""
echo "📝 New server info:"
echo "   PID: $SERVER_PID"
echo "   Binary: $BINARY_PATH"
echo "   Log file: server.log"
echo "   URL: http://$SERVER_HOST:$SERVER_PORT"
echo ""
echo "💡 To view logs: tail -f server.log"
echo "💡 To stop server: kill $SERVER_PID"
echo "⚠️  Server will be terminated when this script exits"

# Keep the script running until interrupted
echo ""
echo "🔄 Script will keep running to maintain server..."
echo "   Press Ctrl+C to stop the server and exit"
echo "   Or run in background with: $0 &"

# Instead of wait, use a loop that can be interrupted
while kill -0 $SERVER_PID 2>/dev/null; do
    sleep 1
done

echo "💀 Server process has ended"
