#!/bin/bash
# Safe Task Review Helper
#
# Usage: ./tools/review-task.sh "<task-title>"
#
# Helps you safely review a task by:
# 1. Generating checklist
# 2. Providing review prompt
# 3. Showing what to verify

TASK_TITLE="$1"

if [ -z "$TASK_TITLE" ]; then
    echo "Usage: $0 '<task-title>'"
    echo "Example: $0 '[1] Project scaffolding (Cargo workspace structure)'"
    exit 1
fi

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║           SAFE TASK REVIEW HELPER                              ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Task: $TASK_TITLE"
echo ""

# Determine spec section
if [[ "$TASK_TITLE" =~ ^\[1\] ]]; then
    SPEC_SECTION="3.1"
    WORKSTREAM="Boot & Core"
elif [[ "$TASK_TITLE" =~ ^\[2\] ]]; then
    SPEC_SECTION="3.2"
    WORKSTREAM="Network Stack"
elif [[ "$TASK_TITLE" =~ ^\[3\] ]]; then
    SPEC_SECTION="3.3"
    WORKSTREAM="TUI Framework"
elif [[ "$TASK_TITLE" =~ ^\[4\] ]]; then
    SPEC_SECTION="3.4"
    WORKSTREAM="LLM API Clients"
elif [[ "$TASK_TITLE" =~ ^\[6\] ]]; then
    SPEC_SECTION="3.6"
    WORKSTREAM="Local Inference"
elif [[ "$TASK_TITLE" =~ ^\[7\] ]]; then
    SPEC_SECTION="3.7"
    WORKSTREAM="Config System"
elif [[ "$TASK_TITLE" =~ ^\[8\] ]]; then
    SPEC_SECTION="3.8"
    WORKSTREAM="Integration"
else
    SPEC_SECTION="N/A"
    WORKSTREAM="Unknown"
fi

echo "═══════════════════════════════════════════════════════════════════"
echo "STEP 1: Generate Review Checklist"
echo "═══════════════════════════════════════════════════════════════════"
echo ""
echo "Run: ./tools/generate-review-checklist.sh \"$TASK_TITLE\""
echo ""

echo "═══════════════════════════════════════════════════════════════════"
echo "STEP 2: Use This Review Prompt in Cursor"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

cat << EOF
Review the code changes for task "$TASK_TITLE" in the moteOS project.

Task Details:
- Title: $TASK_TITLE
- Workstream: $WORKSTREAM
- Specification: docs/TECHNICAL_SPECIFICATIONS.md Section $SPEC_SECTION

Review Requirements:
1. Check against technical specifications exactly (Section $SPEC_SECTION)
2. Verify interface contracts are correct
3. Check Rust OS development best practices
4. Verify no_std compliance (CRITICAL - no std library usage)
5. Check memory safety
6. Verify error handling

Provide:
- Summary of what the code does
- Specification compliance check (detailed, compare to spec)
- Issues found (if any) with severity (High/Medium/Low)
- Specific recommendations
- Clear decision: APPROVE / REQUEST CHANGES / REJECT
- Detailed reasoning for the decision

Be thorough - this is a critical OS component. If anything doesn't match 
the specification exactly, request changes.

EOF

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "STEP 3: Verify Before Approving"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

cat << EOF
Before approving, verify:

1. Does it compile?
   - Find the worktree/branch in Vibe Kanban
   - cd to that directory
   - Run: cargo check --target x86_64-unknown-uefi

2. Does it match the spec?
   - Open: docs/TECHNICAL_SPECIFICATIONS.md Section $SPEC_SECTION
   - Compare code to specification
   - Check interface contracts

3. Is it no_std?
   - Check for #![no_std] in each file
   - Verify no std:: imports
   - Should use alloc:: if needed

4. AI reviewer decision?
   - Did AI explicitly say APPROVE?
   - Were there any issues found?
   - Is reasoning sound?

ONLY approve if ALL checks pass!

EOF

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "STEP 4: Make Decision"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

cat << EOF
Decision Guide:

✅ APPROVE if:
   - AI says APPROVE
   - Code compiles
   - Matches specification
   - no_std compliant
   - No critical issues

⚠️  REQUEST CHANGES if:
   - Minor issues found
   - Doesn't fully match spec
   - Code quality issues
   - Missing error handling

❌ REJECT if:
   - Fundamentally wrong approach
   - Security issues
   - Breaks existing code
   - Major spec violations

When in doubt → REQUEST CHANGES (safer than approving bad code)

EOF

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "Ready to review!"
echo "═══════════════════════════════════════════════════════════════════"
