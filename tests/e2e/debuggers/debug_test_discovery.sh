#!/bin/bash
# Debug script to diagnose test discovery issues

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCENARIOS_DIR="$SCRIPT_DIR/scenarios"

echo "🔍 Test Discovery Diagnostic"
echo "============================"
echo "Script directory: $SCRIPT_DIR"
echo "Scenarios directory: $SCENARIOS_DIR"
echo

# Check if scenarios directory exists
if [ ! -d "$SCENARIOS_DIR" ]; then
    echo "❌ ERROR: Scenarios directory not found: $SCENARIOS_DIR"
    exit 1
fi

echo "✅ Scenarios directory exists"
echo

# List all files in scenarios directory
echo "📁 All files in scenarios directory:"
ls -la "$SCENARIOS_DIR"
echo

# Find .sh files
echo "🔍 Looking for .sh files:"
find "$SCENARIOS_DIR" -name "*.sh" -type f | sort
echo

# Check executable permissions
echo "🔒 Checking executable permissions:"
find "$SCENARIOS_DIR" -name "*.sh" -type f -exec ls -l {} \; | sort
echo

# Find executable .sh files
echo "✅ Executable .sh files:"
find "$SCENARIOS_DIR" -name "*.sh" -type f -executable | sort
echo

# Test the exact discovery method from your script
echo "🧪 Testing discovery method (find with -print0 and sort -z):"
echo "Files found:"
find "$SCENARIOS_DIR" -name "*.sh" -type f -executable -print0 | sort -z | tr '\0' '\n'
echo

# Count tests
executable_count=$(find "$SCENARIOS_DIR" -name "*.sh" -type f -executable | wc -l)
total_sh_count=$(find "$SCENARIOS_DIR" -name "*.sh" -type f | wc -l)

echo "📊 Summary:"
echo "  Total .sh files: $total_sh_count"
echo "  Executable .sh files: $executable_count"
echo

if [ "$executable_count" -ne "$total_sh_count" ]; then
    echo "⚠️  Some .sh files are not executable!"
    echo "Non-executable files:"
    find "$SCENARIOS_DIR" -name "*.sh" -type f ! -executable
    echo
    echo "💡 To fix, run:"
    echo "chmod +x $SCENARIOS_DIR/*.sh"
fi

if [ "$executable_count" -eq 1 ]; then
    echo "🔍 Only 1 executable test found. This explains why only 1 test runs."
    echo "The executable test is:"
    find "$SCENARIOS_DIR" -name "*.sh" -type f -executable
fi
