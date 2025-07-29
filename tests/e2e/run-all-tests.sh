#!/bin/bash
# Main E2E test runner that starts server and runs all test scenarios
# Fixed path resolution to prevent double-appending

set -e

# FIXED: Smart path detection to handle different execution contexts
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Determine the correct E2E directory based on where the script is located
if [[ "$SCRIPT_DIR" == */tests/e2e ]]; then
    # Script is being run from within /tests/e2e directory
    E2E_DIR="$SCRIPT_DIR"
    PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
elif [[ -d "$SCRIPT_DIR/tests/e2e" ]]; then
    # Script is being run from project root
    E2E_DIR="$SCRIPT_DIR/tests/e2e"
    PROJECT_ROOT="$SCRIPT_DIR"
else
    # Try to find tests/e2e directory by walking up
    CURRENT_DIR="$SCRIPT_DIR"
    while [[ "$CURRENT_DIR" != "/" ]]; do
        if [[ -d "$CURRENT_DIR/tests/e2e" ]]; then
            PROJECT_ROOT="$CURRENT_DIR"
            E2E_DIR="$CURRENT_DIR/tests/e2e"
            break
        fi
        CURRENT_DIR="$(dirname "$CURRENT_DIR")"
    done

    if [[ -z "$E2E_DIR" ]]; then
        echo "❌ Error: Could not find tests/e2e directory from $SCRIPT_DIR"
        echo "Please run this script from the project root or ensure tests/e2e exists"
        exit 1
    fi
fi

SCENARIOS_DIR="$E2E_DIR/scenarios"
TEST_RESULTS_DIR="$E2E_DIR/results"
HELPERS_DIR="$E2E_DIR/helpers"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test filtering options
INCLUDE_PATTERN=""
EXCLUDE_PATTERN=""
VERBOSE=false
DRY_RUN=false
DEBUG_PATHS=false

# Server management
SERVER_PID=""
CLEANUP_ON_EXIT=true

