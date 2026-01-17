# Code Review Prompt Template

Use this template when asking AI agents to review code for moteOS tasks.

---

## Basic Review Prompt

```
Review the code changes for task "[TASK_NAME]" in the moteOS project.

Task Details:
- Title: [TASK_NAME]
- Workstream: [WORKSTREAM]
- Specification: docs/TECHNICAL_SPECIFICATIONS.md Section [SECTION]

Review Requirements:
1. Check against technical specifications
2. Verify interface contracts are correct
3. Check Rust OS development best practices
4. Verify no_std compliance
5. Check memory safety
6. Verify error handling

Provide:
- Summary of changes
- Specification compliance check
- Issues found (if any)
- Recommendations
- Approve/Request Changes/Reject decision with reasoning
```

---

## Detailed Review Prompt

```
You are a code reviewer for moteOS, an ultra-lightweight unikernel 
operating system written in Rust.

Review the code changes for task: [TASK_NAME]

Context:
- Workstream: [WORKSTREAM]
- Specification: docs/TECHNICAL_SPECIFICATIONS.md Section [SECTION]
- PRD: docs/moteOS-PRD.md

Review Checklist:

1. SPECIFICATION COMPLIANCE
   - Does the code match the technical specification exactly?
   - Are interface contracts correct?
   - Do function signatures match the spec?
   - Are data structures correct?
   - Are error types appropriate?

2. RUST OS DEVELOPMENT
   - Is #![no_std] present and enforced?
   - Is there any standard library usage?
   - Are unsafe blocks minimal and documented?
   - Is memory safety guaranteed?
   - Is error handling proper?
   - Are there any panics in production paths?

3. WORKSTREAM-SPECIFIC CHECKS
   [WORKSTREAM_SPECIFIC_CHECKS]

4. CODE QUALITY
   - Is the code readable and well-structured?
   - Are comments adequate?
   - Are there any obvious bugs?
   - Is error handling comprehensive?
   - Are there memory or resource leaks?

5. COMPILATION
   - Does the code compile?
   - Are there warnings?
   - Do tests pass (if applicable)?

Provide a detailed review with:
- Summary of what the code does
- Compliance assessment
- List of issues (if any) with severity
- Specific recommendations
- Clear decision: Approve / Request Changes / Reject
- Reasoning for the decision
```

---

## Workstream-Specific Review Prompts

### Boot & Core

```
Focus on:
- Memory allocator correctness and safety
- Interrupt handler safety (minimal work, no blocking)
- Framebuffer access safety (bounds checking)
- Boot sequence correctness
- Memory map handling
- Timer setup correctness
```

### Network Stack

```
Focus on:
- Network driver trait implementation correctness
- smoltcp integration (proper polling, state management)
- TLS certificate verification (security)
- HTTP parsing correctness (edge cases)
- Error handling for network failures
- Timeout handling
```

### TUI Framework

```
Focus on:
- Framebuffer rendering safety (bounds checking)
- Color system matches PRD exactly
- Widget system design (extensible, clean)
- Input handling correctness
- Theme implementation (all colors from PRD)
- Text rendering quality
```

### LLM API Clients

```
Focus on:
- API request/response format correctness
- Streaming parser correctness (SSE parsing)
- Error handling (rate limits, auth, network)
- Provider trait implementation
- Authentication handling (secure)
- Model enumeration
```

### Local Inference

```
Focus on:
- GGUF format parsing correctness
- Tensor operations mathematical correctness
- Transformer implementation correctness
- Memory efficiency (no leaks)
- SIMD optimizations correctness
- Numerical stability
```

### Config System

```
Focus on:
- TOML parsing correctness (all cases)
- EFI variable operations correctness
- Encryption implementation security
- Config validation
- Error handling
```

### Integration

```
Focus on:
- Component integration correctness
- Main loop design (non-blocking)
- Error propagation
- Shutdown sequence
- Build system correctness
```

---

## Quick Review Prompt (For Simple Changes)

```
Quick review for task "[TASK_NAME]":
- Does it compile?
- Matches spec?
- Any obvious bugs?
- Approve or request changes?
```

---

## Review Decision Template

```
REVIEW DECISION: [Approve / Request Changes / Reject]

REASONING:
[Explain why]

ISSUES FOUND:
1. [Issue 1] - [Severity: High/Medium/Low]
2. [Issue 2] - [Severity: High/Medium/Low]

RECOMMENDATIONS:
1. [Recommendation 1]
2. [Recommendation 2]

NEXT STEPS:
- [If approved] Ready to merge
- [If changes] Address issues and re-review
- [If rejected] [Explain what needs to happen]
```

---

## Example: Full Review Prompt

```
Review the code for task "[1] Memory allocator setup" in moteOS.

Specification: docs/TECHNICAL_SPECIFICATIONS.md Section 3.1.5

Check:
1. Does it implement linked_list_allocator correctly?
2. Does it initialize heap from memory map properly?
3. Does it register global allocator correctly?
4. Is it no_std compliant?
5. Is memory safety guaranteed?
6. Are there any leaks or issues?

Provide detailed review with approve/request changes decision.
```

---

**Usage**: Copy the appropriate template and fill in [TASK_NAME], [WORKSTREAM], and [SECTION] placeholders.
