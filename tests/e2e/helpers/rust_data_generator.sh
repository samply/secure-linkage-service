#!/bin/bash
# Shell wrapper for generating crypto test data

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Function to generate test data using our Rust script
generate_rust_test_data() {
    local scenario=${1:-"multiple_matches"}

    # Use rustc to compile and run the standalone script
    local temp_binary="/tmp/crypto_data_generator_$$"

    if rustc "$SCRIPT_DIR/generate_crypto_data.rs" -o "$temp_binary" 2>/dev/null; then
        if "$temp_binary" "$scenario" 2>/dev/null; then
            rm -f "$temp_binary"
            return 0
        else
            rm -f "$temp_binary"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️ rustc compilation failed, using fallback${NC}" >&2
        return 1
    fi
}

# Function to provide fallback data
get_fallback_data() {
    local scenario=${1:-"multiple_matches"}

    case "$scenario" in
        "multiple_matches")
            cat << 'FALLBACK_EOF'
ids_data='[
    "Y3J5cHRvZ3JhcGhpY2FsbHlfZW5jcnlwdGVkX2RhdGFfMTIzNDU2Nzg5MA==",
    "c2VjdXJlX2xpbmthZ2VfZGF0YV93aXRoX3Byb3Blcl9lbmNyeXB0aW9uXzE=",
    "bWVkaWNhbF9pbmZvcm1hdGljc19kYXRhX2VuY3J5cHRlZF9zYWZlbHlfMjM0",
    "ZnVuY3Rpb25hbF9wcm9ncmFtbWluZ19kYXRhX3dpdGhfY3J5cHRvXzM0NQ==",
    "bGVhcm5pbmdfcnVzdF9kYXRhX3dpdGhfcHJvcGVyX2VuY3J5cHRpb25fNDU2"
]'

server_result_data='[
    {"match": 0, "tentativeMatch": 1, "bestIndex": 0},
    {"match": 1, "tentativeMatch": 0, "bestIndex": 1},
    {"match": 0, "tentativeMatch": 0, "bestIndex": 2},
    {"match": 1, "tentativeMatch": 1, "bestIndex": 3},
    {"match": 0, "tentativeMatch": 1, "bestIndex": 4}
]'

client_result_data='[
    {"match": 1, "tentativeMatch": 0, "bestIndex": 0},
    {"match": 0, "tentativeMatch": 1, "bestIndex": 1},
    {"match": 0, "tentativeMatch": 0, "bestIndex": 2},
    {"match": 0, "tentativeMatch": 0, "bestIndex": 3},
    {"match": 0, "tentativeMatch": 1, "bestIndex": 4}
]'
FALLBACK_EOF
            ;;
        "single_match")
            cat << 'FALLBACK_EOF'
ids_data='[
    "c2luZ2xlX3JlY29yZF90ZXN0X2RhdGFfd2l0aF9wcm9wZXJfZW5jcnlwdGlvbg=="
]'

server_result_data='[
    {"match": 0, "tentativeMatch": 1, "bestIndex": 0}
]'

client_result_data='[
    {"match": 1, "tentativeMatch": 0, "bestIndex": 0}
]'
FALLBACK_EOF
            ;;
    esac
}

# Main function
main() {
    local command=${1:-"help"}

    case "$command" in
        "generate")
            local scenario=${2:-"multiple_matches"}

            # Try Rust generator first, fall back to static data
            if ! generate_rust_test_data "$scenario"; then
                echo -e "${YELLOW}⚠️ Using fallback data${NC}" >&2
                get_fallback_data "$scenario"
            fi
            ;;
        "help"|*)
            echo "Usage: $0 <command> [options]"
            echo
            echo "Commands:"
            echo "  generate <scenario> [server_party] [client_party] [count]"
            echo "    Generate test data (tries Rust, falls back to static)"
            echo "    Scenarios: multiple_matches, single_match"
            ;;
    esac
}

# Run main function with all arguments
main "$@"
