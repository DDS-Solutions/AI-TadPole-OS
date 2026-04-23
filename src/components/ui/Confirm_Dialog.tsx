/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: High-fidelity confirmation gate for terminal actions. 
 * Orchestrates destructive intent verification with keyboard entrapment (ESC/Enter).
 * 
 * ### 🔍 Debugging & Observability
 * - **Telemetry Link**: Search for `[Confirm_Dialog]` in browser logs.
 */

import React, { useCallback, useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import { i18n } from '../../i18n';
import { THEME_TOKENS, Z_INDEX_MAP } from './theme_tokens';

interface Confirm_Dialog_Props {
    is_open: boolean;
    title: string;
    message: string;
    confirm_label?: string;
    cancel_label?: string;
    variant?: 'danger' | 'warning' | 'info' | 'success';
    on_confirm: () => void;
    on_cancel: () => void;
}

// Removed local VARIANT_MAP in favor of THEME_TOKENS

export const Confirm_Dialog: React.FC<Confirm_Dialog_Props> = ({
    is_open,
    title,
    message,
    confirm_label = i18n.t('common.confirm'),
    cancel_label = i18n.t('common.cancel'),
    variant = 'danger',
    on_confirm,
    on_cancel,
}) => {
    const dialog_ref = useRef<HTMLDivElement>(null);
    const variantStyle = THEME_TOKENS[variant] || THEME_TOKENS.info;

    const handle_key_down = useCallback(
        (e: KeyboardEvent) => {
            // Check if this modal is the active one (focus is within it)
            // or if it's the only one by checking the DOM order.
            // A simple focus check usually works if we have a focus trap.
            if (e.key === 'Escape') on_cancel();
            if (e.key === 'Enter') {
                // Prevent multiple Enter presses if layered
                const modals = document.querySelectorAll('[role="dialog"]');
                if (modals.length && modals[modals.length - 1] === dialog_ref.current) {
                    on_confirm();
                }
            }

            if (e.key === 'Tab' && dialog_ref.current) {
                const focusableElements = dialog_ref.current.querySelectorAll(
                    'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
                );
                const firstElement = focusableElements[0] as HTMLElement;
                const lastElement = focusableElements[focusableElements.length - 1] as HTMLElement;

                if (e.shiftKey) {
                    if (document.activeElement === firstElement) {
                        lastElement.focus();
                        e.preventDefault();
                    }
                } else {
                    if (document.activeElement === lastElement) {
                        firstElement.focus();
                        e.preventDefault();
                    }
                }
            }
        },
        [on_cancel, on_confirm]
    );

    useEffect(() => {
        if (is_open) {
            document.addEventListener('keydown', handle_key_down);
            
            // Focus trap initial focus
            const timer = setTimeout(() => {
                if (dialog_ref.current) {
                    const focusableElements = dialog_ref.current.querySelectorAll('button');
                    if (focusableElements.length > 0) {
                        focusableElements[focusableElements.length - 1].focus(); // Focus confirm button
                    }
                }
            }, 10);
            
            return () => {
                document.removeEventListener('keydown', handle_key_down);
                clearTimeout(timer);
            };
        }
    }, [is_open, handle_key_down]);

    if (!is_open) return null;

    return createPortal(
        <div
            className="fixed inset-0 flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in duration-200"
            style={{ zIndex: Z_INDEX_MAP.dialog }}
            onClick={on_cancel}
        >
            <div
                ref={dialog_ref}
                role="dialog"
                aria-modal="true"
                aria-labelledby="dialog-title"
                aria-describedby="dialog-message"
                onClick={(e) => e.stopPropagation()}
                className="bg-zinc-900 border border-zinc-800 rounded-xl p-6 max-w-[420px] w-[90%] shadow-2xl animate-in zoom-in-95 slide-in-from-bottom-4 duration-300"
            >
                <h3 id="dialog-title" className={`text-base font-bold mb-2 uppercase tracking-tight ${variantStyle.split(' ')[0]}`}>
                    {title}
                </h3>
                <p id="dialog-message" className="text-sm text-zinc-400 mb-6 leading-relaxed">
                    {message}
                </p>
                <div className="flex gap-3 justify-end">
                    <button
                        onClick={on_cancel}
                        className="px-4 py-2 bg-zinc-800 text-zinc-400 border border-zinc-700 rounded-lg text-xs font-bold uppercase tracking-wider hover:bg-zinc-700 transition-all active:scale-95"
                    >
                        {cancel_label}
                    </button>
                    <button
                        onClick={on_confirm}
                        className={`px-4 py-2 border rounded-lg text-xs font-bold uppercase tracking-wider transition-all active:scale-95 ${variantStyle}`}
                    >
                        {confirm_label}
                    </button>
                </div>
            </div>
        </div>,
        document.getElementById('portal-root') || document.body
    );
};

// Metadata: [Confirm_Dialog]
