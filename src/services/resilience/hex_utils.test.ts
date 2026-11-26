/**
 * @docs ARCHITECTURE:UI-Services:Resilience
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / Resilience / hex_utils.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import { generate_hex_id } from './hex_utils';

describe('generate_hex_id', () => {
    it('generates lowercase hexadecimal identifiers matching 2 * byte length', () => {
        const id16 = generate_hex_id(16);
        const id8 = generate_hex_id(8);
        expect(id16).toMatch(/^[0-9a-f]{32}$/);
        expect(id8).toMatch(/^[0-9a-f]{16}$/);
        expect(id16).not.toBe(generate_hex_id(16));
    });
});
