# 🧪 Directive: Prototype Review (SOP-DEV-05)

## 🎯 Primary Objective
Govern the rapid prototyping and validation of new "Sovereign" capabilities. This directive ensures that experimental features are rigorously tested for technical feasibility BEFORE being integrated into the main `server-rs` core.

---

## 🏗️ Prototyping Lifecycle

### 1. The Sandbox Phase
- **Action**: Develop prototype as a standalone Python/Bash script in `execution/scratch/` or a dedicated feature branch.
- **Requirement**: Zero disruption to the production `tadpole.db` or `memory.lance` registries.

### 2. Capability Validation
- **Goal**: Does it work as intended?
- **Metric**: Run the prototype via a "Test Mission" and analyze the success rates. Identify any unexpected token-burn loops.

### 3. Architectural Alignment
- **Check**: Review the prototype against `ARCHITECTURE.md`.
- **Constraint**: Does the new feature follow the Gateway-Runner-Registry pattern? Is it compliant with the `llm_provider` trait standards?

---

## 🛠️ Review SOP

### 1. Technical Peer Review
- **Action**: Use a specialist agent (e.g., "Refactor Specialist") to audit the prototype code for Rust safety and memory efficiency.
- **Deliverable**: `reports/PROTOTYPE_REVIEW_[NAME]_[DATE].md`.

### 2. The "Go/No-Go" Decision
- **Logic**: If the prototype passes all security and performance gates, it is promoted to the `feature_roadmap.md` for full implementation.

---

## 📊 Integration Strategy
All successful prototypes must eventually be refactored into native Rust or standardized MCP tools, using the `mcp-builder` skill guidelines.