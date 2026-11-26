/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Ui / Toast_Center.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Toast_Center } from './Toast_Center';
import { use_notification_store } from '../../stores/notification_store';

// Mock the notification store
vi.mock('../../stores/notification_store', () => ({
    use_notification_store: vi.fn(),
}));

// Mock framer-motion
vi.mock('framer-motion', () => ({
    motion: {
        div: ({ children, layout: _layoutProp, _layout, ...props }: any) => <div {...props}>{children}</div>,
    },
    AnimatePresence: ({ children }: any) => <>{children}</>,
}));

describe('Toast_Center', () => {
    const mockRemove = vi.fn();

    it('renders notifications from the store', () => {
        (use_notification_store as any).mockReturnValue({
            notifications: [
                { id: '1', title: 'Test Alert', message: 'Something happened', severity: 'info', persistent: false },
            ],
            remove_notification: mockRemove,
        });

        render(<Toast_Center />);

        expect(screen.getByText('Test Alert')).toBeDefined();
        expect(screen.getByText('Something happened')).toBeDefined();
    });

    it('limits the number of visible notifications to 5', () => {
        const manyNotifications = Array.from({ length: 10 }, (_, i) => ({
            id: `${i}`,
            title: `Alert ${i}`,
            message: `Message ${i}`,
            severity: 'info',
            persistent: false,
        }));

        (use_notification_store as any).mockReturnValue({
            notifications: manyNotifications,
            remove_notification: mockRemove,
        });

        render(<Toast_Center />);

        // Should only show the last 5
        expect(screen.queryByText('Alert 0')).toBeNull();
        expect(screen.getByText('Alert 9')).toBeDefined();
        expect(screen.getByText('Alert 5')).toBeDefined();
    });

    it('calls remove_notification when the close button is clicked', () => {
        (use_notification_store as any).mockReturnValue({
            notifications: [
                { id: 'toast-123', title: 'Removable', message: 'Click X', severity: 'success', persistent: false },
            ],
            remove_notification: mockRemove,
        });

        render(<Toast_Center />);

        const closeButton = screen.getByTitle('Close');
        fireEvent.click(closeButton);

        expect(mockRemove).toHaveBeenCalledWith('toast-123');
    });

    it('applies error styling for error severity', () => {
        (use_notification_store as any).mockReturnValue({
            notifications: [
                { id: 'err-1', title: 'Critical', message: 'Failed', severity: 'error', persistent: true },
            ],
            remove_notification: mockRemove,
        });

        const { container } = render(<Toast_Center />);
        const toastItem = container.querySelector('[role="status"]');
        
        expect(toastItem?.className).toContain('border-rose-500');
    });

    it('clamps viewport bounds and makes long response bodies scrollable with a pinned close button', () => {
        const mockClearAll = vi.fn();
        const longMessage = Array.from({ length: 40 }, (_, i) => `Line ${i + 1}: detailed agent output`).join('\n');
        (use_notification_store as any).mockReturnValue({
            notifications: [
                { id: 'long-1', title: 'Agent Alert: Agent of Nine', message: longMessage, severity: 'warning', persistent: true },
                { id: 'long-2', title: 'System Alert', message: 'Second alert', severity: 'info', persistent: false },
            ],
            remove_notification: mockRemove,
            clear_all: mockClearAll,
        });

        const { container } = render(<Toast_Center />);

        const outerContainer = screen.getByTestId('toast-center-container');
        expect(outerContainer.className).toContain('top-16');
        expect(outerContainer.className).toContain('bottom-20');

        const scrollBodies = screen.getAllByTestId('toast-body-scroll');
        expect(scrollBodies[0].className).toContain('overflow-y-auto');
        expect(scrollBodies[0].className).toContain('min-h-0');

        const toastCard = container.querySelector('[role="status"]');
        expect(toastCard?.className).toContain('max-h-');

        const clearAllBtn = screen.getByRole('button', { name: /Clear all notifications/i });
        fireEvent.click(clearAllBtn);
        expect(mockClearAll).toHaveBeenCalledTimes(1);
    });
});
