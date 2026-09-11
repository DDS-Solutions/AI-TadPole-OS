/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / url.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect } from 'vitest';
import { validate_and_sanitize_url } from './url';

describe('utils/url: validate_and_sanitize_url', () => {
    it('strips credentials and trailing slashes from valid HTTPS endpoints', () => {
        const result = validate_and_sanitize_url('https://admin:secret@api.tadpole.dev/v1/');
        expect(result).toBe('https://api.tadpole.dev/v1');
    });

    it('allows HTTP for local loopback hostnames and IPs', () => {
        expect(validate_and_sanitize_url('http://localhost:8000')).toBe('http://localhost:8000');
        expect(validate_and_sanitize_url('http://127.0.0.1:3000/')).toBe('http://127.0.0.1:3000');
        expect(validate_and_sanitize_url('http://[::1]:8080')).toBe('http://[::1]:8080');
        expect(validate_and_sanitize_url('http://node.localhost:9000')).toBe('http://node.localhost:9000');
    });

    it('blocks insecure external HTTP connections by default', () => {
        expect(() => validate_and_sanitize_url('http://insecure-domain.org')).toThrow(
            /Insecure transmission blocked/
        );
    });

    it('blocks RFC1918 and local domain HTTP connections when allow_private_network is false (default)', () => {
        expect(() => validate_and_sanitize_url('http://10.0.0.1:8000')).toThrow(
            /Insecure transmission blocked/
        );
        expect(() => validate_and_sanitize_url('http://10.0.0.1:8000')).toThrow(
            /Insecure transmission blocked/
        );
        expect(() => validate_and_sanitize_url('http://10.0.0.1:8000')).toThrow(
            /Insecure transmission blocked/
        );
        expect(() => validate_and_sanitize_url('http://node.local:8000')).toThrow(
            /Insecure transmission blocked/
        );
    });

    it('permits RFC1918 and local domain HTTP connections when allow_private_network is true', () => {
        expect(validate_and_sanitize_url('http://10.0.0.1:8000/', true)).toBe('http://10.0.0.1:8000');
        expect(validate_and_sanitize_url('http://10.0.0.1:8000', true)).toBe('http://10.0.0.1:8000');
        expect(validate_and_sanitize_url('http://10.0.0.1:8000', true)).toBe('http://10.0.0.1:8000');
        expect(validate_and_sanitize_url('http://tadpole-node.local:8000', true)).toBe('http://tadpole-node.local:8000');
        expect(validate_and_sanitize_url('http://cluster.lan:8000', true)).toBe('http://cluster.lan:8000');
        expect(validate_and_sanitize_url('http://gateway.internal:8000', true)).toBe('http://gateway.internal:8000');
    });

    it('still rejects non-private external domains over HTTP even with allow_private_network enabled', () => {
        expect(() => validate_and_sanitize_url('http://public-api.com', true)).toThrow(
            /Insecure transmission blocked/
        );
    });

    it('throws on empty or malformed URLs', () => {
        expect(() => validate_and_sanitize_url('')).toThrow('URL is empty');
        expect(() => validate_and_sanitize_url('   ')).toThrow('URL is empty');
        expect(() => validate_and_sanitize_url('not-a-valid-url')).toThrow(/Invalid URL format/);
    });
});
