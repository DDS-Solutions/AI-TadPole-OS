/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / General / Lineage_Stream.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Lineage_Stream } from './Lineage_Stream';
import { use_trace_store } from '../stores/trace_store';
import { use_agent_store } from '../stores/agent_store';
import { use_tab_store } from '../stores/tab_store';
import type { Trace_Node } from '../types';

// Mock Zustand Stores
vi.mock('../stores/trace_store', () => ({
    use_trace_store: vi.fn(),
}));

vi.mock('../stores/agent_store', () => ({
    use_agent_store: vi.fn(),
}));

vi.mock('../stores/tab_store', () => ({
    use_tab_store: vi.fn(),
}));

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    }
}));

describe('Lineage_Stream Component', () => {
    const mock_span_success: Trace_Node = {
        id: 'span-root',
        trace_id: 'trace-otel-1',
        name: 'Agent Workflow',
        agent_id: 'agent-1',
        mission_id: 'mission-1',
        status: 'success',
        start_time: Date.now() - 2000,
        end_time: Date.now(),
        attributes: {
            'cluster.id': 'cluster-alpha'
        },
        children: []
    };

    const mock_span_failed: Trace_Node = {
        id: 'span-child',
        trace_id: 'trace-otel-1',
        parent_id: 'span-root',
        name: 'LLM Synthesis',
        agent_id: 'agent-1',
        mission_id: 'mission-1',
        status: 'error',
        error: 'RATE_LIMIT_EXCEEDED',
        error_reason: 'TPM threshold breached on provider',
        http_status: 429,
        start_time: Date.now() - 1000,
        end_time: Date.now(),
        attributes: {
            'provider': 'anthropic'
        },
        children: []
    };

    mock_span_success.children = [mock_span_failed];

    const mock_agent = {
        id: 'agent-1',
        name: 'Agent Prime',
        role: 'Orchestrator'
    };

    beforeEach(() => {
        vi.clearAllMocks();

        (use_trace_store as any).mockReturnValue({
            active_trace_id: 'trace-otel-1',
            get_trace_tree: () => [mock_span_success]
        });

        (use_agent_store as any).mockImplementation((selector?: (state: any) => any) => {
            const state = { get_agent: () => mock_agent };
            return typeof selector === 'function' ? selector(state) : state;
        });

        (use_tab_store as any).mockReturnValue({
            is_lineage_stream_detached: false,
            toggle_lineage_stream_detachment: vi.fn()
        });
    });

    it('renders the OTel Trace Stream header and tree nodes', () => {
        render(<Lineage_Stream />);

        expect(screen.getByText('trace.stream_title')).toBeInTheDocument();
        expect(screen.getByText('trace-otel-1')).toBeInTheDocument();
        expect(screen.getByText('Agent Workflow')).toBeInTheDocument();
        expect(screen.getByText('LLM Synthesis')).toBeInTheDocument();
    });

    it('displays error badge on failed tree nodes', () => {
        render(<Lineage_Stream />);

        // The failed child span should display the error badge
        expect(screen.getByText('RATE_LIMIT_EXCEEDED')).toBeInTheDocument();
    });

    it('opens detail inspection overlay when clicking on a tree node and closes on close button', () => {
        render(<Lineage_Stream />);

        // Click the failed node
        const childNode = screen.getByText('LLM Synthesis');
        fireEvent.click(childNode);

        // Verification: detail overlay is rendered with error details
        const overlay = screen.getByTestId('lineage-detail-overlay');
        expect(overlay).toBeInTheDocument();
        expect(screen.getByTestId('trace-error-banner')).toBeInTheDocument();
        expect(screen.getByText(/Execution Fault \(HTTP 429\)/i)).toBeInTheDocument();
        expect(screen.getByText('TPM threshold breached on provider')).toBeInTheDocument();

        // Close details overlay
        const closeBtn = screen.getByTitle('Close details');
        fireEvent.click(closeBtn);

        expect(screen.queryByTestId('lineage-detail-overlay')).not.toBeInTheDocument();
    });
});
