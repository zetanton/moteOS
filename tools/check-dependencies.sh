#!/bin/bash
# Dependency Checker for Vibe Kanban Tasks
# 
# Usage: ./tools/check-dependencies.sh <task-title>
# 
# Checks if a task's dependencies are complete before starting work.

TASK_TITLE="$1"

if [ -z "$TASK_TITLE" ]; then
    echo "Usage: $0 '<task-title>'"
    echo "Example: $0 '[2] HTTP/1.1 client'"
    exit 1
fi

# Dependency map
declare -A DEPENDENCIES

# Workstream 1
DEPENDENCIES["[1] Project scaffolding (Cargo workspace structure)"]=""
DEPENDENCIES["[1] x86_64 UEFI boot entry point"]="[1] Project scaffolding (Cargo workspace structure)"
DEPENDENCIES["[1] Memory allocator setup"]="[1] Project scaffolding (Cargo workspace structure)"
DEPENDENCIES["[1] Interrupt setup (x86_64)"]="[1] Project scaffolding (Cargo workspace structure)"
DEPENDENCIES["[1] Framebuffer interface"]="[1] Project scaffolding (Cargo workspace structure)"
DEPENDENCIES["[1] Timer setup"]="[1] Project scaffolding (Cargo workspace structure)"

# Workstream 2
DEPENDENCIES["[2] virtio-net driver"]="[1] Memory allocator setup"
DEPENDENCIES["[2] smoltcp integration"]="[2] virtio-net driver"
DEPENDENCIES["[2] DHCP client"]="[2] smoltcp integration"
DEPENDENCIES["[2] DNS resolver"]="[2] DHCP client"
DEPENDENCIES["[2] TLS 1.3 support"]="[2] DNS resolver"
DEPENDENCIES["[2] HTTP/1.1 client"]="[2] TLS 1.3 support"

# Workstream 3
DEPENDENCIES["[3] Framebuffer rendering"]="[1] Framebuffer interface"
DEPENDENCIES["[3] Font system"]="[3] Framebuffer rendering"
DEPENDENCIES["[3] Dark theme implementation"]="[3] Framebuffer rendering"
DEPENDENCIES["[3] Input widget"]="[3] Base widget system"
DEPENDENCIES["[3] Chat screen"]="[3] Message widget"

# Workstream 4
DEPENDENCIES["[4] Common types and LLM provider trait"]=""
DEPENDENCIES["[4] OpenAI client with streaming"]="[2] HTTP/1.1 client"
DEPENDENCIES["[4] Anthropic client with streaming"]="[2] HTTP/1.1 client"
DEPENDENCIES["[4] Groq client with streaming"]="[2] HTTP/1.1 client"
DEPENDENCIES["[4] xAI client with streaming"]="[2] HTTP/1.1 client"

# Workstream 6
DEPENDENCIES["[6] GGUF file format parser"]=""
DEPENDENCIES["[6] BPE tokenizer"]="[6] GGUF file format parser"
DEPENDENCIES["[6] Transformer forward pass"]="[6] Tensor operations"
DEPENDENCIES["[6] Generation loop with streaming"]="[6] Transformer forward pass"

# Workstream 7
DEPENDENCIES["[7] TOML parser (no_std)"]=""
DEPENDENCIES["[7] EFI variable storage"]="[7] TOML parser (no_std)"
DEPENDENCIES["[7] Setup wizard"]="[7] EFI variable storage"

# Workstream 8
DEPENDENCIES["[8] Cargo workspace setup"]=""
DEPENDENCIES["[8] Kernel entry point and main loop"]="[1] Memory allocator setup"
DEPENDENCIES["[8] Component integration"]="[4] OpenAI client with streaming"
DEPENDENCIES["[8] ISO generation and QEMU testing"]="[8] Component integration"

# Check if task exists
if [ -z "${DEPENDENCIES[$TASK_TITLE]}" ]; then
    echo "⚠️  Task not found in dependency map: $TASK_TITLE"
    echo "   This might be a new task. Check docs/DEPENDENCY_MANAGEMENT.md"
    exit 1
fi

DEPENDENCY="${DEPENDENCIES[$TASK_TITLE]}"

if [ -z "$DEPENDENCY" ]; then
    echo "✅ $TASK_TITLE"
    echo "   No dependencies - safe to start!"
    exit 0
fi

echo "🔍 Checking dependencies for: $TASK_TITLE"
echo ""
echo "📋 Required dependency: $DEPENDENCY"
echo ""
echo "⚠️  Before starting this task:"
echo "   1. Check Vibe Kanban - is '$DEPENDENCY' in 'Done' status?"
echo "   2. Verify code is merged to main branch"
echo "   3. Test that dependency works"
echo ""
echo "💡 To check task status, use:"
echo "   - Vibe Kanban UI"
echo "   - MCP: list_tasks with status filter"
echo ""
echo "❌ Do NOT start until dependency is complete!"

exit 2
