# Code Review Guide for moteOS

## Overview

Since you're not an OS developer yourself, this guide explains how to use AI code reviewer agents to review the work done by development agents. This ensures code quality, correctness, and adherence to specifications.

---

## Code Review Workflow

### 1. Development Agent Completes Task

When a development agent finishes a task:
- Code is in isolated worktree/branch
- Task moves to "In Review" status in Vibe Kanban
- PR is created (or ready to be created)

### 2. Code Reviewer Agent Reviews

**Reviewer agent should:**
- Read the code changes
- Check against technical specifications
- Verify interface contracts
- Test compilation (if possible)
- Check for common OS development pitfalls
- Verify no_std compliance
- Check memory safety
- Review error handling

### 3. Review Decision

**Approve** if:
- Code matches specifications
- Interfaces are correct
- No obvious bugs
- Follows Rust best practices
- no_std compliant

**Request Changes** if:
- Doesn't match specifications
- Interface contract violated
- Bugs or issues found
- Missing error handling
- Not no_std compliant

**Reject** if:
- Fundamentally wrong approach
- Security issues
- Breaks existing functionality

---

## Code Review Criteria

### 1. Specification Compliance

**Checklist**:
- [ ] Code matches technical specifications
- [ ] Interface contracts are correct
- [ ] Function signatures match spec
- [ ] Data structures match spec
- [ ] Error types match spec

**Reviewer should reference**:
- `docs/TECHNICAL_SPECIFICATIONS.md` (relevant section)
- `docs/moteOS-PRD.md` (requirements)
- Interface contracts in specs

### 2. Rust OS Development Best Practices

**Checklist**:
- [ ] `#![no_std]` present and enforced
- [ ] No standard library usage
- [ ] Proper use of `alloc` crate if needed
- [ ] Memory safety (no undefined behavior)
- [ ] Proper use of `unsafe` blocks (minimal, documented)
- [ ] No panics in production paths
- [ ] Proper error handling

### 3. Interface Contracts

**Checklist**:
- [ ] Public API matches specification exactly
- [ ] Trait implementations are correct
- [ ] Function signatures match spec
- [ ] Return types match spec
- [ ] Error types match spec
- [ ] No breaking changes to shared interfaces

### 4. Code Quality

**Checklist**:
- [ ] Code is readable and well-structured
- [ ] Comments explain complex logic
- [ ] No obvious bugs
- [ ] Proper error handling
- [ ] No memory leaks
- [ ] No resource leaks

### 5. OS-Specific Concerns

**Checklist**:
- [ ] Interrupt safety (if applicable)
- [ ] Memory management correct
- [ ] No blocking operations in interrupt handlers
- [ ] Proper synchronization (if needed)
- [ ] Hardware abstraction correct
- [ ] Boot-time constraints considered

### 6. Testing & Compilation

**Checklist**:
- [ ] Code compiles without errors
- [ ] No warnings (or justified warnings)
- [ ] Tests pass (if tests exist)
- [ ] Integration points work

---

## Setting Up Code Reviewer Agents

### Option 1: Manual Review Process

**Workflow**:
1. Development agent completes task → "In Review"
2. You assign a reviewer agent to review
3. Reviewer agent reviews code
4. You approve or request changes
5. If approved → merge to main
6. If changes needed → back to development agent

### Option 2: Automated Review Tasks

Create review tasks that automatically trigger when development tasks complete.

### Option 3: Review Checklist Template

Use a standardized checklist for each review.

---

## Review Commands for AI Agents

### For Cursor/Claude Code Review

**Prompt template**:
```
Review the code changes for task "[Task Name]" in the moteOS project.

Check against:
1. Technical specifications in docs/TECHNICAL_SPECIFICATIONS.md Section X.X
2. Interface contracts defined in the spec
3. Rust OS development best practices
4. no_std compliance
5. Memory safety

Focus on:
- Does the code match the specification?
- Are interfaces correct?
- Any bugs or issues?
- Proper error handling?
- OS development best practices?

Provide:
- Summary of changes
- Compliance check
- Issues found (if any)
- Recommendations
- Approve/Request Changes decision
```

### Review Checklist Prompt

```
For the code in [file/path], verify:

1. Specification Compliance:
   - Matches docs/TECHNICAL_SPECIFICATIONS.md Section X.X
   - Interface contract correct
   - Function signatures match spec

2. Rust OS Development:
   - #![no_std] present
   - No std library usage
   - Proper unsafe usage
   - Memory safety

3. Code Quality:
   - Readable and structured
   - Proper error handling
   - No obvious bugs

4. OS Concerns:
   - Interrupt safety
   - Memory management
   - Boot-time constraints

Provide detailed review with approve/request changes decision.
```

---

## Review Process by Workstream

### Workstream 1: Boot & Core

**Critical Review Points**:
- Memory allocator correctness
- Interrupt handler safety
- Framebuffer access safety
- Boot sequence correctness
- Memory map handling

**Reviewer should check**:
- No memory leaks in allocator
- Interrupt handlers are minimal
- Framebuffer operations are safe
- Boot info structure matches spec

### Workstream 2: Network Stack

**Critical Review Points**:
- Network driver trait implementation
- smoltcp integration correctness
- TLS certificate verification
- HTTP parsing correctness
- Error handling for network failures

**Reviewer should check**:
- Driver implements trait correctly
- Network stack polling works
- TLS is secure (no bypass)
- HTTP parsing is correct
- Proper timeout handling

### Workstream 3: TUI Framework

**Critical Review Points**:
- Framebuffer rendering safety
- Color system correctness
- Widget system design
- Input handling
- Theme implementation

