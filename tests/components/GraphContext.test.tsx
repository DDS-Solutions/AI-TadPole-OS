/**
 * @docs ARCHITECTURE:Quality:Verification
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[GraphContext_test]` in observability traces.
 */

import React, { useEffect } from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import { GraphProvider } from '../../src/components/intelligence/knowledge_graph/GraphContext';
import { 
    useGraphDataContext, 
    useSelectionContext, 
    useNavigationContext, 
    useViewportContext 
} from '../../src/components/intelligence/knowledge_graph/graph_context_hooks';
import { intelligence_api_service } from '../../src/services/intelligence_api_service';

// Mock the intelligence api service
vi.mock('../../src/services/intelligence_api_service', () => ({
    intelligence_api_service: {
        get_code_graph: vi.fn(),
        get_blast_radius: vi.fn()
    }
}));

const TestComponent: React.FC = () => {
    const { loading, graphData } = useGraphDataContext();
    const { selectedNode, affectedNodes, selectNode } = useSelectionContext();
    const { nodeHistory, historyIndex, goBack, goForward } = useNavigationContext();
    const { zoomIn, zoomOut, zoomFit, exportPNG, fgRef } = useViewportContext();

    // Set mock fgRef elements
    useEffect(() => {
        fgRef.current = {
            centerAt: vi.fn(),
            zoom: vi.fn().mockReturnValue(1),
            zoomToFit: vi.fn(),
            canvasElement: vi.fn().mockReturnValue({
                toDataURL: vi.fn().mockReturnValue('data:image/png;base64,mock')
            })
        } as any;
    }, [fgRef]);

    if (loading) {
        return <div data-testid="loading">Loading...</div>;
    }

    return (
        <div>
            <div data-testid="nodes-count">{graphData.nodes.length}</div>
            <div data-testid="selected-node">{selectedNode?.name || 'none'}</div>
            <div data-testid="affected-count">{affectedNodes.size}</div>
            <div data-testid="history-len">{nodeHistory.length}</div>
            <div data-testid="history-index">{historyIndex}</div>
            
            <button data-testid="select-btn" onClick={() => selectNode(graphData.nodes[0])}>
                Select Node
            </button>
            <button data-testid="select-btn-2" onClick={() => selectNode(graphData.nodes[1])}>
                Select Node 2
            </button>
            <button data-testid="back-btn" onClick={goBack}>
                Back
            </button>
            <button data-testid="forward-btn" onClick={goForward}>
                Forward
            </button>
            <button data-testid="zoomin-btn" onClick={zoomIn}>
                Zoom In
            </button>
            <button data-testid="zoomout-btn" onClick={zoomOut}>
                Zoom Out
            </button>
            <button data-testid="zoomfit-btn" onClick={zoomFit}>
                Zoom Fit
            </button>
            <button data-testid="export-btn" onClick={exportPNG}>
                Export
            </button>
        </div>
    );
};

