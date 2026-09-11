/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / protocol_codec.test
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
import { ProtocolCodec } from './protocol_codec';
import { encode } from '@msgpack/msgpack';
import { BINARY_HEADER_AUDIO, BINARY_HEADER_SWARM_PULSE } from './binary_headers';
import type { Swarm_Pulse } from '../../../types';

describe('ProtocolCodec', () => {
    it('decodes empty binary buffer to unknown type', () => {
        const empty = new ArrayBuffer(0);
        const result = ProtocolCodec.decode_binary(empty);
        expect(result.type).toBe('unknown');
        expect(result.payload).toBe(empty);
    });

    it('decodes audio binary payload', () => {
        const audio_bytes = new Uint8Array([BINARY_HEADER_AUDIO, 1, 2, 3, 4]);
        const result = ProtocolCodec.decode_binary(audio_bytes.buffer);
        expect(result.type).toBe('audio');
        const payload_view = new Uint8Array(result.payload as ArrayBuffer);
        expect(Array.from(payload_view)).toEqual([1, 2, 3, 4]);
    });

    it('decodes MessagePack swarm pulse payload', () => {
        const mock_pulse: Partial<Swarm_Pulse> = {
            total_active_agents: 3,
            avg_health_score: 98.5
        };
        const packed = encode(mock_pulse);
        const combined = new Uint8Array(packed.length + 1);
        combined[0] = BINARY_HEADER_SWARM_PULSE;
        combined.set(packed, 1);

        const result = ProtocolCodec.decode_binary(combined.buffer);
        expect(result.type).toBe('pulse');
        expect(result.payload).toEqual(mock_pulse);
    });

    it('throws error when MessagePack swarm pulse payload is corrupted', () => {
        const corrupted = new Uint8Array([BINARY_HEADER_SWARM_PULSE, 0xc1, 0x00]); // invalid msgpack byte
        expect(() => ProtocolCodec.decode_binary(corrupted.buffer)).toThrow(/MessagePack decode failed/);
    });

    it('decodes and encodes JSON payloads accurately', () => {
        const mock_json = { type: 'health', healthy: true, timestamp: 12345 };
        const encoded = ProtocolCodec.encode_json(mock_json);
        expect(typeof encoded).toBe('string');
        const decoded = ProtocolCodec.decode_json(encoded);
        expect(decoded).toEqual(mock_json);
    });
});
