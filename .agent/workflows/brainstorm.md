---
description: Structured brainstorming for projects and features. Explores multiple divergent options and evaluates them against Sovereign benchmarks before implementation.
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Sovereign Workflows / brainstorm
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[brainstorm]`)
>
> ### 🤖 AI Persona Directive
> When `/brainstorm` is active, shift from "Executor" mode to "Architect" mode. Prioritize divergent thinking, critical skepticism, and structural integrity over speed of delivery.

# /brainstorm - Structured Idea Exploration

**Usage**: `/brainstorm [problem_vector/feature_request]`

---

## 🎯 Purpose
This command activates **BRAINSTORM** mode. It is designed to prevent "First-Idea Bias" by forcing the exploration of multiple viable paths before any code is written or directives are modified.

> 💡 **Bound Skills**:
> - Use `@[skills/to-spec]` to publish final brainstorm conclusions to `docs/specs/<feature>.md`.
> - Use `@[skills/prototype]` to build executable spikes under `.tmp/prototypes/` when testing UI/logic behavior.

---

## ⚙️ Cognitive Behavior

When `/brainstorm` is triggered, the AI must follow this three-stage reasoning pipeline:

### 1. Context Initialization
- Define the **Problem Vector**: What is the actual friction point?
- Define **Mission Constraints**: What are the non-negotiables (e.g., memory limits, security P0s, API compatibility)?
- Align with the **Sovereign Objective**: How does this increase system autonomy or reliability?

### 2. Aletheia Reasoning (Divergent Generation)
Apply the *Aletheia* process: "Unveil" the truth by stripping away assumed constraints.
- **Generate 3+ Divergent Strategies**: 
    - *Option A (Conservative):* The safest, most incremental path.
    - *Option B (Aggressive):* The most performant or scalable path.
    - *Option C (Radical):* The "clean slate" approach that challenges current architecture.
- **Filter**: Cross-reference each option against `@docs ARCHITECTURE:Core` to ensure no fundamental violations.

### 3. Tradeoff Synthesis
Analyze the options using a deterministic matrix:
- **Fidelity**: How closely does this solve the core problem?
- **Complexity**: How much "cognitive load" does this add to the codebase?
- **Sovereignty**: Does this increase the system's independence or create new dependencies?
- **Graph Check**: (For technical paths) Use `npm run graph:blast` to estimate the actual implementation effort.

---

## 📊 Output Format

```markdown
## 🧠 Brainstorm: [Topic]

### 📍 Context
**Problem Vector**: [Statement]
**Constraints**: [Constraint 1, Constraint 2]

---

### 🗺️ Option Analysis

#### Option A: [Name]
> [High-level description]

- ✅ **Pros**: [Benefit 1], [Benefit 2]
- ❌ **Cons**: [Drawback 1], [Drawback 2]
- ⚙️ **Complexity**: Low | Medium | High
- 🛡️ **Sovereignty**: [Impact on system autonomy]

---

#### Option B: [Name]
> [High-level description]

- ✅ **Pros**: [Benefit 1]
- ❌ **Cons**: [Drawback 1]
- ⚙️ **Complexity**: Low | Medium | High
- 🛡️ **Sovereignty**: [Impact on system autonomy]

---

#### Option C: [Name]
> [High-level description]

- ✅ **Pros**: [Benefit 1]
- ❌ **Cons**: [Drawback 1]
- ⚙️ **Complexity**: Low | Medium | High
- 🛡️ **Sovereignty**: [Impact on system autonomy]

---

## 💡 Sovereign Recommendation

**Recommended Path**: **Option [X]**
**Justification**: [Provide a logical proof why this option optimizes the balance between complexity and sovereignty.]

**Next Step**: Would you like to move this into a `/design` spec or initiate a `/graph` blast-radius check?