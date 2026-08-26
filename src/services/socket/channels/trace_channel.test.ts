/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / trace_channel.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { TraceChannel } from './trace_channel';
import { event_bus } from '../../event_bus';
import type { Socket_Trace_Span_Event, Socket_Trace_Span_Update_Event } from '../types/events';

describe('TraceChannel', () => {
    let trace_channel: TraceChannel;

    beforeEach(() => {
        trace_channel = new TraceChannel();
        vi.restoreAllMocks();
    });

    it('matches trace span and update messages', () => {
        expect(trace_channel.matches({ type: 'trace:span' } as any)).toBe(true);
        expect(trace_channel.matches({ type: 'trace:span_update' } as any)).toBe(true);
        expect(trace_channel.matches({ type: 'log' } as any)).toBe(false);
    });

    it('emits new trace span to event_bus', () => {
        const emit_spy = vi.spyOn(event_bus, 'emit_trace').mockImplementation(() => {});
        const span_event: Socket_Trace_Span_Event = {
            type: 'trace:span',
            span: {
                id: 'span-001',
                name: 'rag_query',
                status: 'running'
            } as any
        };

        trace_channel.handle(span_event);

        expect(emit_spy).toHaveBeenCalledWith(expect.objectContaining({
            id: 'span-001',
            name: 'rag_query',
            status: 'running'
        }));
    });

    it('emits trace span update to event_bus', () => {
        const emit_spy = vi.spyOn(event_bus, 'emit_trace').mockImplementation(() => {});
        const update_event: Socket_Trace_Span_Update_Event = {
            type: 'trace:span_update',
            span_id: 'span-001',
            update: {
                status: 'completed',
                duration_ms: 120
            }
        };

        trace_channel.handle(update_event);

        expect(emit_spy).toHaveBeenCalledWith(expect.objectContaining({
            id: 'span-001',
            status: 'completed',
            duration_ms: 120
        }));
    });
});
