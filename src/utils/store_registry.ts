/**
 * @docs ARCHITECTURE:Stores
 *
 * ### AI Context Alignment
 * - **Subsystem**: Store Architecture / store_registry
 * - **Primary Entrypoints**: `store_reset_registry`, `register_store_for_test_reset`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Zero-dependency module ensuring side-effect-free test isolation across all suites.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

export const store_reset_registry = new Set<() => void>();

export const register_store_for_test_reset = (reset_fn: () => void): void => {
    store_reset_registry.add(reset_fn);
};

