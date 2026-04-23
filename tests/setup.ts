/**
 * @docs ARCHITECTURE:Quality:Verification
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[setup]` in observability traces.
 */

import '@testing-library/jest-dom';
import { vi } from 'vitest';
import en_translations from '../src/locales/en.json';

// Simple nested key resolver for i18n mock
const get_translation = (key: string) => {
    const parts = key.split('.');
    let result: any = en_translations;
    for (const part of parts) {
        if (!result || result[part] === undefined) return key;
        result = result[part];
    }
    return typeof result === 'string' ? result : key;
};

// Mock HTMLCanvasElement.prototype.getContext
HTMLCanvasElement.prototype.getContext = (() => {
    return {} as unknown as RenderingContext;
}) as unknown as typeof HTMLCanvasElement.prototype.getContext;

// Mock localStorage
const local_storage_mock = (() => {
    let store: Record<string, string> = {};
    return {
        getItem: (key: string) => store[key] || null,
        setItem: (key: string, value: string) => {
            store[key] = value.toString();
        },
        removeItem: (key: string) => {
            delete store[key];
        },
        clear: () => {
            store = {};
        },
    };
})();

Object.defineProperty(window, 'localStorage', {
    value: local_storage_mock,
});

// Mock i18n
vi.mock('../src/i18n', () => ({
    i18n: {
        t: (key: string, options?: any) => {
            let t = get_translation(key);
            if (options) {
                Object.keys(options).forEach(opt_key => {
                    t = t.replace(`{{${opt_key}}}`, options[opt_key]);
                });
            }
            return t;
        },
        language: 'en',
    },
}));

// Mock matchMedia for Framer Motion
Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: vi.fn().mockImplementation(query => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(), // deprecated
        removeListener: vi.fn(), // deprecated
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
    })),
});

// ResizeObserver mock for ReactFlow and layout-dependent tests
class Resize_Observer_Mock {
    observe() {}
    unobserve() {}
    disconnect() {}
}

window.ResizeObserver = Resize_Observer_Mock as any;
global.ResizeObserver = Resize_Observer_Mock as any;


// Metadata: [setup]
