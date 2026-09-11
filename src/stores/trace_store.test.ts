/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / trace_store.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { use_trace_store, type Trace_Span } from './trace_store';
import { event_bus } from '../services/event_bus';

describe('use_trace_store', () => {
    beforeEach(() => {
        use_trace_store.getState().clear_all();
    });

    const mock_span_1: Trace_Span = {
        id: 'span1',
        trace_id: 'trace-123',
        name: 'test_span',
        agent_id: 'agent1',
        mission_id: 'mission1',
        start_time: 1000,
        last_activity_at: 1000,
        status: 'success',
        attributes: { key: 'value' }
    };

    const mock_span_2: Trace_Span = {
        id: 'span2',
        trace_id: 'trace-123',
        parent_id: 'span1',
        name: 'child_span',
        agent_id: 'agent1',
        mission_id: 'mission1',
        start_time: 1010,
        last_activity_at: 1010,
        status: 'running',
        attributes: {}
    };

    const mock_span_3: Trace_Span = {
        id: 'span3',
        trace_id: 'trace-other',
        name: 'other_span',
        agent_id: 'agent2',
        mission_id: 'mission2',
        start_time: 2000,
        last_activity_at: 2000,
        status: 'success',
        attributes: {}
    };

    it('adds spans to the store', () => {
        const store = use_trace_store.getState();
        store.add_span(mock_span_1);

        const state = use_trace_store.getState();
        expect(state.spans['span1']).toEqual(mock_span_1);
    });

    it('updates spans in the store', () => {
        const store = use_trace_store.getState();
        store.add_span(mock_span_1);

        store.update_span('span1', { status: 'error', end_time: 1050 });

        const updated_span = use_trace_store.getState().spans['span1'];
        expect(updated_span.status).toBe('error');
        expect(updated_span.end_time).toBe(1050);
        expect(updated_span.name).toBe('test_span'); // Unchanged prop
    });

    it('does not update non-existent spans', () => {
        const store = use_trace_store.getState();
        store.add_span(mock_span_1);

        store.update_span('does-not-exist', { status: 'error' });

        expect(use_trace_store.getState().spans).toEqual({ 'span1': mock_span_1 });
    });

    it('sets the active trace', () => {
        const store = use_trace_store.getState();
        store.set_active_trace('trace-123');

        expect(use_trace_store.getState().active_trace_id).toBe('trace-123');
    });

    it('builds a trace tree correctly', () => {
        const store = use_trace_store.getState();
        store.add_span(mock_span_1);
        store.add_span(mock_span_2);
        store.add_span(mock_span_3);

        const tree = store.get_trace_tree('trace-123');

        expect(tree).toHaveLength(1); // Only root node should be at top level
        expect(tree[0].id).toBe('span1');
        expect(tree[0].children).toHaveLength(1);
        expect(tree[0].children[0].id).toBe('span2');
    });

    it('clears spans for a specific trace', () => {
        const store = use_trace_store.getState();
        store.add_span(mock_span_1);
        store.add_span(mock_span_2);
        store.add_span(mock_span_3);
        store.set_active_trace('trace-123');

        store.clear_trace('trace-123');

        const state = use_trace_store.getState();
        expect(state.spans).toEqual({ 'span3': mock_span_3 });
        expect(state.active_trace_id).toBeNull(); // Should clear active trace if it matches
    });

    it('does not clear active trace if clearing a different trace', () => {
        const store = use_trace_store.getState();
        store.add_span(mock_span_1);
        store.add_span(mock_span_3);
        store.set_active_trace('trace-123');

        store.clear_trace('trace-other');

        const state = use_trace_store.getState();
        expect(state.spans).toEqual({ 'span1': mock_span_1 });
        expect(state.active_trace_id).toBe('trace-123'); 
    });

    it('clears all spans and active trace', () => {
        const store = use_trace_store.getState();
        store.add_span(mock_span_1);
        store.set_active_trace('trace-123');

        store.clear_all();

        const state = use_trace_store.getState();
        expect(state.spans).toEqual({});
        expect(state.active_trace_id).toBeNull();
    });

    it('merges incoming trace span updates with existing spans in event_bus subscription', () => {
        const store = use_trace_store.getState();
        store.add_span(mock_span_1);

        event_bus.emit_trace({
            id: 'span1',
            status: 'error',
            end_time: 1050
        } as any);

        const state = use_trace_store.getState();
        const updated_span = state.spans['span1'];
        expect(updated_span).toBeDefined();
        expect(updated_span.status).toBe('error');
        expect(updated_span.end_time).toBe(1050);
        expect(updated_span.trace_id).toBe('trace-123'); // Preserved!
        expect(updated_span.name).toBe('test_span');     // Preserved!
        expect(updated_span.agent_id).toBe('agent1');    // Preserved!
    });

    it('updates last_activity_at on touch_activity', () => {
        const store = use_trace_store.getState();
        store.add_span(mock_span_2); // running span

        const initial_activity = use_trace_store.getState().spans['span2'].last_activity_at;
        expect(initial_activity).toBeDefined();

        store.touch_activity('span2');
        const updated_activity = use_trace_store.getState().spans['span2'].last_activity_at;
        expect(updated_activity).toBeGreaterThanOrEqual(initial_activity!);
    });

    it('reaps inactive running spans exceeding timeout_seconds', () => {
        const store = use_trace_store.getState();
        const base_time = 100_000;

        store.add_span({
            ...mock_span_2,
            id: 'span-reap-test',
            start_time: base_time,
            last_activity_at: base_time,
            timeout_seconds: 60,
            status: 'running'
        });

        // Test at base_time + 30s -> should NOT reap
        const reaped_early = store.reap_stale_spans(base_time + 30_000);
        expect(reaped_early).toBe(0);
        expect(use_trace_store.getState().spans['span-reap-test'].status).toBe('running');

        // Test at base_time + 70s -> should reap!
        const reaped_late = store.reap_stale_spans(base_time + 70_000);
        expect(reaped_late).toBe(1);

        const reaped_span = use_trace_store.getState().spans['span-reap-test'];
        expect(reaped_span.status).toBe('error');
        expect(reaped_span.attributes.error).toBe('SPAN_TIMEOUT_REAPED');
    });
});
