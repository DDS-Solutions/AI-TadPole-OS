/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[agent_update_channel]` in observability traces.
 */

import { BaseChannel } from './channel';
import type { Incoming_Socket_Message, Agent_Update_Event } from '../types/events';
import { event_bus } from '../../event_bus';

export class AgentUpdateChannel extends BaseChannel<Agent_Update_Event> {
    readonly name = 'agentUpdates';

    matches(message: Incoming_Socket_Message): boolean {
        return (
            message.type === 'agent:create' ||
            message.type === 'agent:update' ||
            message.type === 'agent:status' ||
            message.type === 'engine:ui_invalidate'
        );
    }

    handle(message: Incoming_Socket_Message): void {
        const data = message as Agent_Update_Event;
        const normalized_agent_id = this.get_agent_id(data) || '';
        
        const normalized_data = data.type === 'agent:status'
            ? { ...data, type: 'agent:update' as const, agent_id: normalized_agent_id, data: { status: data.status } }
            : { ...data, agent_id: normalized_agent_id };

        if (data.type === 'engine:ui_invalidate') {
            event_bus.emit_log({
                source: 'System',
                text: `UI Invalidated: ${data.resource}${data.id ? ` (#${data.id})` : ''}`,
                severity: 'info'
            });
        }

        this.emit(normalized_data as Agent_Update_Event);
    }

    private get_agent_id(data: Incoming_Socket_Message): string | undefined {
        const d = data as Record<string, unknown>;
        if ('agent_id' in d && typeof d.agent_id === 'string') return d.agent_id;
        if ('agentId' in d && typeof d.agentId === 'string') return d.agentId;
        if ('id' in d && typeof d.id === 'string') return d.id;
        return undefined;
    }
}

// Metadata: [agent_update_channel]
