/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Layout / Observability_Sidebar.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Observability_Sidebar } from './Observability_Sidebar';
import { use_tab_store } from '../../stores/tab_store';

// Mock child observability components
vi.mock('../Lineage_Stream', () => ({
    Lineage_Stream: () => <div data-testid="lineage-stream-component" />
}));

vi.mock('../Neural_Waterfall', () => ({
    Neural_Waterfall: () => <div data-testid="waterfall-component" />
}));

vi.mock('../dashboard/System_Log', () => ({
    System_Log: () => <div data-testid="system-log-component" />
}));

vi.mock('../../stores/tab_store', () => ({
    use_tab_store: Object.assign(
        vi.fn(),
        {
            getState: vi.fn(),
            setState: vi.fn()
        }
    )
}));

describe('Observability_Sidebar', () => {
    const mock_toggle_lineage = vi.fn();
    const mock_toggle_trace = vi.fn();
    const mock_toggle_system_log = vi.fn();
    const mock_reattach_all = vi.fn();

    const default_store_state = {
        is_system_log_detached: false,
        is_trace_stream_detached: false,
        is_lineage_stream_detached: false,
        toggle_lineage_stream_detachment: mock_toggle_lineage,
        toggle_trace_stream_detachment: mock_toggle_trace,
        toggle_system_log_detachment: mock_toggle_system_log,
        reattach_observability_sectors: mock_reattach_all,
    };

    beforeEach(() => {
        vi.clearAllMocks();
        vi.mocked(use_tab_store).mockReturnValue(default_store_state as any);
        vi.mocked(use_tab_store.getState).mockReturnValue(default_store_state as any);
    });

    it('renders all 3 observability monitors when none are detached', () => {
        render(<Observability_Sidebar />);

        expect(screen.getByTestId('lineage-stream-component')).toBeInTheDocument();
        expect(screen.getByTestId('waterfall-component')).toBeInTheDocument();
        expect(screen.getByTestId('system-log-component')).toBeInTheDocument();
    });

    it('renders placeholder with recall button when Neural_Waterfall is detached', () => {
        vi.mocked(use_tab_store).mockReturnValue({
            ...default_store_state,
            is_trace_stream_detached: true,
        } as any);

        render(<Observability_Sidebar />);

        expect(screen.getByTestId('lineage-stream-component')).toBeInTheDocument();
        expect(screen.queryByTestId('waterfall-component')).not.toBeInTheDocument();
        expect(screen.getByTestId('system-log-component')).toBeInTheDocument();

        expect(screen.getByText(/Neural Waterfall/i)).toBeInTheDocument();
        const recall_btn = screen.getByTitle(/Dock Neural Waterfall back to sidebar/i);
        expect(recall_btn).toBeInTheDocument();

        fireEvent.click(recall_btn);
        expect(mock_toggle_trace).toHaveBeenCalled();
    });

    it('renders placeholder with recall button when Lineage_Stream is detached', () => {
        vi.mocked(use_tab_store).mockReturnValue({
            ...default_store_state,
            is_lineage_stream_detached: true,
        } as any);

        render(<Observability_Sidebar />);

        expect(screen.queryByTestId('lineage-stream-component')).not.toBeInTheDocument();
        expect(screen.getByTestId('waterfall-component')).toBeInTheDocument();

        const recall_btn = screen.getByTitle(/Dock .* back to sidebar/i);
        expect(recall_btn).toBeInTheDocument();

        fireEvent.click(recall_btn);
        expect(mock_toggle_lineage).toHaveBeenCalled();
    });

    it('renders empty state and Recall All Monitors button when all 3 sectors are detached', () => {
        vi.mocked(use_tab_store).mockReturnValue({
            ...default_store_state,
            is_system_log_detached: true,
            is_trace_stream_detached: true,
            is_lineage_stream_detached: true,
        } as any);

        render(<Observability_Sidebar />);

        expect(screen.queryByTestId('lineage-stream-component')).not.toBeInTheDocument();
        expect(screen.queryByTestId('waterfall-component')).not.toBeInTheDocument();
        expect(screen.queryByTestId('system-log-component')).not.toBeInTheDocument();

        const recall_all_btn = screen.getByRole('button', { name: /Recall All Monitors/i });
        expect(recall_all_btn).toBeInTheDocument();

        fireEvent.click(recall_all_btn);
        expect(mock_reattach_all).toHaveBeenCalled();
    });
});

