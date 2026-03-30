import React, { useState, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Search, Command, User, Users, FileText, Settings, Zap, Database } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAgentStore } from '../stores/agentStore';
import { useSovereignStore } from '../stores/sovereignStore';
import { getSettings } from '../stores/settingsStore';
import clsx from 'clsx';
import { i18n } from '../i18n';

interface CommandItem {
    id: string;
    title: string;
    description: string;
    icon: React.ElementType;
    category: 'Agent' | 'Page' | 'Action' | 'Memory';
    action: () => void;
}

export const CommandPalette: React.FC<{ isOpen: boolean; onClose: () => void }> = ({ isOpen, onClose }): React.ReactElement | null => {
    const [query, setQuery] = useState('');
    const [selectedIndex, setSelectedIndex] = useState(0);
    const [memoryResults, setMemoryResults] = useState<CommandItem[]>([]);
    const [isSearching, setIsSearching] = useState(false);
    const { agents } = useAgentStore();
    const { setSelectedAgentId, setScope } = useSovereignStore();
    const navigate = useNavigate();
    const inputRef = useRef<HTMLInputElement>(null);

    // Debounced memory search
    useEffect(() => {
        if (!query || query.length < 3) {
            setMemoryResults([]);
            return;
        }

        const timeout = setTimeout(async (): Promise<void> => {
            setIsSearching(true);
            try {
                const { TadpoleOSUrl, TadpoleOSApiKey } = getSettings();
                const res = await fetch(`${TadpoleOSUrl}/v1/search/memory?query=${encodeURIComponent(query)}`, {
                    headers: {
                        'Authorization': `Bearer ${TadpoleOSApiKey}`
                    }
                });
                const data = await res.json();
                if (data.status === 'success') {
                    const results = data.entries.map((m: { id: string; text: string }): CommandItem => ({
                        id: `mem-${m.id}`,
                        title: m.text,
                        description: i18n.t('command.memory_found'),
                        icon: Database,
                        category: 'Memory',
                        action: () => {
                            onClose();
                        }
                    }));
                    setMemoryResults(results);
                }
            } catch (err) {
                console.error("Memory search failed:", err);
            } finally {
                setIsSearching(false);
            }
        }, 300);

        return () => clearTimeout(timeout);
    }, [query, onClose]);

    const staticItems: CommandItem[] = [
        ...agents.map((agent): CommandItem => ({
            id: `agent-${agent.id}`,
            title: agent.name,
            description: agent.role,
            icon: User,
            category: 'Agent' as const,
            action: () => {
                setSelectedAgentId(agent.id);
                setScope('agent');
                onClose();
            }
        })),
        { id: 'page-ops', title: i18n.t('command.ops_center'), description: i18n.t('command.main_dashboard'), icon: Zap, category: 'Page', action: () => { navigate('/'); onClose(); } },
        { id: 'page-org', title: i18n.t('command.agent_hierarchy'), description: i18n.t('command.org_chart'), icon: Users, category: 'Page', action: () => { navigate('/org-chart'); onClose(); } },
        { id: 'page-caps', title: i18n.t('command.skills_workflows'), description: i18n.t('command.skills_hub'), icon: Settings, category: 'Page', action: () => { navigate('/skills'); onClose(); } },
        { id: 'action-clear', title: i18n.t('command.clear_history'), description: i18n.t('command.reset_chat'), icon: FileText, category: 'Action', action: () => { useSovereignStore.getState().clearHistory(); onClose(); } },
    ];

    const filteredStatic = staticItems.filter((item): boolean =>
        (item.title?.toLowerCase() || '').includes(query.toLowerCase()) ||
        (item.description?.toLowerCase() || '').includes(query.toLowerCase())
    );

    const allItems = [...filteredStatic, ...memoryResults].slice(0, 10);

    useEffect(() => {
        if (isOpen) {
            setTimeout(() => inputRef.current?.focus(), 10);
        }
    }, [isOpen]);

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent): void => {
            if (!isOpen) return;
            if (e.key === 'ArrowDown') {
                setSelectedIndex(prev => (prev + 1) % allItems.length);
                e.preventDefault();
            } else if (e.key === 'ArrowUp') {
                setSelectedIndex(prev => (prev - 1 + allItems.length) % allItems.length);
                e.preventDefault();
            } else if (e.key === 'Enter') {
                allItems[selectedIndex]?.action();
                e.preventDefault();
            } else if (e.key === 'Escape') {
                onClose();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, allItems, selectedIndex, onClose]);

    return (
        <AnimatePresence>
            {isOpen && (
                <div className="fixed inset-0 z-[200] flex items-start justify-center pt-[15vh] px-4">
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        className="fixed inset-0 bg-black/40 backdrop-blur-sm z-0"
                        onClick={onClose}
                        onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && onClose()}
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
                            <Search className={clsx("transition-colors", isSearching ? "text-blue-500 animate-pulse" : "text-zinc-500")} size={20} />
                            <input
                                ref={inputRef}
                                type="text"
                                value={query}
                                    onChange={e => setQuery(e.target.value)}
                                    placeholder={i18n.t('command.search_placeholder')}
                                    aria-label={i18n.t('command.search_aria')}
                                    className="flex-1 bg-transparent border-none focus:ring-0 text-zinc-100 placeholder:text-zinc-600 font-medium"
                            />
                            <div className="px-1.5 py-0.5 rounded border border-zinc-800 text-[10px] text-zinc-600 font-mono">
                                {i18n.t('command.esc')}
                            </div>
                        </div>

                        <div className="max-h-[400px] overflow-y-auto p-2 custom-scrollbar">
                            {allItems.map((item, index): React.ReactElement => {
                                const Icon = item.icon;
                                return (
                                    <button
                                        key={item.id}
                                        onMouseEnter={() => setSelectedIndex(index)}
                                        onClick={item.action}
                                        className={clsx(
                                            "w-full flex items-center gap-3 p-3 rounded-xl transition-all text-left group",
                                            index === selectedIndex ? "bg-zinc-800/80 text-zinc-100 shadow-inner" : "text-zinc-400 hover:text-zinc-200"
                                        )}
                                        role="option"
                                        aria-selected={index === selectedIndex}
                                        id={`command-item-${item.id}`}
                                    >
                                        <div className={clsx(
                                            "p-2 rounded-lg transition-colors",
                                            index === selectedIndex ? "bg-zinc-700 text-blue-400" : "bg-zinc-950 text-zinc-600"
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
                            {allItems.length === 0 && (
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
