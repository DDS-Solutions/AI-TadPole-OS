/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / index
 * - **Primary Entrypoints**: `ApiError`, `AuthError`, `RateLimitError`, `ValidationError`, `ServerError`, `map_api_error_to_subclass`, `map_api_error`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
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

export class AuthError extends ApiError {
    constructor(message: string, type: string, status: number, error_code: string | null = null, help_link: string | null = null) {
        super(message, type, status, error_code, help_link);
        this.name = 'AuthError';
        Object.setPrototypeOf(this, AuthError.prototype);
    }
}

export class RateLimitError extends ApiError {
    constructor(message: string, type: string, status: number, error_code: string | null = null, help_link: string | null = null) {
        super(message, type, status, error_code, help_link);
        this.name = 'RateLimitError';
        Object.setPrototypeOf(this, RateLimitError.prototype);
    }
}

export class ValidationError extends ApiError {
    constructor(message: string, type: string, status: number, error_code: string | null = null, help_link: string | null = null) {
        super(message, type, status, error_code, help_link);
        this.name = 'ValidationError';
        Object.setPrototypeOf(this, ValidationError.prototype);
    }
}

export class ServerError extends ApiError {
    constructor(message: string, type: string, status: number, error_code: string | null = null, help_link: string | null = null) {
        super(message, type, status, error_code, help_link);
        this.name = 'ServerError';
        Object.setPrototypeOf(this, ServerError.prototype);
    }
}

export function map_api_error_to_subclass(err: ApiError): ApiError {
    if (err.status === 401 || err.status === 403) {
        return new AuthError(err.message, err.type, err.status, err.error_code, err.help_link);
    }
    if (err.status === 429) {
        return new RateLimitError(err.message, err.type, err.status, err.error_code, err.help_link);
    }
    if (err.status === 400) {
        return new ValidationError(err.message, err.type, err.status, err.error_code, err.help_link);
    }
    if (err.status >= 500) {
        return new ServerError(err.message, err.type, err.status, err.error_code, err.help_link);
    }
    return err;
}

/**
 * @deprecated Use map_api_error_to_subclass instead.
 */
export function map_api_error(err: unknown): never {
    if (err instanceof ApiError) {
        throw map_api_error_to_subclass(err);
    }
    throw err;
}
