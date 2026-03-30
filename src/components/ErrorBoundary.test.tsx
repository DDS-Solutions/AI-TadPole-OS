import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import ErrorBoundary, { SectorBoundary } from './ErrorBoundary';

// A component that throws an error to test the boundary
const BuggyComponent = ({ shouldThrow }: { shouldThrow: boolean }) => {
    if (shouldThrow) {
        throw new Error('Neural Spike Detected! Simulation crash.');
    }
    return <div>Sector Stable</div>;
};

describe('ErrorBoundary', () => {
    let consoleErrorSpy: ReturnType<typeof vi.spyOn>;

    beforeEach(() => {
        // Suppress console.error to avoid polluting test output with expected errors
        consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => { });
    });

    afterEach(() => {
        consoleErrorSpy.mockRestore();
    });

    it('renders children when there is no error', () => {
        render(
            <ErrorBoundary>
                <div>Safe Zone</div>
            </ErrorBoundary>
        );
        expect(screen.getByText('Safe Zone')).toBeInTheDocument();
    });

    it('displays fallback UI when a child crashes', () => {
        render(
            <ErrorBoundary name="Test Sector">
                <BuggyComponent shouldThrow={true} />
            </ErrorBoundary>
        );

        expect(screen.getByText(/Neural Sector Fault: Test Sector/i)).toBeInTheDocument();
        expect(screen.getByText(/Neural Spike Detected!/i)).toBeInTheDocument();
    });

    it('recovers on reset button click', () => {
        const { rerender } = render(
            <ErrorBoundary name="Test Sector">
                <BuggyComponent shouldThrow={true} />
            </ErrorBoundary>
        );

        expect(screen.getByText(/Neural Sector Fault/i)).toBeInTheDocument();

        // 1. Rerender with a fixed/safe component first
        rerender(
            <ErrorBoundary name="Test Sector">
                <BuggyComponent shouldThrow={false} />
            </ErrorBoundary>
        );

        // 2. Now click the reset button which is still visible in the error state
        const resetButton = screen.getByText(/Re-initialize Sector/i);
        fireEvent.click(resetButton);

        expect(screen.getByText('Sector Stable')).toBeInTheDocument();
        expect(screen.queryByText(/Neural Sector Fault/i)).not.toBeInTheDocument();
    });
});

describe('SectorBoundary', () => {
    let consoleErrorSpy: ReturnType<typeof vi.spyOn>;

    beforeEach(() => {
        consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => { });
    });

    afterEach(() => {
        consoleErrorSpy.mockRestore();
    });

    it('renders as a named boundary', () => {
        render(
            <SectorBoundary name="Widget A">
                <BuggyComponent shouldThrow={true} />
            </SectorBoundary>
        );

        expect(screen.getByText(/Neural Sector Fault: Widget A/i)).toBeInTheDocument();
    });
});
