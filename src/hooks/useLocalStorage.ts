/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Safe LocalStorage React Hook**: Encapsulates browser storage interactions with structured error handling,
 * defensive JSON parsing, and graceful in-memory fallback for environments with blocked storage (e.g. strict CSP / private windows).
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: LocalStorage quota exceeded, corrupted JSON payloads, or security policy blocking window.localStorage.
 * - **Telemetry Link**: Search `[useLocalStorage]` in browser console debug traces.
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

// Metadata: [useLocalStorage]
