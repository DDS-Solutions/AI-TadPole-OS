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
            end_line: 20,
            community: expect.any(Number)
        });

        // Node 1: kind struct normalized to class, end_line coerced to 0
        expect(data.nodes[1]).toEqual({
            name: 'struct1',
            path: 'src/main.rs',
            kind: 'class',
            signature: '',
            start_line: 25,
            end_line: 0,
            community: expect.any(Number)
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

    it('computes communities using Label Propagation Algorithm', () => {
        const rawData = {
            nodes: [
                { name: 'A', path: 'src/a.rs', kind: 'class' },
                { name: 'B', path: 'src/a.rs', kind: 'function' },
                { name: 'C', path: 'src/a.rs', kind: 'function' },
                { name: 'D', path: 'src/b.rs', kind: 'class' },
                { name: 'E', path: 'src/b.rs', kind: 'function' },
                { name: 'F', path: 'src/b.rs', kind: 'function' }
            ],
            links: [
                // Cluster 1 (A, B, C)
                { source: 'src/a.rs:A', target: 'src/a.rs:B' },
                { source: 'src/a.rs:A', target: 'src/a.rs:C' },
                { source: 'src/a.rs:B', target: 'src/a.rs:C' },
                
                // Bridge link
                { source: 'src/a.rs:C', target: 'src/b.rs:D' },
                
                // Cluster 2 (D, E, F)
                { source: 'src/b.rs:D', target: 'src/b.rs:E' },
                { source: 'src/b.rs:D', target: 'src/b.rs:F' },
                { source: 'src/b.rs:E', target: 'src/b.rs:F' }
            ],
            anomalies: []
        };

        const { data } = sanitize_graph_data(rawData);
        
        expect(data.nodes).toHaveLength(6);
        
        // Retrieve communities
        const commA = data.nodes.find(n => n.name === 'A')?.community;
        const commB = data.nodes.find(n => n.name === 'B')?.community;
        const commC = data.nodes.find(n => n.name === 'C')?.community;
        const commD = data.nodes.find(n => n.name === 'D')?.community;
        const commE = data.nodes.find(n => n.name === 'E')?.community;
        const commF = data.nodes.find(n => n.name === 'F')?.community;

        expect(commA).toBeDefined();
        expect(commB).toBeDefined();
        expect(commC).toBeDefined();
        expect(commD).toBeDefined();
        expect(commE).toBeDefined();
        expect(commF).toBeDefined();

        // Nodes within Cluster 1 should belong to the same community
        expect(commA).toEqual(commB);
        expect(commB).toEqual(commC);

        // Nodes within Cluster 2 should belong to the same community
        expect(commD).toEqual(commE);
        expect(commE).toEqual(commF);

        // The two clusters should form different communities
        expect(commA).not.toEqual(commD);
    });

    it('rejects nodes with script injection attempts', () => {
        const rawData = {
            nodes: [
                { name: '<script>alert(1)</script>', path: 'src/a.rs', kind: 'class' },
                { name: 'funcA', path: 'src/b.rs', kind: 'function' },
                { name: 'funcB', path: 'src/c.rs" onload="alert(1)', kind: 'function' }
            ],
            links: [],
            anomalies: []
        };

        const { data, issues } = sanitize_graph_data(rawData);
        
        expect(data.nodes).toHaveLength(1);
        expect(data.nodes[0].name).toBe('funcA');

        const securityIssues = issues.filter(i => i.category === 'security-validation');
        expect(securityIssues).toHaveLength(2);
    });

    it('guarantees deterministic community detection', () => {
        const rawData = {
            nodes: [
                { name: 'A', path: 'src/a.rs' },
                { name: 'B', path: 'src/a.rs' },
                { name: 'C', path: 'src/a.rs' }
            ],
            links: [
                { source: 'src/a.rs:A', target: 'src/a.rs:B' },
                { source: 'src/a.rs:B', target: 'src/a.rs:C' }
            ],
            anomalies: []
        };

        const runs = Array.from({ length: 10 }, () => sanitize_graph_data(rawData));
        const firstRunResults = runs[0].data.nodes.map(n => n.community);

        runs.forEach((run) => {
            const runResults = run.data.nodes.map(n => n.community);
            expect(runResults).toEqual(firstRunResults);
        });
    });

    it('enforces DoS limits on nodes and links collections', () => {
        const excessNodes = Array.from({ length: 10005 }, (_, i) => ({
            name: `Node${i}`,
            path: 'src/dos.rs'
        }));
        const excessLinks = Array.from({ length: 50005 }, () => ({
            source: 'src/dos.rs:Node0',
            target: 'src/dos.rs:Node1'
        }));

        const rawData = {
            nodes: excessNodes,
            links: excessLinks,
            anomalies: []
        };

        const { data, issues } = sanitize_graph_data(rawData);
        expect(data.nodes).toHaveLength(10000);
        expect(data.links).toHaveLength(50000);

        const dosIssues = issues.filter(i => i.category === 'dos-limit');
        expect(dosIssues).toHaveLength(2);
    });

    it('does not mutate input node objects by reference', () => {
        const rawNode = { name: 'A', path: 'src/a.rs' };
        const rawData = {
            nodes: [rawNode],
            links: [],
            anomalies: []
        };

        const { data } = sanitize_graph_data(rawData);
        expect(rawNode).not.toHaveProperty('community');
        expect(data.nodes[0].community).toBeDefined();
    });

    it('blocks custom HTML tag and script injections using a strict allowlist but allows safe symbol characters', () => {
        const rawData = {
            nodes: [
                { name: '<img src=x onerror=alert(1)>', path: 'src/a.rs' },
                { name: 'javascript:alert(1)', path: 'src/a.rs' },
                { name: 'A', path: 'src/a.rs' },
                { name: 'B', path: 'src/b.rs;background:url(javascript:alert(1))' },
                { name: 'my_func', path: 'src/app.rs' },
                { name: 'MyStruct', path: 'src/utils.rs' }
            ],
            links: []
        };

        const { data, issues } = sanitize_graph_data(rawData);
        expect(data.nodes).toHaveLength(3);
        expect(data.nodes.map(n => n.name)).toContain('A');
        expect(data.nodes.map(n => n.name)).toContain('my_func');
        expect(data.nodes.map(n => n.name)).toContain('MyStruct');

        const securityIssues = issues.filter(i => i.category === 'security-validation');
        expect(securityIssues).toHaveLength(3);
    });
});

// Metadata: [graphSanitizer_test]

