import '@testing-library/jest-dom';
import { vi } from 'vitest';
import enTranslations from '../src/locales/en.json';

// Simple nested key resolver for i18n mock
const getTranslation = (key: string) => {
    const parts = key.split('.');
    let result: any = enTranslations;
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
const localStorageMock = (() => {
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
    value: localStorageMock,
});

// Mock i18n
vi.mock('../src/i18n', () => ({
    i18n: {
        t: (key: string, options?: any) => {
            let t = getTranslation(key);
            if (options) {
                Object.keys(options).forEach(optKey => {
                    t = t.replace(`{{${optKey}}}`, options[optKey]);
                });
            }
            return t;
        },
        language: 'en',
        changeLanguage: vi.fn(),
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


