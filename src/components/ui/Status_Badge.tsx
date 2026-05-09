/**
 * @docs ARCHITECTURE:Interface
 * @docs DESIGN_SYNERGY:Components
 * 
 * ### AI Assist Note
 * **Atomic UI Component**: Universal status signaling token. 
 * Maps system states to deterministic Tailwind classes.
 * 
 * ### 🔍 Debugging & Observability
 * - **Telemetry Link**: Search for `[Status_Badge]` in UI tracing.
 */

import React from 'react';
import { THEME_TOKENS } from './theme_tokens';

const STATUS_MAP = {
    idle: 'bg-zinc-500/10 text-zinc-400 border-zinc-500/20',
    active: `${THEME_TOKENS.success} shadow-[0_0_8px_rgba(34,197,94,0.2)]`,
    thinking: `${THEME_TOKENS.info} shadow-[0_0_8px_rgba(59,130,246,0.2)]`,
    running: `${THEME_TOKENS.info} shadow-[0_0_8px_rgba(59,130,246,0.2)]`,
    paused: THEME_TOKENS.warning,
    error: THEME_TOKENS.danger,
    failed: THEME_TOKENS.danger,
    completed: THEME_TOKENS.success,
    pending: THEME_TOKENS.warning,
    connected: THEME_TOKENS.success,
    disconnected: THEME_TOKENS.danger,
};

interface Status_Badge_Props {
    status: keyof typeof STATUS_MAP | (string & {});
    size?: 'sm' | 'md' | 'lg';
    pulse?: boolean;
    className?: string;
}



const SIZE_CLASSES = {
    sm: 'text-[9px] px-1.5 py-0.5 gap-1',
    md: 'text-[10px] px-2 py-1 gap-1.5',
    lg: 'text-[11px] px-2.5 py-1.5 gap-2',
};

const DOT_SIZES = {
    sm: 'w-1 h-1',
    md: 'w-1.5 h-1.5',
    lg: 'w-2 h-2',
};

export const Status_Badge: React.FC<Status_Badge_Props> = ({
    status,
    size = 'md',
    pulse = false,
    className = '',
}) => {
    const key = status.toLowerCase() as keyof typeof STATUS_MAP;
    const styleClass = STATUS_MAP[key] ?? STATUS_MAP.idle;
    const sizeClass = SIZE_CLASSES[size];
    const dotSize = DOT_SIZES[size];

    return (
        <span className={`inline-flex items-center font-bold font-mono uppercase tracking-wider border rounded-md transition-all duration-300 ${styleClass} ${sizeClass} ${className}`}>
            <span className={`${dotSize} rounded-full bg-current ${pulse ? 'animate-pulse' : ''}`} />
            {status}
        </span>
    );
};

// Metadata: [Status_Badge]
