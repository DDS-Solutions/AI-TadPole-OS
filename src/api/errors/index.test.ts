/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / index.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import {
    ApiError,
    AuthError,
    RateLimitError,
    ValidationError,
    ServerError,
    map_api_error_to_subclass,
} from './index';

describe('API Errors & Mapping', () => {
    it('instantiates ApiError correctly', () => {
        const err = new ApiError('Not found', 'NOT_FOUND', 404, 'E404', 'https://docs.tadpole.org');
        expect(err.message).toBe('Not found');
        expect(err.type).toBe('NOT_FOUND');
        expect(err.status).toBe(404);
        expect(err.error_code).toBe('E404');
        expect(err.help_link).toBe('https://docs.tadpole.org');
        expect(err.name).toBe('ApiError');
        expect(err instanceof ApiError).toBe(true);
        expect(err instanceof Error).toBe(true);
    });

    it('maps 401 and 403 to AuthError', () => {
        const err401 = new ApiError('Unauthorized', 'UNAUTHORIZED', 401);
        const mapped401 = map_api_error_to_subclass(err401);
        expect(mapped401).toBeInstanceOf(AuthError);
        expect(mapped401.name).toBe('AuthError');

        const err403 = new ApiError('Forbidden', 'FORBIDDEN', 403);
        const mapped403 = map_api_error_to_subclass(err403);
        expect(mapped403).toBeInstanceOf(AuthError);
    });

    it('maps 429 to RateLimitError', () => {
        const err = new ApiError('Rate limit exceeded', 'RATE_LIMIT', 429);
        const mapped = map_api_error_to_subclass(err);
        expect(mapped).toBeInstanceOf(RateLimitError);
        expect(mapped.name).toBe('RateLimitError');
    });

    it('maps 400 to ValidationError', () => {
        const err = new ApiError('Bad request', 'VALIDATION_FAILED', 400);
        const mapped = map_api_error_to_subclass(err);
        expect(mapped).toBeInstanceOf(ValidationError);
        expect(mapped.name).toBe('ValidationError');
    });

    it('maps 500+ to ServerError', () => {
        const err500 = new ApiError('Internal Error', 'SERVER_ERROR', 500);
        const mapped500 = map_api_error_to_subclass(err500);
        expect(mapped500).toBeInstanceOf(ServerError);
        expect(mapped500.name).toBe('ServerError');

        const err503 = new ApiError('Service Unavailable', 'UNAVAILABLE', 503);
        const mapped503 = map_api_error_to_subclass(err503);
        expect(mapped503).toBeInstanceOf(ServerError);
    });

    it('returns unmodified ApiError for unmapped status codes like 404', () => {
        const err404 = new ApiError('Not found', 'NOT_FOUND', 404);
        const mapped404 = map_api_error_to_subclass(err404);
        expect(mapped404).toBe(err404);
        expect(mapped404.name).toBe('ApiError');
    });
});
