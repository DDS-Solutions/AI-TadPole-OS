/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **@deprecated Use map_api_error_to_subclass instead.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[index]` in observability traces.
 */

import { ApiError } from './api-error';
import { AuthError } from './auth-error';
import { RateLimitError } from './rate-limit-error';
import { ValidationError } from './validation-error';
import { ServerError } from './server-error';

export * from './api-error';
export * from './auth-error';
export * from './rate-limit-error';
export * from './validation-error';
export * from './server-error';

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

// Metadata: [index]
