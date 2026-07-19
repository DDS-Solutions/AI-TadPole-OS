/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Assist Note
 * **UI Sub-Component**: Extracted header bar for the Sovereign Chat.
 * Contains the drag handle, status LED, voice waveform animation,
 * and action buttons (transcript toggle, minimize, detach, clear).
 * Memoized to prevent re-renders from message list changes.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Drag handle not responding (if dragControls undefined in detached mode).
 * - **Telemetry Link**: Search for `[Chat_Header]` in browser logs.
 */

import React from 'react';
import { motion } from 'framer-motion';
import {
    X,
    Minimize2,
    Zap,
    ExternalLink,
    GripVertical,
    Activity
} from 'lucide-react';
import clsx from 'clsx';
import { type DragControls } from 'framer-motion';
import { type Voice_Status } from '../../services/voice_client';
import { type Sovereign_Scope } from '../../stores/sovereign_store';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

export interface Chat_Header_Props {
    is_detached: boolean;
    active_scope: Sovereign_Scope;
    target_node: string;
    is_speaking: boolean;
    voice_status: Voice_Status;
    show_transcript: boolean;
    on_toggle_transcript: () => void;
    on_minimize: () => void;
    on_toggle_detach: () => void;
    on_clear_history: () => void;
    drag_controls?: DragControls;
    on_header_click?: () => void;
}

/**
 * Chat_Header
 * Renders the title bar with drag-to-move, status indicators, and action buttons.
 * Memoized to avoid re-rendering when messages change.
 */
export const Chat_Header = React.memo<Chat_Header_Props>(({
    is_detached,
    active_scope,
    target_node,
    is_speaking,
    voice_status,
    show_transcript,
    on_toggle_transcript,
    on_minimize,
    on_toggle_detach,
    on_clear_history,
    drag_controls,
    on_header_click,
}) => {
    return (
        <div
            onPointerDown={(e) => {
                if (!is_detached && drag_controls) {
                    drag_controls.start(e);
                }
            }}
            className={clsx(
                "relative z-10 p-4 border-b border-[color:var(--color-border)]/50 bg-[color:var(--color-background)]/40 backdrop-blur-md flex items-center justify-between shrink-0 overflow-hidden cursor-pointer select-none",
                !is_detached && "cursor-grab active:cursor-grabbing"
            )}
            onDoubleClick={on_header_click}
            onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && on_header_click?.()}
            role="button"
            tabIndex={0}
            title={i18n.t('chat.header_drag_hint')}
            aria-label={i18n.t('chat.header_drag_aria')}
        >
            <div className="flex items-center gap-3">
                {!is_detached && <GripVertical size={14} className="text-zinc-700" />}
                <div className="relative bg-zinc-100 p-1.5 rounded-md text-black shadow-lg">
                    <Zap size={14} className="fill-current" />
                </div>
                <div>
                    <span className="font-bold text-[11px] tracking-[0.2em] text-zinc-100 uppercase">{i18n.t('chat.title')}</span>
                    <div className="flex items-center gap-1.5">
                        <div className={clsx(
                            "h-1 w-1 rounded-full animate-pulse",
                            voice_status === 'active' ? "bg-emerald-500" :
                            voice_status === 'initializing' ? "bg-green-500" :
                            voice_status === 'stalled' ? "bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.8)]" : "bg-emerald-500"
                        )} />
                        <span className="text-[9px] text-zinc-500 font-mono uppercase tracking-tighter">
                            {i18n.t(`chat.scope_${active_scope}`)} / {active_scope !== 'swarm' ? target_node : i18n.t('chat.sovereign_link')}
                        </span>
                        {is_speaking && (
                            <div className="flex items-center gap-0.5 ml-2 mr-1">
                                {[1, 2, 3, 4].map(i => (
                                    <motion.div
                                        key={i}
                                        animate={{ height: [4, 12, 4] }}
                                        transition={{ repeat: Infinity, duration: 0.6, delay: i * 0.1 }}
                                        className="w-0.5 bg-green-500 rounded-full"
                                    />
                                ))}
                            </div>
                        )}
                    </div>
                </div>
            </div>
            <div className="flex items-center gap-1">
                <Tooltip content={show_transcript ? i18n.t('chat.show_chat_tooltip') : i18n.t('chat.show_transcript_tooltip')} position="top">
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            on_toggle_transcript();
                        }}
                        className={clsx(
                            "p-2 rounded-lg transition-all active:scale-95",
                            show_transcript ? "text-green-400 bg-green-500/10 border border-green-500/30 shadow-[0_0_15px_rgba(59,130,246,0.15)]" : "text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800/50"
                        )}
                        aria-label={i18n.t('chat.toggle_transcript_aria')}
                    >
                        <Activity size={16} />
                    </button>
                </Tooltip>
                {!is_detached && (
                    <Tooltip content={i18n.t('chat.minimize_tooltip')} position="top">
                        <button
                            onClick={(e) => {
                                e.stopPropagation();
                                on_minimize();
                            }}
                            className="p-2 text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800/50 rounded-lg transition-colors"
                            aria-label={i18n.t('chat.minimize_aria')}
                        >
                            <Minimize2 size={16} />
                        </button>
                    </Tooltip>
                )}
                <Tooltip content={is_detached ? i18n.t('chat.restore_tooltip') : i18n.t('chat.detach_tooltip')} position="top">
                    <button
                        onClick={(e) => { e.stopPropagation(); on_toggle_detach(); }}
                        className={clsx(
                            "p-2 rounded-lg transition-colors",
                            is_detached ? "text-green-400 bg-green-500/10" : "text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800/50"
                        )}
                        aria-label={is_detached ? i18n.t('chat.restore_aria') : i18n.t('chat.detach_aria')}
                    >
                        <ExternalLink size={16} />
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('chat.close_tooltip')} position="top">
                    <button
                        onClick={(e) => { e.stopPropagation(); on_clear_history(); }}
                        className="p-2 text-red-500/50 hover:text-red-400 hover:bg-red-500/10 rounded-lg transition-colors"
                        aria-label={i18n.t('chat.close_aria')}
                    >
                        <X size={16} />
                    </button>
                </Tooltip>
            </div>
        </div>
    );
});

Chat_Header.displayName = 'Chat_Header';

// Metadata: [Chat_Header]
