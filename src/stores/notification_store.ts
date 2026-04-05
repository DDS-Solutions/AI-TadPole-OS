/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: Swarm-wide alert and notification bus. 
 * Orchestrates the queueing, display, and auto-dismissal of system alerts, mission milestones, and security warnings.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Notification queue starvation (alerts not showing), or persistent "zombie" toasts after manual dismissal.
 * - **Telemetry Link**: Search for `[NotificationStore]` or `ALERT_PULSE` in UI logs.
 */

import { create } from 'zustand';

export type Notification_Severity = 'info' | 'success' | 'warning' | 'error';

export interface Notification {
    id: string;
    severity: Notification_Severity;
    title: string;
    message: string;
    type_id?: string;
    persistent: boolean;
    timestamp: Date;
}

interface Notification_State {
    notifications: Notification[];
    add_notification: (notification: Omit<Notification, 'id' | 'timestamp'>) => void;
    remove_notification: (id: string) => void;
    clear_all: () => void;
}

/**
 * use_notification_store
 * Manages global application notifications and toast alerts.
 * Features automatic dismissal for non-persistent alerts and severity-based grouping.
 */
export const use_notification_store = create<Notification_State>((set) => ({
    notifications: [],

    add_notification: (notification) => {
        const id = Math.random().toString(36).substring(2, 9);
        const new_notification: Notification = {
            ...notification,
            id,
            timestamp: new Date(),
        };

        set((state) => ({
            notifications: [new_notification, ...state.notifications],
        }));

        // Auto-dismiss logic for non-persistent notifications
        if (!notification.persistent) {
            setTimeout(() => {
                set((state) => ({
                    notifications: state.notifications.filter((n) => n.id !== id),
                }));
            }, 6000); // 6 seconds for standard toasts
        }
    },

    remove_notification: (id) => {
        set((state) => ({
            notifications: state.notifications.filter((n) => n.id !== id),
        }));
    },

    clear_all: () => set({ notifications: [] }),
}));

