# Safe Code Review Process for moteOS

## Current Status

You have **5 tasks in review**. This guide walks you through reviewing them safely, one by one.

---

## Review Priority Order

**Review in this order** (most critical first):

1. **[1] Project scaffolding** ⚠️ CRITICAL - Blocks everything
2. **[8] Cargo workspace setup** - Foundation
3. **[4] Common types and LLM provider trait** - Foundation for LLM clients
4. **[6] GGUF file format parser** - Foundation for inference
5. **[7] TOML parser (no_std)** - Foundation for config

---

## Step-by-Step Safe Review Process

### Step 1: Get the Code

For each task in review:

1. **Open Vibe Kanban** and click on the task
2. **Find the PR or branch** - Vibe Kanban will show the branch/worktree
3. **Open in Cursor** - Click to open the worktree or PR

### Step 2: Generate Review Checklist

```bash
./tools/generate-review-checklist.sh "[1] Project scaffolding (Cargo workspace structure)"
```

This gives you a focused checklist for that specific task.

### Step 3: Use AI Reviewer Agent

**In Cursor chat**, use this prompt (customize for each task):

```
Review the code changes for task "[TASK_NAME]" in the moteOS project.

Task Details:
- Title: [TASK_NAME]
- Specification: docs/TECHNICAL_SPECIFICATIONS.md Section [SECTION]

Review Requirements:
1. Check against technical specifications exactly
2. Verify interface contracts are correct
3. Check Rust OS development best practices
4. Verify no_std compliance
5. Check memory safety
6. Verify error handling

Provide:
- Summary of what the code does
- Specification compliance check (detailed)
- Issues found (if any) with severity
- Specific recommendations
- Clear decision: APPROVE / REQUEST CHANGES / REJECT
- Reasoning for the decision

Be thorough - this is a critical OS component.
```

### Step 4: Verify Key Points

**Before approving**, manually verify:

1. **Does it compile?**
   ```bash
   cd [worktree-path]
   cargo check --target x86_64-unknown-uefi
   ```

2. **Does it match the spec?**
   - Compare code to `docs/TECHNICAL_SPECIFICATIONS.md`
   - Check interface contracts
   - Verify function signatures

3. **Is it no_std?**
   - Check for `#![no_std]`
   - No `std::` imports
   - Uses `alloc::` if needed

### Step 5: Make Decision

**Only approve if**:
- ✅ AI reviewer says APPROVE
- ✅ Code compiles
- ✅ Matches specification
- ✅ No critical issues

**Request changes if**:
- ⚠️ Minor issues found
- ⚠️ Doesn't fully match spec
- ⚠️ Code quality issues

**Reject if**:
- ❌ Fundamentally wrong
- ❌ Security issues
- ❌ Breaks existing code

### Step 6: Update Task Status

**In Vibe Kanban**:
- If approved → Merge PR → Task moves to "Done"
- If changes needed → Add comments → Task back to "In Progress"
- If rejected → Reject PR → Create new task

---

## Task-Specific Review Prompts

### Task 1: [1] Project scaffolding (Cargo workspace structure)

**CRITICAL - Review this first!**

**Review Prompt**:
```
Review the code for task "[1] Project scaffolding (Cargo workspace structure)".

Specification: docs/TECHNICAL_SPECIFICATIONS.md Section 3.1

Check:
1. Is the workspace Cargo.toml structure correct?
2. Are all crates configured for no_std?
3. Does the crate structure match the spec (boot/, shared/, etc.)?
4. Are dependencies correctly specified?
5. Is the project structure as defined in the spec?

This is CRITICAL - it blocks all other work. Be very thorough.

Provide detailed review with approve/request changes decision.
```

**What to verify**:
- [ ] Root `Cargo.toml` has workspace members
- [ ] Each crate has `#![no_std]`
- [ ] Crate structure matches spec
- [ ] Dependencies are correct
- [ ] Compiles successfully

---

### Task 2: [8] Cargo workspace setup

**Review Prompt**:
```
Review the code for task "[8] Cargo workspace setup".

Specification: docs/TECHNICAL_SPECIFICATIONS.md Section 3.8.1

Check:
1. Is the workspace configuration correct?
2. Are shared dependencies properly configured?
3. Does it match the project structure from Task 1?
4. Are build targets configured?

Provide detailed review with approve/request changes decision.
```

**What to verify**:
- [ ] Workspace members are correct
- [ ] Shared dependencies are configured
- [ ] Build targets are set up
- [ ] Compiles successfully

---

### Task 3: [4] Common types and LLM provider trait

**Review Prompt**:
```
Review the code for task "[4] Common types and LLM provider trait".

Specification: docs/TECHNICAL_SPECIFICATIONS.md Section 3.4.2-3

Check:
1. Are Message, Role, GenerationConfig structs correct?
2. Does LlmProvider trait match the spec exactly?
3. Are error types appropriate?
4. Is the interface contract correct?
5. Is it no_std compliant?

This defines the interface for all LLM clients - must be correct.

Provide detailed review with approve/request changes decision.
```

