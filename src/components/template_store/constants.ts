/**
 * @docs ARCHITECTURE:Constants
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / constants
 * - **Primary Entrypoints**: `REPO_URL`, `REGISTRY_RAW`, `BASE_RAW_URL`, `COMPANY_SIZES`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export const REPO_URL = 'https://github.com/DDS-Solutions/AI-Tadpole-OS-Industry-Templates.git';
export const REGISTRY_RAW = 'https://raw.githubusercontent.com/DDS-Solutions/AI-Tadpole-OS-Industry-Templates/main/registry.json';
export const BASE_RAW_URL = 'https://raw.githubusercontent.com/DDS-Solutions/AI-Tadpole-OS-Industry-Templates/main';

export const COMPANY_SIZES = ['All', '25', '50', '100', '150', '200'] as const;
export const APP_REFRESH_AGENTS_EVENT = 'app:refresh-agents';

// Metadata: [Template_Store]



// [Template_Store]
