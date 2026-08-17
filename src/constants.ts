/**
 * @docs ARCHITECTURE:Infrastructure
 * 
 * ### AI Assist Note
 * **Root Constants Barrel**: Re-exports from `./constants/index`.
 * GAP-FE-01: Modularized constants into `src/constants/` submodules while maintaining
 * backwards compatibility for existing imports.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Incorrect provider string mapping or missing model IDs.
 * - **Telemetry Link**: Search for `[Constants]` in source audits.
 */

export * from './constants/index';

// Metadata: [constants]
