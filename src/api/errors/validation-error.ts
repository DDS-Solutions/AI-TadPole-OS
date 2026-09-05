/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / validation-error
 * - **Primary Entrypoints**: `ValidationError`
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

export class ValidationError extends ApiError {
    constructor(message: string, type: string, status: number, error_code: string | null = null, help_link: string | null = null) {
        super(message, type, status, error_code, help_link);
        this.name = 'ValidationError';
        Object.setPrototypeOf(this, ValidationError.prototype);
    }
}
