import React, { useCallback, useEffect, useRef } from 'react';

interface ConfirmDialogProps {
    isOpen: boolean;
    title: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    variant?: 'danger' | 'warning' | 'info';
    onConfirm: () => void;
    onCancel: () => void;
}

const VARIANT_COLORS = {
    danger: { accent: '#f87171', bg: 'rgba(248, 113, 113, 0.1)', border: 'rgba(248, 113, 113, 0.25)' },
    warning: { accent: '#fbbf24', bg: 'rgba(251, 191, 36, 0.1)', border: 'rgba(251, 191, 36, 0.25)' },
    info: { accent: '#60a5fa', bg: 'rgba(96, 165, 250, 0.1)', border: 'rgba(96, 165, 250, 0.25)' },
};

/**
 * Reusable confirmation dialog for destructive actions.
 *
 * Replaces the inline confirm/prompt patterns in:
 * - AgentConfigPanel (delete agent)
 * - ModelManager (delete model/provider)
 * - Skills (delete skill/workflow)
 * - OversightDashboard (approve/reject oversight entry)
 */
export const ConfirmDialog: React.FC<ConfirmDialogProps> = ({
    isOpen,
    title,
    message,
    confirmLabel = 'Confirm',
    cancelLabel = 'Cancel',
    variant = 'danger',
    onConfirm,
    onCancel,
}) => {
    const dialogRef = useRef<HTMLDivElement>(null);
    const colors = VARIANT_COLORS[variant];

    const handleKeyDown = useCallback(
        (e: KeyboardEvent) => {
            if (e.key === 'Escape') onCancel();
            if (e.key === 'Enter') onConfirm();
        },
        [onCancel, onConfirm]
    );

    useEffect(() => {
        if (isOpen) {
            document.addEventListener('keydown', handleKeyDown);
            return () => document.removeEventListener('keydown', handleKeyDown);
        }
    }, [isOpen, handleKeyDown]);

    if (!isOpen) return null;

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
            onClick={onCancel}
        >
            <div
                ref={dialogRef}
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
                        onClick={onCancel}
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
                        {cancelLabel}
                    </button>
                    <button
                        onClick={onConfirm}
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
                        {confirmLabel}
                    </button>
                </div>
            </div>
        </div>
    );
};
