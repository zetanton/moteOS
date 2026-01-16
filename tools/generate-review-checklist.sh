#!/bin/bash
# Generate Review Checklist for a Task
#
# Usage: ./tools/generate-review-checklist.sh "<task-title>"
#
# Generates a review checklist based on the task's workstream

TASK_TITLE="$1"

if [ -z "$TASK_TITLE" ]; then
    echo "Usage: $0 '<task-title>'"
    echo "Example: $0 '[1] Memory allocator setup'"
    exit 1
fi

# Determine workstream from task title
if [[ "$TASK_TITLE" =~ ^\[1\] ]]; then
    WORKSTREAM="Boot & Core"
    SPEC_SECTION="3.1"
    FOCUS="Memory management, interrupts, boot sequence"
elif [[ "$TASK_TITLE" =~ ^\[2\] ]]; then
    WORKSTREAM="Network Stack"
    SPEC_SECTION="3.2"
    FOCUS="Network drivers, TCP/IP, TLS, HTTP"
elif [[ "$TASK_TITLE" =~ ^\[3\] ]]; then
    WORKSTREAM="TUI Framework"
    SPEC_SECTION="3.3"
    FOCUS="Framebuffer rendering, widgets, themes"
elif [[ "$TASK_TITLE" =~ ^\[4\] ]]; then
    WORKSTREAM="LLM API Clients"
    SPEC_SECTION="3.4"
    FOCUS="API integration, streaming, error handling"
elif [[ "$TASK_TITLE" =~ ^\[6\] ]]; then
    WORKSTREAM="Local Inference"
    SPEC_SECTION="3.6"
    FOCUS="GGUF parsing, tensor ops, transformer"
elif [[ "$TASK_TITLE" =~ ^\[7\] ]]; then
    WORKSTREAM="Config System"
    SPEC_SECTION="3.7"
    FOCUS="TOML parsing, EFI storage, encryption"
elif [[ "$TASK_TITLE" =~ ^\[8\] ]]; then
    WORKSTREAM="Integration"
    SPEC_SECTION="3.8"
    FOCUS="Component integration, main loop"
else
    WORKSTREAM="Unknown"
    SPEC_SECTION="N/A"
    FOCUS="General review"
fi

cat << EOF
╔════════════════════════════════════════════════════════════════╗
║           CODE REVIEW CHECKLIST                                ║
╚════════════════════════════════════════════════════════════════╝

Task: $TASK_TITLE
Workstream: $WORKSTREAM
Specification: docs/TECHNICAL_SPECIFICATIONS.md Section $SPEC_SECTION

═══════════════════════════════════════════════════════════════════

1. SPECIFICATION COMPLIANCE
   ───────────────────────────────────────────────────────────────
   ☐ Code matches technical specifications (Section $SPEC_SECTION)
   ☐ Interface contracts are correct
   ☐ Function signatures match spec
   ☐ Data structures match spec
   ☐ Error types match spec
   ☐ Return types match spec

2. RUST OS DEVELOPMENT
   ───────────────────────────────────────────────────────────────
   ☐ #![no_std] present and enforced
   ☐ No standard library usage
   ☐ Proper use of alloc crate (if needed)
   ☐ Memory safety (no undefined behavior)
   ☐ Proper use of unsafe blocks (minimal, documented)
   ☐ No panics in production paths
   ☐ Proper error handling

3. WORKSTREAM-SPECIFIC CHECKS
   ───────────────────────────────────────────────────────────────
   Focus: $FOCUS
   
   $(case "$WORKSTREAM" in
     "Boot & Core")
       echo "   ☐ Memory allocator correctness"
       echo "   ☐ Interrupt handler safety"
       echo "   ☐ Framebuffer access safety"
       echo "   ☐ Boot sequence correctness"
       ;;
     "Network Stack")
       echo "   ☐ Network driver trait implementation"
       echo "   ☐ smoltcp integration correctness"
       echo "   ☐ TLS certificate verification"
       echo "   ☐ HTTP parsing correctness"
       ;;
     "TUI Framework")
       echo "   ☐ Framebuffer rendering safety"
       echo "   ☐ Color system matches PRD"
       echo "   ☐ Widget system design"
       echo "   ☐ Theme implementation"
       ;;
     "LLM API Clients")
       echo "   ☐ API request/response correctness"
       echo "   ☐ Streaming parser correctness"
       echo "   ☐ Provider trait implementation"
       echo "   ☐ Authentication handling"
       ;;
     "Local Inference")
       echo "   ☐ GGUF parsing correctness"
       echo "   ☐ Tensor operations correctness"
       echo "   ☐ Memory efficiency"
       echo "   ☐ SIMD optimizations"
       ;;
     "Config System")
       echo "   ☐ TOML parsing correctness"
       echo "   ☐ EFI variable handling"
       echo "   ☐ Encryption implementation"
       echo "   ☐ Config validation"
       ;;
     "Integration")
       echo "   ☐ Component integration"
       echo "   ☐ Main loop correctness"
       echo "   ☐ Error propagation"
       echo "   ☐ Shutdown sequence"
       ;;
     *)
       echo "   ☐ General code quality"
       echo "   ☐ Interface correctness"
       ;;
   esac)

4. CODE QUALITY
   ───────────────────────────────────────────────────────────────
   ☐ Code is readable and well-structured
   ☐ Comments explain complex logic
   ☐ No obvious bugs
   ☐ Proper error handling
   ☐ No memory leaks
   ☐ No resource leaks

5. COMPILATION & TESTING
   ───────────────────────────────────────────────────────────────
   ☐ Code compiles without errors
   ☐ No warnings (or justified warnings)
   ☐ Tests pass (if tests exist)
   ☐ Integration points work

═══════════════════════════════════════════════════════════════════

REVIEW DECISION:
   ☐ Approve - Ready to merge
   ☐ Request Changes - Minor issues
   ☐ Reject - Major issues

ISSUES FOUND:
   (List any issues here)

RECOMMENDATIONS:
   (List recommendations here)

═══════════════════════════════════════════════════════════════════

Reviewer: _____________________
Date: _____________________

EOF
