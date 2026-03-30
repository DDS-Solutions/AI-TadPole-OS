import { create } from 'zustand';

export type NotificationSeverity = 'info' | 'success' | 'warning' | 'error';

export interface Notification {
    id: string;
    severity: NotificationSeverity;
    title: string;
    message: string;
    typeId?: string;
    persistent: boolean;
    timestamp: Date;
}

interface NotificationState {
    notifications: Notification[];
    addNotification: (notification: Omit<Notification, 'id' | 'timestamp'>) => void;
    removeNotification: (id: string) => void;
    clearAll: () => void;
}

export const useNotificationStore = create<NotificationState>((set) => ({
    notifications: [],

    addNotification: (notification) => {
        const id = Math.random().toString(36).substring(2, 9);
        const newNotification: Notification = {
            ...notification,
            id,
            timestamp: new Date(),
        };

        set((state) => ({
            notifications: [newNotification, ...state.notifications],
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

    removeNotification: (id) => {
        set((state) => ({
            notifications: state.notifications.filter((n) => n.id !== id),
        }));
    },

    clearAll: () => set({ notifications: [] }),
}));
