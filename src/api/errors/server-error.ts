/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / server-error
 * - **Primary Entrypoints**: `ServerError`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { ApiError } from './api-error';

export class ServerError extends ApiError {
    constructor(message: string, type: string, status: number, error_code: string | null = null, help_link: string | null = null) {
        super(message, type, status, error_code, help_link);
        this.name = 'ServerError';
        Object.setPrototypeOf(this, ServerError.prototype);
    }
}
