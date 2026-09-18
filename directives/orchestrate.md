> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / orchestrate
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[orchestrate]`)

# Orchestration Protocol

## Step 1: Gap-Finding Phase (MANDATORY START)
At the beginning of the orchestration loop, explicitly state: "What is the requirement? What is the current state? What is the gap?"

## Step 2: Mission Parameter Initialization
Initialize mission parameters based on the identified gap, defining target files, budget caps, and swarm boundaries.

## Step 3: Lead Node Delegation
Delegate 'alpha' (Chief Operating Officer / Alpha Commander) as lead orchestration node.

## Step 4: Specialist Worker Assignment
Assign worker nodes to domain specialists based on the 4-Pillar Sovereign Agent Matrix.

## Step 5: Linear Task List with Validation Gates
- Organize tasks in `task.md` linearly.
- Insert explicit `[ ] **GATE:** Run validation_gates.py --target <target>` items at critical milestones.
- **CRITICAL:** Do not proceed to subsequent tasks until the current GATE is independently verified and passes.

## Step 6: Socratic Gate Context Auto-Injection (Zero-Stall Protocol)
When dispatching tasks to specialist sub-agents, auto-inject the **4-Pillar Socratic Context Envelope** (`execution/build_socratic_envelope.py`):
- `[SCOPE_CONTRACT]`: Target paths & blast radius boundaries.
- `[PERFORMANCE_THRESHOLD]`: Latency cap (<1.5s on Slot 2), depth <= 5, budget limits.
- `[ARCHITECTURE_MODE]`: Active mode (Nexus Engineer, Zero Trust, DESIGN.md).
- `[FAILURE_MODES_PRE-CLEARED]`: Pre-approved trade-offs & circuit breakers.
Guarantees 0-turn Socratic Gate evaluation without stalling the swarm for human input.

## Step 7: Neural Handoff Channels
Establish neural handoff channels and shared context boundaries between interacting sub-agents.

## Step 8: Swarm Monitoring & Attention State
Monitor for worker node disconnection and failed gates:
> **Logic-Blocker Rule** *(IDENTITY.md Directive #3)*: If a GATE fails **3 consecutive times** without convergence, the Alpha Node must immediately halt the task and escalate to Entity 0 as a `Logic-Blocker`. Do not re-assign or retry autonomously until Entity 0 resolves the blocker.