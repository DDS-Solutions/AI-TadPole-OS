/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: ADG Must-Fail Fixture / v1_header
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` This fixture must FAIL lint-headers.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

// This file intentionally retains the v1 "Witness Tests: none declared" field.
// ADG v2 lint-headers MUST flag this as a v1 violation.
// If this file passes lint, the must-fail fixture is broken.
export const FIXTURE_MARKER = 'v1_header';
