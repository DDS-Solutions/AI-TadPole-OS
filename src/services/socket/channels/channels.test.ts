/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / channels.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi } from 'vitest';
import { AgentUpdateChannel } from './agent_update_channel';
import { LogChannel } from './log_channel';
import { AudioStreamChannel } from './audio_stream_channel';
import { TraceChannel } from './trace_channel';
import { SwarmPulseChannel } from './swarm_pulse_channel';
import { HandoffChannel } from './handoff_channel';
import { McpPulseChannel } from './mcp_pulse_channel';
import { HealthChannel } from './health_channel';
import { event_bus } from '../../event_bus';

// Mock event_bus
vi.mock('../../event_bus', () => ({
    event_bus: {
        emit_log: vi.fn(),
        emit_trace: vi.fn(),
    }
}));

describe('WebSocket Channels', () => {
    it('AgentUpdateChannel matches and routes agent state events', () => {
        const channel = new AgentUpdateChannel();
        const listener = vi.fn();
        channel.subscribe(listener);

        const msg_valid = { type: 'agent:update', agent_id: 'agent-1' } as any;
        const msg_invalid = { type: 'other:type' } as any;

        expect(channel.matches(msg_valid)).toBe(true);
        expect(channel.matches(msg_invalid)).toBe(false);

        channel.handle(msg_valid);
        expect(listener).toHaveBeenCalledWith(msg_valid);
    });

    it('LogChannel matches and routes agent logs', () => {
        const channel = new LogChannel();
        const listener = vi.fn();
        channel.subscribe(listener);

        channel.set_agent_name_cache([{ id: 'agent-1', name: 'Agent Alpha' }]);

        const msg_valid = { type: 'log', agent_id: 'agent-1', text: 'Initiating sequence...' } as any;
        const msg_invalid = { type: 'other:type' } as any;

        expect(channel.matches(msg_valid)).toBe(true);
        expect(channel.matches(msg_invalid)).toBe(false);

        channel.handle(msg_valid);
        expect(listener).toHaveBeenCalledWith(msg_valid);
        
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            agent_id: 'agent-1',
            agent_name: 'Agent Alpha',
            text: 'Initiating sequence...'
        }));
    });

    it('AudioStreamChannel handles and emits binary audio chunks', () => {
        const channel = new AudioStreamChannel();
        const listener = vi.fn();
        channel.subscribe(listener);

        const chunk = new Uint8Array([1, 2, 3]).buffer;
        channel.handle_binary(chunk);

        expect(listener).toHaveBeenCalledWith(chunk);
    });

    it('TraceChannel matches and routes trace events', () => {
        const channel = new TraceChannel();
        const listener = vi.fn();
        channel.subscribe(listener);

        const msg_valid = { type: 'trace:span', span: { id: 'span-123', name: 'process' } } as any;
        const msg_invalid = { type: 'engine:health' } as any;

        expect(channel.matches(msg_valid)).toBe(true);
        expect(channel.matches(msg_invalid)).toBe(false);

        channel.handle(msg_valid);
        expect(listener).toHaveBeenCalledWith(msg_valid);
        expect(event_bus.emit_trace).toHaveBeenCalledWith(msg_valid.span);
    });

    it('SwarmPulseChannel handles and emits binary swarm pulses', () => {
        const channel = new SwarmPulseChannel();
        const listener = vi.fn();
        channel.subscribe(listener);

        const payload = { active_agents: 4 } as any;
        channel.emit(payload);

        expect(listener).toHaveBeenCalledWith(payload);
    });

    it('HandoffChannel matches and routes handoffs', () => {
        const channel = new HandoffChannel();
        const listener = vi.fn();
        channel.subscribe(listener);

        const msg_valid = { type: 'agent:handoff', agent_id: 'agent-1', from_cluster: 'a', to_cluster: 'b' } as any;
        const msg_invalid = { type: 'agent:update' } as any;

        expect(channel.matches(msg_valid)).toBe(true);
        expect(channel.matches(msg_invalid)).toBe(false);

        channel.handle(msg_valid);
        expect(listener).toHaveBeenCalledWith(msg_valid);
    });

    it('McpPulseChannel matches and routes mcp events', () => {
        const channel = new McpPulseChannel();
        const listener = vi.fn();
        channel.subscribe(listener);

        const msg_valid = { type: 'engine:mcp_pulse', tool: 'git_add' } as any;
        const msg_invalid = { type: 'engine:health' } as any;

        expect(channel.matches(msg_valid)).toBe(true);
        expect(channel.matches(msg_invalid)).toBe(false);

        channel.handle(msg_valid);
        expect(listener).toHaveBeenCalledWith(msg_valid);
    });

    it('HealthChannel matches and routes health pulse events', () => {
        const channel = new HealthChannel();
        const listener = vi.fn();
        channel.subscribe(listener);

        const msg_valid = { type: 'engine:health', cpu: 5.5 } as any;
        const msg_invalid = { type: 'engine:system_log' } as any;

        expect(channel.matches(msg_valid)).toBe(true);
        expect(channel.matches(msg_invalid)).toBe(false);

        channel.handle(msg_valid);
        expect(listener).toHaveBeenCalledWith(msg_valid);
    });
});
