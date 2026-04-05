/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Manages the Benchmark Analytics.test page. 
 * Connects the global state stores to the layout orchestration.
 */

/**
 * @file Benchmark_Analytics.test.tsx
 * @description Suite for the Performance Analytics and Benchmarking page.
 * @module Pages/Benchmark_Analytics
 * @testedBehavior
 * - Benchmark Retrieval: Fetching and displaying historical performance data.
 * - Live Execution: Triggering new benchmark runs and handling telemetry feedback via event_bus.
 * - Competitive Analysis: Delta calculations and "isImprovement" logic between selected tests.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks tadpole_os_service for benchmark data and run triggering.
 * - Mocks framer-motion to prevent Vitest/Jsdom animation interference.
 * - Mocks i18n to return keys for stable assertion matching.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import Benchmark_Analytics from './Benchmark_Analytics';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { event_bus } from '../services/event_bus';
import { i18n } from '../i18n';
import '@testing-library/jest-dom/vitest';

// Mock Services
vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        get_benchmarks: vi.fn(),
        run_benchmark: vi.fn(),
    }
}));

vi.mock('../services/event_bus', () => ({
    event_bus: {
        emit_log: vi.fn(),
        get_history: vi.fn(() => []),
        subscribe_logs: vi.fn(() => () => { }),
    }
}));

// Mock i18n to return keys for stable testing
vi.mock('../i18n', () => ({
    i18n: {
        t: vi.fn((key) => key)
    }
}));

// Mock framer-motion to avoid animation issues in tests
vi.mock('framer-motion', () => ({
    motion: {
        div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
        tr: ({ children, ...props }: any) => <tr {...props}>{children}</tr>,
    },
    AnimatePresence: ({ children }: any) => <>{children}</>,
}));

// Mock UI components
vi.mock('../components/ui', () => ({
    Tooltip: ({ children, content }: any) => (
        <div data-testid="tooltip-wrapper">
            {children}
            <span style={{ display: 'none' }}>{content}</span>
        </div>
    )
}));

const mock_benchmarks = [
    {
        id: '1',
        name: 'Runner Bench',
        category: 'execution',
        test_id: 'BM-RUN-01',
        mean_ms: 120.5,
        p95_ms: 150.0,
        p99_ms: 180.0,
        target_value: '< 150ms',
        status: 'PASS',
        created_at: new Date().toISOString()
    },
    {
        id: '2',
        name: 'DB Latency',
        category: 'persistence',
        test_id: 'BM-DB-01',
        mean_ms: 45.2,
        p95_ms: 60.0,
        p99_ms: 80.0,
        target_value: '< 50ms',
        status: 'PASS',
        created_at: new Date().toISOString()
    }
];

describe('Benchmark_Analytics', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        (tadpole_os_service.get_benchmarks as any).mockResolvedValue(mock_benchmarks);
        (i18n.t as any).mockImplementation((key: string) => key);
    });

    it('renders and fetches benchmarks on mount', async () => {
        render(<Benchmark_Analytics />);
        
        expect(screen.getByText('benchmark.title')).toBeInTheDocument();
        expect(screen.getByText('benchmark.loading')).toBeInTheDocument();

        await waitFor(() => {
            expect(tadpole_os_service.get_benchmarks).toHaveBeenCalled();
            expect(screen.getByText('Runner Bench')).toBeInTheDocument();
            expect(screen.getByText('DB Latency')).toBeInTheDocument();
        });
    });

    it('handles benchmark execution', async () => {
        (tadpole_os_service.run_benchmark as any).mockResolvedValue({ status: 'success' });
        render(<Benchmark_Analytics />);
        await waitFor(() => expect(screen.getByText('Runner Bench')).toBeInTheDocument());

        const run_btn = screen.getByText('benchmark.btn_run_runner');
        fireEvent.click(run_btn);

        expect(screen.getByText('benchmark.btn_executing')).toBeInTheDocument();
        
        await waitFor(() => {
            expect(tadpole_os_service.run_benchmark).toHaveBeenCalledWith('BM-RUN-01');
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                text: expect.stringContaining('benchmark.event_success')
            }));
        });
    });

    it('handles benchmark execution failure', async () => {
        (tadpole_os_service.run_benchmark as any).mockRejectedValue(new Error('Telemetry Timeout'));
        render(<Benchmark_Analytics />);
        await waitFor(() => expect(screen.getByText('Runner Bench')).toBeInTheDocument());

        const run_btn = screen.getByText('benchmark.btn_run_db');
        fireEvent.click(run_btn);

        await waitFor(() => {
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'error',
                text: expect.stringContaining('benchmark.event_failed')
            }));
        });
    });

    it('toggles selection and shows comparison', async () => {
        render(<Benchmark_Analytics />);
        await waitFor(() => expect(screen.getByText('Runner Bench')).toBeInTheDocument());

        // Select first bench
        fireEvent.click(screen.getByText('Runner Bench'));
        
        // Select second bench
        fireEvent.click(screen.getByText('DB Latency'));

        // Comparison should appear
        await waitFor(() => {
            expect(screen.getByText('benchmark.label_delta_analysis')).toBeInTheDocument();
            expect(screen.getByText('benchmark.label_baseline')).toBeInTheDocument();
            expect(screen.getByText('benchmark.label_current_target')).toBeInTheDocument();
        });

        // Verification of delta calculation (120.5 vs 45.2)
        // Delta = 120.5 - 45.2 = 75.3. Percentage = (75.3 / 45.2) * 100 = 166.59% -> +166.6%
        expect(screen.getByText('+166.6%')).toBeInTheDocument();
        
        // Clear selection
        fireEvent.click(screen.getByText('benchmark.btn_clear'));
        await waitFor(() => {
            expect(screen.queryByText('benchmark.label_delta_analysis')).not.toBeInTheDocument();
        });
    });

    it('shows empty state when no benchmarks available', async () => {
        (tadpole_os_service.get_benchmarks as any).mockResolvedValue([]);
        render(<Benchmark_Analytics />);
        
        await waitFor(() => {
            expect(screen.getByText('benchmark.empty')).toBeInTheDocument();
        });
    });
});

