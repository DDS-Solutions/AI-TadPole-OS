/**
 * @docs ARCHITECTURE:UI
 * @docs UTILITY:CommandTableUtils
 *
 * ### AI Assist Note
 * Utility functions for Command_Table metrics and utilization formatting.
 * Isolated into a dedicated utility file to preserve React HMR Fast Refresh compliance.
 *
 * ### 🔍 Debugging & Observability
 * Traceability via `execution/parity_guard.py`.
 */

/**
 * Calculates allocation utilization percentage with zero-budget safety.
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
