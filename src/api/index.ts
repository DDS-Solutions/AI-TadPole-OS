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
