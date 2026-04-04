/**
 * @file Benchmark_Analytics.test.tsx
 * @description Suite for the Performance Analytics and Benchmarking page.
 * @module Pages/Benchmark_Analytics
 * @testedBehavior
 * - Benchmark Retrieval: Fetching and displaying historical performance data.
 * - Live Execution: Triggering new benchmark runs and handling telemetry feedback via event_bus.
 * - Competitive Analysis: Delta calculations and "isImprovement" logic between selected tests.
 * @aiContext
 * - Mocks tadpole_os_service for benchmark data and run triggering.
 * - Checks for specific delta percentage strings (e.g., "+166.6%") to verify calculation logic.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import Benchmark_Analytics from './Benchmark_Analytics';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { event_bus } from '../services/event_bus';

// Mock Services
vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        get_benchmarks: vi.fn(),
        run_benchmark: vi.fn(),
    }
}));

vi.mock('../services/event_bus', () => ({
    event_bus: {
        emit: vi.fn(),
        subscribe: vi.fn(() => () => { }),
        getHistory: vi.fn(() => []),
    }
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

const mockBenchmarks = [
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
        (tadpole_os_service.get_benchmarks as any).mockResolvedValue(mockBenchmarks);
    });

    it('renders and fetches benchmarks on mount', async () => {
        render(<Benchmark_Analytics />);
        
        expect(screen.getByText(/Performance Analytics/i)).toBeInTheDocument();
        expect(screen.getByText(/Decrypting performance logs/i)).toBeInTheDocument();

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

        const runBtn = screen.getByText('Run Runner Bench');
        fireEvent.click(runBtn);

        expect(screen.getByText('Executing...')).toBeInTheDocument();
        expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
            text: expect.stringContaining('Triggering BM-RUN-01')
        }));

        await waitFor(() => {
            expect(tadpole_os_service.run_benchmark).toHaveBeenCalledWith('BM-RUN-01');
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                text: expect.stringContaining('completed successfully')
            }));
        });
    });

    it('handles benchmark execution failure', async () => {
        (tadpole_os_service.run_benchmark as any).mockRejectedValue(new Error('Telemetry Timeout'));
        render(<Benchmark_Analytics />);
        await waitFor(() => expect(screen.getByText('Runner Bench')).toBeInTheDocument());

        const runBtn = screen.getByText('Run DB Bench');
        fireEvent.click(runBtn);

        await waitFor(() => {
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'error',
                text: expect.stringContaining('failed')
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
            expect(screen.getByText('Delta Analysis')).toBeInTheDocument();
            expect(screen.getByText('Baseline')).toBeInTheDocument();
            expect(screen.getByText('Current Target')).toBeInTheDocument();
        });

        // Verification of delta calculation (120.5 vs 45.2)
        // Delta = 120.5 - 45.2 = 75.3. Percentage = (75.3 / 45.2) * 100 = 166.59%
        // But the code: calculateDelta(v1, v2) where v1 is target (first selected), v2 is baseline (second selected)
        // Wait, toggling selection: [...selectedTests, id]
        // comparisonData: t1 = first selected, t2 = second selected.
        // t2 (baseline) = second selected = 'DB Latency' (45.2ms)
        // t1 (target) = first selected = 'Runner Bench' (120.5ms)
        // Percentage = ((120.5 - 45.2) / 45.2) * 100 = 166.6%
        // Since delta > 0, it's NOT an improvement (isImprovement = false).
        expect(screen.getByText('+166.6%')).toBeInTheDocument();
        
        // Clear selection
        fireEvent.click(screen.getByText('Clear Selection'));
        await waitFor(() => {
            expect(screen.queryByText('Delta Analysis')).not.toBeInTheDocument();
        });
    });

    it('shows empty state when no benchmarks available', async () => {
        (tadpole_os_service.get_benchmarks as any).mockResolvedValue([]);
        render(<Benchmark_Analytics />);
        
        await waitFor(() => {
            expect(screen.getByText(/No benchmarks recorded/i)).toBeInTheDocument();
        });
    });
});
