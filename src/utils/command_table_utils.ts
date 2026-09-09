/**
 * @docs ARCHITECTURE:UI
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / command_table_utils
 * - **Primary Entrypoints**: `calculate_utilization`, `format_cost_usd`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export function calculate_utilization(cost_usd: number = 0, budget_usd: number = 1): number {
  if (budget_usd === 0) {
    return cost_usd > 0 ? 100 : 0;
  }
  return (cost_usd / budget_usd) * 100;
}

/**
 * Formats accreted USD costs with micro-precision for sub-cent values.
 */
export function format_cost_usd(cost_usd: number = 0): string {
  if (cost_usd > 0 && cost_usd < 0.0001) {
    return cost_usd.toExponential(2);
  }
  return cost_usd.toFixed(4);
}
