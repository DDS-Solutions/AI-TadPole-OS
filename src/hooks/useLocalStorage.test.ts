/**
 * @docs ARCHITECTURE:UI-Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useLocalStorage.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useLocalStorage } from './useLocalStorage';

describe('useLocalStorage', () => {
    beforeEach(() => {
        window.localStorage.clear();
        vi.restoreAllMocks();
    });

    it('returns initialValue when storage is empty', () => {
        const { result } = renderHook(() => useLocalStorage('test_key', 'initial_val'));
        expect(result.current[0]).toBe('initial_val');
    });

    it('reads existing item from localStorage', () => {
        window.localStorage.setItem('test_key', JSON.stringify('persisted_val'));
        const { result } = renderHook(() => useLocalStorage('test_key', 'default_val'));
        expect(result.current[0]).toBe('persisted_val');
    });

    it('updates state and persists to localStorage on setValue', () => {
        const { result } = renderHook(() => useLocalStorage('test_key', 'initial'));
        act(() => {
            result.current[1]('updated');
        });
        expect(result.current[0]).toBe('updated');
        expect(JSON.parse(window.localStorage.getItem('test_key') || '""')).toBe('updated');
    });

    it('supports functional updates', () => {
        const { result } = renderHook(() => useLocalStorage('test_counter', 10));
        act(() => {
            result.current[1]((prev) => prev + 5);
        });
        expect(result.current[0]).toBe(15);
        expect(window.localStorage.getItem('test_counter')).toBe('15');
    });

    it('removes item and resets state to initial value', () => {
        const { result } = renderHook(() => useLocalStorage('test_key', 'default'));
        act(() => {
            result.current[1]('custom');
        });
        expect(result.current[0]).toBe('custom');

        act(() => {
            result.current[2]();
        });
        expect(result.current[0]).toBe('default');
        expect(window.localStorage.getItem('test_key')).toBeNull();
    });

    it('handles corrupted JSON in localStorage gracefully without throwing', () => {
        window.localStorage.setItem('corrupted_key', '{invalid json');
        const { result } = renderHook(() => useLocalStorage('corrupted_key', { fallback: true }));
        expect(result.current[0]).toEqual({ fallback: true });
    });

    it('switches keys without an effect-driven stale render', () => {
        window.localStorage.setItem('first_key', JSON.stringify('first'));
        window.localStorage.setItem('second_key', JSON.stringify('second'));
        const { result, rerender } = renderHook(
            ({ storageKey }) => useLocalStorage(storageKey, 'default'),
            { initialProps: { storageKey: 'first_key' } },
        );

        expect(result.current[0]).toBe('first');
        rerender({ storageKey: 'second_key' });
        expect(result.current[0]).toBe('second');

        act(() => result.current[1]('updated second'));
        expect(window.localStorage.getItem('first_key')).toBe(JSON.stringify('first'));
        expect(window.localStorage.getItem('second_key')).toBe(JSON.stringify('updated second'));
    });
});
