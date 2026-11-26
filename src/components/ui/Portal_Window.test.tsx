/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Ui / Portal_Window.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { StrictMode } from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, act } from '@testing-library/react';
import { Portal_Window } from './Portal_Window';

describe('Portal_Window', () => {
    let mockWindow: any;
    const mockOnClose = vi.fn();

    beforeEach(async () => {
        // Flush deferred popup closes left by the previous test's auto-unmount.
        await new Promise(resolve => setTimeout(resolve, 0));
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

    it('opens a new window on mount', async () => {
        await act(async () => {
            render(
                <Portal_Window id="test-tab" title="Test Tab" on_close={mockOnClose}>
                    <div>Content</div>
                </Portal_Window>
            );
        });

        expect(window.open).toHaveBeenCalledTimes(1);
        expect(window.open).toHaveBeenCalledWith('', 'tadpole-detached-test-tab', expect.any(String));
    });

    it('does NOT re-open the window if the on_close callback changes (strobe fix)', async () => {
        let rerenderFn: any;
        await act(async () => {
            const { rerender } = render(
                <Portal_Window id="test-tab" title="Test Tab" on_close={mockOnClose}>
                    <div>Content</div>
                </Portal_Window>
            );
            rerenderFn = rerender;
        });

        expect(window.open).toHaveBeenCalledTimes(1);

        // Re-render with a NEW inline function (simulating parent re-render)
        await act(async () => {
            rerenderFn(
                <Portal_Window id="test-tab" title="Test Tab" on_close={() => {}}>
                    <div>Content</div>
                </Portal_Window>
            );
        });

        // Should STILL only have been called once
        expect(window.open).toHaveBeenCalledTimes(1);
    });

    it('updates document title when title prop changes without re-opening window', async () => {
        let rerenderFn: any;
        await act(async () => {
            const { rerender } = render(
                <Portal_Window id="test-tab" title="Initial Title" on_close={mockOnClose}>
                    <div>Content</div>
                </Portal_Window>
            );
            rerenderFn = rerender;
        });

        expect(window.open).toHaveBeenCalledTimes(1);
        expect(mockWindow.document.title).toBe('Initial Title | Tadpole OS Detached');

        await act(async () => {
            rerenderFn(
                <Portal_Window id="test-tab" title="Updated Title" on_close={mockOnClose}>
                    <div>Content</div>
                </Portal_Window>
            );
        });

        // Title should update
        expect(mockWindow.document.title).toBe('Updated Title | Tadpole OS Detached');
        
        // But window should NOT re-open
        expect(window.open).toHaveBeenCalledTimes(1);
    });

    it('closes the window and calls cleanup on unmount', async () => {
        let unmountFn: any;
        await act(async () => {
            const { unmount } = render(
                <Portal_Window id="test-tab" title="Test Tab" on_close={mockOnClose}>
                    <div>Content</div>
                </Portal_Window>
            );
            unmountFn = unmount;
        });

        await act(async () => {
            unmountFn();
        });

        await act(async () => {
            await new Promise(resolve => setTimeout(resolve, 0));
        });

        expect(mockWindow.close).toHaveBeenCalledTimes(1);
    });

    it('survives React StrictMode double-mount when the popup blocker denies a second open', async () => {
        // Browsers consume the click's transient user activation on the first window.open().
        // A second open (StrictMode remount) is popup-blocked and returns null.
        vi.mocked(window.open)
            .mockReset()
            .mockReturnValueOnce(mockWindow as any)
            .mockReturnValue(null);
        const onPopupBlock = vi.fn();

        await act(async () => {
            render(
                <StrictMode>
                    <Portal_Window id="strict-tab" title="Strict Tab" on_close={mockOnClose} on_popup_block={onPopupBlock}>
                        <div>Content</div>
                    </Portal_Window>
                </StrictMode>
            );
        });
        await act(async () => {
            await new Promise(resolve => setTimeout(resolve, 0));
        });

        expect(window.open).toHaveBeenCalledTimes(1);
        expect(onPopupBlock).not.toHaveBeenCalled();
        expect(mockOnClose).not.toHaveBeenCalled();
        expect(mockWindow.close).not.toHaveBeenCalled();
        expect(mockWindow.document.body.textContent).toContain('Content');
    });

    it('invokes on_close at most once even when beforeunload, pagehide, and unload all fire', async () => {
        const listeners: Record<string, Array<() => void>> = {};
        mockWindow.addEventListener = vi.fn((event: string, cb: () => void) => {
            listeners[event] = listeners[event] || [];
            listeners[event].push(cb);
        });

        await act(async () => {
            render(
                <Portal_Window id="once-tab" title="Once Tab" on_close={mockOnClose}>
                    <div>Content</div>
                </Portal_Window>
            );
        });

        listeners.beforeunload?.forEach((fn) => fn());
        listeners.pagehide?.forEach((fn) => fn());
        listeners.unload?.forEach((fn) => fn());

        expect(mockOnClose).toHaveBeenCalledTimes(1);
    });

    it('resizes without tearing down or reopening the popup when width/height props change', async () => {
        mockWindow.resizeTo = vi.fn();
        let rerenderFn: any;
        await act(async () => {
            const { rerender } = render(
                <Portal_Window id="resize-tab" title="Resize Tab" width={1000} height={700} on_close={mockOnClose}>
                    <div>Content</div>
                </Portal_Window>
            );
            rerenderFn = rerender;
        });

        expect(window.open).toHaveBeenCalledTimes(1);

        await act(async () => {
            rerenderFn(
                <Portal_Window id="resize-tab" title="Resize Tab" width={1400} height={900} on_close={mockOnClose}>
                    <div>Content</div>
                </Portal_Window>
            );
        });

        expect(window.open).toHaveBeenCalledTimes(1);
        expect(mockWindow.resizeTo).toHaveBeenCalledWith(1400, 900);
    });
});
