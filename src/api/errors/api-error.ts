/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[api_error]` in observability traces.
 */

export class ApiError extends Error {
    public type: string;
    public status: number;
    public error_code: string | null;
    public help_link: string | null;

    constructor(
        message: string,
        type: string,
        status: number,
        error_code: string | null = null,
        help_link: string | null = null
    ) {
        super(message);
        this.type = type;
        this.status = status;
        this.error_code = error_code;
        this.help_link = help_link;
        this.name = 'ApiError';
        // Ensure the prototype is set correctly for stack traces
        Object.setPrototypeOf(this, ApiError.prototype);
    }
}

// Metadata: [api_error]
