# Resource Allocation & Budget Governance

This protocol defines the standard operating procedure for agent-led resource allocation and fiscal management within the Tadpole OS swarm.

## 1. Primary Directives
- **Efficiency First**: Optimize token usage across all clusters to maximize mission momentum.
- **Fiscal Control**: Adhere strictly to the `budget_limit` defined in the agent's neural configuration.
- **Sector Priority**: Allocate resources based on mission criticality (Critical > Strategic > Operational).

## 2. Allocation Lifecycle
1. **Request Phase**: Agent identifies a tool/skill requirement and estimates token consumption.
2. **Audit Phase**: Governance hooks verify current `cost_usd` against the `budget_limit`.
3. **Execution Phase**: If approved, the agent initiates the task with a designated model slot.
4. **Recording Phase**: Consumption results are committed to the transaction ledger and vector memory.

## 3. Throttling & Governance
- **Breach Alert**: If `cost_usd` exceeds 90% of the budget, initiate a `Neural Link Warning`.
- **Hard Stop**: If budget is exhausted, the agent enters `breached` status and halts autonomous tool calls.
- **Oversight Gate**: High-cost operations (USD > $0.50) require manual user authorization via the Neural Link hub.

*This workflow is monitored by the Continuity Scheduler and the Hive Mind Telemetry stream.*