**Reviewer should check**:
- Framebuffer bounds checking
- Colors match PRD specification
- Widget system is extensible
- Input handling is correct
- Theme colors are exact

### Workstream 4: LLM API Clients

**Critical Review Points**:
- API request/response correctness
- Streaming parser correctness
- Error handling
- Provider trait implementation
- Authentication handling

**Reviewer should check**:
- API calls match provider docs
- SSE parsing is correct
- Error types are appropriate
- Trait implementation is correct
- API keys are handled securely

### Workstream 6: Local Inference

**Critical Review Points**:
- GGUF parsing correctness
- Tensor operations correctness
- Transformer implementation
- Memory efficiency
- SIMD optimizations

**Reviewer should check**:
- GGUF format parsing is correct
- Math operations are correct
- Memory usage is reasonable
- Optimizations are correct
- No numerical instabilities

### Workstream 7: Config System

**Critical Review Points**:
- TOML parsing correctness
- EFI variable handling
- Encryption implementation
- Config validation

**Reviewer should check**:
- TOML parser handles all cases
- EFI operations are correct
- Encryption is secure
- Config validation works

### Workstream 8: Integration

**Critical Review Points**:
- Component integration
- Main loop correctness
- Error propagation
- Shutdown sequence

**Reviewer should check**:
- All components wired correctly
- Main loop doesn't block
- Errors are handled
- Shutdown is clean

---

## Review Tools & Scripts

### Review Checklist Generator

Create a script that generates a review checklist for a specific task:

```bash
./tools/generate-review-checklist.sh "[1] Memory allocator setup"
```

### Compilation Checker

Check if code compiles:

```bash
cd worktree/[task-name]
cargo check --target x86_64-unknown-uefi
```

### Specification Compliance Checker

Compare code against specifications (manual or automated).

---

## Using AI Agents for Review

### In Cursor

**Ask Cursor to review**:
```
Review the code changes for task "[Task Name]". 
Check against technical specifications and provide 
approve/request changes decision.
```

### In Vibe Kanban

**Create review tasks**:
- When a development task moves to "In Review"
- Create a corresponding review task
- Assign to reviewer agent
- Reviewer agent reviews and provides decision

### Review Agent Prompt

```
You are a code reviewer for moteOS, an ultra-lightweight 
unikernel OS written in Rust.

Review the code changes for [task name].

Requirements:
1. Check against docs/TECHNICAL_SPECIFICATIONS.md
2. Verify interface contracts
3. Check Rust OS development best practices
4. Verify no_std compliance
5. Check memory safety

Provide:
- Detailed review
- List of issues (if any)
- Approve or Request Changes decision
- Specific feedback for improvements
```

---

## Review Decision Workflow

### Approve

**When to approve**:
- Code matches specifications
- Interfaces are correct
- No bugs found
- Follows best practices
- Ready to merge

**Action**: Merge PR to main branch

### Request Changes

**When to request changes**:
- Minor issues found
- Specification mismatch (fixable)
- Code quality issues
- Missing error handling
- Documentation needed

**Action**: 
- Add comments to PR
- Move task back to "In Progress"
- Development agent fixes issues
- Re-review

### Reject

**When to reject**:
- Fundamentally wrong approach
- Security vulnerabilities
- Breaks existing functionality
- Major specification violations

**Action**:
- Reject PR
- Create new task with correct approach
- Start over

---

## Review Automation

### Automated Checks

**Before human/AI review**:
1. Compilation check
2. Linting (if available)
3. Format check
4. Basic specification compliance

**Script example**:
```bash
#!/bin/bash
# auto-review.sh

TASK_NAME="$1"
WORKTREE_PATH="worktrees/$TASK_NAME"

cd "$WORKTREE_PATH"

# Compilation check
cargo check --target x86_64-unknown-uefi || exit 1

# Format check
cargo fmt --check || exit 1

# Basic checks
echo "✅ Compilation: OK"
echo "✅ Format: OK"
echo "Ready for detailed review"
```

### Review Templates

Create review templates for each workstream type to ensure consistency.

---

## Common Issues to Watch For

### 1. Standard Library Usage

**Problem**: Using `std` in `no_std` code

**Example**:
```rust
// ❌ BAD
use std::vec::Vec;

// ✅ GOOD
use alloc::vec::Vec;
```

### 2. Panic in Production

**Problem**: Unwrapping without error handling

**Example**:
```rust
// ❌ BAD
let value = option.unwrap();

// ✅ GOOD
let value = option.ok_or(Error::MissingValue)?;
```

### 3. Unsafe Without Documentation

**Problem**: Unsafe blocks without explanation

**Example**:
```rust
// ❌ BAD
unsafe { *ptr = value; }

// ✅ GOOD
// SAFETY: ptr is valid and aligned (checked at boot)
unsafe { *ptr = value; }
```

### 4. Interface Mismatch

**Problem**: Function signature doesn't match spec

**Example**:
```rust
// Spec says: fn new() -> Result<Self, Error>
// Code has: fn new() -> Self  // ❌ Wrong
```

### 5. Memory Leaks

**Problem**: Allocating without freeing

**Reviewer should check**: All allocations have corresponding frees

---

## Review Metrics

Track review quality:
- Time to review
- Issues found per review
- Approval rate
- Re-review rate

---

## Next Steps

1. **Set up review process**: Choose manual or automated
2. **Create review tasks**: For each development task
3. **Train reviewer agents**: Use review prompts
4. **Establish review criteria**: Use checklists
5. **Monitor review quality**: Track metrics

---

**Last Updated**: January 2026
