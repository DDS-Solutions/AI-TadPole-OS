import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/react';
import { PortalWindow } from './PortalWindow';

describe('PortalWindow', () => {
    let mockWindow: any;
    const mockOnClose = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();

        // Use a real JSDOM document for the detached window to ensure compatibility with createPortal
        const childDoc = document.implementation.createHTMLDocument('Tadpole OS Detached');
        
        // Mock window.open to return a window-like object
        mockWindow = {
            document: childDoc,
            addEventListener: vi.fn(),
            removeEventListener: vi.fn(),
            focus: vi.fn(),
            close: vi.fn(),
        };

        // Standard Vitest way to mock global window properties
        vi.spyOn(window, 'open').mockReturnValue(mockWindow as any);
        
        // Provide styleSheets if needed by the component
        Object.defineProperty(window.document, 'styleSheets', {
            value: [],
            writable: true,
            configurable: true,
        });
    });

    it('opens a new window on mount', () => {
        render(
            <PortalWindow id="test-tab" title="Test Tab" onClose={mockOnClose}>
                <div>Content</div>
            </PortalWindow>
        );

        expect(window.open).toHaveBeenCalledTimes(1);
        expect(window.open).toHaveBeenCalledWith('', 'tadpole-detached-test-tab', expect.any(String));
    });

    it('does NOT re-open the window if the onClose callback changes (strobe fix)', () => {
        const { rerender } = render(
            <PortalWindow id="test-tab" title="Test Tab" onClose={mockOnClose}>
                <div>Content</div>
            </PortalWindow>
        );

        expect(window.open).toHaveBeenCalledTimes(1);

        // Re-render with a NEW inline function (simulating parent re-render)
        rerender(
            <PortalWindow id="test-tab" title="Test Tab" onClose={() => {}}>
                <div>Content</div>
            </PortalWindow>
        );

        // Should STILL only have been called once
        expect(window.open).toHaveBeenCalledTimes(1);
    });

    it('updates document title when title prop changes without re-opening window', () => {
        const { rerender } = render(
            <PortalWindow id="test-tab" title="Initial Title" onClose={mockOnClose}>
                <div>Content</div>
            </PortalWindow>
        );

        expect(window.open).toHaveBeenCalledTimes(1);
        expect(mockWindow.document.title).toBe('Initial Title | Tadpole OS Detached');

        rerender(
            <PortalWindow id="test-tab" title="Updated Title" onClose={mockOnClose}>
                <div>Content</div>
            </PortalWindow>
        );

        // Title should update
        expect(mockWindow.document.title).toBe('Updated Title | Tadpole OS Detached');
        
        // But window should NOT re-open
        expect(window.open).toHaveBeenCalledTimes(1);
    });

    it('closes the window and calls cleanup on unmount', () => {
        const { unmount } = render(
            <PortalWindow id="test-tab" title="Test Tab" onClose={mockOnClose}>
                <div>Content</div>
            </PortalWindow>
        );

        unmount();

        expect(mockWindow.close).toHaveBeenCalledTimes(1);
    });
});
