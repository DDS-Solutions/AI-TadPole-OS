/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Sub-Component**: Extracted message list renderer for the Sovereign Chat.
 * Memoized to prevent re-renders when parent scope/target selectors change.
 * Uses content-visibility: auto for implicit DOM virtualization.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Message key collisions, or stale rendered_messages reference after scope switch.
 * - **Telemetry Link**: Search for `[Chat_Message_List]` in React DevTools Profiler.
 */

import React, { useMemo } from 'react';
import { motion } from 'framer-motion';
import { Bot, User, Zap } from 'lucide-react';
import clsx from 'clsx';
import { type Message_Part } from '../../stores/sovereign_store';
import { i18n } from '../../i18n';

export interface Chat_Message {
    id: string;
    sender_id: string;
    sender_name: string;
    text: string;
    scope: string;
    target_node?: string;
    timestamp: number | string | Date;
    parts?: Message_Part[];
    agent_id?: string;
}

interface Chat_Message_Item_Props {
    msg: Chat_Message;
}

/**
 * Individual message bubble — memoized by ID to avoid re-rendering
 * unchanged messages when the list grows.
 */
const Chat_Message_Item = React.memo<Chat_Message_Item_Props>(({ msg }) => {
    return (
        <motion.div
            initial={{ opacity: 0, x: msg.sender_id === '0' ? 10 : -10 }}
            animate={{ opacity: 1, x: 0 }}
            style={{ contentVisibility: 'auto', containIntrinsicSize: '180px' }}
            className={clsx(
                "flex flex-col gap-2 max-w-[90%]",
                msg.sender_id === '0' ? "ml-auto items-end" : "items-start"
            )}
        >
            <div className="flex items-center gap-2 px-1">
                {msg.sender_id !== '0' && (
                    <Bot size={12} className="text-zinc-500" />
                )}
                <span className="text-[9px] font-bold text-zinc-500 uppercase tracking-widest">
                    {msg.sender_name}
                </span>
                {msg.sender_id === '0' && (
                    <User size={12} className="text-zinc-500" />
                )}
            </div>
            <div
                className={clsx(
                    "px-4 py-3 rounded-2xl text-[13px] leading-relaxed relative group border transition-all duration-300 overflow-hidden break-words whitespace-pre-wrap flex flex-col gap-3",
                    msg.sender_id === '0'
                        ? "bg-zinc-100 text-zinc-900 rounded-tr-sm shadow-[0_5px_20px_-5px_rgba(255,255,255,0.2)]"
                        : "bg-zinc-800/80 text-zinc-200 rounded-tl-sm border-zinc-700/50 shadow-xl",
                    (msg.text.length > 800 || (msg.parts?.length || 0) > 5) ? "max-h-[400px] overflow-y-auto custom-scrollbar text-xs" : ""
                )}
            >
                {msg.parts && (msg.parts as Message_Part[]).length > 0 ? (
                    ((msg.parts as Message_Part[]) || []).map((part: Message_Part, idx) => (
                        <div key={`${msg.id}-part-${idx}`} className="animate-in fade-in slide-in-from-bottom-1 duration-300">
                            {part.type === 'text' && (
                                <span className="opacity-90">{part.content}</span>
                            )}
                            {part.type === 'thought' && (
                                <div className="flex items-start gap-2 py-1 px-2 bg-black/20 rounded-lg border border-white/5 italic text-zinc-400 text-[11px] leading-tight group/thought">
                                    <div className="mt-1">
                                        {part.status === 'thinking' ? (
                                            <div className="w-2 h-2 rounded-full bg-blue-500/50 animate-ping" />
                                        ) : (
                                            <Bot size={10} className="text-blue-400/60" />
                                        )}
                                    </div>
                                    <span>{part.content}</span>
                                </div>
                            )}
                            {part.type === 'tool' && (
                                <div className="flex flex-col gap-1 py-2 px-3 bg-zinc-900/40 rounded-lg border border-zinc-700/30 group/tool">
                                    <div className="flex items-center gap-2">
                                        <Zap size={10} className="text-amber-400" />
                                        <span className="font-mono text-[10px] uppercase tracking-wider text-zinc-300">
                                            {i18n.t('chat.label_execution')} {part.name || 'unknown'}
                                        </span>
                                    </div>
                                    {Boolean(part.output) && (
                                        <div className="mt-1 text-[10px] font-mono text-zinc-500 overflow-hidden text-ellipsis">
                                            {i18n.t('chat.label_result')} {typeof part.output === 'string' ? part.output : JSON.stringify(part.output).slice(0, 100) + '...'}
                                        </div>
                                    )}
                                </div>
                            )}
                        </div>
                    ))
                ) : (
                    msg.text
                )}
                {msg.scope === 'swarm' && msg.sender_id !== '0' && (
                    <div className="absolute top-0 right-0 p-1">
                        <Zap size={8} className="text-blue-400" />
                    </div>
                )}
            </div>
            <div className="flex flex-row items-center gap-2 px-1 text-[8px] font-mono text-zinc-600 uppercase tracking-tighter whitespace-nowrap">
                <span>{new Date(msg.timestamp).toLocaleTimeString([], { hour12: false, minute: '2-digit' })}</span>
                <span>•</span>
                <span className="text-zinc-400">{i18n.t(`chat.scope_${msg.scope}`)}</span>
            </div>
        </motion.div>
    );
}, (prev, next) => prev.msg.id === next.msg.id);

Chat_Message_Item.displayName = 'Chat_Message_Item';

// ── Exported List Component ──────────────────────────────

interface Chat_Message_List_Props {
    messages: Chat_Message[];
    max_rendered: number;
    active_scope: string;
    target_node: string;
}

/**
 * Chat_Message_List
 * Renders the filtered and capped message list for Sovereign Chat.
 * Memoized: only re-renders when the messages array reference or count changes.
 */
export const Chat_Message_List = React.memo<Chat_Message_List_Props>(({
    messages,
    max_rendered,
    active_scope,
    target_node,
}) => {
    const rendered = useMemo(() => {
        if (messages.length <= max_rendered) return messages;
        return messages.slice(messages.length - max_rendered);
    }, [messages, max_rendered]);

    const hidden_count = messages.length - rendered.length;

    if (messages.length === 0) {
        return (
            <div className="h-full flex flex-col items-center justify-center text-zinc-700 space-y-4 opacity-40 text-center">
                <Bot size={32} className="neural-pulse" />
                <div>
                    <p className="text-xs font-bold uppercase tracking-[0.3em]">
                        {active_scope === 'agent' ? i18n.t('chat.isolated_stream', { target: target_node }) : i18n.t('chat.waiting_packets')}
                    </p>
                    <p className="text-[9px] font-mono mt-1 opacity-60">{i18n.t('chat.ready_for_input')}</p>
                </div>
            </div>
        );
    }

    return (
        <>
            {hidden_count > 0 && (
                <div className="text-[9px] font-mono uppercase tracking-[0.14em] text-zinc-500 bg-zinc-900/50 border border-zinc-800 rounded-lg px-3 py-2 text-center">
                    {i18n.t('chat.rendering_capped', { count: max_rendered, hidden: hidden_count })}
                </div>
            )}
            {(rendered || []).map((msg) => (
                <Chat_Message_Item key={msg.id} msg={msg} />
            ))}
        </>
    );
});

Chat_Message_List.displayName = 'Chat_Message_List';
