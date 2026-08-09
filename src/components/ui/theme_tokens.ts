/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[theme_tokens]` in observability traces.
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

// Metadata: [theme_tokens]
