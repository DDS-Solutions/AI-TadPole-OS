/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[headers]` in observability traces.
 */

import type { HttpClientAdapter } from '../types/ports';

export function mint_headers(
    crypto: Crypto | HttpClientAdapter['crypto'],
    token: string,
    custom_request_id?: string
): { 
    headers: Record<string, string>; 
    context: { span_id: string; trace_id: string; traceparent: string; request_id: string } 
} {
    const cleanToken = token.trim();
    if (!cleanToken) {
        throw new Error('Tadpole OS API token is missing. Configure NEURAL_TOKEN in Settings before making requests.');
    }

    const request_id = custom_request_id || 
        (typeof crypto.randomUUID === 'function' 
            ? crypto.randomUUID() 
            : `tr-${Date.now()}`);

    let trace_id: string;
    if (custom_request_id) {
        const stripped = custom_request_id.replace(/-/g, '');
        if (/^[0-9a-f]{32}$/i.test(stripped) && stripped !== '00000000000000000000000000000000') {
            trace_id = stripped.toLowerCase();
        } else {
            trace_id = Array.from(crypto.getRandomValues(new Uint8Array(16)))
                .map(b => b.toString(16).padStart(2, '0'))
                .join('');
        }
    } else {
        trace_id = Array.from(crypto.getRandomValues(new Uint8Array(16)))
            .map(b => b.toString(16).padStart(2, '0'))
            .join('');
    }

    const span_id = Array.from(crypto.getRandomValues(new Uint8Array(8)))
        .map(b => b.toString(16).padStart(2, '0'))
        .join('');

    const traceparent = `00-${trace_id}-${span_id}-01`;

    return {
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${cleanToken}`,
            'X-Request-Id': request_id,
            'traceparent': traceparent
        },
        context: { span_id, trace_id, traceparent, request_id }
    };
}

// Metadata: [headers]
