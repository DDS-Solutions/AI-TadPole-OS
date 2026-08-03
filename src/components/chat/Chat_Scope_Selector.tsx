/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Assist Note
 * **UI Sub-Component**: Extracted scope/target selector for the Sovereign Chat.
 * Contains the Agent/Cluster/Swarm tab bar and the agent + cluster
 * dropdown selectors with animated transitions.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Dropdown not closing (if setOpenDropdown not called on selection),
 *   or stale agent list after roster change.
 * - **Telemetry Link**: Search for `[Chat_Scope_Selector]` in browser logs.
 */

import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Target as TargetIcon, ChevronDown } from 'lucide-react';
import clsx from 'clsx';
import { type Sovereign_Scope } from '../../stores/sovereign_store';
import { type Mission_Cluster } from '../../stores/workspace_store';
import type { Agent } from '../../types';
import { i18n } from '../../i18n';

export interface Chat_Scope_Selector_Props {
    active_scope: Sovereign_Scope;
    on_set_scope: (scope: Sovereign_Scope) => void;
    target_agent: string;
    target_cluster: string;
    open_dropdown: 'agent' | 'cluster' | null;
    set_open_dropdown: (val: 'agent' | 'cluster' | null) => void;
    sorted_agents: Agent[];
    clusters: Mission_Cluster[];
    set_target_agent: (name: string) => void;
    set_selected_agent_id: (id: string) => void;
    set_target_cluster: (name: string) => void;
    set_input_text: (text: string) => void;
}

/**
 * Chat_Scope_Selector
 * Renders scope tabs (agent/cluster/swarm) and the agent + cluster dropdowns.
 * Memoized to avoid re-rendering when message list changes.
 */
