/**
 * @docs ARCHITECTURE:Infrastructure
 * 
 * ### AI Assist Note
 * **Constants Barrel**: Re-exports all constant modules from the `constants/` directory.
 * GAP-FE-01: Migrated provider/model constants from root `constants.ts` to
 * `constants/providers.ts` to resolve the file vs directory import ambiguity.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Incorrect provider string mapping or missing model IDs.
 * - **Telemetry Link**: Search for `[Constants]` in source audits.
 */

export { PROVIDERS, DEFAULT_PROVIDER, MODEL_IDS } from './providers';

// Metadata: [constants_index]
