/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Global neural command hub for rapid navigation and memory search. 
 * Implements fuzzy matching and debounced SME data retrieval via `agent_api_service`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Debounced search timeout (API drop), keyboard trap if focus loss occurs during animation, or memory result rendering of malformed DTOs.
 * - **Telemetry Link**: Search for `[Command_Palette]` or `search_memory` in service logs.
 */

import React, { useState, useEffect, useRef } from 'react';
import type { Agent_Memory_Entry } from '../contracts/agent';
import { motion, AnimatePresence } from 'framer-motion';
import { Search, Command, User, Users, FileText, Settings, Zap, Database, Store, ShoppingBag, Lock } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { use_agent_store } from '../stores/agent_store';
import { use_sovereign_store } from '../stores/sovereign_store';
import { agent_api_service } from '../services/agent_api_service';
import clsx from 'clsx';
import { i18n } from '../i18n';

interface Command_Item {
    id: string;
    title: string;
    description: string;
    icon: React.ElementType;
    category: 'Agent' | 'Page' | 'Action' | 'Memory';
    action: () => void;
}

/**
 * Command_Palette
 * Global command interface for rapid navigation, agent selection, and neural memory search.
 * Features fuzzy matching and debounced SME data retrieval.
 */
export const Command_Palette: React.FC<{ is_open: boolean; on_close: () => void }> = ({ is_open, on_close }): React.ReactElement | null => {
    const [query, set_query] = useState('');
    const [selected_index, set_selected_index] = useState(0);
    const [memory_results, set_memory_results] = useState<Command_Item[]>([]);
    const [is_searching, set_is_searching] = useState(false);
    const { agents } = use_agent_store();
    const { set_selected_agent_id, set_scope } = use_sovereign_store();
    const navigate = useNavigate();
    const input_ref = useRef<HTMLInputElement>(null);

    // Debounced memory search
    useEffect(() => {
        let is_active = true;
        if (!query || query.length < 3) {
            void Promise.resolve().then(() => set_memory_results([]));
            return;
        }

        const timeout = setTimeout(async (): Promise<void> => {
            set_is_searching(true);
            try {
                const data = await agent_api_service.search_memory(query);
                if (is_active && data.status === 'success') {
                    const results = data.entries.map((m: Agent_Memory_Entry): Command_Item => ({
                        id: `mem-${m.id}`,
                        title: m.text,
                        description: i18n.t('command.memory_found'),
                        icon: Database,
                        category: 'Memory',
                        action: () => {
                            on_close();
                        }
                    }));
                    set_memory_results(results);
                }
            } catch (err) {
                if (is_active) console.error("Memory search failed:", err);
            } finally {
                if (is_active) set_is_searching(false);
            }
        }, 300);

        return () => {
            is_active = false;
            clearTimeout(timeout);
        };
    }, [query, on_close]);

    const static_items: Command_Item[] = [
        ...(agents || []).map((agent): Command_Item => ({
            id: `agent-${agent.id}`,
            title: agent.name,
            description: agent.role,
            icon: User,
            category: 'Agent' as const,
            action: () => {
                set_selected_agent_id(agent.id);
                set_scope('agent');
                on_close();
            }
        })),
        { id: 'page-ops', title: i18n.t('command.ops_center'), description: i18n.t('command.main_dashboard'), icon: Zap, category: 'Page', action: () => { navigate('/'); on_close(); } },
        { id: 'page-org', title: i18n.t('command.agent_hierarchy'), description: i18n.t('command.org_chart'), icon: Users, category: 'Page', action: () => { navigate('/org-chart'); on_close(); } },
        { id: 'page-caps', title: i18n.t('command.skills_workflows'), description: i18n.t('command.skills_hub'), icon: Settings, category: 'Page', action: () => { navigate('/skills'); on_close(); } },
        { id: 'page-model-store', title: i18n.t('command.model_store'), description: i18n.t('command.model_store_desc'), icon: Store, category: 'Page', action: () => { navigate('/infra/model-store'); on_close(); } },
        { id: 'page-template-store', title: i18n.t('command.template_store'), description: i18n.t('command.template_store_desc'), icon: ShoppingBag, category: 'Page', action: () => { navigate('/store'); on_close(); } },
        { id: 'page-security', title: i18n.t('command.security_dashboard'), description: i18n.t('command.security_dashboard_desc'), icon: Lock, category: 'Page', action: () => { navigate('/security'); on_close(); } },
        { id: 'action-clear', title: i18n.t('command.clear_history'), description: i18n.t('command.reset_chat'), icon: FileText, category: 'Action', action: () => { use_sovereign_store.getState().clear_history(); on_close(); } },
    ];

    const filtered_static = static_items.filter((item): boolean =>
        (item.title?.toLowerCase() || '').includes(query.toLowerCase()) ||
        (item.description?.toLowerCase() || '').includes(query.toLowerCase())
    );

    const all_items = [...filtered_static, ...(memory_results || [])].slice(0, 10);

    useEffect(() => {
        if (is_open) {
            setTimeout(() => input_ref.current?.focus(), 10);
        }
    }, [is_open]);

    const handle_key_down = (e: React.KeyboardEvent): void => {
        if (e.key === 'ArrowDown') {
            set_selected_index(prev => (prev + 1) % all_items.length);
            e.preventDefault();
        } else if (e.key === 'ArrowUp') {
            set_selected_index(prev => (prev - 1 + all_items.length) % all_items.length);
            e.preventDefault();
        } else if (e.key === 'Enter') {
            all_items[selected_index]?.action();
            e.preventDefault();
        } else if (e.key === 'Escape') {
            on_close();
        } else if (e.key === 'Tab') {
            // Focus Trap / Cycle results
            if (e.shiftKey) {
                set_selected_index(prev => (prev - 1 + all_items.length) % all_items.length);
            } else {
                set_selected_index(prev => (prev + 1) % all_items.length);
            }
            e.preventDefault();
        }
    };

    return (
        <AnimatePresence>
            {is_open && (
                <div className="fixed inset-0 z-[200] flex items-start justify-center pt-[15vh] px-4">
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        className="fixed inset-0 bg-black/40 backdrop-blur-sm z-0"
                        onClick={on_close}
                        onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && on_close()}
                        role="button"
                        tabIndex={-1}
                        aria-hidden="true"
                    />

                    <motion.div
                        initial={{ opacity: 0, scale: 0.95, y: -20 }}
                        animate={{ opacity: 1, scale: 1, y: 0 }}
                        exit={{ opacity: 0, scale: 0.95, y: -20 }}
                        className="relative w-full max-w-xl bg-zinc-900 border border-zinc-800 rounded-2xl shadow-[0_30px_60px_-15px_rgba(0,0,0,0.7)] overflow-hidden"
                    >
                        <div className="p-4 border-b border-zinc-800 flex items-center gap-3">
                            <Search className={clsx("transition-colors", is_searching ? "text-green-500 animate-pulse" : "text-zinc-500")} size={20} />
                            <input
                                ref={input_ref}
                                type="text"
                                value={query}
                                onChange={e => set_query(e.target.value)}
                                onKeyDown={handle_key_down}
                                placeholder={i18n.t('command.search_placeholder')}
                                aria-label={i18n.t('command.search_aria')}
                                className="flex-1 bg-transparent border-none focus:ring-0 text-zinc-100 placeholder:text-zinc-600 font-medium"
                            />
                            <div className="px-1.5 py-0.5 rounded border border-zinc-800 text-[10px] text-zinc-600 font-mono">
                                {i18n.t('command.esc')}
                            </div>
                        </div>

                        <div className="max-h-[400px] overflow-y-auto p-2 custom-scrollbar">
                            {(all_items || []).map((item, index): React.ReactElement => {
                                const Icon = item.icon;
                                return (
                                    <button
                                        key={item.id}
                                        onMouseEnter={() => set_selected_index(index)}
                                        onClick={item.action}
                                        className={clsx(
                                            "w-full flex items-center gap-3 p-3 rounded-xl transition-all text-left group",
                                            index === selected_index ? "bg-zinc-800/80 text-zinc-100 shadow-inner" : "text-zinc-400 hover:text-zinc-200"
                                        )}
                                        role="option"
                                        aria-selected={index === selected_index}
                                        id={`command-item-${item.id}`}
                                    >
                                        <div className={clsx(
                                            "p-2 rounded-lg transition-colors",
                                            index === selected_index ? "bg-zinc-700 text-green-400" : "bg-zinc-950 text-zinc-600"
                                        )}>
                                            <Icon size={18} aria-hidden="true" />
                                        </div>
                                        <div className="flex-1 min-w-0">
                                            <div className="text-sm font-bold truncate">{item.title}</div>
                                            <div className="text-[10px] text-zinc-600 font-medium truncate uppercase tracking-wider">{item.description}</div>
                                        </div>
                                        <div className="text-[9px] text-zinc-700 font-mono uppercase tracking-widest">{i18n.t(`command.cat_${item.category.toLowerCase()}`)}</div>
                                    </button>
                                );
                            })}
                            {(all_items || []).length === 0 && (
                                <div className="p-12 text-center text-zinc-600 font-mono text-xs uppercase tracking-widest">
                                    {i18n.t('command.no_results', { query })}
                                </div>
                            )}
                        </div>

                        <div className="p-3 border-t border-zinc-800 bg-zinc-950/50 flex items-center justify-between text-[10px] text-zinc-600 font-bold uppercase tracking-widest">
                            <div className="flex gap-4">
                                <span className="flex items-center gap-1"><Command size={10} /> {i18n.t('command.hint_close')}</span>
                                <span className="flex items-center gap-1">{i18n.t('command.hint_navigate')}</span>
                                <span className="flex items-center gap-1">{i18n.t('command.hint_select')}</span>
                            </div>
                            <span className="text-zinc-800">{i18n.t('command.hub_footer')}</span>
                        </div>
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    );
};


// Metadata: [Command_Palette]
