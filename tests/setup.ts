import '@testing-library/jest-dom';

// Mock HTMLCanvasElement.prototype.getContext
HTMLCanvasElement.prototype.getContext = (() => {
    return {} as unknown as RenderingContext;
}) as unknown as typeof HTMLCanvasElement.prototype.getContext;

// Mock localStorage
// This is critical for stabilizing tests that use store persistence or direct 
// localStorage access in components (e.g., OversightDashboard).
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

