/**
 * @docs ARCHITECTURE:Logic:Commands
 */

import { event_bus } from '../../services/event_bus';
import type { Agent } from '../../types';

export function find_agent(name_or_id: string | undefined, agents: Agent[]): Agent | null {
    if (!name_or_id) {
        event_bus.emit_log({ source: 'System', text: 'Missing agent name. Usage: /<command> <agent-name>', severity: 'error' });
        return null;
    }
    const lower = name_or_id.toLowerCase();
    const found = agents.find(a => 
        a.name.toLowerCase() === lower || 
        a.id === name_or_id || 
        a.name.toLowerCase().includes(lower)
    );
    
    if (!found) {
        event_bus.emit_log({ 
            source: 'System', 
            text: `Agent "${name_or_id}" not found. Available: ${agents.map(a => a.name).slice(0, 8).join(', ')}...`, 
            severity: 'error' 
        });
        return null;
    }
    return found;
}