export const Chat_Scope_Selector = React.memo<Chat_Scope_Selector_Props>(({
    active_scope,
    on_set_scope,
    target_agent,
    target_cluster,
    open_dropdown,
    set_open_dropdown,
    sorted_agents,
    clusters,
    set_target_agent,
    set_selected_agent_id,
    set_target_cluster,
    set_input_text,
}) => {
    return (
        <div className="relative z-20 flex flex-col border-b border-[color:var(--color-border)]/30">
            <div className="flex p-1.5 bg-[color:var(--color-background)]/20 backdrop-blur-sm gap-1">
                {(['agent', 'cluster', 'swarm'] as Sovereign_Scope[]).map(scope => (
                    <button
                        key={scope}
                        onClick={() => on_set_scope(scope)}
                        className={clsx(
                            "flex-1 py-1.5 px-2 text-[10px] font-bold uppercase tracking-[0.15em] rounded-md transition-all relative overflow-hidden",
                            active_scope === scope ? "text-zinc-100" : "text-zinc-600 hover:text-zinc-400"
                        )}
                        aria-pressed={active_scope === scope}
                    >
                        {active_scope === scope && (
                            <motion.div layoutId="scope-bg" className="absolute inset-0 bg-zinc-800 border border-zinc-700/50 shadow-inner rounded-md" />
                        )}
                        <span className="relative z-10">{i18n.t(`chat.scope_${scope}`)}</span>
                    </button>
                ))}
            </div>

            {active_scope !== 'swarm' && (
                <div className="px-3 pb-2 flex items-center gap-2">
                    {/* Agent Selector */}
                    <div className="relative flex-1 min-w-0">
                        <button
                            onClick={() => {
                                set_open_dropdown(open_dropdown === 'agent' ? null : 'agent');
                                if (active_scope !== 'agent') on_set_scope('agent');
                            }}
                            className={clsx(
                                "w-full flex items-center justify-between gap-2 text-[10px] font-bold transition-colors uppercase tracking-widest bg-[color:var(--color-surface)]/50 px-2 py-1.5 rounded border group",
                                active_scope === 'agent' ? "border-green-500/50 text-green-400" : "border-[color:var(--color-border)] text-zinc-500 hover:text-zinc-300"
                            )}
                            aria-haspopup="listbox"
                            aria-expanded={open_dropdown === 'agent'}
                            aria-label={i18n.t('chat.select_agent_aria')}
                        >
                            <div className="flex items-center gap-1.5 truncate">
                                <TargetIcon size={12} className={active_scope === 'agent' ? "text-green-500" : "text-zinc-600"} />
                                <span className="truncate">{i18n.t('chat.agent_prefix')}<span className={active_scope === 'agent' ? "text-zinc-100" : "text-zinc-400"}>{target_agent || i18n.t('chat.select_placeholder')}</span></span>
                            </div>
                            <ChevronDown size={12} className={clsx("transition-transform flex-shrink-0", open_dropdown === 'agent' && "rotate-180")} />
                        </button>

                        <AnimatePresence>
                            {open_dropdown === 'agent' && (
                                <motion.div
                                    initial={{ opacity: 0, y: -10 }}
                                    animate={{ opacity: 1, y: 0 }}
                                    exit={{ opacity: 0, y: -10 }}
                                    className="absolute left-0 top-full mt-1 w-full min-w-[160px] bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-lg shadow-2xl z-20 py-1 overflow-y-auto max-h-64 custom-scrollbar backdrop-blur-xl"
                                >
                                    {(sorted_agents || []).map((agent: Agent) => (
                                        <button
                                            key={agent.id}
                                            onClick={() => {
                                                set_target_agent(agent.name);
                                                set_selected_agent_id(agent.id);
                                                on_set_scope('agent');
                                                set_open_dropdown(null);
                                                set_input_text(`@${agent.name}: `);
                                            }}
                                            className="w-full text-left px-3 py-2 text-xs hover:bg-zinc-800 text-zinc-400 hover:text-zinc-100 flex items-center gap-2 transition-colors"
                                        >
                                            <div className={clsx("w-2 h-2 rounded-full flex-shrink-0", agent.status === 'offline' ? "opacity-30" : "")} style={{ backgroundColor: agent.theme_color || '#52525b' }} />
                                            <span className={clsx("truncate flex-1 max-w-[100px]", agent.status === 'offline' && "text-zinc-600")}>{agent.name}</span>
                                            {agent.status !== 'offline' && agent.status !== 'idle' && (
                                                <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse ml-auto" />
                                            )}
                                        </button>
                                    ))}
                                </motion.div>
                            )}
                        </AnimatePresence>
                    </div>

                    {/* Cluster Selector */}
                    <div className="relative flex-1 min-w-0">
                        <button
                            onClick={() => {
                                set_open_dropdown(open_dropdown === 'cluster' ? null : 'cluster');
                                if (active_scope !== 'cluster') on_set_scope('cluster');
                            }}
                            className={clsx(
                                "w-full flex items-center justify-between gap-2 text-[10px] font-bold transition-colors uppercase tracking-widest bg-[color:var(--color-surface)]/50 px-2 py-1.5 rounded border group",
                                active_scope === 'cluster' ? "border-emerald-500/50 text-emerald-400" : "border-[color:var(--color-border)] text-zinc-500 hover:text-zinc-300"
                            )}
                            aria-haspopup="listbox"
                            aria-expanded={open_dropdown === 'cluster'}
                            aria-label={i18n.t('chat.select_cluster_aria')}
                        >
                            <div className="flex items-center gap-1.5 truncate">
                                <TargetIcon size={12} className={active_scope === 'cluster' ? "text-emerald-500" : "text-zinc-600"} />
                                <span className="truncate">{i18n.t('chat.cluster_prefix')}<span className={active_scope === 'cluster' ? "text-zinc-100" : "text-zinc-400"}>{target_cluster || i18n.t('chat.select_placeholder')}</span></span>
                            </div>
                            <ChevronDown size={12} className={clsx("transition-transform flex-shrink-0", open_dropdown === 'cluster' && "rotate-180")} />
                        </button>

                        <AnimatePresence>
                            {open_dropdown === 'cluster' && (
                                <motion.div
                                    initial={{ opacity: 0, y: -10 }}
                                    animate={{ opacity: 1, y: 0 }}
                                    exit={{ opacity: 0, y: -10 }}
                                    className="absolute right-0 top-full mt-1 w-full min-w-[160px] bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-lg shadow-2xl z-20 py-1 overflow-y-auto max-h-64 custom-scrollbar backdrop-blur-xl"
                                >
                                    {(clusters || []).map(cluster => (
                                        <button
                                            key={cluster.id}
                                            onClick={() => {
                                                set_target_cluster(cluster.name);
                                                on_set_scope('cluster');
                                                set_open_dropdown(null);
                                            }}
                                            className="w-full text-left px-3 py-2 text-xs hover:bg-zinc-800 text-zinc-400 hover:text-zinc-100 flex items-center gap-2 transition-colors"
                                        >
                                            <div className={clsx(
                                                "w-2 h-2 rounded-full flex-shrink-0",
                                                cluster.theme === 'cyan' ? 'bg-cyan-500' :
                                                    cluster.theme === 'zinc' ? 'bg-zinc-500' :
                                                        cluster.theme === 'amber' ? 'bg-amber-500' : 'bg-green-500'
                                            )} />
                                            <span className="truncate">{cluster.name}</span>
                                        </button>
                                    ))}
                                </motion.div>
                            )}
                        </AnimatePresence>
                    </div>
                </div>
            )}
        </div>
    );
});

Chat_Scope_Selector.displayName = 'Chat_Scope_Selector';

// Metadata: [Chat_Scope_Selector]
