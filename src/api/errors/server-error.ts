/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[server_error]` in observability traces.
 */

import { ApiError } from './api-error';

export class ServerError extends ApiError {
    constructor(message: string, type: string, status: number, error_code: string | null = null, help_link: string | null = null) {
        super(message, type, status, error_code, help_link);
        this.name = 'ServerError';
        Object.setPrototypeOf(this, ServerError.prototype);
    }
}

// Metadata: [server_error]
