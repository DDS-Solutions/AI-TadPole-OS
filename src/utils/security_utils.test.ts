/**
 * @docs ARCHITECTURE:Security
 * 
 * ### AI Context Alignment
 * - **Subsystem**: Sovereign Frontend / Utilities / security_utils.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import { sanitize_telemetry, sanitize_payload, scan_and_redact_secrets } from './security_utils';

describe('Security Utils - Sanitization', () => {
    it('should strip script tags', () => {
        const input = 'Hello <script>alert("xss")</script> world';
        const expected = 'Hello  world';
        expect(sanitize_telemetry(input)).toBe(expected);
    });

    it('should strip event handlers', () => {
        const input = '<img src="x" onerror="alert(1)">';
        const expected = '<img src="x" >';
        expect(sanitize_telemetry(input)).toBe(expected);
    });

    it('should strip dangerous tags like iframe', () => {
        const input = 'Check this out <iframe src="javascript:alert(1)"></iframe>';
        const expected = 'Check this out ';
        expect(sanitize_telemetry(input)).toBe(expected);
    });

    it('should handle nested/complex dangerous attributes', () => {
        const input = '<div onmouseover="doSomething()" onclick="doOther()">Content</div>';
        const expected = '<div  >Content</div>';
        expect(sanitize_telemetry(input)).toBe(expected);
    });

    it('should sanitize entire payloads recursively', () => {
        const payload = {
            type: 'log',
            text: '<script>evil()</script>Safe text',
            thought: 'Deep <iframe src="evil.com"></iframe> thoughts',
            other: 'data'
        };
        const sanitized = sanitize_payload(payload);
        expect(sanitized.text).toBe('Safe text');
        expect(sanitized.thought).toBe('Deep  thoughts');
        expect(sanitized.other).toBe('data');
    });

    it('should handle empty or null input gracefully', () => {
        expect(sanitize_telemetry('')).toBe('');
        // @ts-expect-error testing null
        expect(sanitize_telemetry(null)).toBe('');
    });
});

describe('Security Utils - DLP Secret Redaction', () => {
    it('should detect and redact private keys', () => {
        const text = 'Here is the key: -----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----';
        const res = scan_and_redact_secrets(text);
        expect(res.has_secrets).toBe(true);
        expect(res.redacted_count).toBe(1);
        expect(res.detected_types).toContain('Private Key');
        expect(res.sanitized).toContain('[REDACTED_PRIVATE_KEY]');
        expect(res.sanitized).not.toContain('MIIEowIBAAKCAQEA');
    });

    it('should detect and redact OpenAI API keys', () => {
        const text = 'Use sk-proj-1234567890abcdef1234567890 for auth';
        const res = scan_and_redact_secrets(text);
        expect(res.has_secrets).toBe(true);
        expect(res.sanitized).toBe('Use [REDACTED_AI_KEY] for auth');
    });

    it('should detect and redact Anthropic API keys', () => {
        const text = 'Anthropic key: sk-ant-api03-1234567890abcdef1234567890-test';
        const res = scan_and_redact_secrets(text);
        expect(res.has_secrets).toBe(true);
        expect(res.sanitized).toBe('Anthropic key: [REDACTED_AI_KEY]');
    });

    it('should detect and redact Google API keys', () => {
        const text = 'API key is AIzaSyD1234567890abcdefghijklmnopqrstuv';
        const res = scan_and_redact_secrets(text);
        expect(res.has_secrets).toBe(true);
        expect(res.sanitized).toBe('API key is [REDACTED_GOOGLE_KEY]');
    });

    it('should detect and redact GitHub tokens', () => {
        const text = 'Token: ghp_1234567890abcdefghijklmnopqrstuvwxyz12';
        const res = scan_and_redact_secrets(text);
        expect(res.has_secrets).toBe(true);
        expect(res.sanitized).toBe('Token: [REDACTED_GITHUB_TOKEN]');
    });

    it('should detect and redact Bearer tokens', () => {
        const text = 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.abcdefghij';
        const res = scan_and_redact_secrets(text);
        expect(res.has_secrets).toBe(true);
        expect(res.sanitized).toContain('Bearer [REDACTED_BEARER_TOKEN]');
    });

    it('should return clean text without modification when no secrets are present', () => {
        const text = 'This is a normal message about user authentication flow.';
        const res = scan_and_redact_secrets(text);
        expect(res.has_secrets).toBe(false);
        expect(res.redacted_count).toBe(0);
        expect(res.sanitized).toBe(text);
    });
});
