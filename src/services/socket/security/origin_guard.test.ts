/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / origin_guard.test
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
import { is_allowed_origin } from './origin_guard';

describe('is_allowed_origin', () => {
    it('allows loopback hostnames (localhost, 127.0.0.1, [::1])', () => {
        expect(is_allowed_origin('http://localhost:8000')).toBe(true);
        expect(is_allowed_origin('ws://127.0.0.1:8000')).toBe(true);
        expect(is_allowed_origin('http://[::1]:8000')).toBe(true);
    });

    it('allows RFC 1918 private LAN IP subnets and .local domains when private networks are trusted', () => {
        expect(is_allowed_origin('http://10.0.0.1:8000')).toBe(true);
        expect(is_allowed_origin('http://10.0.0.1:9000')).toBe(true);
        expect(is_allowed_origin('http://10.0.0.1:8000')).toBe(true);
        expect(is_allowed_origin('http://sovereign-node.local:8000')).toBe(true);
    });

    it('can reject private LAN IPs if allow_private_network is explicitly false', () => {
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
