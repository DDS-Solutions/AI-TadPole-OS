import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, AlertTriangle, CheckCircle, Info, ShieldAlert } from 'lucide-react';
import { useNotificationStore } from '../../stores/notificationStore';
import type { Notification } from '../../stores/notificationStore';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

const ToastItem: React.FC<{ notification: Notification }> = ({ notification }) => {
    const { removeNotification } = useNotificationStore();

    const icons = {
        info: <Info className="w-5 h-5 text-blue-400" />,
        success: <CheckCircle className="w-5 h-5 text-emerald-400" />,
        warning: <AlertTriangle className="w-5 h-5 text-amber-400" />,
        error: <ShieldAlert className="w-5 h-5 text-rose-500" />,
    };

    return (
        <motion.div
            layout
            initial={{ opacity: 0, x: 50, scale: 0.9 }}
            animate={{ opacity: 1, x: 0, scale: 1 }}
            exit={{ opacity: 0, scale: 0.95, transition: { duration: 0.2 } }}
            className={cn(
                "relative group flex items-start gap-3 p-4 mb-3 min-w-[320px] max-w-md",
                "bg-black/60 backdrop-blur-xl border rounded-xl shadow-2xl overflow-hidden",
                notification.severity === 'error' ? "border-rose-500/50 shadow-rose-900/20" : "border-white/10 shadow-black/40"
            )}
        >
            {/* Background Glow for Errors */}
            {notification.severity === 'error' && (
                <div className="absolute inset-0 bg-rose-500/5 pointer-events-none" />
            )}

            <div className="flex-shrink-0 mt-0.5">
                {icons[notification.severity]}
            </div>

            <div className="flex-grow flex flex-col gap-1 pr-6">
                <span className={cn(
                    "text-sm font-bold tracking-tight uppercase",
                    notification.severity === 'error' ? "text-rose-400" : "text-white/90"
                )}>
                    {notification.title}
                </span>
                <p className="text-sm text-white/60 leading-relaxed">
                    {notification.message}
                </p>
                {notification.typeId && (
                    <span className="text-[10px] font-mono text-white/30 uppercase mt-1">
                        ID: {notification.typeId}
                    </span>
                )}
            </div>

            <button
                onClick={() => removeNotification(notification.id)}
                className="absolute top-3 right-3 p-1 rounded-md text-white/30 hover:text-white hover:bg-white/10 transition-colors"
                title={notification.persistent ? "Manual Dismiss Required" : "Close"}
            >
                <X className="w-4 h-4" />
            </button>

            {/* Persistence Indicator */}
            {notification.persistent && (
                <div className="absolute bottom-0 left-0 h-[2px] w-full bg-rose-500/30" />
            )}
        </motion.div>
    );
};

export const ToastCenter: React.FC = () => {
    const { notifications } = useNotificationStore();

    return (
        <div className="fixed bottom-6 right-6 z-[9999] flex flex-col items-end pointer-events-none">
            <div className="pointer-events-auto">
                <AnimatePresence mode="popLayout">
                    {notifications.map((n) => (
                        <ToastItem key={n.id} notification={n} />
                    ))}
                </AnimatePresence>
            </div>
        </div>
    );
};
