/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Intelligence / KnowledgeGraph.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { KnowledgeGraph } from './KnowledgeGraph';
import { intelligence_api_service } from '../../services/intelligence_api_service';

// Mock react-force-graph-2d
vi.mock('react-force-graph-2d', () => ({
    default: React.forwardRef(({ graphData }: any, ref: any) => {
        React.useImperativeHandle(ref, () => ({
            centerAt: vi.fn(),
            zoom: vi.fn().mockReturnValue(1),
            zoomToFit: vi.fn(),
            graphData: () => graphData,
        }));
        return (
            <div data-testid="mock-graph-canvas">
                <div data-testid="graph-nodes-count">{graphData?.nodes?.length ?? 0}</div>
            </div>
        );
    }),
}));

// Mock intelligence API service
vi.mock('../../services/intelligence_api_service', () => ({
    intelligence_api_service: {
        get_code_graph: vi.fn(),
        get_markdown_memory_graph: vi.fn(),
        get_blast_radius: vi.fn(),
        get_knowledge_peers: vi.fn(),
    },
}));

describe('KnowledgeGraph Component', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        (intelligence_api_service.get_code_graph as any).mockResolvedValue({
            nodes: [
                { id: 'sym-1', name: 'AppRouter', path: 'src/router.rs', kind: 'struct' },
                { id: 'sym-2', name: 'AuthHandler', path: 'src/auth.rs', kind: 'function' },
            ],
            edges: [
                { source: 'sym-1', target: 'sym-2', edge_type: 'calls' },
            ],
        });
        (intelligence_api_service.get_markdown_memory_graph as any).mockResolvedValue({
            nodes: [
                { id: 'mem-1', label: 'system_architecture.md', path: '.agent/memory/system.md', link_count: 1 },
            ],
            edges: [],
        });
    });

    it('renders the graph HUD with Symbols, Memory, and Knowledge mode buttons', async () => {
        render(<KnowledgeGraph />);

        await waitFor(() => {
            expect(screen.getByRole('button', { name: /symbols/i })).toBeDefined();
            expect(screen.getByRole('button', { name: /memory/i })).toBeDefined();
            expect(screen.getByRole('button', { name: /knowledge/i })).toBeDefined();
        });
    });

    it('switches between Symbols and Memory view modes on user interaction', async () => {
        render(<KnowledgeGraph />);

        // Wait for initial code graph load
        await waitFor(() => {
            expect(intelligence_api_service.get_code_graph).toHaveBeenCalled();
        });

        // Click Memory mode
        const memoryBtn = screen.getByRole('button', { name: /memory/i });
        fireEvent.click(memoryBtn);

        await waitFor(() => {
            expect(intelligence_api_service.get_markdown_memory_graph).toHaveBeenCalled();
        });
    });
});
