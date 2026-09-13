/**
 * @docs ARCHITECTURE:Types
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / vite-env.d
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

interface ImportMetaEnv {
  readonly VITE_NEURAL_TOKEN?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
