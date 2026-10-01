/**
 * @docs ARCHITECTURE:Telemetry
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / General / Telemetry_Graph.test
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
import { Telemetry_Graph } from './Telemetry_Graph';
import { use_trace_store } from '../stores/trace_store';
import { use_agent_store } from '../stores/agent_store';

// Mock reactflow for jsdom environment
vi.mock('reactflow', async () => {
    const actual = await vi.importActual<Record<string, unknown>>('reactflow');
    return {
        ...actual,
        default: ({ children, nodes, edges }: any) => (
            <div data-testid="reactflow-container">
                <div data-testid="node-count">{nodes?.length ?? 0}</div>
                <div data-testid="edge-count">{edges?.length ?? 0}</div>
                {children}
            </div>
        ),
        Background: () => <div data-testid="rf-background" />,
        Controls: () => <div data-testid="rf-controls" />,
        Panel: ({ children, className }: any) => <div className={className}>{children}</div>,
        Handle: () => <div data-testid="rf-handle" />,
        useNodesState: (initial: any) => {
            const [nodes, setNodes] = React.useState(initial);
            return [nodes, setNodes, vi.fn()];
        },
        useEdgesState: (initial: any) => {
            const [edges, setEdges] = React.useState(initial);
            return [edges, setEdges, vi.fn()];
        },
    };
});

describe('Telemetry_Graph Component', () => {
    beforeEach(() => {
        use_trace_store.setState({
            spans: {},
            active_traces: {},
            last_trace_id: null,
            trace_speed: 1,
            is_recording: true,
        });
        use_agent_store.setState({
            agents: [
                {
                    id: 'agent-1',
                    name: 'Sovereign Agent Alpha',
                    role: 'Lead Architect',
                    department: 'Engineering' as any,
                    status: 'idle' as any,
                },
            ],
        });
    });

    it('renders the telemetry graph controls and empty state', () => {
        render(<Telemetry_Graph />);
        expect(screen.getByTestId('reactflow-container')).toBeDefined();
        expect(screen.getByTestId('node-count').textContent).toBe('0');
        expect(screen.getByTestId('edge-count').textContent).toBe('0');
        expect(screen.getByRole('button', { name: /purge/i })).toBeDefined();
    });

    it('projects trace spans into DAG nodes and parent-child edges', async () => {
        use_trace_store.setState({
            spans: {
                'span-root': {
                    id: 'span-root',
                    trace_id: 'trace-1',
                    name: 'agent:execute_goal',
                    start_time: 1000,
                    status: 'completed',
                    attributes: {},
                    events: [],
                    agent_id: 'agent-1',
                    mission_id: 'msn-100',
                },
                'span-child': {
                    id: 'span-child',
                    trace_id: 'trace-1',
                    parent_id: 'span-root',
                    name: 'execute_tool:read_file',
                    start_time: 1050,
                    status: 'running',
                    attributes: {},
                    events: [],
                    agent_id: 'agent-1',
                    mission_id: 'msn-100',
                },
            },
        });

        render(<Telemetry_Graph />);
        await waitFor(() => {
            expect(screen.getByTestId('node-count').textContent).toBe('2');
            expect(screen.getByTestId('edge-count').textContent).toBe('1');
        }, { timeout: 2000 });
    });

    it('filters spans by mission ID when selected from dropdown', async () => {
        use_trace_store.setState({
            spans: {
                'span-m1': {
                    id: 'span-m1',
                    trace_id: 'trace-1',
                    name: 'task:mission_1',
                    start_time: 1000,
                    status: 'completed',
                    attributes: {},
                    events: [],
                    mission_id: 'msn-alpha',
                },
                'span-m2': {
                    id: 'span-m2',
                    trace_id: 'trace-2',
                    name: 'task:mission_2',
                    start_time: 2000,
                    status: 'completed',
                    attributes: {},
                    events: [],
                    mission_id: 'msn-beta',
                },
            },
        });

        render(<Telemetry_Graph />);
        const select = screen.getByRole('combobox');
        expect(select).toBeDefined();

        // Initially both spans rendered (2 nodes)
        await waitFor(() => {
            expect(screen.getByTestId('node-count').textContent).toBe('2');
        }, { timeout: 2000 });

        // Filter to msn-alpha (1 node)
        fireEvent.change(select, { target: { value: 'msn-alpha' } });
        await waitFor(() => {
            expect(screen.getByTestId('node-count').textContent).toBe('1');
        }, { timeout: 2000 });
    });

    it('clears all trace spans when purge button is clicked', async () => {
        use_trace_store.setState({
            spans: {
                'span-1': {
                    id: 'span-1',
                    trace_id: 't-1',
                    name: 'root',
                    start_time: 100,
                    status: 'completed',
                    attributes: {},
                    events: [],
                },
            },
        });

        render(<Telemetry_Graph />);
        await waitFor(() => {
            expect(screen.getByTestId('node-count').textContent).toBe('1');
        }, { timeout: 2000 });

        const purgeBtn = screen.getByRole('button', { name: /purge/i });
        fireEvent.click(purgeBtn);

        expect(Object.keys(use_trace_store.getState().spans)).toHaveLength(0);
        await waitFor(() => {
            expect(screen.getByTestId('node-count').textContent).toBe('0');
        }, { timeout: 2000 });
    });
});

