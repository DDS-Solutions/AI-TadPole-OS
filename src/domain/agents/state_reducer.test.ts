/**
 * @docs ARCHITECTURE:Stores
 * ### AI Assist Note
 * Exercise partial updates and event ordering independently of the transport.
 * ### 🔍 Debugging & Observability
 * Fixtures cover omitted versus explicitly removed model slots.
 */
import { describe, expect, it } from 'vitest';
import type { Agent } from '../../contracts/agent';
import { reduce_agent_state } from './state_reducer';

describe('agent state ingestion', () => {
    const agent = { id: 'a', name: 'Alpha', role: 'Engineer', department: 'Engineering', model_2: 'planning', _server_timestamp: 20 } as Agent;
    it('preserves omitted fields and applies explicit slot deletion', () => {
        const updated = reduce_agent_state([agent], { type: 'patch', id: 'a', source: 'SOCKET', workspace: '/work', patch: { status: 'active' }, timestamp: 21 });
        expect(updated[0]).toMatchObject({ name: 'Alpha', model_2: 'planning', status: 'active' });
        const removed = reduce_agent_state(updated, { type: 'patch', id: 'a', source: 'SOCKET', workspace: '/work', patch: { model_2: null } as unknown as Partial<Agent>, timestamp: 22 });
        expect(removed[0].model_2).toBeUndefined();
    });
    it('ignores old events and incomplete unknown socket records', () => {
        const state = [agent];
        expect(reduce_agent_state(state, { type: 'patch', id: 'a', source: 'SOCKET', timestamp: 19, patch: { name: 'Old' } })).toBe(state);
        expect(reduce_agent_state(state, { type: 'patch', id: 'unknown', source: 'SOCKET', patch: { name: 'Unknown' } })).toBe(state);
    });
    it('normalizes snapshots through the same domain contract', () => {
        const snapshot = reduce_agent_state([agent], { type: 'snapshot', agents: [{ id: 'a', name: 'Alpha', model2: 'planning' }], workspace: () => '/work' });
        const patch = reduce_agent_state([agent], { type: 'patch', id: 'a', source: 'SOCKET', workspace: '/work', patch: { name: 'Alpha', model_2: 'planning' } });
        expect(snapshot[0]).toEqual(patch[0]);
    });
});
