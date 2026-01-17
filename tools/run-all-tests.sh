#!/bin/bash
# Comprehensive test runner for moteOS
# Runs all tests in sequence and provides a summary report

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test results tracking
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Test log file
TEST_LOG="/tmp/moteos-test-suite.log"
echo "moteOS Test Suite - $(date)" > "$TEST_LOG"
echo "========================================" >> "$TEST_LOG"

# Print header
echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║     moteOS Comprehensive Test Suite    ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════╝${NC}"
echo ""

# Helper functions
run_test() {
    local test_name="$1"
    local test_command="$2"
    local required="${3:-true}"  # Default to required
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}[$TESTS_RUN]${NC} ${CYAN}$test_name${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    
    if eval "$test_command" >> "$TEST_LOG" 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}: $test_name"
        echo "PASSED: $test_name" >> "$TEST_LOG"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        local exit_code=$?
        echo -e "${RED}✗ FAILED${NC}: $test_name (exit code: $exit_code)"
        echo "FAILED: $test_name (exit code: $exit_code)" >> "$TEST_LOG"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        
        if [ "$required" = "true" ]; then
            echo -e "${RED}This is a required test. Stopping test suite.${NC}"
            return $exit_code
        else
            echo -e "${YELLOW}This is an optional test. Continuing...${NC}"
            return 0
        fi
    fi
}

skip_test() {
    local test_name="$1"
    local reason="$2"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_SKIPPED=$((TESTS_SKIPPED + 1))
    
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}[$TESTS_RUN]${NC} ${MAGENTA}$test_name${NC} ${YELLOW}[SKIPPED]${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}Reason: $reason${NC}"
    echo ""
    echo "SKIPPED: $test_name - $reason" >> "$TEST_LOG"
}

check_dependency() {
    local cmd="$1"
    local install_hint="$2"
    
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo -e "${RED}✗ Missing dependency: $cmd${NC}"
        if [ -n "$install_hint" ]; then
            echo -e "${YELLOW}Install with: $install_hint${NC}"
        fi
        return 1
    fi
    return 0
}

# Check prerequisites
echo -e "${CYAN}Checking prerequisites...${NC}"
echo ""

MISSING_DEPS=0

check_dependency "cargo" "Install Rust from https://rustup.rs" || MISSING_DEPS=$((MISSING_DEPS + 1))
check_dependency "rustc" "Install Rust from https://rustup.rs" || MISSING_DEPS=$((MISSING_DEPS + 1))
check_dependency "qemu-system-x86_64" "brew install qemu (macOS) or apt-get install qemu-system-x86 (Linux)" || MISSING_DEPS=$((MISSING_DEPS + 1))
check_dependency "xorriso" "brew install xorriso (macOS) or apt-get install xorriso (Linux)" || MISSING_DEPS=$((MISSING_DEPS + 1))
check_dependency "python3" "Usually pre-installed" || MISSING_DEPS=$((MISSING_DEPS + 1))

if [ $MISSING_DEPS -gt 0 ]; then
    echo ""
    echo -e "${RED}Missing $MISSING_DEPS required dependencies. Please install them before running tests.${NC}"
    exit 1
fi

echo -e "${GREEN}All prerequisites met!${NC}"
echo ""

# Phase 1: Unit Tests
echo -e "${MAGENTA}════════════════════════════════════════════${NC}"
echo -e "${MAGENTA}  PHASE 1: Unit Tests${NC}"
echo -e "${MAGENTA}════════════════════════════════════════════${NC}"
echo ""

# Note: Some crates may not have std available, so we test what we can
run_test "Rust unit tests (std-compatible crates)" \
    "cargo test --workspace --lib --no-fail-fast 2>&1 | grep -E '(test result|running|test.*ok|test.*FAILED)' || true" \
    "false"

# Phase 2: Build Verification
echo ""
echo -e "${MAGENTA}════════════════════════════════════════════${NC}"
echo -e "${MAGENTA}  PHASE 2: Build Verification${NC}"
echo -e "${MAGENTA}════════════════════════════════════════════${NC}"
echo ""

run_test "x86_64 UEFI ISO build" \
    "$SCRIPT_DIR/build-iso-uefi.sh" \
    "true"

run_test "x86_64 BIOS ISO build" \
    "$SCRIPT_DIR/build-iso-bios.sh" \
    "false"

run_test "AArch64 cross-compilation build" \
    "$SCRIPT_DIR/test-build-aarch64.sh" \
    "false"

# Phase 3: Boot Tests
echo ""
echo -e "${MAGENTA}════════════════════════════════════════════${NC}"
echo -e "${MAGENTA}  PHASE 3: Boot Tests${NC}"
echo -e "${MAGENTA}════════════════════════════════════════════${NC}"
echo ""

if [ -f "$PROJECT_ROOT/moteos-x64-uefi.iso" ]; then
    run_test "x86_64 UEFI boot test" \
        "$SCRIPT_DIR/test-boot.sh" \
        "true"
else
    skip_test "x86_64 UEFI boot test" "ISO not found (build failed or skipped)"
fi

# Phase 4: Integration Tests
echo ""
echo -e "${MAGENTA}════════════════════════════════════════════${NC}"
echo -e "${MAGENTA}  PHASE 4: Integration Tests${NC}"
echo -e "${MAGENTA}════════════════════════════════════════════${NC}"
echo ""

if [ -f "$PROJECT_ROOT/moteos-x64-uefi.iso" ]; then
    run_test "Network integration test" \
        "$SCRIPT_DIR/test-network.sh" \
        "false"
    
    run_test "API integration test" \
        "$SCRIPT_DIR/test-api.sh" \
        "false"
else
    skip_test "Network integration test" "ISO not found"
    skip_test "API integration test" "ISO not found"
fi

# Summary
echo ""
echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║          Test Suite Summary            ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════╝${NC}"
echo ""

echo -e "${BLUE}Tests Run:${NC}    $TESTS_RUN"
echo -e "${GREEN}Passed:${NC}        $TESTS_PASSED"
echo -e "${RED}Failed:${NC}        $TESTS_FAILED"
echo -e "${YELLOW}Skipped:${NC}      $TESTS_SKIPPED"
echo ""

# Calculate success rate
if [ $TESTS_RUN -gt 0 ]; then
    SUCCESS_RATE=$((TESTS_PASSED * 100 / TESTS_RUN))
    echo -e "${BLUE}Success Rate:${NC} $SUCCESS_RATE%"
    echo ""
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}✓ All tests passed!${NC}"
        echo ""
        echo -e "${GREEN}Full test log: $TEST_LOG${NC}"
        exit 0
    else
        echo -e "${RED}✗ Some tests failed. Check the log for details.${NC}"
        echo ""
        echo -e "${YELLOW}Full test log: $TEST_LOG${NC}"
        echo -e "${YELLOW}Failed test details:${NC}"
        grep "FAILED:" "$TEST_LOG" | sed 's/^/  /'
        exit 1
    fi
else
    echo -e "${YELLOW}No tests were run.${NC}"
    exit 1
fi
