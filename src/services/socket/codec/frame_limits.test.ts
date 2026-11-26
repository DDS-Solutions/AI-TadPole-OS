/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / Socket Codec / frame_limits.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import { MAX_BINARY_FRAME_SIZE, MAX_TEXT_FRAME_SIZE } from './frame_limits';
import { BINARY_HEADER_AUDIO, BINARY_HEADER_SWARM_PULSE } from './binary_headers';

describe('Socket codec framing contracts', () => {
    it('enforces bounded positive frame limits matching transport safety caps', () => {
        expect(MAX_BINARY_FRAME_SIZE).toBe(1024 * 1024);
        expect(MAX_TEXT_FRAME_SIZE).toBe(5 * 1024 * 1024);
        expect(MAX_TEXT_FRAME_SIZE).toBeGreaterThan(MAX_BINARY_FRAME_SIZE);
    });

    it('defines distinct single-byte binary frame headers', () => {
        expect(BINARY_HEADER_AUDIO).toBe(0x01);
        expect(BINARY_HEADER_SWARM_PULSE).toBe(0x02);
        expect(BINARY_HEADER_AUDIO).not.toBe(BINARY_HEADER_SWARM_PULSE);
    });
});