usage() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  -i, --include PATTERN    Only run tests matching this pattern (regex)"
    echo "  -e, --exclude PATTERN    Exclude tests matching this pattern (regex)"
    echo "  -v, --verbose           Show detailed output during test execution"
    echo "  -n, --dry-run           Show which tests would run without executing them"
    echo "  -d, --debug             Show path debugging information"
    echo "  --no-server            Don't start/stop server (assume it's already running)"
    echo "  -h, --help              Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                           # Start server and run all tests"
    echo "  $0 -i 'basic|simple'         # Run only tests with 'basic' or 'simple' in name"
    echo "  $0 -e 'stress|memory'        # Run all tests except stress/memory tests"
    echo "  $0 -i '0[1-5]_'              # Run only tests 01 through 05"
    echo "  $0 -n                        # Show what tests would run"
    echo "  $0 -d                        # Show debugging info and paths"
    echo "  $0 --no-server              # Run tests without managing server"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -i|--include)
            INCLUDE_PATTERN="$2"
            shift 2
            ;;
        -e|--exclude)
            EXCLUDE_PATTERN="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -n|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -d|--debug)
            DEBUG_PATHS=true
            VERBOSE=true
            shift
            ;;
        --no-server)
            CLEANUP_ON_EXIT=false
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Debug path information
debug_paths() {
    echo -e "${BLUE}🔍 Path Debug Information${NC}"
    echo "=========================="
    echo "Script location: ${BASH_SOURCE[0]}"
    echo "Script directory: $SCRIPT_DIR"
    echo "Project root: $PROJECT_ROOT"
    echo "E2E directory: $E2E_DIR"
    echo "Scenarios directory: $SCENARIOS_DIR"
    echo "Helpers directory: $HELPERS_DIR"
    echo "Results directory: $TEST_RESULTS_DIR"
    echo "Current working directory: $(pwd)"
    echo

    echo "📁 Directory structure check:"
    if [ -d "$PROJECT_ROOT" ]; then
        echo "✅ Project root exists: $PROJECT_ROOT"
        echo "Project root contents:"
        ls -la "$PROJECT_ROOT" | head -10
    else
        echo "❌ Project root NOT found: $PROJECT_ROOT"
    fi
    echo

    if [ -d "$E2E_DIR" ]; then
        echo "✅ E2E directory exists: $E2E_DIR"
        echo "E2E directory contents:"
        ls -la "$E2E_DIR"
    else
        echo "❌ E2E directory NOT found: $E2E_DIR"
    fi
    echo

    if [ -d "$SCENARIOS_DIR" ]; then
        echo "✅ Scenarios directory exists: $SCENARIOS_DIR"
        echo "Found $(ls -1 "$SCENARIOS_DIR"/*.sh 2>/dev/null | wc -l) .sh files"
        ls -la "$SCENARIOS_DIR"/*.sh 2>/dev/null | head -5 || echo "No .sh files found"
    else
        echo "❌ Scenarios directory NOT found: $SCENARIOS_DIR"
    fi
    echo

    if [ -f "$HELPERS_DIR/test_helpers.sh" ]; then
        echo "✅ Test helpers found: $HELPERS_DIR/test_helpers.sh"
    else
        echo "❌ Test helpers NOT found: $HELPERS_DIR/test_helpers.sh"
    fi
    echo
}

if [ "$DEBUG_PATHS" = true ]; then
    debug_paths
    if [ "$DRY_RUN" = true ]; then
        echo "Debug mode complete. Exiting."
        exit 0
    fi
fi

# Validate required directories exist
if [ ! -d "$E2E_DIR" ]; then
    echo -e "${RED}❌ Error: E2E test directory not found: $E2E_DIR${NC}"
    echo "Expected structure: <project>/tests/e2e/scenarios/"
    echo ""
    echo "🔍 Debugging info:"
    echo "  Script was run from: $SCRIPT_DIR"
    echo "  Looking for E2E dir at: $E2E_DIR"
    echo ""
    echo "💡 Try running the script with -d flag for detailed path debugging"
    exit 1
fi

if [ ! -d "$SCENARIOS_DIR" ]; then
    echo -e "${RED}❌ Error: Scenarios directory not found: $SCENARIOS_DIR${NC}"
    exit 1
fi

if [ ! -f "$HELPERS_DIR/test_helpers.sh" ]; then
    echo -e "${RED}❌ Error: Test helpers not found: $HELPERS_DIR/test_helpers.sh${NC}"
    exit 1
fi

# Create results directory
mkdir -p "$TEST_RESULTS_DIR"

# Cleanup function
cleanup() {
    if [ "$CLEANUP_ON_EXIT" = true ] && [ -n "$SERVER_PID" ]; then
        echo "Cleaning up test environment..."
        echo "🛑 Terminating server started by this script (PID: $SERVER_PID)"
        if kill -TERM "$SERVER_PID" 2>/dev/null; then
            echo "📤 Sending SIGTERM to server..."
            echo "⏳ Waiting for graceful shutdown... (1s)"
            sleep 1
            if kill -0 "$SERVER_PID" 2>/dev/null; then
                echo "🔨 Force killing server..."
                kill -KILL "$SERVER_PID" 2>/dev/null || true
            fi
            echo "✅ Server terminated successfully"
        else
            echo "ℹ️  Server process already terminated"
        fi
    fi
}

# Set up cleanup trap
trap cleanup EXIT

# Start server if requested
if [ "$CLEANUP_ON_EXIT" = true ] && [ "$DRY_RUN" = false ]; then
    echo "Starting server..."
    if [ -f "$PROJECT_ROOT/start-server.sh" ]; then
        # Start server in background and capture PID
        "$PROJECT_ROOT/start-server.sh" &
        SERVER_PID=$!
        echo "🚀 Server started with PID: $SERVER_PID"

        # Wait a moment for server to start
        echo "⏳ Waiting for server to initialize..."
        sleep 3

        # Basic health check
        if ! curl -s -f "http://localhost:5000/health" >/dev/null 2>&1; then
            echo -e "${YELLOW}⚠️ Server health check failed, but continuing...${NC}"
        else
            echo "✅ Server health check passed"
        fi
    else
        echo -e "${YELLOW}⚠️ start-server.sh not found at $PROJECT_ROOT/start-server.sh, assuming server is already running${NC}"
    fi
fi

echo "Running E2E tests..."

# Change to E2E directory so relative paths in test_helpers.sh work correctly
cd "$E2E_DIR"

# Rest of the test runner logic (unchanged)
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
LOG_FILE="$TEST_RESULTS_DIR/test_run_$TIMESTAMP.log"

# Test tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

verbose_log() {
    if [ "$VERBOSE" = true ]; then
        echo "$1" | tee -a "$LOG_FILE"
    else
        echo "$1" >> "$LOG_FILE"
    fi
}

print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}  Secure Linkage Service E2E Tests${NC}"
    echo -e "${BLUE}================================${NC}"
    echo
}

print_summary() {
    echo
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}  Test Summary${NC}"
    echo -e "${BLUE}================================${NC}"
    echo -e "Total Tests: ${TOTAL_TESTS}"
    echo -e "Passed: ${GREEN}${PASSED_TESTS}${NC}"
    echo -e "Failed: ${RED}${FAILED_TESTS}${NC}"
    if [ $SKIPPED_TESTS -gt 0 ]; then
        echo -e "Skipped: ${YELLOW}${SKIPPED_TESTS}${NC}"
    fi
    if [ $TOTAL_TESTS -gt 0 ]; then
        echo -e "Success Rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%"
    fi
    echo -e "Log File: ${LOG_FILE}"
    echo
}

get_test_name() {
    local filename="$1"
    echo "$filename" | sed 's/\.sh$//' | sed 's/_/ /g' | sed 's/\b\w/\U&/g'
}

should_run_test() {
    local test_file="$1"
    local test_name="$2"

    if [ -n "$INCLUDE_PATTERN" ]; then
        if ! echo "$test_file" | grep -E "$INCLUDE_PATTERN" >/dev/null && \
           ! echo "$test_name" | grep -iE "$INCLUDE_PATTERN" >/dev/null; then
            return 1
        fi
    fi

    if [ -n "$EXCLUDE_PATTERN" ]; then
        if echo "$test_file" | grep -E "$EXCLUDE_PATTERN" >/dev/null || \
           echo "$test_name" | grep -iE "$EXCLUDE_PATTERN" >/dev/null; then
            return 1
        fi
    fi

    return 0
}

run_test() {
    local test_name="$1"
    local test_script="$2"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    echo -e "${YELLOW}Running: $test_name${NC}"
    log "Starting test: $test_name ($test_script)"

    if [ "$VERBOSE" = true ]; then
        if bash "$test_script"; then
            echo -e "${GREEN}✅ PASSED: $test_name${NC}"
            log "PASSED: $test_name"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            echo -e "${RED}❌ FAILED: $test_name${NC}"
            log "FAILED: $test_name"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    else
        if bash "$test_script" >> "$LOG_FILE" 2>&1; then
            echo -e "${GREEN}✅ PASSED: $test_name${NC}"
            log "PASSED: $test_name"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            echo -e "${RED}❌ FAILED: $test_name${NC}"
            log "FAILED: $test_name"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    fi
    echo
}

discover_tests() {
    if [ ! -d "$SCENARIOS_DIR" ]; then
        echo -e "${RED}Error: Scenarios directory not found: $SCENARIOS_DIR${NC}" >&2
        return 1
    fi

    verbose_log "Scanning for tests in: $SCENARIOS_DIR"

    local all_sh_files=()
    while IFS= read -r -d '' file; do
        all_sh_files+=("$file")
    done < <(find "$SCENARIOS_DIR" -name "*.sh" -type f -print0 | sort -z)

    verbose_log "Found ${#all_sh_files[@]} .sh files total"

    local executable_files=()
    local non_executable_files=()

    for file in "${all_sh_files[@]}"; do
        if [ -x "$file" ]; then
            executable_files+=("$file")
            verbose_log "  Executable: $(basename "$file")"
        else
            non_executable_files+=("$file")
            verbose_log "  Non-executable: $(basename "$file")"
        fi
    done

    if [ ${#executable_files[@]} -eq 0 ] && [ ${#non_executable_files[@]} -gt 0 ]; then
        echo -e "${YELLOW}Warning: No executable test files found, but ${#non_executable_files[@]} .sh files detected${NC}"
        echo -e "${YELLOW}Attempting to make test files executable...${NC}"

        for file in "${non_executable_files[@]}"; do
            if chmod +x "$file" 2>/dev/null; then
                executable_files+=("$file")
                echo -e "${GREEN}Made executable: $(basename "$file")${NC}"
            else
                echo -e "${RED}Failed to make executable: $(basename "$file")${NC}"
            fi
        done
    fi

    if [ ${#executable_files[@]} -eq 0 ]; then
        echo -e "${YELLOW}Warning: No test files found in $SCENARIOS_DIR${NC}"
        return 1
    fi

    verbose_log "Final executable test count: ${#executable_files[@]}"
    printf '%s\n' "${executable_files[@]}"
}

# Main test execution
print_header

if [ "$DRY_RUN" = true ]; then
    echo -e "${BLUE}DRY RUN MODE - Showing tests that would be executed:${NC}"
    echo
fi

log "Starting E2E test suite (PID: $$)"
verbose_log "Running from E2E directory: $(pwd)"
verbose_log "Scenarios directory: $SCENARIOS_DIR"
verbose_log "Results directory: $TEST_RESULTS_DIR"

if [ -n "$INCLUDE_PATTERN" ]; then
    verbose_log "Include pattern: $INCLUDE_PATTERN"
fi

if [ -n "$EXCLUDE_PATTERN" ]; then
    verbose_log "Exclude pattern: $EXCLUDE_PATTERN"
fi

# Discover all test files
if ! discovered_tests=$(discover_tests); then
    echo -e "${RED}No tests discovered. Exiting.${NC}"
    exit 1
fi

mapfile -t test_files <<< "$discovered_tests"
verbose_log "Processing ${#test_files[@]} discovered test files"

# Process each discovered test
for test_file in "${test_files[@]}"; do
    [ -z "$test_file" ] && continue

    test_filename=$(basename "$test_file")
    test_name=$(get_test_name "$test_filename")

    if should_run_test "$test_filename" "$test_name"; then
        if [ "$DRY_RUN" = true ]; then
            echo -e "${GREEN}Would run:${NC} $test_name ($test_filename)"
            TOTAL_TESTS=$((TOTAL_TESTS + 1))
        else
            run_test "$test_name" "$test_file"
        fi
    else
        verbose_log "Skipping: $test_name ($test_filename)"
        SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
    fi
done

if [ "$DRY_RUN" = true ]; then
    echo
    echo -e "${BLUE}Summary:${NC}"
    echo -e "Tests that would run: ${TOTAL_TESTS}"
    echo -e "Tests that would be skipped: ${SKIPPED_TESTS}"
    exit 0
fi

print_summary

echo "E2E tests completed"

# Show recent failures for quick debugging
if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}Recent test failures (last 20 lines from log):${NC}"
    echo -e "${RED}================================================${NC}"
    tail -20 "$LOG_FILE" | grep -E "(FAILED|ERROR|❌)" || tail -20 "$LOG_FILE"
    echo -e "${RED}================================================${NC}"
    echo -e "Full log: ${LOG_FILE}"
    echo

    log "Test suite completed with $FAILED_TESTS failures"
    exit 1
fi

log "All tests completed successfully"
