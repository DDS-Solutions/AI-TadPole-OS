/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Voice / Audio_Visualizer.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