describe('GraphContext', () => {
    const mockGraph = {
        nodes: [
            { name: 'NodeA', path: 'src/a.rs', kind: 'class', start_line: 0, end_line: 10 },
            { name: 'NodeB', path: 'src/b.rs', kind: 'func', start_line: 5, end_line: 15 }
        ],
        links: [
            { source: 'src/a.rs:NodeA', target: 'src/b.rs:NodeB' }
        ],
        anomalies: []
    };

    beforeEach(() => {
        vi.clearAllMocks();
        (intelligence_api_service.get_code_graph as any).mockResolvedValue(mockGraph);
        (intelligence_api_service.get_blast_radius as any).mockResolvedValue([]);
    });

    it('initializes and loads graph data', async () => {
        render(
            <GraphProvider>
                <TestComponent />
            </GraphProvider>
        );

        expect(screen.getByTestId('loading')).toBeInTheDocument();

        await waitFor(() => {
            expect(screen.queryByTestId('loading')).not.toBeInTheDocument();
        });

        expect(screen.getByTestId('nodes-count')).toHaveTextContent('2');
        expect(screen.getByTestId('selected-node')).toHaveTextContent('none');
    });

    it('selects a node, updates selection, fetches blast radius, and updates history', async () => {
        (intelligence_api_service.get_blast_radius as any).mockResolvedValue([
            { name: 'NodeB', path: 'src/b.rs', kind: 'func' }
        ]);

        render(
            <GraphProvider>
                <TestComponent />
            </GraphProvider>
        );

        await waitFor(() => {
            expect(screen.queryByTestId('loading')).not.toBeInTheDocument();
        });

        const selectBtn = screen.getByTestId('select-btn');
        await act(async () => {
            fireEvent.click(selectBtn);
        });

        await waitFor(() => {
            expect(screen.getByTestId('selected-node')).toHaveTextContent('NodeA');
            expect(screen.getByTestId('affected-count')).toHaveTextContent('1');
            expect(screen.getByTestId('history-len')).toHaveTextContent('1');
            expect(screen.getByTestId('history-index')).toHaveTextContent('0');
        });
    });

    it('navigates history back and forward', async () => {
        render(
            <GraphProvider>
                <TestComponent />
            </GraphProvider>
        );

        await waitFor(() => {
            expect(screen.queryByTestId('loading')).not.toBeInTheDocument();
        });

        // Select Node 1
        await act(async () => {
            fireEvent.click(screen.getByTestId('select-btn'));
        });
        
        await waitFor(() => {
            expect(screen.getByTestId('selected-node')).toHaveTextContent('NodeA');
        });

        // Select Node 2
        await act(async () => {
            fireEvent.click(screen.getByTestId('select-btn-2'));
        });

        await waitFor(() => {
            expect(screen.getByTestId('selected-node')).toHaveTextContent('NodeB');
            expect(screen.getByTestId('history-len')).toHaveTextContent('2');
            expect(screen.getByTestId('history-index')).toHaveTextContent('1');
        });

        // Go Back
        await act(async () => {
            fireEvent.click(screen.getByTestId('back-btn'));
        });

        await waitFor(() => {
            expect(screen.getByTestId('selected-node')).toHaveTextContent('NodeA');
            expect(screen.getByTestId('history-index')).toHaveTextContent('0');
        });

        // Go Forward
        await act(async () => {
            fireEvent.click(screen.getByTestId('forward-btn'));
        });

        await waitFor(() => {
            expect(screen.getByTestId('selected-node')).toHaveTextContent('NodeB');
            expect(screen.getByTestId('history-index')).toHaveTextContent('1');
        });
    });

    it('aborts rapid-fire selectNode API requests to avoid race conditions', async () => {
        let resolveRequestA: (value: any) => void = () => {};
        const promiseA = new Promise((resolve) => {
            resolveRequestA = resolve;
        });

        (intelligence_api_service.get_blast_radius as any)
            .mockImplementationOnce(() => promiseA) // Node A blast radius
            .mockResolvedValueOnce([]); // Node B blast radius (immediate)

        render(
            <GraphProvider>
                <TestComponent />
            </GraphProvider>
        );

        await waitFor(() => {
            expect(screen.queryByTestId('loading')).not.toBeInTheDocument();
        });

        // Click Node A (takes time)
        await act(async () => {
            fireEvent.click(screen.getByTestId('select-btn'));
        });

        // Click Node B immediately
        await act(async () => {
            fireEvent.click(screen.getByTestId('select-btn-2'));
        });

        // Resolve Node A request
        await act(async () => {
            resolveRequestA([{ name: 'NodeB', path: 'src/b.rs', kind: 'func' }]);
        });

        // Verify that the final selected node and affected count map to Node B (aborted A request does not overwrite)
        await waitFor(() => {
            expect(screen.getByTestId('selected-node')).toHaveTextContent('NodeB');
            expect(screen.getByTestId('affected-count')).toHaveTextContent('0');
        });
    });
});

// Metadata: [GraphContext_test]
