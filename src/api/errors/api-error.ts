/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / api-error
 * - **Primary Entrypoints**: `ApiError`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
