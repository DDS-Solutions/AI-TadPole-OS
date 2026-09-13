/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Ui / theme_tokens
 * - **Primary Entrypoints**: `THEME_TOKENS`, `Z_INDEX_MAP`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export const THEME_TOKENS = {
    danger: 'text-[var(--color-danger-text)] border-[var(--color-danger-border)] bg-[var(--color-danger-bg)]',
    warning: 'text-[var(--color-warning-text)] border-[var(--color-warning-border)] bg-[var(--color-warning-bg)]',
    info: 'text-[var(--color-info-text)] border-[var(--color-info-border)] bg-[var(--color-info-bg)]',
    success: 'text-[var(--color-success-text)] border-[var(--color-success-border)] bg-[var(--color-success-bg)]',
};

export const Z_INDEX_MAP = {
    banner: 9990,
    portal: 9997,
    tooltip: 9998,
    toast: 9999,
    dialog: 10000,
};