**What to verify**:
- [ ] Message struct matches spec
- [ ] Role enum is correct
- [ ] LlmProvider trait signature matches spec
- [ ] Error types are defined
- [ ] no_std compliant
- [ ] Compiles successfully

---

### Task 4: [6] GGUF file format parser

**Review Prompt**:
```
Review the code for task "[6] GGUF file format parser".

Specification: docs/TECHNICAL_SPECIFICATIONS.md Section 3.6.2

Check:
1. Does it parse GGUF header correctly?
2. Does it extract metadata properly?
3. Does it load tensor data correctly?
4. Is the file format parsing correct?
5. Are error cases handled?
6. Is it no_std compliant?

This is complex binary format parsing - verify correctness carefully.

Provide detailed review with approve/request changes decision.
```

**What to verify**:
- [ ] GGUF header parsing is correct
- [ ] Metadata extraction works
- [ ] Tensor loading is correct
- [ ] Error handling is comprehensive
- [ ] no_std compliant
- [ ] Compiles successfully

---

### Task 5: [7] TOML parser (no_std)

**Review Prompt**:
```
Review the code for task "[7] TOML parser (no_std)".

Specification: docs/TECHNICAL_SPECIFICATIONS.md Section 3.7.3

Check:
1. Does it parse TOML correctly (key-value, nested tables, arrays)?
2. Is it truly no_std (no standard library)?
3. Does it handle all required TOML features?
4. Are error cases handled?
5. Is it minimal as specified?

This must be no_std - verify no std library usage.

Provide detailed review with approve/request changes decision.
```

**What to verify**:
- [ ] Parses key-value pairs
- [ ] Handles nested tables
- [ ] Handles arrays
- [ ] No std library usage
- [ ] Error handling
- [ ] Compiles successfully

---

## Safety Checklist Before Approving ANY Task

**Before clicking "Approve" or merging, verify**:

- [ ] AI reviewer explicitly says "APPROVE"
- [ ] Code compiles: `cargo check` succeeds
- [ ] Matches specification: Compare to `docs/TECHNICAL_SPECIFICATIONS.md`
- [ ] no_std compliant: No `std::` imports, has `#![no_std]`
- [ ] Interface contracts correct: Function signatures match spec
- [ ] No critical issues: AI reviewer found no high-severity issues
- [ ] Dependencies satisfied: If this task depends on others, those are done

**If ANY of these fail → Request Changes or Reject**

---

## Review Workflow Diagram

```
Task in Review
    ↓
Generate Checklist
    ↓
Use AI Reviewer Agent
    ↓
AI Provides Review
    ↓
    ├─→ APPROVE → Verify Checklist → Merge PR → Done ✅
    │
    ├─→ REQUEST CHANGES → Add Comments → Back to In Progress
    │
    └─→ REJECT → Reject PR → Create New Task
```

---

## What to Do Right Now

### Immediate Actions

1. **Start with Task 1** (Project scaffolding) - It's critical
2. **Open the task in Vibe Kanban** - Find the PR/branch
3. **Open in Cursor** - View the code
4. **Run the review prompt** - Use the prompt above for Task 1
5. **Generate checklist** - `./tools/generate-review-checklist.sh "[1] Project scaffolding"`
6. **Verify compilation** - Check if it compiles
7. **Make decision** - Based on AI review + your verification

### For Each Task

Repeat the process for all 5 tasks, in priority order.

---

## Common Red Flags

**Reject immediately if you see**:
- ❌ `use std::` anywhere (not no_std)
- ❌ Missing `#![no_std]`
- ❌ Function signatures don't match spec
- ❌ Security vulnerabilities
- ❌ Breaking changes to shared interfaces

**Request changes if you see**:
- ⚠️ Minor spec mismatches
- ⚠️ Missing error handling
- ⚠️ Code quality issues
- ⚠️ Missing comments
- ⚠️ Warnings that should be fixed

---

## Getting Help

**If unsure about a review decision**:
1. Ask the AI reviewer to explain their reasoning
2. Compare more carefully to the specification
3. Check if code compiles
4. When in doubt → Request Changes (safer than approving bad code)

**Remember**: It's safer to request changes than to approve bad code. You can always approve later after fixes.

---

## Next Steps After Review

**After reviewing all 5 tasks**:

1. **If Task 1 (scaffolding) is approved**:
   - Merge it immediately
   - Start dependent tasks (Memory allocator, UEFI boot, etc.)

2. **If any task needs changes**:
   - Add specific comments
   - Move task back to "In Progress"
   - Development agent fixes
   - Re-review

3. **If any task is rejected**:
   - Understand why
   - Create new task with correct approach
   - Start over

---

**Last Updated**: January 2026
