/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **src/api/index.ts — TRANSITIONAL barrel**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[index]` in observability traces.
 */

// src/api/index.ts — TRANSITIONAL barrel
// @deprecated Prefer deep imports from @/api/<module>; this barrel will be removed in v3.0

export * from './types';
export * from './errors';
export * from './utils';
export * from './channels';
export * from './trace';
export * from './service';
export * from './constants';
export { createApiService } from './factory';
export {
    base_api_service_instance,
    api_request,
    get_headers,
} from './legacy';

// Metadata: [index]
