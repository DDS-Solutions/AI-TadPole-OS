/**
 * @docs ARCHITECTURE:Security
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_003_telemetry_scrub.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-003: Telemetry Scrub — API keys and secrets are redacted before logging.
 *
 * Asserts that `scrub_string` and `scrub_secrets` correctly redact
 * Google, Groq, Anthropic, HuggingFace, and GitHub API keys,
 * Bearer tokens, and sensitive object keys.
 */

import { describe, it, expect } from 'vitest';
import { scrub_string, scrub_secrets, scrub_secrets_object, is_sensitive_key } from '../../src/api/utils/scrub';

describe('INV-003: Telemetry secret scrubbing', () => {
    describe('scrub_string', () => {
        it('redacts OpenAI/Anthropic sk- keys', () => {
            const input = 'Authorization: sk-abc123def456ghi789jkl012';
            expect(scrub_string(input)).not.toContain('sk-abc123');
            expect(scrub_string(input)).toContain('[REDACTED]');
        });

        it('redacts Google AI API keys (AIza...)', () => {
            const input = 'key=AIzaSyBdefghijklmnopqrstuv';
            expect(scrub_string(input)).not.toContain('AIzaSyB');
            expect(scrub_string(input)).toContain('[REDACTED]');
        });

        it('redacts Groq API keys (gsk_...)', () => {
            const input = 'token: gsk_abcdefghijklmnopqrstuvwxyz';
            expect(scrub_string(input)).not.toContain('gsk_');
            expect(scrub_string(input)).toContain('[REDACTED]');
        });

        it('redacts HuggingFace API keys (hf_...)', () => {
            const input = 'api_key: hf_abcdefghijklmnopqrstuv';
            expect(scrub_string(input)).not.toContain('hf_');
            expect(scrub_string(input)).toContain('[REDACTED]');
        });

        it('redacts GitHub PATs (ghp_...)', () => {
            const input = 'token=ghp_abcdefghijklmnopqrstuvwxyz';
            expect(scrub_string(input)).not.toContain('ghp_');
            expect(scrub_string(input)).toContain('[REDACTED]');
        });

        it('redacts GitHub fine-grained PATs (github_pat_...)', () => {
            const input = 'auth: github_pat_' + 'a'.repeat(55);
            expect(scrub_string(input)).not.toContain('github_pat_');
            expect(scrub_string(input)).toContain('[REDACTED]');
        });

        it('redacts Bearer tokens', () => {
            const input = 'Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.payload.signature';
            expect(scrub_string(input)).not.toContain('eyJhbGci');
            expect(scrub_string(input)).toContain('Bearer [REDACTED]');
        });

        it('handles strings with no secrets', () => {
            const input = 'This is a normal log message with no keys';
            expect(scrub_string(input)).toBe(input);
        });
    });

    describe('scrub_secrets (body-level)', () => {
        it('scrubs string bodies containing keys', () => {
            const result = scrub_secrets('my key is sk-1234567890abcdef');
            expect(result).not.toContain('sk-1234567890');
        });

        it('scrubs JSON string bodies', () => {
            const body = JSON.stringify({ api_key: 'sk-secret123456789012' });
            const result = scrub_secrets(body) as string;
            expect(result).not.toContain('sk-secret');
        });

        it('scrubs object bodies with sensitive keys', () => {
            const body = { token: 'super-secret-value', name: 'safe' };
            const result = scrub_secrets(body) as Record<string, unknown>;
            expect(result).toHaveProperty('token', '[REDACTED]');
            expect(result).toHaveProperty('name', 'safe');
        });

        it('returns null/undefined as-is', () => {
            expect(scrub_secrets(null)).toBeNull();
            expect(scrub_secrets(undefined)).toBeUndefined();
        });
    });

    describe('is_sensitive_key', () => {
        it.each([
            'key', 'token', 'secret', 'password', 'auth', 'authorization',
            'cookie', 'jwt', 'bearer',
            'api_key', 'api_token', 'auth_token', 'access_token',
            'secret_key', 'jwt_token',
            'apiKey', 'authToken', 'secretKey',
        ])('detects "%s" as sensitive', (key) => {
            expect(is_sensitive_key(key)).toBe(true);
        });

        it.each([
            'name', 'email', 'url', 'host', 'port', 'model',
            // NOTE: 'max_tokens' is intentionally NOT here — it ends with '_token(s)'
            // which the scrubber correctly flags as sensitive. This is a tradeoff:
            // the scrubber over-redacts rather than under-redacts.
            'temperature', 'description',
        ])('does NOT flag "%s" as sensitive', (key) => {
            expect(is_sensitive_key(key)).toBe(false);
        });
    });

    describe('scrub_secrets_object (deep)', () => {
        it('recursively scrubs nested objects', () => {
            const obj = {
                config: {
                    provider: {
                        api_key: 'sk-nested-secret-key-value',
                        model: 'gpt-4',
                    },
                },
            };
            const result = scrub_secrets_object(obj) as any;
            expect(result.config.provider.api_key).toBe('[REDACTED]');
            expect(result.config.provider.model).toBe('gpt-4');
        });

        it('scrubs arrays of objects', () => {
            const arr = [
                { token: 'secret1', name: 'a' },
                { token: 'secret2', name: 'b' },
            ];
            const result = scrub_secrets_object(arr) as any[];
            expect(result[0].token).toBe('[REDACTED]');
            expect(result[1].token).toBe('[REDACTED]');
            expect(result[0].name).toBe('a');
        });
    });
});
