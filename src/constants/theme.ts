/**
 * @docs ARCHITECTURE:DesignSystem
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / theme
 * - **Primary Entrypoints**: `THEME_COLORS`, `GRAPH_THEME`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export const THEME_COLORS = {
    // Functional States
    IDLE: '#71717a',      // zinc-500
    BUSY: '#22d3ee',      // cyan-400
    RUNNING: '#06b6d4',   // cyan-500 (more saturated for active pulses)
    ERROR: '#f43f5e',     // rose-500
    DEGRADED: '#f59e0b',   // amber-500
    SUCCESS: '#10b981',    // emerald-500
    
    // UI Accents
    PRIMARY: '#22d3ee',   // cyan-400
    SECONDARY: '#71717a', // zinc-500
    WARNING: '#f59e0b',   // amber-500
    DANGER: '#f43f5e',    // rose-500

    // Borders & Traces
    BORDER_DIM: '#27272a', // zinc-800
    BORDER_BRIGHT: '#3f3f46', // zinc-700
    
    // Special Effects & UI Accents
    GLOW_CYAN: 'rgba(34, 211, 238, 0.4)',
    GLOW_ROSE: 'rgba(244, 63, 94, 0.4)',
    NEURAL_GRID: 'rgba(255, 255, 255, 0.05)',
    AMBER: '#eab308',
    AMBER_LIGHT: '#fbbf24',
    AMBER_GLOW: 'rgba(245, 158, 11, 0.4)',
    SKY: '#38bdf8',
    CYBER_GREEN: '#22c55e',
    CYBER_RED: '#ef4444',
    ROSE_LIGHT: '#fda4af',
    AMBER_LIGHT_TEXT: '#fef08a',
    FOCUS_RING: '#10b981',
    
    // Backgrounds (Matching zinc-950/900)
    DARK_BG: '#09090b',
    DARK_SURFACE: '#18181b',
};

export const GRAPH_THEME = {
    NODE_RADIUS: 2.5,
    LINK_WIDTH: 1,
    LINK_DISTANCE: 60,
    CHARGE_STRENGTH: -150,
    PARTICLE_SPEED: 0.005,
    LABEL_FONT: 'Inter, system-ui',
    TELEMETRY_BG: 'rgba(255, 255, 255, 0.1)',
};
