/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / index
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export * from './headers';
export * from './response-headers';
export * from './attributes';
