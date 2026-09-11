/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / utils
 * - **Primary Entrypoints**: `find_agent`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { event_bus } from '../../services/event_bus';
import type { Agent } from '../../types';
import { use_agent_store } from '../../stores/agent_store';

export function find_agent(name_or_id: string | undefined, agents: Agent[]): Agent | null {
    if (!name_or_id) {
        event_bus.emit_log({ source: 'System', text: 'Missing agent name. Usage: /<command> <agent-name>', severity: 'error' });
        return null;
    }
    const lower = name_or_id.toLowerCase();
    
    // Helper to evaluate ranked matching on a candidate list
    const search_list = (list: Agent[]): Agent | null => {
        // 1. Exact ID or case-sensitive name match
        let match = list.find(a => a.id === name_or_id || a.name === name_or_id);
        if (match) return match;

        // 2. Case-insensitive exact match (ID or name)
        match = list.find(a => (a.id && a.id.toLowerCase() === lower) || (a.name && a.name.toLowerCase() === lower));
        if (match) return match;

        // 3. Case-insensitive prefix match (name or ID)
        match = list.find(a => (a.name && a.name.toLowerCase().startsWith(lower)) || (a.id && a.id.toLowerCase().startsWith(lower)));
        if (match) return match;

        // 4. Case-insensitive substring match (name or ID)
        match = list.find(a => (a.name && a.name.toLowerCase().includes(lower)) || (a.id && a.id.toLowerCase().includes(lower)));
        if (match) return match;

        return null;
    };

    // First search in passed context agents
    let result = search_list(agents);

    // If not found in context agents, fallback to live store agents
    const live_agents = use_agent_store.getState().agents || [];
    if (!result) {
        result = search_list(live_agents);
    } else {
        // Rehydrate with freshest store data if matching agent exists in store
        const live = live_agents.find(a => a.id === result!.id || a.name === result!.name);
        if (live) result = live;
    }

    if (result) return result;
    
    const available_names = (agents.length > 0 ? agents : live_agents)
        .map(a => a.name)
        .filter(Boolean)
        .slice(0, 8)
        .join(', ');

    event_bus.emit_log({ 
        source: 'System', 
        text: `Agent "${name_or_id}" not found. Available: ${available_names || 'none'}...`, 
        severity: 'error' 
    });
    return null;
}
