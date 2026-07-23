/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **ProtocolCodec**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[protocol_codec]` in observability traces.
 */

import { decode } from '@msgpack/msgpack';
import type { Swarm_Pulse } from '../../../types';
import type { Incoming_Socket_Message } from '../types/events';
import { BINARY_HEADER_AUDIO, BINARY_HEADER_SWARM_PULSE } from './binary_headers';

/**
 * ProtocolCodec
 * Handles encoding and decoding of WebSocket payload framing.
 */
export class ProtocolCodec {
    static decode_binary(data: ArrayBuffer): { type: 'audio' | 'pulse' | 'unknown'; payload: ArrayBuffer | Swarm_Pulse } {
        const view = new Uint8Array(data);
        if (view.length === 0) {
            return { type: 'unknown', payload: data };
        }
        const header = view[0];
        const payload = data.slice(1);
        if (header === BINARY_HEADER_AUDIO) {
            return { type: 'audio', payload };
        } else if (header === BINARY_HEADER_SWARM_PULSE) {
            try {
                const pulse = decode(payload, {
                    maxStrLength: 1024 * 1024,
                    maxBinLength: 1024 * 1024,
                    maxArrayLength: 10000,
                    maxMapLength: 10000
                }) as Swarm_Pulse;
                return { type: 'pulse', payload: pulse };
            } catch (e) {
                throw new Error(`MessagePack decode failed: ${e instanceof Error ? e.message : String(e)}`, { cause: e });
            }
        }
        return { type: 'unknown', payload: data };
    }

    static decode_json(data: string): Incoming_Socket_Message {
        return JSON.parse(data) as Incoming_Socket_Message;
    }

    static encode_json(data: Record<string, unknown>): string {
        return JSON.stringify(data);
    }
}

// Metadata: [protocol_codec]
