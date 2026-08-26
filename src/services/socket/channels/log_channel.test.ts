/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / log_channel.test
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
import { LogChannel } from './log_channel';
import { event_bus } from '../../event_bus';
import type { Socket_Log_Event, Socket_Scheduled_Job_Complete_Event } from '../types/events';

describe('LogChannel', () => {
    let log_channel: LogChannel;

    beforeEach(() => {
        log_channel = new LogChannel();
        vi.restoreAllMocks();
    });

    it('matches supported log event types', () => {
        expect(log_channel.matches({ type: 'log' } as any)).toBe(true);
        expect(log_channel.matches({ type: 'thought' } as any)).toBe(true);
        expect(log_channel.matches({ type: 'agent:message' } as any)).toBe(true);
        expect(log_channel.matches({ type: 'engine:scheduled_job_complete' } as any)).toBe(true);
        expect(log_channel.matches({ type: 'unrelated:event' } as any)).toBe(false);
    });

    it('emits log events to event_bus with resolved agent names', () => {
        const emit_spy = vi.spyOn(event_bus, 'emit_log').mockImplementation(() => {});
        log_channel.set_agent_name_cache([
            { id: 'agent-123', name: 'Commander Alpha' }
        ]);

        const log_event: Socket_Log_Event = {
            type: 'log',
            id: 'req-01',
            agent_id: 'agent-123',
            text: 'Executing neural plan',
            level: 'info'
        };

        log_channel.handle(log_event);

        expect(emit_spy).toHaveBeenCalledWith(expect.objectContaining({
            source: 'Agent',
            agent_id: 'agent-123',
            agent_name: 'Commander Alpha',
            text: 'Executing neural plan',
            severity: 'info'
        }));
    });

    it('handles scheduled job completion events', () => {
        const emit_spy = vi.spyOn(event_bus, 'emit_log').mockImplementation(() => {});
        const job_event: Socket_Scheduled_Job_Complete_Event = {
            type: 'engine:scheduled_job_complete',
            job_id: 'job-01',
            job_name: 'Nightly Sync',
            status: 'completed',
            cost_usd: 0.045
        };

        log_channel.handle(job_event);

        expect(emit_spy).toHaveBeenCalledWith(expect.objectContaining({
            source: 'System',
            text: expect.stringContaining("Scheduled Job 'Nightly Sync' completed"),
            severity: 'success'
        }));
    });

    it('clears listeners and agent cache on clear()', () => {
        log_channel.set_agent_name_cache([{ id: 'a1', name: 'Agent One' }]);
        log_channel.clear();
        const emit_spy = vi.spyOn(event_bus, 'emit_log').mockImplementation(() => {});
        
        log_channel.handle({
            type: 'log',
            agent_id: 'a1',
            text: 'After clear'
        } as Socket_Log_Event);

        expect(emit_spy).toHaveBeenCalledWith(expect.objectContaining({
            agent_name: 'a1' // cache cleared, falls back to ID
        }));
    });
});
