/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: ADG Must-Fail Fixture / fabricated_witness
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` This fixture must FAIL lint-headers.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `nonexistent_test_that_does_not_exist.test.ts`
 */

// This file intentionally contains a fabricated witness test declaration.
// ADG v2 lint-headers MUST flag this file as a violation.
// If this file passes lint, the must-fail fixture is broken.
export const FIXTURE_MARKER = 'fabricated_witness';
