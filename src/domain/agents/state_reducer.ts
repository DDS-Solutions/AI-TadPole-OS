/**
 * @docs ARCHITECTURE:Stores
 * ### AI Assist Note
 * API snapshots and socket patches share one agent merge boundary.
 * ### 🔍 Debugging & Observability
 * Ordering and explicit-null cases are verified as pure transitions.
 */
import type { Agent, AgentDto } from '../../contracts/agent';
import { normalize_agent_dto } from './normalizers';

export type Agent_State_Event =
    | { type: 'snapshot'; agents: AgentDto[]; workspace: (id: string) => string }
    | { type: 'patch'; id: string; patch: Partial<Agent>; source: 'API' | 'SOCKET' | 'SYNC'; workspace?: string; timestamp?: number };

export function reduce_agent_state(agents: Agent[], event: Agent_State_Event): Agent[] {
    if (event.type === 'snapshot') {
        const existing = new Map(agents.map(agent => [agent.id, agent]));
        return event.agents.map(raw => ({
            ...existing.get(raw.id),
            ...normalize_agent_dto(raw, event.workspace(raw.id), existing.get(raw.id))
        }));
    }
    const index = agents.findIndex(agent => agent.id === event.id);
    const previous = agents[index];
    if (event.source === 'SOCKET' && event.timestamp) {
        if (previous?._server_timestamp && event.timestamp < previous._server_timestamp) return agents;
    }
    const patch = event.source === 'SOCKET' && event.workspace
        ? normalize_agent_dto({ id: event.id, ...event.patch } as AgentDto, event.workspace, previous)
        : event.patch;
    if (!previous && (event.source === 'API' || (event.source === 'SOCKET' && !event.workspace))) return agents;
    const next = {
        ...previous, ...patch, id: event.id,
        ...(event.timestamp ? { _server_timestamp: event.timestamp } : {})
    } as Agent;
    if (!previous) return [...agents, next];
    const result = [...agents];
    result[index] = next;
    return result;
}
