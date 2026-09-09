/**
 * @docs ARCHITECTURE:UI-Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useLocalStorage
 * - **Primary Entrypoints**: `useLocalStorage`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[useLocalStorage]`
 * - **Witness Tests**: none declared
 */

import { useState, useCallback } from 'react';

export function useLocalStorage<T>(
    key: string,
    initialValue: T | (() => T)
): [T, (value: T | ((prev: T) => T)) => void, () => void] {
    const readValue = useCallback((): T => {
        if (typeof window === 'undefined') {
            return initialValue instanceof Function ? initialValue() : initialValue;
        }

        try {
            const item = window.localStorage.getItem(key);
            if (!item || item === 'undefined') {
                return initialValue instanceof Function ? initialValue() : initialValue;
            }
            try {
                return JSON.parse(item) as T;
            } catch {
                const fallback = initialValue instanceof Function ? initialValue() : initialValue;
                if (typeof fallback === 'string') {
                    return item as unknown as T;
                }
                throw new Error(`Failed to parse JSON for key "${key}"`);
            }
        } catch (error) {
            console.debug(`[useLocalStorage] Error reading key "${key}":`, error);
            return initialValue instanceof Function ? initialValue() : initialValue;
        }
    }, [key, initialValue]);

    const [storedState, setStoredState] = useState<{ key: string; value: T }>(() => ({
        key,
        value: readValue(),
    }));
    const storedValue = storedState.key === key ? storedState.value : readValue();

    const setValue = useCallback(
        (value: T | ((prev: T) => T)) => {
            setStoredState((prev) => {
                const currentValue = prev.key === key ? prev.value : readValue();
                const nextValue = value instanceof Function ? value(currentValue) : value;
                if (typeof window !== 'undefined') {
                    try {
                        window.localStorage.setItem(key, JSON.stringify(nextValue));
                    } catch (error) {
                        console.debug(`[useLocalStorage] Error setting key "${key}":`, error);
                    }
                }
                return { key, value: nextValue };
            });
        },
        [key, readValue]
    );

    const removeValue = useCallback(() => {
        if (typeof window !== 'undefined') {
            try {
                window.localStorage.removeItem(key);
            } catch (error) {
                console.debug(`[useLocalStorage] Error removing key "${key}":`, error);
            }
        }
        setStoredState({
            key,
            value: initialValue instanceof Function ? initialValue() : initialValue,
        });
    }, [key, initialValue]);

    return [storedValue, setValue, removeValue];
}
