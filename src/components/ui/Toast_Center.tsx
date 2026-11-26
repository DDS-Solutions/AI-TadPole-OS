/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Ui / Toast_Center
 * - **Primary Entrypoints**: `Toast_Center`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, AlertTriangle, CheckCircle, Info, ShieldAlert } from 'lucide-react';
import { use_notification_store } from '../../stores/notification_store';
import type { Notification } from '../../stores/notification_store';
import { i18n } from '../../i18n';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';
import { Z_INDEX_MAP } from './theme_tokens';

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

/**
 * Toast_Item
 * Individual notification card with severity-based styling and framer-motion animations.
 */
const Toast_Item: React.FC<{ notification: Notification }> = ({ notification }) => {
    const { remove_notification } = use_notification_store();

    const icons = {
        info: <Info className="w-5 h-5 text-green-400" />,
        success: <CheckCircle className="w-5 h-5 text-emerald-400" />,
        warning: <AlertTriangle className="w-5 h-5 text-amber-400" />,
        error: <ShieldAlert className="w-5 h-5 text-rose-500" />,
    };

    const close_label = notification.persistent ? i18n.t('common.manual_dismiss') : i18n.t('common.close');

    return (
        <motion.div
            layout
            role="status"
            aria-live={notification.severity === 'error' ? 'assertive' : 'polite'}
            initial={{ opacity: 0, x: 50, scale: 0.9 }}
            animate={{ opacity: 1, x: 0, scale: 1 }}
            exit={{ opacity: 0, scale: 0.95, transition: { duration: 0.2 } }}
            className={cn(
                "relative group flex flex-col mb-3 min-w-[320px] w-[min(28rem,calc(100vw-3rem))] max-h-[min(420px,calc(100vh-11rem))]",
                "bg-zinc-950/90 backdrop-blur-xl border rounded-xl shadow-2xl overflow-hidden shrink-0",
                notification.severity === 'error' ? "border-rose-500/50 shadow-rose-900/20" : "border-white/10 shadow-black/40"
            )}
        >
            {/* Background Glow for Errors */}
            {notification.severity === 'error' && (
                <div className="absolute inset-0 bg-rose-500/5 pointer-events-none" />
            )}

            {/* Pinned Header with Always-Visible Close Button */}
            <div className="relative z-10 flex items-start justify-between gap-3 px-4 pt-3.5 pb-2 border-b border-white/5 shrink-0">
                <div className="flex items-start gap-2.5 min-w-0">
                    <div className="flex-shrink-0 mt-0.5">
                        {icons[notification.severity]}
                    </div>
                    <span className={cn(
                        "text-sm font-bold tracking-tight uppercase break-words",
                        notification.severity === 'error' ? "text-rose-400" : "text-white/90"
                    )}>
                        {notification.title}
                    </span>
                </div>

                <button
                    type="button"
                    onClick={() => remove_notification(notification.id)}
                    aria-label={close_label}
                    className="shrink-0 p-1 rounded-md text-zinc-400 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
                    title={close_label}
                >
                    <X className="w-4 h-4" />
                </button>
            </div>

            {/* Scrollable Body for Long Responses / Trace Details */}
            <div
                data-testid="toast-body-scroll"
                className="relative z-10 flex-1 min-h-0 overflow-y-auto overscroll-contain custom-scrollbar px-4 py-2.5"
            >
                <p className="text-sm text-zinc-300 leading-relaxed whitespace-pre-wrap break-words">
                    {notification.message}
                </p>
            </div>

            {notification.type_id && (
                <div className="relative z-10 px-4 pb-3 pt-1 border-t border-white/5 shrink-0">
                    <span className="text-[10px] font-mono text-zinc-400 uppercase">
                        ID: {notification.type_id}
                    </span>
                </div>
            )}

            {/* Persistence Indicator */}
            {notification.persistent && (
                <div className="absolute bottom-0 left-0 h-[2px] w-full bg-rose-500/30 pointer-events-none" />
            )}
        </motion.div>
    );
};

/**
 * Toast_Center
 * Global notification container for real-time system alerts and user feedback.
 * Clamped inside viewport top/bottom bounds so close controls are never clipped by the browser header.
 */
export const Toast_Center: React.FC = () => {
    const { notifications, clear_all } = use_notification_store();
    const visible_notifications = (notifications || []).slice(-5); // Prevent DOM lag during error loops

    return (
        <div
            data-testid="toast-center-container"
            className="fixed top-16 bottom-20 right-6 flex flex-col justify-end items-end max-w-[calc(100vw-3rem)] pointer-events-none"
            style={{ zIndex: Z_INDEX_MAP.toast }}
        >
            <div className="pointer-events-auto flex flex-col items-end max-h-full overflow-y-auto overflow-x-hidden custom-scrollbar pr-1">
                <AnimatePresence mode="popLayout">
                    {visible_notifications.map((n) => (
                        <Toast_Item key={n.id} notification={n} />
                    ))}
                </AnimatePresence>
                {visible_notifications.length > 1 && typeof clear_all === 'function' && (
                    <button
                        type="button"
                        onClick={clear_all}
                        aria-label="Clear all notifications"
                        className="mt-1 px-3 py-1 rounded-lg bg-zinc-900/90 hover:bg-zinc-800 border border-white/10 text-[10px] font-mono uppercase tracking-wider text-zinc-300 hover:text-white transition-colors shadow-lg cursor-pointer shrink-0"
                    >
                        Clear All ({visible_notifications.length})
                    </button>
                )}
            </div>
        </div>
    );
};
