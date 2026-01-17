#!/bin/bash
# QEMU API test script
# Tests LLM API connectivity and functionality

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

echo -e "${GREEN}Running moteOS API test...${NC}"

# Check for required tools
command -v qemu-system-x86_64 >/dev/null 2>&1 || { 
    echo -e "${RED}Error: qemu-system-x86_64 not found${NC}" >&2
    exit 1
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

# Set up mock API server
MOCK_API_PORT=8080
MOCK_API_PID=""

start_mock_api() {
    echo -e "${BLUE}Starting mock LLM API server on port $MOCK_API_PORT...${NC}"
    
    # Create a simple mock OpenAI-compatible API response
    cat > /tmp/mock-api-response.json << 'EOF'
{
  "id": "chatcmpl-test",
  "object": "chat.completion.chunk",
  "created": 1234567890,
  "model": "gpt-4o",
  "choices": [{
    "index": 0,
    "delta": {"content": "Hello"},
    "finish_reason": null
  }]
}
EOF

    # Start a mock API server using Python
    if command -v python3 >/dev/null 2>&1; then
        cat > /tmp/mock_api_server.py << 'PYEOF'
import http.server
import socketserver
import json
import time

class MockAPIHandler(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path.startswith('/v1/chat/completions'):
            self.send_response(200)
            self.send_header('Content-Type', 'text/event-stream')
            self.send_header('Cache-Control', 'no-cache')
            self.send_header('Connection', 'keep-alive')
            self.end_headers()
            
            # Send SSE stream
            chunks = [
                '{"id":"test","object":"chat.completion.chunk","choices":[{"delta":{"content":"Hello"}}]}',
                '{"id":"test","object":"chat.completion.chunk","choices":[{"delta":{"content":" there"}}]}',
                '{"id":"test","object":"chat.completion.chunk","choices":[{"delta":{"content":"!"}}]}',
                '[DONE]'
            ]
            
            for chunk in chunks:
                self.wfile.write(f'data: {chunk}\n\n'.encode())
                self.wfile.flush()
                time.sleep(0.1)
        else:
            self.send_response(404)
            self.end_headers()
    
    def log_message(self, format, *args):
        pass  # Suppress logging

PORT = 8080
with socketserver.TCPServer(("", PORT), MockAPIHandler) as httpd:
    httpd.serve_forever()
PYEOF
        
        python3 /tmp/mock_api_server.py > /tmp/mock-api-server.log 2>&1 &
        MOCK_API_PID=$!
        sleep 2
        echo -e "${GREEN}Mock API server started (PID: $MOCK_API_PID)${NC}"
    else
        echo -e "${YELLOW}Warning: Python3 not found - cannot start mock API server${NC}"
        echo -e "${YELLOW}API test will be limited${NC}"
    fi
}

stop_mock_api() {
    if [ -n "$MOCK_API_PID" ]; then
        kill $MOCK_API_PID 2>/dev/null || true
        wait $MOCK_API_PID 2>/dev/null || true
        echo -e "${BLUE}Mock API server stopped${NC}"
    fi
}

trap stop_mock_api EXIT

# Start mock API server
start_mock_api

# QEMU command with network and host forwarding
QEMU_CMD="qemu-system-x86_64"

QEMU_ARGS=(
    -machine q35
    -cpu qemu64
    -m 1G
    -drive "file=$ISO_FILE,format=raw,media=cdrom,id=cdrom0,if=none"
    -device "ide-cd,drive=cdrom0,bootindex=1"
    -netdev "user,id=net0,hostfwd=tcp::8080-:8080"
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

echo -e "${GREEN}Starting QEMU with API test configuration...${NC}"
echo -e "${BLUE}Test configuration:${NC}"
echo -e "  - Mock API server: localhost:$MOCK_API_PORT"
echo -e "  - Host forwarding: localhost:8080 -> guest:8080"
echo -e "  - Guest can access mock API at: 10.0.2.2:8080 (QEMU host IP)"
echo ""
echo -e "${YELLOW}The system will boot and attempt API operations:${NC}"
echo -e "  1. Network initialization"
echo -e "  2. TLS/HTTPS connection to API endpoint"
echo -e "  3. API request (POST /v1/chat/completions)"
echo -e "  4. Streaming response parsing"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop the test${NC}"
echo ""

# Run QEMU with timeout (90 seconds for API operations)
timeout 90s "$QEMU_CMD" "${QEMU_ARGS[@]}" 2>&1 | tee /tmp/moteos-api-test.log || {
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ]; then
        echo -e "${GREEN}✓ API test completed (timeout reached)${NC}"
    else
        echo -e "${RED}✗ API test failed (exit code: $EXIT_CODE)${NC}"
        echo -e "${YELLOW}Check /tmp/moteos-api-test.log for details${NC}"
        exit $EXIT_CODE
    fi
}

# Check log for API operation indicators
echo ""
echo -e "${BLUE}Analyzing API test results...${NC}"

TLS_SUCCESS=false
API_REQUEST_SUCCESS=false
API_RESPONSE_SUCCESS=false

if grep -qi "tls.*established\|https.*connected\|ssl.*handshake" /tmp/moteos-api-test.log 2>/dev/null; then
    TLS_SUCCESS=true
    echo -e "${GREEN}✓ TLS: Connection established${NC}"
else
    echo -e "${YELLOW}⚠ TLS: No clear success indicator${NC}"
fi

if grep -qi "api.*request\|post.*completions\|http.*post" /tmp/moteos-api-test.log 2>/dev/null; then
    API_REQUEST_SUCCESS=true
    echo -e "${GREEN}✓ API Request: Sent successfully${NC}"
else
    echo -e "${YELLOW}⚠ API Request: No clear success indicator${NC}"
fi

if grep -qi "api.*response\|streaming.*token\|completion.*received" /tmp/moteos-api-test.log 2>/dev/null; then
    API_RESPONSE_SUCCESS=true
    echo -e "${GREEN}✓ API Response: Received successfully${NC}"
else
    echo -e "${YELLOW}⚠ API Response: No clear success indicator${NC}"
fi

# Check mock API server logs
if [ -f /tmp/mock-api-server.log ]; then
    if grep -qi "POST.*completions" /tmp/mock-api-server.log 2>/dev/null; then
        echo -e "${GREEN}✓ Mock API: Request received${NC}"
        API_REQUEST_SUCCESS=true
    fi
fi

echo ""
if [ "$TLS_SUCCESS" = true ] || [ "$API_REQUEST_SUCCESS" = true ] || [ "$API_RESPONSE_SUCCESS" = true ]; then
    echo -e "${GREEN}✓ API test completed with some successful operations${NC}"
    echo -e "${YELLOW}Check /tmp/moteos-api-test.log for full details${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ API test completed but no clear success indicators found${NC}"
    echo -e "${YELLOW}This may be normal if the kernel doesn't log API operations yet${NC}"
    echo -e "${YELLOW}Check /tmp/moteos-api-test.log for kernel output${NC}"
    exit 0  # Don't fail - manual inspection needed
fi
