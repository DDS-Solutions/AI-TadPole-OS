/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:UI-Components**
 * Resilient Graph Sanitizer.
 * Provides client-side normalization, sanitization, and referential integrity checks.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Fails to sanitize, returns empty list, or drops too many elements.
 * - **Telemetry Link**: Search `[graph_sanitizer]` in observability traces.
 */

import type { SymbolNode } from '../../../services/intelligence_api_service';
import type { ForceGraphLink } from './types';

export interface SanitizerIssue {
    level: 'auto-corrected' | 'dropped';
    category: string;
    message: string;
    path: string;
}

export interface SanitizedGraphResult {
    data: {
        nodes: SymbolNode[];
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

/**
 * Sanitizes and normalizes the raw graph data from the API.
 * Ensures the app doesn't crash on invalid input or broken links.
 */
export const sanitize_graph_data = (raw: unknown): SanitizedGraphResult => {
    const issues: SanitizerIssue[] = [];
    const rawData = raw as Record<string, unknown> | null | undefined;

    // 1. Core Collections Check
    const rawNodes = Array.isArray(rawData?.nodes) ? rawData.nodes : [];
    if (!Array.isArray(rawData?.nodes)) {
        issues.push({
            level: 'auto-corrected',
            category: 'invalid-collection',
            message: 'nodes collection was missing or invalid; initialized as empty array',
            path: 'nodes'
        });
    }

    const rawLinks = Array.isArray(rawData?.links) ? rawData.links : [];
    if (!Array.isArray(rawData?.links)) {
        issues.push({
            level: 'auto-corrected',
            category: 'invalid-collection',
            message: 'links collection was missing or invalid; initialized as empty array',
            path: 'links'
        });
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

    // 2. Validate and Sanitize Nodes
    const validNodes: SymbolNode[] = [];
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
            end_line
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

    return {
        data: {
            nodes: validNodes,
            links: validLinks,
            anomalies: sanitizedAnomalies
        },
        issues
    };
};

// Metadata: [graph_sanitizer]
