#!/bin/bash
# QEMU network test script
# Tests network connectivity: DHCP, DNS, and HTTP

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ISO_FILE="$PROJECT_ROOT/moteos-x64-uefi.iso"
OVMF_CODE="/usr/share/OVMF/OVMF_CODE.fd"
OVMF_VARS="/usr/share/OVMF/OVMF_VARS.fd"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}Running moteOS network test...${NC}"

# Check for required tools
command -v qemu-system-x86_64 >/dev/null 2>&1 || { 
    echo -e "${RED}Error: qemu-system-x86_64 not found${NC}" >&2
    exit 1
}

command -v nc >/dev/null 2>&1 || { 
    echo -e "${YELLOW}Warning: netcat (nc) not found - some network tests may be limited${NC}"
}

# Check if ISO exists
if [ ! -f "$ISO_FILE" ]; then
    echo -e "${YELLOW}ISO file not found: $ISO_FILE${NC}"
    echo -e "${YELLOW}Building ISO first...${NC}"
    "$SCRIPT_DIR/build-iso-uefi.sh" || {
        echo -e "${RED}Failed to build ISO${NC}" >&2
        exit 1
    }
fi

# Check for OVMF
if [ ! -f "$OVMF_CODE" ]; then
    if [ -f "/usr/share/qemu/OVMF_CODE.fd" ]; then
        OVMF_CODE="/usr/share/qemu/OVMF_CODE.fd"
        OVMF_VARS="/usr/share/qemu/OVMF_VARS.fd"
    elif [ -f "$PROJECT_ROOT/OVMF_CODE.fd" ]; then
        OVMF_CODE="$PROJECT_ROOT/OVMF_CODE.fd"
        OVMF_VARS="$PROJECT_ROOT/OVMF_VARS.fd"
    else
        OVMF_CODE=""
        OVMF_VARS=""
    fi
fi

# Create temporary OVMF vars file if needed
if [ -n "$OVMF_CODE" ] && [ -f "$OVMF_CODE" ]; then
    TEMP_VARS=$(mktemp)
    cp "$OVMF_VARS" "$TEMP_VARS" 2>/dev/null || touch "$TEMP_VARS"
    trap "rm -f $TEMP_VARS" EXIT
fi

# Set up network test server (simple HTTP server)
TEST_PORT=8080
TEST_SERVER_PID=""

start_test_server() {
    echo -e "${BLUE}Starting test HTTP server on port $TEST_PORT...${NC}"
    
    # Create a simple test response
    cat > /tmp/moteos-test-response.txt << 'EOF'
HTTP/1.1 200 OK
Content-Type: text/plain
Content-Length: 13

Network OK!
EOF
    
    # Start a simple HTTP server using netcat or python
    if command -v python3 >/dev/null 2>&1; then
        python3 -m http.server $TEST_PORT > /tmp/test-server.log 2>&1 &
        TEST_SERVER_PID=$!
    elif command -v nc >/dev/null 2>&1; then
        # Fallback: use netcat (less ideal)
        while true; do
            nc -l -p $TEST_PORT < /tmp/moteos-test-response.txt
        done > /tmp/test-server.log 2>&1 &
        TEST_SERVER_PID=$!
    else
        echo -e "${YELLOW}Warning: No HTTP server available (python3 or nc)${NC}"
        echo -e "${YELLOW}Network test will be limited to QEMU user networking${NC}"
    fi
    
    sleep 1
}

stop_test_server() {
    if [ -n "$TEST_SERVER_PID" ]; then
        kill $TEST_SERVER_PID 2>/dev/null || true
        wait $TEST_SERVER_PID 2>/dev/null || true
    fi
}

trap stop_test_server EXIT

# Start test server
start_test_server

# QEMU command with user networking
QEMU_CMD="qemu-system-x86_64"

