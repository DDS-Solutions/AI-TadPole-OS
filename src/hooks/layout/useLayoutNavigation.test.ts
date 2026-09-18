/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useLayoutNavigation.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useLayoutNavigation } from './useLayoutNavigation';
import { use_tab_store } from '../../stores/tab_store';

const mock_navigate = vi.fn();
let mock_pathname = '/';

vi.mock('react-router-dom', () => ({
    useNavigate: () => mock_navigate,
    useLocation: () => ({ pathname: mock_pathname })
}));

describe('useLayoutNavigation', () => {
    const mock_set_palette = vi.fn();

    beforeEach(() => {
        vi.restoreAllMocks();
        mock_pathname = '/';
        use_tab_store.setState({
            tabs: [
                { id: 'tab-1', title: 'Dashboard', path: '/' } as any,
                { id: 'tab-2', title: 'Settings', path: '/settings' } as any
            ],
            active_tab_id: 'tab-1'
        });
    });

    it('toggles command palette on Ctrl+K shortcut', () => {
        renderHook(() => useLayoutNavigation(mock_set_palette));

        act(() => {
            const event = new KeyboardEvent('keydown', { key: 'k', ctrlKey: true });
            window.dispatchEvent(event);
        });

        expect(mock_set_palette).toHaveBeenCalled();
    });

    it('navigates via number keys when not focused in an input or contenteditable', () => {
        renderHook(() => useLayoutNavigation(mock_set_palette));

        act(() => {
            const event = new KeyboardEvent('keydown', { key: '6' });
            window.dispatchEvent(event);
        });

        expect(mock_navigate).toHaveBeenCalledWith('/settings');
    });

    it('ignores number key shortcuts when focused in input, textarea, or contenteditable elements', () => {
        renderHook(() => useLayoutNavigation(mock_set_palette));

        // Simulated contenteditable div
        const editableDiv = document.createElement('div');
        Object.defineProperty(editableDiv, 'isContentEditable', { value: true });

        act(() => {
            const event = new KeyboardEvent('keydown', { key: '6', bubbles: true });
            Object.defineProperty(event, 'target', { value: editableDiv });
            window.dispatchEvent(event);
        });

        expect(mock_navigate).not.toHaveBeenCalled();
    });

    it('does not trigger navigation shortcuts inside detached window paths', () => {
        mock_pathname = '/detached/chat';
        renderHook(() => useLayoutNavigation(mock_set_palette));

        act(() => {
            const event = new KeyboardEvent('keydown', { key: '2' });
            window.dispatchEvent(event);
        });

        expect(mock_navigate).not.toHaveBeenCalled();
    });
});
