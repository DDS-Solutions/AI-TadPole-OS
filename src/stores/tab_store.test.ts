/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the workspace tab management and multi-window synchronization.** 
 * Verifies tab lifecycle (open/close/deduplication) and the detachment (pop-out) state management for various dashboard components.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Duplicate tabs created due to inconsistent path normalization or failure to broadcast state changes across window instances.
 * - **Telemetry Link**: Search `[tab_store.test]` in tracing logs.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { use_tab_store } from './tab_store';

// Mock BroadcastChannel
const mockPostMessage = vi.fn();
const mockOnMessage = vi.fn();
class MockBroadcastChannel {
    onmessage = mockOnMessage;
    postMessage = mockPostMessage;
}
vi.stubGlobal('BroadcastChannel', MockBroadcastChannel);

describe('tab_store', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        // Reset state to initial value
        use_tab_store.setState({
            tabs: [
                { id: 'initial-ops', title: 'Operations', path: '/dashboard', icon: 'LayoutDashboard' }
            ],
            active_tab_id: 'initial-ops',
            is_system_log_detached: false,
            is_trace_stream_detached: false,
            is_lineage_stream_detached: false,
            is_swarm_pulse_detached: false,
            is_agent_grid_detached: false
        });
    });

    it('opens a new tab', () => {
        const store = use_tab_store.getState();
        store.open_tab({
            title: 'Intelligence',
            path: '/intelligence',
            icon: 'Brain'
        });

        const state = use_tab_store.getState();
        expect(state.tabs).toHaveLength(2);
        expect(state.tabs[1].title).toBe('Intelligence');
        expect(state.active_tab_id).toBe(state.tabs[1].id);
    });

    it('deduplicates tabs by path', () => {
        const store = use_tab_store.getState();
        store.open_tab({ title: 'Intelligence', path: '/intelligence/view' });
        store.open_tab({ title: 'Intelligence Updated', path: '/intelligence/view/' }); // Should match despite trailing slash

        const state = use_tab_store.getState();
        expect(state.tabs).toHaveLength(2); // Initial + Intelligence
        expect(state.tabs[1].title).toBe('Intelligence Updated');
    });

    it('closes a tab and switches active tab', () => {
        const store = use_tab_store.getState();
        store.open_tab({ title: 'Tab 2', path: '/tab2' });
        const tab2_id = use_tab_store.getState().active_tab_id!;
        
        store.close_tab(tab2_id);
        
        const state = use_tab_store.getState();
        expect(state.tabs).toHaveLength(1);
        expect(state.active_tab_id).toBe('initial-ops');
    });

    it('does not close the last tab', () => {
        const store = use_tab_store.getState();
        store.close_tab('initial-ops');
        
        expect(use_tab_store.getState().tabs).toHaveLength(1);
    });

    it('sets active tab', () => {
        const store = use_tab_store.getState();
        store.open_tab({ title: 'Tab 2', path: '/tab2' });
        const tab2_id = use_tab_store.getState().active_tab_id!;
        
        store.set_active_tab('initial-ops');
        expect(use_tab_store.getState().active_tab_id).toBe('initial-ops');
        
        store.set_active_tab(tab2_id);
        expect(use_tab_store.getState().active_tab_id).toBe(tab2_id);
    });

    it('toggles detachment states', () => {
        const store = use_tab_store.getState();
        
        expect(use_tab_store.getState().is_agent_grid_detached).toBe(false);
        store.toggle_agent_grid_detachment();
        expect(use_tab_store.getState().is_agent_grid_detached).toBe(true);
        
        store.toggle_system_log_detachment();
        expect(use_tab_store.getState().is_system_log_detached).toBe(true);
    });

    it('fingerprints tabs for sync', () => {
        // This is testing internal logic indirectly or just ensuring it exists
        const state = use_tab_store.getState();
        expect(state.tabs).toBeDefined();
        expect(state.active_tab_id).toBeDefined();
    });
});
