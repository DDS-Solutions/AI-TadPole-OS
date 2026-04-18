/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: High-fidelity confirmation gate for terminal actions. 
 * Orchestrates destructive intent verification with keyboard entrapment (ESC/Enter) and variant-aware styling.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Keydown listener leak if not unmounted, z-index collision with other fixed overlays (Portals), or variant color mismatch.
 * - **Telemetry Link**: Search for `[Confirm_Dialog]` or `on_confirm` in browser logs.
 */

import React, { useCallback, useEffect, useRef } from 'react';
import { i18n } from '../../i18n';

interface Confirm_Dialog_Props {
    is_open: boolean;
    title: string;
    message: string;
    confirm_label?: string;
    cancel_label?: string;
    variant?: 'danger' | 'warning' | 'info';
    on_confirm: () => void;
    on_cancel: () => void;
}

const VARIANT_COLORS = {
    danger: { accent: '#f87171', bg: 'rgba(248, 113, 113, 0.1)', border: 'rgba(248, 113, 113, 0.25)' },
    warning: { accent: '#fbbf24', bg: 'rgba(251, 191, 36, 0.1)', border: 'rgba(251, 191, 36, 0.25)' },
    info: { accent: '#60a5fa', bg: 'rgba(96, 165, 250, 0.1)', border: 'rgba(96, 165, 250, 0.25)' },
};

/**
 * Confirm_Dialog
 * Reusable confirmation dialog for destructive actions.
 * Managed with high-visibility overlays and variant-aware styling.
 */
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
    const colors = VARIANT_COLORS[variant];

    const handle_key_down = useCallback(
        (e: KeyboardEvent) => {
            if (e.key === 'Escape') on_cancel();
            if (e.key === 'Enter') on_confirm();
        },
        [on_cancel, on_confirm]
    );

    useEffect(() => {
        if (is_open) {
            document.addEventListener('keydown', handle_key_down);
            return () => document.removeEventListener('keydown', handle_key_down);
        }
    }, [is_open, handle_key_down]);

    if (!is_open) return null;

    return (
        <div
            style={{
                position: 'fixed',
                inset: 0,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                background: 'rgba(0, 0, 0, 0.6)',
                backdropFilter: 'blur(4px)',
                zIndex: 10000,
                animation: 'fadeIn 0.15s ease-out',
            }}
            onClick={on_cancel}
        >
            <div
                ref={dialog_ref}
                onClick={(e) => e.stopPropagation()}
                style={{
                    background: 'var(--surface-elevated, #1e293b)',
                    border: `1px solid ${colors.border}`,
                    borderRadius: 12,
                    padding: '24px 28px',
                    maxWidth: 420,
                    width: '90%',
                    boxShadow: '0 20px 40px rgba(0, 0, 0, 0.5)',
                    animation: 'slideUp 0.2s ease-out',
                }}
            >
                <h3
                    style={{
                        margin: '0 0 8px',
                        fontSize: '1rem',
                        fontWeight: 700,
                        color: colors.accent,
                    }}
                >
                    {title}
                </h3>
                <p
                    style={{
                        margin: '0 0 20px',
                        fontSize: '0.8rem',
                        color: 'var(--text-secondary, #94a3b8)',
                        lineHeight: 1.5,
                    }}
                >
                    {message}
                </p>
                <div style={{ display: 'flex', gap: 10, justifyContent: 'flex-end' }}>
                    <button
                        onClick={on_cancel}
                        style={{
                            background: 'rgba(148, 163, 184, 0.1)',
                            color: '#94a3b8',
                            border: '1px solid rgba(148, 163, 184, 0.2)',
                            borderRadius: 6,
                            padding: '8px 16px',
                            fontSize: '0.75rem',
                            fontWeight: 600,
                            cursor: 'pointer',
                        }}
                    >
                        {cancel_label}
                    </button>
                    <button
                        onClick={on_confirm}
                        style={{
                            background: colors.bg,
                            color: colors.accent,
                            border: `1px solid ${colors.border}`,
                            borderRadius: 6,
                            padding: '8px 16px',
                            fontSize: '0.75rem',
                            fontWeight: 600,
                            cursor: 'pointer',
                        }}
                    >
                        {confirm_label}
                    </button>
                </div>
            </div>
        </div>
    );
};

