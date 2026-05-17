/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification of the global Error Boundary recovery shell.** 
 * Validates functional fallback UI rendering when components throw during render, ensuring the main dashboard remains interactive.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Infinite re-render loops or failure to clear the error state during manual user rescue attempts.
 * - **Telemetry Link**: Search `[Error_Boundary.test]` in tracing logs.
 */


import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import Error_Boundary, { SectorBoundary } from './Error_Boundary';

// A component that throws an error to test the boundary
const BuggyComponent = ({ shouldThrow }: { shouldThrow: boolean }) => {
    if (shouldThrow) {
        throw new Error('Neural Spike Detected! Simulation crash.');
    }
    return <div>Sector Stable</div>;
};

describe('Error_Boundary', () => {
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
            <Error_Boundary>
                <div>Safe Zone</div>
            </Error_Boundary>
        );
        expect(screen.getByText('Safe Zone')).toBeInTheDocument();
    });

    it('displays fallback UI when a child crashes', () => {
        render(
            <Error_Boundary name="Test Sector">
                <BuggyComponent shouldThrow={true} />
            </Error_Boundary>
        );

        expect(screen.getByText(/Neural Sector Fault: Test Sector/i)).toBeInTheDocument();
        expect(screen.getByText(/Neural Spike Detected!/i)).toBeInTheDocument();
    });

    it('recovers on reset button click', () => {
        const { rerender } = render(
            <Error_Boundary name="Test Sector">
                <BuggyComponent shouldThrow={true} />
            </Error_Boundary>
        );

        expect(screen.getByText(/Neural Sector Fault/i)).toBeInTheDocument();

        // 1. Rerender with a fixed/safe component first
        rerender(
            <Error_Boundary name="Test Sector">
                <BuggyComponent shouldThrow={false} />
            </Error_Boundary>
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


// Metadata: [Error_Boundary_test]

// Metadata: [Error_Boundary_test]
