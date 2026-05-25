/**
 * @docs ARCHITECTURE:Quality:Verification
 * 
 * ### AI Assist Note
 * **Verification for the Client-side Resilient Graph Sanitizer.**
 * Checks validation, normalization, and referential integrity of symbol graphs.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Unit test failures on sanitization logic.
 * - **Telemetry Link**: Search `[graphSanitizer_test]` in observability traces.
 */

import { describe, it, expect } from 'vitest';
import { sanitize_graph_data } from '../src/components/intelligence/knowledge_graph/graph_sanitizer';

describe('graph_sanitizer', () => {
    it('handles null, undefined, or empty objects gracefully', () => {
        const result = sanitize_graph_data(null);
        expect(result.data.nodes).toEqual([]);
        expect(result.data.links).toEqual([]);
        expect(result.data.anomalies).toEqual([]);
        expect(result.issues.length).toBeGreaterThanOrEqual(3); // nodes, links, anomalies missing
    });

    it('validates nodes and maps kind aliases', () => {
        const rawData = {
            nodes: [
                { name: 'func1', path: 'src/main.rs', kind: 'func', start_line: '10', end_line: 20 },
                { name: 'struct1', path: 'src/main.rs', kind: 'struct', start_line: 25, end_line: 'abc' },
                { name: '', path: 'src/empty.rs', kind: 'fn' } // Invalid: empty name, should be dropped
            ],
            links: [],
            anomalies: []
        };

        const { data, issues } = sanitize_graph_data(rawData);

        // check nodes
        expect(data.nodes).toHaveLength(2);
        
        // Node 0: kind func normalized to function, start_line coerced
        expect(data.nodes[0]).toEqual({
            name: 'func1',
            path: 'src/main.rs',
            kind: 'function',
            signature: '',
            start_line: 10,
            end_line: 20
        });

        // Node 1: kind struct normalized to class, end_line coerced to 0
        expect(data.nodes[1]).toEqual({
            name: 'struct1',
            path: 'src/main.rs',
            kind: 'class',
            signature: '',
            start_line: 25,
            end_line: 0
        });

        // Issues check
        const aliasIssue = issues.find(i => i.category === 'alias' && i.message.includes('func1'));
        expect(aliasIssue).toBeDefined();
        expect(aliasIssue?.level).toBe('auto-corrected');

        const dropIssue = issues.find(i => i.level === 'dropped' && i.category === 'missing-field');
        expect(dropIssue).toBeDefined();
    });

    it('enforces link referential integrity', () => {
        const rawData = {
            nodes: [
                { name: 'A', path: 'src/a.rs', kind: 'class' },
                { name: 'B', path: 'src/b.rs', kind: 'function' }
            ],
            links: [
                { source: 'src/a.rs:A', target: 'src/b.rs:B' }, // valid
                { source: 'src/a.rs:A', target: 'src/c.rs:C' }, // invalid: target doesn't exist
                { source: 'src/d.rs:D', target: 'src/b.rs:B' }, // invalid: source doesn't exist
                { source: '', target: 'src/b.rs:B' } // invalid: missing source
            ]
        };

        const { data, issues } = sanitize_graph_data(rawData);

        expect(data.links).toHaveLength(1);
        expect(data.links[0]).toEqual({ source: 'src/a.rs:A', target: 'src/b.rs:B' });

        const invalidRefIssues = issues.filter(i => i.category === 'invalid-reference');
        expect(invalidRefIssues).toHaveLength(2); // target c.rs:C and source d.rs:D
    });

    it('coerces and normalizes anomalies', () => {
        const rawData = {
            nodes: [],
            links: [],
            anomalies: [
                'Unused function helper()',
                12345 // non-string coerced
            ]
        };

        const { data } = sanitize_graph_data(rawData);
        expect(data.anomalies).toEqual(['Unused function helper()', '12345']);
    });
});

// Metadata: [graphSanitizer_test]
