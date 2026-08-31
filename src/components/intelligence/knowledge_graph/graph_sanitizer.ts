/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Intelligence / graph_sanitizer
 * - **Primary Entrypoints**: `sanitize_graph_data`, `SanitizerIssue`, `SanitizedSymbolNode`, `SanitizedGraphResult`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import type { SymbolNode } from '../../../services/intelligence_api_service';
import type { ForceGraphLink, ExtendedGraphNode } from './types';

export interface SanitizerIssue {
    level: 'auto-corrected' | 'dropped';
    category: string;
    message: string;
    path: string;
}

export interface SanitizedSymbolNode extends SymbolNode {
    community?: number;
}

export interface SanitizedGraphResult {
    data: {
        nodes: SanitizedSymbolNode[];
        links: ForceGraphLink[];
        anomalies: string[];
    };
    issues: SanitizerIssue[];
}

const KIND_ALIASES: Record<string, string> = {
    'func': 'function',
    'fn': 'function',
    'method': 'function',
    'struct': 'class',
    'interface': 'trait',
};

// Seeded Mulberry32 PRNG for deterministic community assignments
const mulberry32 = (seed: number) => {
    return () => {
        let t = seed += 0x6D2B79F5;
        t = Math.imul(t ^ (t >>> 15), t | 1);
        t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
};

/**
 * Sanitizes and normalizes the raw graph data from the API.
 * Ensures the app doesn't crash on invalid input or broken links.
 */
export const sanitize_graph_data = (raw: unknown): SanitizedGraphResult => {
    const issues: SanitizerIssue[] = [];
    const rawData = raw as Record<string, unknown> | null | undefined;

    // Hard limits on input sizes for DoS mitigation
    const MAX_NODES = 10000;
    const MAX_LINKS = 50000;

    // 1. Core Collections Check & DoS Truncation
    let rawNodes = Array.isArray(rawData?.nodes) ? rawData.nodes : [];
    if (!Array.isArray(rawData?.nodes)) {
        issues.push({
            level: 'auto-corrected',
            category: 'invalid-collection',
            message: 'nodes collection was missing or invalid; initialized as empty array',
            path: 'nodes'
        });
    }
    if (rawNodes.length > MAX_NODES) {
        issues.push({
            level: 'dropped',
            category: 'dos-limit',
            message: `nodes collection exceeded maximum limit of ${MAX_NODES}; truncated from ${rawNodes.length}`,
            path: 'nodes'
        });
        rawNodes = rawNodes.slice(0, MAX_NODES);
    }

    let rawLinks = Array.isArray(rawData?.links) ? rawData.links : [];
    if (!Array.isArray(rawData?.links)) {
        issues.push({
            level: 'auto-corrected',
            category: 'invalid-collection',
            message: 'links collection was missing or invalid; initialized as empty array',
            path: 'links'
        });
    }
    if (rawLinks.length > MAX_LINKS) {
        issues.push({
            level: 'dropped',
            category: 'dos-limit',
            message: `links collection exceeded maximum limit of ${MAX_LINKS}; truncated from ${rawLinks.length}`,
            path: 'links'
        });
        rawLinks = rawLinks.slice(0, MAX_LINKS);
    }

    const rawAnomalies = Array.isArray(rawData?.anomalies) ? rawData.anomalies : [];
    if (!Array.isArray(rawData?.anomalies)) {
        issues.push({
            level: 'auto-corrected',
            category: 'invalid-collection',
            message: 'anomalies collection was missing or invalid; initialized as empty array',
            path: 'anomalies'
        });
    }

    // Strict XSS/Injection validation check using a safe character allowlist
    const SAFE_IDENTIFIER = /^[a-zA-Z0-9_./\\:\-@[\]#~\s]{1,300}$/;
    const isSafeString = (str: string) => {
        return SAFE_IDENTIFIER.test(str);
    };

    // 2. Validate and Sanitize Nodes
    const validNodes: SanitizedSymbolNode[] = [];
    rawNodes.forEach((node: unknown, idx: number) => {
        if (!node || typeof node !== 'object') {
            issues.push({
                level: 'dropped',
                category: 'invalid-node',
                message: `nodes[${idx}] was not a valid object; dropped`,
                path: `nodes[${idx}]`
            });
            return;
        }

        const nodeObj = node as Record<string, unknown>;
        const name = typeof nodeObj.name === 'string' ? nodeObj.name.trim() : '';
        const path = typeof nodeObj.path === 'string' ? nodeObj.path.trim() : '';

        if (!name) {
            issues.push({
                level: 'dropped',
                category: 'missing-field',
                message: `nodes[${idx}] is missing a valid "name"; dropped`,
                path: `nodes[${idx}].name`
            });
            return;
        }

        if (!isSafeString(name) || !isSafeString(path)) {
            issues.push({
                level: 'dropped',
                category: 'security-validation',
                message: `Node "${name}" contains forbidden injection sequences or invalid characters; dropped for security`,
                path: `nodes[${idx}]`
            });
            return;
        }

        if (!path) {
            issues.push({
                level: 'auto-corrected',
                category: 'missing-field',
                message: `Node "${name}" has missing "path"; defaulted to empty string`,
                path: `nodes[${idx}].path`
            });
        }

        // Normalize Kind
        let kind = typeof nodeObj.kind === 'string' ? nodeObj.kind.trim().toLowerCase() : 'unknown';
        if (KIND_ALIASES[kind]) {
            const original = kind;
            kind = KIND_ALIASES[kind];
            issues.push({
                level: 'auto-corrected',
                category: 'alias',
                message: `Node "${name}": kind "${original}" normalized to "${kind}"`,
                path: `nodes[${idx}].kind`
            });
        }

        // Coerce Line Ranges
        let start_line = typeof nodeObj.start_line === 'number' 
            ? nodeObj.start_line 
            : parseInt(String(nodeObj.start_line), 10);
        if (isNaN(start_line)) {
            start_line = 0;
            issues.push({
                level: 'auto-corrected',
                category: 'type-coercion',
                message: `Node "${name}": start_line was invalid; defaulted to 0`,
                path: `nodes[${idx}].start_line`
            });
        }

        let end_line = typeof nodeObj.end_line === 'number' 
            ? nodeObj.end_line 
            : parseInt(String(nodeObj.end_line), 10);
        if (isNaN(end_line)) {
            end_line = 0;
            issues.push({
                level: 'auto-corrected',
                category: 'type-coercion',
                message: `Node "${name}": end_line was invalid; defaulted to 0`,
                path: `nodes[${idx}].end_line`
            });
        }

        const signature = typeof nodeObj.signature === 'string' ? nodeObj.signature : '';

        validNodes.push({
            name,
            path,
            kind,
            signature,
            start_line,
            end_line,
            docstring: null,
            docstring_range: null,
        });
    });

    // 3. Validate Links (Strict String IDs)
    const validNodeIds = new Set(validNodes.map(n => `${n.path}:${n.name}`));
    const validLinks: ForceGraphLink[] = [];

    rawLinks.forEach((link: unknown, idx: number) => {
        if (!link || typeof link !== 'object') {
            issues.push({
                level: 'dropped',
                category: 'invalid-link',
                message: `links[${idx}] was not a valid object; dropped`,
                path: `links[${idx}]`
            });
            return;
        }

        const linkObj = link as Record<string, unknown>;
        const source = typeof linkObj.source === 'string' ? linkObj.source.trim() : '';
        const target = typeof linkObj.target === 'string' ? linkObj.target.trim() : '';

        if (!source || !target) {
            issues.push({
                level: 'dropped',
                category: 'missing-field',
                message: `links[${idx}]: missing source or target; dropped`,
                path: `links[${idx}]`
            });
            return;
        }

        if (!validNodeIds.has(source)) {
            issues.push({
                level: 'dropped',
                category: 'invalid-reference',
                message: `links[${idx}]: source "${source}" does not exist in nodes list; dropped link`,
                path: `links[${idx}].source`
            });
            return;
        }

        if (!validNodeIds.has(target)) {
            issues.push({
                level: 'dropped',
                category: 'invalid-reference',
                message: `links[${idx}]: target "${target}" does not exist in nodes list; dropped link`,
                path: `links[${idx}].target`
            });
            return;
        }

        validLinks.push({ source, target });
    });

    const sanitizedAnomalies = rawAnomalies.map((a: unknown) => typeof a === 'string' ? a : String(a));

    // Run Label Propagation Algorithm for community clustering - deterministic & non-mutating
    const communityMap = compute_communities(validNodes, validLinks);

    const nodesWithCommunities = validNodes.map(node => ({
        ...node,
        community: communityMap.get(`${node.path}:${node.name}`) ?? 0
    }));

    return {
        data: {
            nodes: nodesWithCommunities,
            links: validLinks,
            anomalies: sanitizedAnomalies
        },
        issues
    };
};

/**
 * Computes community modularity using a lightweight Label Propagation Algorithm (LPA).
 * Features hub node exclusion and re-attachment to prevent network bridging collapse.
 * Returns a Map of node ID (path:name) to computed community ID.
 */
const compute_communities = (nodes: SanitizedSymbolNode[], links: ForceGraphLink[]): Map<string, number> => {
    const communityMap = new Map<string, number>();
    if (nodes.length === 0) return communityMap;

    const rng = mulberry32(1337);

    // 1. Build adjacency list and map node IDs (path:name) to node objects
    const nodeDegrees = new Map<string, number>();
    const adj = new Map<string, string[]>();

    nodes.forEach(node => {
        const id = `${node.path}:${node.name}`;
        nodeDegrees.set(id, 0);
        adj.set(id, []);
    });

    links.forEach(link => {
        const u = typeof link.source === 'string' ? link.source : (link.source as ExtendedGraphNode).id;
        const v = typeof link.target === 'string' ? link.target : (link.target as ExtendedGraphNode).id;
        if (adj.has(u) && adj.has(v)) {
            adj.get(u)!.push(v);
            adj.get(v)!.push(u);
            nodeDegrees.set(u, (nodeDegrees.get(u) || 0) + 1);
            nodeDegrees.set(v, (nodeDegrees.get(v) || 0) + 1);
        }
    });

    // 2. Hub Exclusion: identify hub nodes (degree > 15)
    const hubThreshold = 15;
    const hubNodes = new Set<string>();
    nodes.forEach(node => {
        const id = `${node.path}:${node.name}`;
        if ((nodeDegrees.get(id) || 0) > hubThreshold) {
            hubNodes.add(id);
        }
    });

    // 3. Initialize communities for non-hub nodes
    let communityCounter = 0;
    nodes.forEach(node => {
        const id = `${node.path}:${node.name}`;
        if (!hubNodes.has(id)) {
            communityMap.set(id, communityCounter++);
        }
    });

    // 4. Label Propagation Iterations
    const iterations = 8;
    // Alphabetically sort non-hub node IDs prior to shuffling to guarantee ordering determinism
    const nonHubIds = nodes
        .map(node => `${node.path}:${node.name}`)
        .filter(id => !hubNodes.has(id))
        .sort();

    for (let iter = 0; iter < iterations; iter++) {
        // Shuffle node list deterministically to avoid ordering bias
        for (let i = nonHubIds.length - 1; i > 0; i--) {
            const j = Math.floor(rng() * (i + 1));
            [nonHubIds[i], nonHubIds[j]] = [nonHubIds[j], nonHubIds[i]];
        }

        let changed = false;
        nonHubIds.forEach(id => {
            const neighbors = adj.get(id) || [];
            const neighborLabels = neighbors
                .filter(nb => !hubNodes.has(nb)) // ignore hub neighbors during propagation
                .map(nb => communityMap.get(nb))
                .filter((label): label is number => label !== undefined);

            if (neighborLabels.length === 0) return;

            // Count label frequencies
            const counts = new Map<number, number>();
            let maxCount = 0;
            let bestLabels: number[] = [];

            neighborLabels.forEach(label => {
                const count = (counts.get(label) || 0) + 1;
                counts.set(label, count);
                if (count > maxCount) {
                    maxCount = count;
                    bestLabels = [label];
                } else if (count === maxCount) {
                    bestLabels.push(label);
                }
            });

            if (bestLabels.length > 0) {
                const currentLabel = communityMap.get(id)!;
                let bestLabel: number;
                if (bestLabels.includes(currentLabel)) {
                    bestLabel = currentLabel;
                } else {
                    bestLabel = bestLabels[Math.floor(rng() * bestLabels.length)];
                }

                if (communityMap.get(id) !== bestLabel) {
                    communityMap.set(id, bestLabel);
                    changed = true;
                }
            }
        });

        if (!changed) break; // early convergence
    }

    // 5. Hub Node Re-attachment via majority vote of neighbors
    hubNodes.forEach(hubId => {
        const neighbors = adj.get(hubId) || [];
        const neighborLabels = neighbors
            .map(nb => communityMap.get(nb))
            .filter((label): label is number => label !== undefined);

        if (neighborLabels.length === 0) {
            communityMap.set(hubId, communityCounter++);
            return;
        }

        const counts = new Map<number, number>();
        let maxCount = 0;
        let bestLabels: number[] = [];

        neighborLabels.forEach(label => {
            const count = (counts.get(label) || 0) + 1;
            counts.set(label, count);
            if (count > maxCount) {
                maxCount = count;
                bestLabels = [label];
            } else if (count === maxCount) {
                bestLabels.push(label);
            }
        });

        const bestLabel = bestLabels.length > 0
            ? bestLabels[Math.floor(rng() * bestLabels.length)]
            : communityCounter++;

        communityMap.set(hubId, bestLabel);
    });

    // 6. Map internal labels to dense, sequential IDs starting from 0 (ordered by size)
    const communitySizes = new Map<number, number>();
    communityMap.forEach((label) => {
        communitySizes.set(label, (communitySizes.get(label) || 0) + 1);
    });

    const sortedLabels = Array.from(communitySizes.keys())
        .sort((a, b) => (communitySizes.get(b) || 0) - (communitySizes.get(a) || 0));

    const labelToNewId = new Map<number, number>();
    sortedLabels.forEach((label, idx) => {
        labelToNewId.set(label, idx);
    });

    // 7. Map each node's ID to its final sequential community ID
    const resultMap = new Map<string, number>();
    nodes.forEach(node => {
        const id = `${node.path}:${node.name}`;
        const label = communityMap.get(id);
        const communityId = label !== undefined ? labelToNewId.get(label) : 0;
        resultMap.set(id, communityId ?? 0);
    });

    return resultMap;
};
