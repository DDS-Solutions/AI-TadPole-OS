/**
 * @docs ARCHITECTURE:Domain
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / agents_domain
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export * from './normalizers';
export * from './serializers';
export * from './form_state';
