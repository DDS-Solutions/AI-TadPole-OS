/**
 * @file traceStore.test.ts
 * @description Suite for observability, distributed tracing, and task breakdown.
 * @module Stores/TraceStore
 * @testedBehavior
 * - Tree Construction: Building hierarchical parent-child relationships between spans.
 * - Filtering: Retrieving specific trace branches by trace ID.
 * - Lifecycle: Real-time updates to span status (running -> success/error).
 * @aiContext
 * - Tests recursive tree building logic using mock span hierarchies.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { useTraceStore, type TraceSpan } from './traceStore';

describe('useTraceStore', () => {
    beforeEach(() => {
        useTraceStore.getState().clearAll();
    });

    const mockSpan1: TraceSpan = {
        id: 'span1',
        traceId: 'trace-123',
        name: 'test_span',
        agentId: 'agent1',
        missionId: 'mission1',
        startTime: 1000,
        status: 'success',
        attributes: { key: 'value' }
    };

    const mockSpan2: TraceSpan = {
        id: 'span2',
        traceId: 'trace-123',
        parentId: 'span1',
        name: 'child_span',
        agentId: 'agent1',
        missionId: 'mission1',
        startTime: 1010,
        status: 'running',
        attributes: {}
    };

    const mockSpan3: TraceSpan = {
        id: 'span3',
        traceId: 'trace-other',
        name: 'other_span',
        agentId: 'agent2',
        missionId: 'mission2',
        startTime: 2000,
        status: 'success',
        attributes: {}
    };

    it('adds spans to the store', () => {
        const store = useTraceStore.getState();
        store.addSpan(mockSpan1);

        const state = useTraceStore.getState();
        expect(state.spans['span1']).toEqual(mockSpan1);
    });

    it('updates spans in the store', () => {
        const store = useTraceStore.getState();
        store.addSpan(mockSpan1);

        store.updateSpan('span1', { status: 'error', endTime: 1050 });

        const updatedSpan = useTraceStore.getState().spans['span1'];
        expect(updatedSpan.status).toBe('error');
        expect(updatedSpan.endTime).toBe(1050);
        expect(updatedSpan.name).toBe('test_span'); // Unchanged prop
    });

    it('does not update non-existent spans', () => {
        const store = useTraceStore.getState();
        store.addSpan(mockSpan1);

        store.updateSpan('does-not-exist', { status: 'error' });

        expect(useTraceStore.getState().spans).toEqual({ 'span1': mockSpan1 });
    });

    it('sets the active trace', () => {
        const store = useTraceStore.getState();
        store.setActiveTrace('trace-123');

        expect(useTraceStore.getState().activeTraceId).toBe('trace-123');
    });

    it('builds a trace tree correctly', () => {
        const store = useTraceStore.getState();
        store.addSpan(mockSpan1);
        store.addSpan(mockSpan2);
        store.addSpan(mockSpan3);

        const tree = store.getTraceTree('trace-123');

        expect(tree).toHaveLength(1); // Only root node should be at top level
        expect(tree[0].id).toBe('span1');
        expect(tree[0].children).toHaveLength(1);
        expect(tree[0].children[0].id).toBe('span2');
    });

    it('clears spans for a specific trace', () => {
        const store = useTraceStore.getState();
        store.addSpan(mockSpan1);
        store.addSpan(mockSpan2);
        store.addSpan(mockSpan3);
        store.setActiveTrace('trace-123');

        store.clearTrace('trace-123');

        const state = useTraceStore.getState();
        expect(state.spans).toEqual({ 'span3': mockSpan3 });
        expect(state.activeTraceId).toBeNull(); // Should clear active trace if it matches
    });

    it('does not clear active trace if clearing a different trace', () => {
        const store = useTraceStore.getState();
        store.addSpan(mockSpan1);
        store.addSpan(mockSpan3);
        store.setActiveTrace('trace-123');

        store.clearTrace('trace-other');

        const state = useTraceStore.getState();
        expect(state.spans).toEqual({ 'span1': mockSpan1 });
        expect(state.activeTraceId).toBe('trace-123'); 
    });

    it('clears all spans and active trace', () => {
        const store = useTraceStore.getState();
        store.addSpan(mockSpan1);
        store.setActiveTrace('trace-123');

        store.clearAll();

        const state = useTraceStore.getState();
        expect(state.spans).toEqual({});
        expect(state.activeTraceId).toBeNull();
    });
});
