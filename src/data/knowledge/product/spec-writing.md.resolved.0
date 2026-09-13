---
name: spec-writing
description: Creating clear, comprehensive Product Requirement Documents (PRDs).
---

# Spec Writing Protocol

A PRD is a contract between Product and Engineering. It must be unambiguous, complete, and feasible.

## Architecture

```mermaid
graph LR
    P[Problem] --> U[User Stories]
    U --> F[Functional Req]
    U --> NF[Non-Functional Req]
    F --> UI[UI/UX Mocks]
    NF --> E[Eng Review]
    UI --> E
    E --> S[Signed Spec]
```

### 1. Problem Statement
Start with *why*. What user pain are we solving? What is the business value?

### 2. User Stories
Format: "As a [Role], I want to [Action], so that [Benefit]."
- *Example*: "As a CEO, I want to see a burn rate chart, so that I don't run out of cash."

### 3. Requirements (The "What")
- **Functional**: Inputs, outputs, logic.
- **Non-Functional**: Speed, security, supported devices.
- **Out of Scope**: Explicitly list what we are NOT building.

## When to Use
- **New Features**: Essential for alignment.
- **Micro-Updates**: Can be a lightweight ticket description using the same principles.

## Operational Principles
1. **No Ambiguity**: Avoid words like "fast", "easy", "better". Use metrics (e.g., "< 200ms").
2. **Living Document**: The spec changes as we learn. Keep it updated.
3. **Visuals > Text**: A mockup is worth 1000 words.
