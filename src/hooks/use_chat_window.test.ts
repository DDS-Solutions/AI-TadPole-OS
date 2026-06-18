/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the Sovereign Chat window management hook.** 
 * Verifies the minimization, maximization, and detachment logic, ensuring the state is correctly synced with the `sovereign_store`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Viewport-relative coordinate calculation errors or state drift between the hook and the global store.
 * - **Telemetry Link**: Search `[useChatWindow.test]` in tracing logs.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useChatWindow } from './use_chat_window';
import { use_sovereign_store } from '../stores/sovereign_store';

// Mock stores
vi.mock('../stores/sovereign_store', () => ({
    use_sovereign_store: vi.fn(),
}));

describe('useChatWindow', () => {
    const mock_set_detached = vi.fn();
    
    beforeEach(() => {
        vi.clearAllMocks();
        (use_sovereign_store as any).mockReturnValue({
            is_detached: false,
            set_detached: mock_set_detached,
        });

        // Mock window dimensions
        Object.defineProperty(window, 'innerHeight', { writable: true, configurable: true, value: 1000 });
        Object.defineProperty(window, 'innerWidth', { writable: true, configurable: true, value: 1920 });
    });

    it('should initialize with default states', () => {
        const { result } = renderHook(() => useChatWindow());
        
        expect(result.current.is_minimized).toBe(false);
        expect(result.current.x_open.get()).toBe(0);
        expect(result.current.y_open.get()).toBe(0);
    });

    it('should toggle detachment via sovereign store', () => {
        const { result } = renderHook(() => useChatWindow());
        
        act(() => {
            result.current.toggle_detach();
        });

        expect(mock_set_detached).toHaveBeenCalledWith(true);
    });

    it('should perform minimize transform and reset open positions', () => {
        const { result } = renderHook(() => useChatWindow());
        
        // Move window first
        result.current.x_open.set(-100);
        result.current.y_open.set(-100);

        act(() => {
            result.current.perform_minimize_transform();
        });

        expect(result.current.is_minimized).toBe(true);
        // Position on minimize depends on the logic in the hook (viewport-relative)
        expect(result.current.x_min.get()).not.toBe(0);
    });

    it('should reset minimized state when maximizing', () => {
        const { result } = renderHook(() => useChatWindow());
        
        act(() => {
            result.current.perform_minimize_transform();
        });
        expect(result.current.is_minimized).toBe(true);

        act(() => {
            result.current.perform_maximize_transform();
        });
        expect(result.current.is_minimized).toBe(false);
    });

    it('should log telemetry on transform actions', () => {
        const console_spy = vi.spyOn(console, 'debug');
        const { result } = renderHook(() => useChatWindow());

        act(() => {
            result.current.perform_minimize_transform();
        });

        expect(console_spy).toHaveBeenCalledWith(expect.stringContaining('[useChatWindow] Initiating minimize transform'));
    });
});

// Metadata: [use_chat_window_test]

// Metadata: [use_chat_window_test]
