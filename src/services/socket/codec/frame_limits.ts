/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / frame_limits
 * - **Primary Entrypoints**: `MAX_BINARY_FRAME_SIZE`, `MAX_TEXT_FRAME_SIZE`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export const MAX_BINARY_FRAME_SIZE = 1 * 1024 * 1024; // 1MB
export const MAX_TEXT_FRAME_SIZE = 5 * 1024 * 1024; // 5MB
