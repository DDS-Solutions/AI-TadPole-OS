import React from 'react';

/**
 * Status_Badge_Props
 * Defines the strict interface for the status badge component.
 */
interface Status_Badge_Props {
    status: string;
    size?: 'sm' | 'md' | 'lg';
    pulse?: boolean;
}

const STATUS_COLORS: Record<string, { bg: string; text: string; glow: string }> = {
    idle: { bg: 'rgba(100, 116, 139, 0.15)', text: '#94a3b8', glow: 'none' },
    active: { bg: 'rgba(52, 211, 153, 0.15)', text: '#34d399', glow: '0 0 8px rgba(52, 211, 153, 0.4)' },
    thinking: { bg: 'rgba(96, 165, 250, 0.15)', text: '#60a5fa', glow: '0 0 8px rgba(96, 165, 250, 0.4)' },
    running: { bg: 'rgba(96, 165, 250, 0.15)', text: '#60a5fa', glow: '0 0 8px rgba(96, 165, 250, 0.4)' },
    paused: { bg: 'rgba(251, 191, 36, 0.15)', text: '#fbbf24', glow: '0 0 8px rgba(251, 191, 36, 0.3)' },
    error: { bg: 'rgba(248, 113, 113, 0.15)', text: '#f87171', glow: '0 0 8px rgba(248, 113, 113, 0.4)' },
    failed: { bg: 'rgba(248, 113, 113, 0.15)', text: '#f87171', glow: '0 0 8px rgba(248, 113, 113, 0.4)' },
    completed: { bg: 'rgba(52, 211, 153, 0.15)', text: '#34d399', glow: 'none' },
    pending: { bg: 'rgba(251, 191, 36, 0.15)', text: '#fbbf24', glow: '0 0 6px rgba(251, 191, 36, 0.25)' },
    connected: { bg: 'rgba(52, 211, 153, 0.15)', text: '#34d399', glow: '0 0 8px rgba(52, 211, 153, 0.4)' },
    disconnected: { bg: 'rgba(248, 113, 113, 0.15)', text: '#f87171', glow: 'none' },
};

const SIZES = {
    sm: { font_size: '0.65rem', padding: '2px 6px', dot_size: 5 },
    md: { font_size: '0.7rem', padding: '3px 8px', dot_size: 6 },
    lg: { font_size: '0.8rem', padding: '4px 10px', dot_size: 8 },
};

/**
 * Status_Badge
 * Reusable status badge with color-coded styling, optional pulse animation,
 * and status-aware glowing effects.
 */
export const Status_Badge: React.FC<Status_Badge_Props> = ({
    status,
    size = 'md',
    pulse = false,
}) => {
    const key = status.toLowerCase();
    const colors = STATUS_COLORS[key] ?? STATUS_COLORS.idle;
    const dim = SIZES[size];

    return (
        <span
            style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 4,
                background: colors.bg,
                color: colors.text,
                fontSize: dim.font_size,
                fontWeight: 600,
                fontFamily: "'JetBrains Mono', monospace",
                padding: dim.padding,
                borderRadius: 4,
                textTransform: 'uppercase',
                letterSpacing: '0.5px',
                boxShadow: colors.glow,
            }}
        >
            <span
                style={{
                    width: dim.dot_size,
                    height: dim.dot_size,
                    borderRadius: '50%',
                    backgroundColor: colors.text,
                    animation: pulse ? 'pulse-dot 2s ease-in-out infinite' : 'none',
                }}
            />
            {status}
        </span>
    );
};
