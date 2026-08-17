/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Assist Note
 * Regression coverage for the adjacent production module and its public contracts.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Contract, rendering, state transition, or error-handling regression.
 * - **Trace Scope**: Vitest assertions and test-local mocks.
 */

import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { Audio_Visualizer } from './Audio_Visualizer';

describe('Audio_Visualizer Component', () => {
    it('renders 8 audio equalizer bars in idle state', () => {
        const { container } = render(<Audio_Visualizer is_active={false} />);
        const bars = container.querySelectorAll('.w-2');
        expect(bars.length).toBe(8);
        expect(bars[0].className).toContain('bg-zinc-800');
    });

    it('animates bars when active', () => {
        const { container } = render(<Audio_Visualizer is_active={true} />);
        const bars = container.querySelectorAll('.w-2');
        expect(bars.length).toBe(8);
        expect(bars[0].className).toContain('animate-pulse');
    });
});
