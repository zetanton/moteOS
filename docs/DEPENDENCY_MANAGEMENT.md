# Dependency Management & Agent Coordination Guide

## Overview

This document explains how to prevent agents from stepping on each other and ensure tasks are started only when their dependencies are met.

---

## How Vibe Kanban Prevents Conflicts

### 1. Isolated Git Worktrees

**Each task runs in its own worktree** - agents cannot interfere with each other's work:
- Task A creates branch `task-a` in worktree `/worktrees/task-a/`
- Task B creates branch `task-b` in worktree `/worktrees/task-b/`
- Changes are isolated until merged to main

**This means**: Agents can work in parallel on different tasks without conflicts.

### 2. Tasks Don't Auto-Start

**Tasks are created in "To Do" status** - they won't start until you explicitly:
- Call `start_task_attempt` via MCP
- Click "Start" in Vibe Kanban UI
- Use "Create & Start" (which we didn't use)

**This means**: Tasks won't begin work until you're ready.

### 3. Manual Review Gates

Tasks move through: **To Do → In Progress → In Review → Done**

- **In Review** stage requires manual approval before merging
- You can review code, check dependencies, and approve/reject
- No auto-merging without review

---

## Dependency Map

### Critical Path (Must Complete in Order)

```
[1] Project scaffolding
    ↓
[1] Memory allocator setup
    ↓
[2] virtio-net driver (needs memory allocator)
    ↓
[2] smoltcp integration
    ↓
[2] DHCP client
    ↓
[2] DNS resolver
    ↓
[2] TLS 1.3 support
    ↓
[2] HTTP/1.1 client
    ↓
[4] LLM clients (need HTTP client)
    ↓
[8] Component integration (needs LLM clients)
```

### Parallel Workstreams (Can Work Simultaneously)

These can run in parallel **after** their dependencies are met:

**Workstream 1 (Boot & Core)**:
- All tasks can run in parallel after "Project scaffolding"
- No dependencies between them

**Workstream 3 (TUI)**:
- Can start after "[1] Framebuffer interface" is done
- TUI tasks can run in parallel with each other

**Workstream 6 (Inference)**:
- Can start immediately (no dependencies)
- All inference tasks can run in parallel

**Workstream 7 (Config)**:
- Can start immediately (no dependencies)
- Config tasks can run in parallel

**Workstream 8 (Integration)**:
- "[8] Cargo workspace setup" can start immediately
- Other integration tasks need their dependencies

---

## Dependency Rules by Workstream

### Workstream 1: Boot & Core

| Task | Can Start After | Blocks |
|------|----------------|--------|
| Project scaffolding | Nothing | Everything |
| Memory allocator | Project scaffolding | Network stack |
| UEFI boot | Project scaffolding | Nothing |
| Interrupt setup | Project scaffolding | Nothing |
| Framebuffer interface | Project scaffolding | TUI |
| Timer setup | Project scaffolding | Nothing |

**Rule**: Start "Project scaffolding" first. Others can start after it's done.

### Workstream 2: Network Stack

| Task | Can Start After | Blocks |
|------|----------------|--------|
| virtio-net driver | [1] Memory allocator | Everything else |
| smoltcp integration | [2] virtio-net driver | DHCP, DNS, TLS, HTTP |
| DHCP client | [2] smoltcp integration | DNS |
| DNS resolver | [2] DHCP client | TLS, HTTP |
| TLS 1.3 support | [2] DNS resolver | HTTP |
| HTTP/1.1 client | [2] TLS 1.3 support | LLM clients |

**Rule**: Must follow the chain. Each task blocks the next.

### Workstream 3: TUI Framework

| Task | Can Start After | Blocks |
|------|----------------|--------|
| Framebuffer rendering | [1] Framebuffer interface | Font, themes, widgets |
| Font system | [3] Framebuffer rendering | Nothing |
| Dark theme | [3] Framebuffer rendering | Nothing |
| Input widget | [3] Base widget system | Chat screen |
| Chat screen | [3] Message widget | Integration |

**Rule**: Start after "[1] Framebuffer interface" is done. Most can run in parallel.

### Workstream 4: LLM API Clients

| Task | Can Start After | Blocks |
|------|----------------|--------|
| Common types | Nothing | All LLM clients |
| OpenAI client | [2] HTTP/1.1 client | Integration |
| Anthropic client | [2] HTTP/1.1 client | Integration |
| Groq client | [2] HTTP/1.1 client | Integration |
| xAI client | [2] HTTP/1.1 client | Integration |

**Rule**: All LLM clients can start in parallel after HTTP client is ready.

### Workstream 6: Local Inference

| Task | Can Start After | Blocks |
|------|----------------|--------|
| GGUF parser | Nothing | Tokenizer, transformer |
| BPE tokenizer | [6] GGUF parser | Transformer |
| Transformer forward | [6] Tensor operations | Generation loop |
| Generation loop | [6] Transformer forward | Integration |

**Rule**: Can start immediately. Follow the chain within workstream.

### Workstream 7: Config System

| Task | Can Start After | Blocks |
|------|----------------|--------|
| TOML parser | Nothing | EFI storage, wizard |
| EFI storage | [7] TOML parser | Wizard |
| Setup wizard | [7] EFI storage | Integration |

**Rule**: Can start immediately. Follow the chain within workstream.

### Workstream 8: Integration

| Task | Can Start After | Blocks |
|------|----------------|--------|
| Cargo workspace | Nothing | Everything |
| Kernel entry point | [1] Memory allocator | Component integration |
| Component integration | [4] OpenAI client | ISO generation |
| ISO generation | [8] Component integration | Nothing |

**Rule**: Start "Cargo workspace" first. Others need their dependencies.

---

## How to Enforce Dependencies

### Method 1: Check Before Starting

**Before starting any task**, check:

1. **Are dependencies done?**
   - Look at the task description - it lists dependencies
   - Check Vibe Kanban - are dependency tasks in "Done" status?
   - Verify code is merged to main branch

2. **Is the base branch ready?**
   - When starting a task, set `base_branch` to `main`
   - Ensure dependencies are merged to `main` first

3. **Use tags to mark status**:
   - `blocked` - Task waiting for dependencies
   - `ready` - Dependencies met, can start
   - `in-progress` - Currently being worked on

### Method 2: Update Task Descriptions

Each task description includes:
- **Prerequisites**: What must be done first
- **Dependencies**: Specific tasks that must be complete
- **Blocks**: What this task blocks

**Example**:
```
Prerequisites:
- [1] Memory allocator setup must be complete
- Code merged to main branch

Dependencies:
- Task: "[1] Memory allocator setup"
- Status: Must be "Done" before starting

Blocks:
- All Network Stack tasks
```

### Method 3: Use Task Status

**Don't start tasks that are blocked**:
- If a task depends on another, wait until dependency is "Done"
- Check the dependency chain before starting
- Use Vibe Kanban filters to see what's ready

### Method 4: Manual Coordination

**For critical path tasks**:
- Review before starting
- Ensure dependencies are merged
- Test that dependencies work before proceeding

---

## Starting Tasks Safely

### Checklist Before Starting a Task

- [ ] Read the task description
- [ ] Check prerequisites listed
- [ ] Verify dependency tasks are "Done" in Vibe Kanban
- [ ] Confirm dependency code is merged to main
- [ ] Check that no other agent is working on conflicting code
- [ ] Verify base branch is set correctly
- [ ] Review interface contracts (if task depends on interfaces)

### Starting a Task

**Via MCP**:
```json
{
  "tool": "start_task_attempt",
  "parameters": {
    "task_id": "task-xyz",
    "executor": "claude-code",
    "base_branch": "main"  // Ensure dependencies are here
  }
}
```

**In Vibe Kanban UI**:
1. Click on task
2. Click "Start" button
3. Select agent/executor
4. Verify base branch is `main`

---

## Preventing Conflicts

### 1. Clear Module Ownership

Each workstream owns its module:
- **Workstream 1**: `boot/` crate
- **Workstream 2**: `network/` crate
- **Workstream 3**: `tui/` crate
- **Workstream 4**: `llm/` crate
- **Workstream 6**: `inference/` crate
- **Workstream 7**: `config/` crate
- **Workstream 8**: `kernel/` crate and integration

**Rule**: Agents should only modify their own workstream's code.

### 2. Shared Interfaces

**`shared/` crate** contains:
- Common types
- Error types
- Interface definitions

**Rule**: Changes to `shared/` require coordination. Update technical specs first.

### 3. Integration Points

**Integration agent (Workstream 8)** is the only one that:
- Wires components together
- Modifies `kernel/src/main.rs`
- Creates build scripts

**Rule**: Other agents don't touch integration code.

### 4. Review Before Merge

**Always review before merging**:
- Check that code follows interface contracts
- Verify no conflicts with other workstreams
- Ensure tests pass (when available)
- Confirm dependencies are satisfied

---

## Monitoring & Alerts

### What to Watch For

1. **Agents starting blocked tasks**
   - Check task status before starting
   - Verify dependencies are done

2. **Conflicts in shared code**
   - Monitor `shared/` crate changes
   - Coordinate interface changes

3. **Breaking interface contracts**
   - Review changes to public APIs
   - Update technical specs if needed

4. **Parallel work on same module**
   - Each workstream owns its module
   - Don't duplicate work

### Daily Check

1. Review tasks in "In Progress"
2. Check for blocked tasks
3. Verify dependencies are met
4. Review PRs before merging
5. Update task status/tags

---

## Emergency Procedures

### If Agents Conflict

1. **Stop both agents** immediately
2. **Review the conflict**:
   - What code is conflicting?
   - Which workstreams are involved?
   - What was the intended change?
3. **Resolve manually**:
   - Merge one change first
   - Update the other to work with merged code
   - Coordinate the fix
4. **Update dependencies** if needed
5. **Restart tasks** after resolution

### If Dependencies Are Missing

1. **Identify the missing dependency**
2. **Check if it's actually needed**:
   - Review technical specs
   - Check interface contracts
3. **Either**:
   - Wait for dependency to complete
   - Adjust task to not need dependency
   - Create a minimal stub interface

---

## Best Practices Summary

1. ✅ **Always check dependencies** before starting
2. ✅ **Use isolated worktrees** (automatic in Vibe Kanban)
3. ✅ **Review before merging** (manual gate)
4. ✅ **Respect module ownership** (each workstream owns its code)
5. ✅ **Update technical specs** if interfaces change
6. ✅ **Coordinate shared changes** (especially `shared/` crate)
7. ✅ **Test dependencies** before starting dependent tasks
8. ✅ **Use tags** to mark blocked/ready status
9. ✅ **Monitor progress** regularly
10. ✅ **Communicate** when dependencies are complete

---

## Quick Reference: Safe to Start?

| Task | Safe to Start? | Why |
|------|----------------|-----|
| [1] Project scaffolding | ✅ Yes | No dependencies |
| [1] Memory allocator | ⚠️ After scaffolding | Needs workspace |
| [2] virtio-net | ⚠️ After memory allocator | Needs heap |
| [2] HTTP client | ⚠️ After TLS | Needs TLS first |
| [4] LLM clients | ⚠️ After HTTP client | Needs HTTP |
| [3] TUI tasks | ⚠️ After framebuffer | Needs framebuffer |
| [6] Inference | ✅ Yes | No dependencies |
| [7] Config | ✅ Yes | No dependencies |
| [8] Integration | ⚠️ After components | Needs all components |

---

**Last Updated**: January 2026
