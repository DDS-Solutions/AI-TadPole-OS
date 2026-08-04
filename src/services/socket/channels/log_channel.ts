/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[log_channel]` in observability traces.
 */

import { BaseChannel } from './channel';
import type { Incoming_Socket_Message, Socket_Log_Event, Socket_Agent_Message_Event, Socket_Scheduled_Job_Complete_Event } from '../types/events';
import { event_bus } from '../../event_bus';

export class LogChannel extends BaseChannel<Incoming_Socket_Message> {
    readonly name = 'log';
    private agent_name_cache = new Map<string, string>();

    matches(message: Incoming_Socket_Message): boolean {
        return (
            message.type === 'log' ||
            message.type === 'thought' ||
            message.type === 'agent:message' ||
            message.type === 'engine:scheduled_job_complete'
        );
    }

    handle(message: Incoming_Socket_Message): void {
        this.emit(message);
        
        switch (message.type) {
            case 'log':
            case 'thought':
                this.handle_log(message as Socket_Log_Event);
                break;
            case 'agent:message':
                this.handle_agent_message(message as Socket_Agent_Message_Event);
                break;
            case 'engine:scheduled_job_complete':
                this.handle_scheduled_job_complete(message as Socket_Scheduled_Job_Complete_Event);
                break;
        }
    }

    public set_agent_name_cache(agents: Array<{ id: string; name: string }>): void {
        this.agent_name_cache.clear();
        for (const a of agents) {
            if (a && a.id && a.name) {
                this.agent_name_cache.set(a.id, a.name);
            }
        }
    }

    private get_agent_id(data: Incoming_Socket_Message): string | undefined {
        const d = data as Record<string, unknown>;
        if ('agent_id' in d && typeof d.agent_id === 'string') return d.agent_id;
        if ('agentId' in d && typeof d.agentId === 'string') return d.agentId;
        if ('id' in d && typeof d.id === 'string') return d.id;
        return undefined;
    }

    private get_agent_name(agent_id: string | undefined, fallback_name?: string): string {
        if (agent_id) {
            const cached = this.agent_name_cache.get(agent_id);
            if (cached) return cached;
        }
        return fallback_name || agent_id || '';
    }

    private handle_log(data: Socket_Log_Event): void {
        const agent_id = this.get_agent_id(data);
        const agent_name = this.get_agent_name(agent_id, data.agent_name);
        const metadata = Object.assign(Object.create(null), data);

        event_bus.emit_log({
            id: (data.id || data.request_id || data.requestId || '') as string,
            source: agent_id ? 'Agent' : 'System',
            agent_id,
            agent_name,
            text: (data.text || data.message || JSON.stringify(data)) as string,
            severity: data.level === 'error' ? 'error' : 'info',
            metadata: metadata as Record<string, unknown>
        });
    }

    private handle_agent_message(data: Socket_Agent_Message_Event): void {
        const agent_id = this.get_agent_id(data);
        const agent_name = this.get_agent_name(agent_id, data.agent_name);
        const metadata = Object.assign(Object.create(null), data);

        event_bus.emit_log({
            id: (data.id || data.message_id || data.messageId || '') as string,
            source: 'Agent',
            agent_id,
            agent_name,
            text: (data.text || data.message || data.content || 'Mission action complete.') as string,
            severity: 'info',
            metadata: metadata as Record<string, unknown>
        });
    }

    private handle_scheduled_job_complete(data: Socket_Scheduled_Job_Complete_Event): void {
        event_bus.emit_log({
            source: 'System',
            text: `Scheduled Job '${data.job_name}' completed. Cost: $${(data.cost_usd || 0).toFixed(4)}`,
            severity: data.status === 'failed' ? 'error' : 'success'
        });
    }

    override clear(): void {
        super.clear();
        this.agent_name_cache.clear();
    }
}

// Metadata: [log_channel]
