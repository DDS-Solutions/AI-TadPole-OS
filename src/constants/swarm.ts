/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / swarm
 * - **Primary Entrypoints**: `SWARM_NODE_STATUS`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export const SWARM_NODE_STATUS = {
    IDLE: 0,
    BUSY: 1,
    ERROR: 2,
    DEGRADED: 3,
    MISSION_HUB: 4, // UI-specific extension
} as const;