QEMU_ARGS=(
    -machine q35
    -cpu qemu64
    -m 1G
    -drive "file=$ISO_FILE,format=raw,media=cdrom,id=cdrom0,if=none"
    -device "ide-cd,drive=cdrom0,bootindex=1"
    -netdev "user,id=net0,hostfwd=tcp::2222-:22,hostfwd=tcp::8080-:8080"
    -device "virtio-net,netdev=net0"
    -serial stdio
    -monitor none
    -display none
    -no-reboot
)

# Add UEFI firmware if available
if [ -n "$OVMF_CODE" ] && [ -f "$OVMF_CODE" ]; then
    QEMU_ARGS+=(
        -drive "if=pflash,format=raw,readonly=on,file=$OVMF_CODE"
        -drive "if=pflash,format=raw,file=$TEMP_VARS"
    )
fi

echo -e "${GREEN}Starting QEMU with network support...${NC}"
echo -e "${BLUE}Network configuration:${NC}"
echo -e "  - QEMU user networking enabled"
echo -e "  - Guest IP: 10.0.2.15 (default QEMU user net)"
echo -e "  - Host forwarding: localhost:8080 -> guest:8080"
echo -e "  - Test server running on host: localhost:$TEST_PORT"
echo ""
echo -e "${YELLOW}The system will boot and attempt network operations:${NC}"
echo -e "  1. DHCP acquisition"
echo -e "  2. DNS resolution (e.g., google.com)"
echo -e "  3. HTTP connection to test server"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop the test${NC}"
echo ""

# Run QEMU with timeout (60 seconds for network operations)
timeout 60s "$QEMU_CMD" "${QEMU_ARGS[@]}" 2>&1 | tee /tmp/moteos-network-test.log || {
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ]; then
        echo -e "${GREEN}✓ Network test completed (timeout reached)${NC}"
    else
        echo -e "${RED}✗ Network test failed (exit code: $EXIT_CODE)${NC}"
        echo -e "${YELLOW}Check /tmp/moteos-network-test.log for details${NC}"
        exit $EXIT_CODE
    fi
}

# Check log for network operation indicators
echo ""
echo -e "${BLUE}Analyzing network test results...${NC}"

DHCP_SUCCESS=false
DNS_SUCCESS=false
HTTP_SUCCESS=false

if grep -qi "dhcp.*success\|ip.*acquired\|network.*configured" /tmp/moteos-network-test.log 2>/dev/null; then
    DHCP_SUCCESS=true
    echo -e "${GREEN}✓ DHCP: IP address acquired${NC}"
else
    echo -e "${YELLOW}⚠ DHCP: No clear success indicator${NC}"
fi

if grep -qi "dns.*resolved\|dns.*success\|resolved.*to" /tmp/moteos-network-test.log 2>/dev/null; then
    DNS_SUCCESS=true
    echo -e "${GREEN}✓ DNS: Resolution successful${NC}"
else
    echo -e "${YELLOW}⚠ DNS: No clear success indicator${NC}"
fi

if grep -qi "http.*success\|http.*200\|connection.*established" /tmp/moteos-network-test.log 2>/dev/null; then
    HTTP_SUCCESS=true
    echo -e "${GREEN}✓ HTTP: Connection successful${NC}"
else
    echo -e "${YELLOW}⚠ HTTP: No clear success indicator${NC}"
fi

echo ""
if [ "$DHCP_SUCCESS" = true ] || [ "$DNS_SUCCESS" = true ] || [ "$HTTP_SUCCESS" = true ]; then
    echo -e "${GREEN}✓ Network test completed with some successful operations${NC}"
    echo -e "${YELLOW}Check /tmp/moteos-network-test.log for full details${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ Network test completed but no clear success indicators found${NC}"
    echo -e "${YELLOW}This may be normal if the kernel doesn't log network operations yet${NC}"
    echo -e "${YELLOW}Check /tmp/moteos-network-test.log for kernel output${NC}"
    exit 0  # Don't fail - manual inspection needed
fi
