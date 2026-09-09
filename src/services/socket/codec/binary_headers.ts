/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / binary_headers
 * - **Primary Entrypoints**: `BINARY_HEADER_AUDIO`, `BINARY_HEADER_SWARM_PULSE`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export const BINARY_HEADER_AUDIO = 0x01;
export const BINARY_HEADER_SWARM_PULSE = 0x02;
