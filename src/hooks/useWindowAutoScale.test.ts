/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useWindowAutoScale.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useWindowAutoScale } from './useWindowAutoScale';

describe('useWindowAutoScale', () => {
    beforeEach(() => {
        document.body.style.zoom = '';
    });

    afterEach(() => {
        vi.unstubAllGlobals();
        document.body.style.zoom = '';
    });

    it('computes and applies scale within min and max boundaries', () => {
        vi.stubGlobal('innerWidth', 1200);
        vi.stubGlobal('innerHeight', 800);

        renderHook(() => useWindowAutoScale(1200, 800, 0.55, 1.0));

        // 1200 / 1200 = 1.0, 800 / 800 = 1.0 -> 1.0
        expect(Number(document.body.style.zoom)).toBe(1.0);
    });

    it('clamps scale to min_scale when viewport is very small', () => {
        vi.stubGlobal('innerWidth', 400);
        vi.stubGlobal('innerHeight', 300);

        renderHook(() => useWindowAutoScale(1200, 800, 0.55, 1.0));

        // 400 / 1200 = 0.33 < 0.55 -> clamped to 0.55
        expect(Number(document.body.style.zoom)).toBe(0.55);
    });

    it('resets document.body.style.zoom on unmount', () => {
        vi.stubGlobal('innerWidth', 600);
        vi.stubGlobal('innerHeight', 400);

        const { unmount } = renderHook(() => useWindowAutoScale(1200, 800, 0.55, 1.0));
        expect(document.body.style.zoom).not.toBe('');

        unmount();
        expect(document.body.style.zoom).toBe('');
    });

    it('updates scale dynamically on window resize event', () => {
        vi.stubGlobal('innerWidth', 1200);
        vi.stubGlobal('innerHeight', 800);

        renderHook(() => useWindowAutoScale(1200, 800, 0.55, 1.0));
        expect(Number(document.body.style.zoom)).toBe(1.0);

        act(() => {
            vi.stubGlobal('innerWidth', 900);
            vi.stubGlobal('innerHeight', 600);
            window.dispatchEvent(new Event('resize'));
        });

        // 900 / 1200 = 0.75, 600 / 800 = 0.75 -> 0.75
        expect(Number(document.body.style.zoom)).toBe(0.75);
    });
});
