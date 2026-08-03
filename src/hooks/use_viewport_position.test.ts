/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **useViewportPosition Unit Tests**: Validates trigger layout coordinates calculations,
 * viewport edge flipping logic, and final clamping behavior under custom window bounds.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Sub-pixel clamping loops or unmocked element bounds.
 * - **Telemetry Link**: Search `[useViewportPosition.test]` in tracing logs.
 */

import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useViewportPosition } from './use_viewport_position';

describe('useViewportPosition', () => {
    let mock_trigger: any;
    let mock_content: any;
    let trigger_ref: any;
    let content_ref: any;

    beforeEach(() => {
        mock_trigger = {
            getBoundingClientRect: vi.fn().mockReturnValue({
                top: 100,
                bottom: 150,
                left: 100,
                right: 200,
                width: 100,
                height: 50
            })
        };
        mock_content = {
            getBoundingClientRect: vi.fn().mockReturnValue({
                width: 80,
                height: 40
            })
        };
        trigger_ref = { current: mock_trigger };
        content_ref = { current: mock_content };

        // Stub global window dimensions
        vi.stubGlobal('innerWidth', 1024);
        vi.stubGlobal('innerHeight', 768);
    });

    afterEach(() => {
        vi.unstubAllGlobals();
    });

    it('calculates optimal position without flipping when far from boundaries', () => {
        const { result } = renderHook(() => useViewportPosition({
            trigger_ref,
            content_ref,
            position: 'top',
            is_visible: true
        }));

        act(() => {
            result.current.update_position();
        });

        // Far from boundary: actual_position stays 'top'
        expect(result.current.actual_position).toBe('top');
        // coords x should be left (100) + width / 2 (50) = 150
        // coords y should be top (100) - offset (8) = 92
        expect(result.current.coords.x).toBe(150);
        expect(result.current.coords.y).toBe(92);
    });

    it('flips position from top to bottom when too close to top boundary', () => {
        mock_trigger.getBoundingClientRect.mockReturnValue({
            top: 20, // less than 60 threshold
            bottom: 70,
            left: 100,
            right: 200,
            width: 100,
            height: 50
        });

        const { result } = renderHook(() => useViewportPosition({
            trigger_ref,
            content_ref,
            position: 'top',
            is_visible: true
        }));

        act(() => {
            result.current.update_position();
        });

        expect(result.current.actual_position).toBe('bottom');
        // x should be 150
        // y should be bottom (70) + offset (8) = 78
        expect(result.current.coords.x).toBe(150);
        expect(result.current.coords.y).toBe(78);
    });

    it('flips position from bottom to top when too close to bottom boundary', () => {
        mock_trigger.getBoundingClientRect.mockReturnValue({
            top: 720,
            bottom: 750, // view_height - bottom (768 - 750) = 18 < 60 threshold
            left: 100,
            right: 200,
            width: 100,
            height: 30
        });

        const { result } = renderHook(() => useViewportPosition({
            trigger_ref,
            content_ref,
            position: 'bottom',
            is_visible: true
        }));

        act(() => {
            result.current.update_position();
        });

        expect(result.current.actual_position).toBe('top');
    });

    it('flips position from left to right when too close to left boundary', () => {
        mock_trigger.getBoundingClientRect.mockReturnValue({
            top: 300,
            bottom: 350,
            left: 40, // < 100 threshold
            right: 140,
            width: 100,
            height: 50
        });

        const { result } = renderHook(() => useViewportPosition({
            trigger_ref,
            content_ref,
            position: 'left',
            is_visible: true
        }));

        act(() => {
            result.current.update_position();
        });

        expect(result.current.actual_position).toBe('right');
    });

    it('flips position from right to left when too close to right boundary', () => {
        mock_trigger.getBoundingClientRect.mockReturnValue({
            top: 300,
            bottom: 350,
            left: 880,
            right: 980, // view_width - right (1024 - 980) = 44 < 100 threshold
            width: 100,
            height: 50
        });

        const { result } = renderHook(() => useViewportPosition({
            trigger_ref,
            content_ref,
            position: 'right',
            is_visible: true
        }));

        act(() => {
            result.current.update_position();
        });

        expect(result.current.actual_position).toBe('left');
    });

    it('clamps coordinates to stay inside viewport padding', () => {
        // Positioned top, but trigger is far left (x would be 20)
        // content_rect width is 80, padding is 8.
        // x - content_rect.width / 2 = 20 - 40 = -20 < padding (8)
        // should adjust x to content_rect.width / 2 + padding = 40 + 8 = 48
        mock_trigger.getBoundingClientRect.mockReturnValue({
            top: 200,
            bottom: 250,
            left: -30,
            right: 70,
            width: 100,
            height: 50
        });

        const { result } = renderHook(() => useViewportPosition({
            trigger_ref,
            content_ref,
            position: 'top',
            is_visible: true,
            padding: 8
        }));

        act(() => {
            result.current.update_position();
        });

        expect(result.current.coords.x).toBe(48);
    });
});

// Metadata: [use_viewport_position_test]
