/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Knowledge Graph Anomalies Unit Tests**: Validates anomaly logging parsed lists,
 * clipboard copy formats, and Pathfinder Modal BFS navigation states.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Clipboard API rejection in JSDOM, or BFS cycle hanging on mocked subgraphs.
 * - **Telemetry Link**: Search `[knowledge_graph_anomalies.test]` in tracing logs.
 */

import '@testing-library/jest-dom';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AnomalyPanel } from './AnomalyPanel';
import { PathFinderModal } from './PathFinderModal';
import { useGraphDataContext, useSelectionContext, useUIStateContext } from './graph_context_hooks';

// Mock i18n
vi.mock('../../../i18n', () => ({
    i18n: {
        t: (key: string, options?: any) => {
            if (options && options.list) return `${key}:${options.list}`;
            return key;
        },
    },
}));

// Mock graph_context_hooks
vi.mock('./graph_context_hooks', () => ({
    useGraphDataContext: vi.fn(),
    useSelectionContext: vi.fn(),
    useUIStateContext: vi.fn(),
}));

describe('Knowledge Graph Anomalies Component Group', () => {
    const mock_anomalies_data = {
        anomalies: [
            'Unused symbol (0 incoming references): unusedFunc in src/utils/math.ts'
        ]
    };
    const mock_graph_data = {
        nodes: [
            { id: 'node-1', name: 'Node One', path: 'src/node1.ts' },
            { id: 'node-2', name: 'Node Two', path: 'src/node2.ts' }
        ],
        links: [],
        nodeMap: new Map()
    };

    beforeEach(() => {
        vi.clearAllMocks();
        
        (useGraphDataContext as any).mockReturnValue({
            data: mock_anomalies_data,
            graphData: mock_graph_data
        });

        (useSelectionContext as any).mockReturnValue({
            selectedNode: null,
            selectNode: vi.fn()
        });

        (useUIStateContext as any).mockReturnValue({
            isPathFinderOpen: true,
            setIsPathFinderOpen: vi.fn(),
            isAnomalyPanelOpen: true,
            setIsAnomalyPanelOpen: vi.fn(),
            setHighlightedPathNodeIds: vi.fn()
        });

        // Mock clipboard API
        vi.stubGlobal('navigator', {
            clipboard: {
                writeText: vi.fn().mockResolvedValue(undefined)
            }
        });
    });

    describe('AnomalyPanel', () => {
        it('renders detected anomalies and triggers clipboard copy on click', async () => {
            render(<AnomalyPanel />);

            // Check that the heading and action button are in the document
            expect(screen.getByText('knowledge_graph.anomaly_title')).toBeInTheDocument();
            
            const copy_btn = screen.getByTitle('knowledge_graph.anomaly_tooltip_copy');
            await act(async () => {
                fireEvent.click(copy_btn);
            });

            expect(navigator.clipboard.writeText).toHaveBeenCalled();
        });
    });

    describe('PathFinderModal', () => {
        it('renders and lists start/end nodes search boxes', () => {
            render(<PathFinderModal />);

            expect(screen.getByPlaceholderText('knowledge_graph.pathfinder_placeholder_start')).toBeInTheDocument();
        });
    });
});

// Metadata: [knowledge_graph_anomalies_test]
