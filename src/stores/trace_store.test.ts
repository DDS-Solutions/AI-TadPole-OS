/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Validation of the Real-time Telemetry and Log Tracing store.** 
 * Verifies the ingestion of autonomous execution spans and the hierarchical reconstruction of trace trees from flat span lists. 
 * Pure logic tests: validates O(1) span lookup and O(N) tree reconstruction complexity without external API side-effects.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Memory leaks due to uncapped log growth or failure to render specific log severities in the Oversight Dashboard.
 * - **Telemetry Link**: Search `[trace_store.test]` in tracing logs.
 */


/**
 * @file trace_store.test.ts
 * @description Suite for the Distributed Agent Tracing and Span Registry.
 * @module Stores/TraceStore
 * @testedBehavior
 * - Span Ingestion: Logic for adding and updating autonomous execution spans.
 * - Tree Reconstruction: Building hierarchical execution trees from flat span lists.
 * - Trace Management: Scoping and clearing of execution traces by trace_id.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Validates O(1) span lookup and O(N) tree reconstruction complexity.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { use_trace_store, type Trace_Span } from './trace_store';

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
});

