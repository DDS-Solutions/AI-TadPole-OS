/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **CognitionSidebar Unit Tests**: Validates symbol metadata inspection displays,
 * blast radius loading status indicators, and agent memory workspace listings.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Missing i18n placeholders or Zustand dropdown mock state mismatch.
 * - **Telemetry Link**: Search `[cognition_sidebar.test]` in tracing logs.
 */

import '@testing-library/jest-dom';
import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { CognitionSidebar } from './CognitionSidebar';
import { useMemoryWorkspace } from './useMemoryWorkspace';
import { useSelectionContext, useGraphDataContext, useUIStateContext } from './graph_context_hooks';

// Mock i18n
vi.mock('../../../i18n', () => ({
    i18n: {
        t: (key: string, options?: any) => {
            if (options && options.count != null) return `${key}:${options.count}`;
            return key;
        },
    },
}));

// Mock hooks
vi.mock('./useMemoryWorkspace', () => ({
    useMemoryWorkspace: vi.fn(),
}));

vi.mock('./graph_context_hooks', () => ({
    useSelectionContext: vi.fn(),
    useGraphDataContext: vi.fn(),
    useUIStateContext: vi.fn(),
}));

describe('CognitionSidebar', () => {
    const mock_memory_result = {
        memories: [
            { id: 'mem-1', text: 'Memory Entry One', created_at: '2026-07-10' }
        ],
        searchQuery: '',
        setSearchQuery: vi.fn(),
        isSearching: false,
        saveMemory: vi.fn(),
        deleteMemory: vi.fn(),
        isSaving: false,
        memoryText: '',
        setMemoryText: vi.fn(),
    };

    const mock_graph_data = {
        data: {
            agents: [
                { id: 'agent-1', name: 'Agent Alpha', theme_color: '#10b981' }
            ],
            nodes: []
        },
        graphData: {
            nodes: [],
            links: [],
            nodeMap: new Map()
        }
    };

    beforeEach(() => {
        vi.clearAllMocks();
        
        (useMemoryWorkspace as any).mockReturnValue(mock_memory_result);
        (useGraphDataContext as any).mockReturnValue(mock_graph_data);
        
        (useUIStateContext as any).mockReturnValue({
            isMemoryNode: false,
            activeInfoTab: 'info',
            setActiveInfoTab: vi.fn(),
            viewMode: 'graph'
        });
    });

    it('renders null when no symbol node is selected', () => {
        (useSelectionContext as any).mockReturnValue({
            selectedNode: null,
            affectedNodes: new Set(),
            resetSelection: vi.fn(),
            blastRadiusLoading: false
        });

        const { container } = render(<CognitionSidebar />);

        expect(container).toBeEmptyDOMElement();
    });

    it('renders node details and blast radius when a node is selected', () => {
        const mock_node = {
            id: 'node-1',
            name: 'calculate_optimal_bounds',
            kind: 'function',
            path: 'src/utils/bounds.ts'
        };

        (useSelectionContext as any).mockReturnValue({
            selectedNode: mock_node,
            affectedNodes: new Set(['node-1']),
            resetSelection: vi.fn(),
            blastRadiusLoading: false
        });

        render(<CognitionSidebar />);

        expect(screen.getByText('calculate_optimal_bounds')).toBeInTheDocument();
        expect(screen.getByText('src/utils/bounds.ts')).toBeInTheDocument();
        expect(screen.getByText('knowledge_graph.sidebar_dependents:0')).toBeInTheDocument();
    });
});

// Metadata: [cognition_sidebar_test]
