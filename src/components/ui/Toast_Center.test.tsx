/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Toast_Center_test]` in observability traces.
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Toast_Center } from './Toast_Center';
import { use_notification_store } from '../../stores/notification_store';
import React from 'react';

// Mock the notification store
vi.mock('../../stores/notification_store', () => ({
    use_notification_store: vi.fn(),
}));

// Mock framer-motion
vi.mock('framer-motion', () => ({
    motion: {
        div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
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
});

// Metadata: [Toast_Center_test]
