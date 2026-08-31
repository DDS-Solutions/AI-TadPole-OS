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

export function find_agent(name_or_id: string | undefined, agents: Agent[]): Agent | null {
    if (!name_or_id) {
        event_bus.emit_log({ source: 'System', text: 'Missing agent name. Usage: /<command> <agent-name>', severity: 'error' });
        return null;
    }
    const lower = name_or_id.toLowerCase();
    
    // 1. Exact ID or case-sensitive name match
    let found = agents.find(a => a.id === name_or_id || a.name === name_or_id);
    if (found) return found;

    // 2. Case-insensitive exact name match
    found = agents.find(a => a.name.toLowerCase() === lower);
    if (found) return found;

    // 3. Case-insensitive prefix match
    found = agents.find(a => a.name.toLowerCase().startsWith(lower));
    if (found) return found;

    // 4. Case-insensitive substring match
    found = agents.find(a => a.name.toLowerCase().includes(lower));
    if (found) return found;
    
    event_bus.emit_log({ 
        source: 'System', 
        text: `Agent "${name_or_id}" not found. Available: ${agents.map(a => a.name).slice(0, 8).join(', ')}...`, 
        severity: 'error' 
    });
    return null;
}
