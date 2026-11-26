/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / origin_guard.test
 *
 * ### AI Assist Note
 * - Origin equivalence must retain TLS and effective-port restrictions.
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect, vi, afterEach } from 'vitest';
import { is_allowed_origin } from './origin_guard';

describe('is_allowed_origin', () => {
    afterEach(() => vi.unstubAllGlobals());

    it.each([['https:', 'wss:', '443'], ['http:', 'ws:', '80']])('allows %s browser origins over %s on the same port', (page_scheme, socket_scheme, port) => {
        vi.stubGlobal('window', { location: { protocol: page_scheme, hostname: 'same-origin.example', port: '' } });
        expect(is_allowed_origin(`${socket_scheme}//same-origin.example:${port}/v1/engine/ws`)).toBe(true);
        expect(is_allowed_origin(`${socket_scheme}//same-origin.example:9443/v1/engine/ws`)).toBe(false);
        expect(is_allowed_origin(`${socket_scheme}//other-origin.example:${port}/v1/engine/ws`)).toBe(false);
    });

    it('does not downgrade an HTTPS origin or widen a URL allowlist to another port', () => {
        vi.stubGlobal('window', { location: { protocol: 'https:', hostname: 'secure-page.example', port: '' } });
        expect(is_allowed_origin('ws://secure-page.example/v1/engine/ws')).toBe(false);
        expect(is_allowed_origin('wss://trusted-url.example/v1/engine/ws', ['https://trusted-url.example'])).toBe(true);
        expect(is_allowed_origin('wss://trusted-url.example:9443/v1/engine/ws', ['https://trusted-url.example'])).toBe(false);
        expect(is_allowed_origin('ws://trusted-url.example/v1/engine/ws', ['https://trusted-url.example'])).toBe(false);
    });

    it('allows loopback hostnames (localhost, 127.0.0.1, [::1])', () => {
        expect(is_allowed_origin('http://localhost:8000')).toBe(true);
        expect(is_allowed_origin('ws://127.0.0.1:8000')).toBe(true);
        expect(is_allowed_origin('http://[::1]:8000')).toBe(true);
    });

    it('allows RFC 1918 private LAN IP subnets and .local domains when private networks are trusted', () => {
        expect(is_allowed_origin('http://10.0.0.1:8000', [], true)).toBe(true);
        expect(is_allowed_origin('http://10.0.0.1:9000', [], true)).toBe(true);
        expect(is_allowed_origin('http://10.0.0.1:8000', [], true)).toBe(true);
        expect(is_allowed_origin('http://sovereign-node.local:8000', [], true)).toBe(true);
    });

    it('rejects private LAN IPs by default or if allow_private_network is explicitly false', () => {
        expect(is_allowed_origin('http://10.0.0.1:8000')).toBe(false);
        expect(is_allowed_origin('http://10.0.0.1:8000', [], false)).toBe(false);
    });

    it('rejects unlisted external origins', () => {
        expect(is_allowed_origin('http://evil.com')).toBe(false);
        expect(is_allowed_origin('https://attacker.org:8080')).toBe(false);
    });

    it('allows explicit runtime allowed origins with port matching', () => {
        const allowed = ['https://trusted.domain.com:8443', 'api.internal.net:3000', '*.corp.internal'];
        expect(is_allowed_origin('https://trusted.domain.com:8443/ws', allowed)).toBe(true);
        expect(is_allowed_origin('https://trusted.domain.com:9999/ws', allowed)).toBe(false);
        expect(is_allowed_origin('http://api.internal.net:3000', allowed)).toBe(true);
        expect(is_allowed_origin('http://api.internal.net:4000', allowed)).toBe(false);
        expect(is_allowed_origin('http://node1.corp.internal:8000', allowed)).toBe(true);
        expect(is_allowed_origin('http://other.domain.com', allowed)).toBe(false);
    });

    it('handles malformed URLs safely by returning false', () => {
        expect(is_allowed_origin('not-a-valid-url')).toBe(false);
        expect(is_allowed_origin('')).toBe(false);
    });
});
