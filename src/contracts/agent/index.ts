/**
 * @docs ARCHITECTURE:Contracts
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / index
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export * from './shared';
export * from './wire';
export * from './domain';
export * from './form';
