import React from 'react';
import { Settings, Search, Upload } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface Skill_Header_Props {
    active_category: 'user' | 'ai';
    set_active_category: (category: 'user' | 'ai') => void;
    search_query: string;
    set_search_query: (query: string) => void;
    handle_import_click: () => void;
    is_saving: boolean;
    user_registry_count: number;
    ai_services_count: number;
}

export const Skill_Header: React.FC<Skill_Header_Props> = ({
    active_category,
    set_active_category,
    search_query,
    set_search_query,
    handle_import_click,
    is_saving,
    user_registry_count,
    ai_services_count
}) => {
    return (
        <div className="flex items-center justify-between border-b border-zinc-900 pb-2 px-1 shrink-0">
            <div>
                <h2 className="text-xl font-bold flex items-center gap-2 text-zinc-100">
                    <Tooltip content={i18n.t('skills.tooltip_main')} position="right">
                        <Settings className="text-blue-500 cursor-help" />
                    </Tooltip>
                    {i18n.t('skills.title')}
                </h2>
                <div className="mt-1"></div>
            </div>

            <div className="flex items-center gap-4">
                <div className="flex items-center p-1 bg-zinc-900 rounded-lg border border-zinc-800 self-center">
                    <button
                        onClick={() => set_active_category('user')}
                        className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all flex items-center gap-2 ${active_category === 'user'
                            ? 'bg-blue-600 text-white shadow-lg'
                            : 'text-zinc-500 hover:text-zinc-300'
                            }`}
                    >
                        User Registry
                        <span className={`px-1 rounded ${active_category === 'user' ? 'bg-black/20 text-white' : 'bg-zinc-800 text-zinc-600'}`}>
                            {user_registry_count}
                        </span>
                    </button>
                    <button
                        onClick={() => set_active_category('ai')}
                        className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all flex items-center gap-2 ${active_category === 'ai'
                            ? 'bg-indigo-600 text-white shadow-lg'
                            : 'text-zinc-500 hover:text-zinc-300'
                            }`}
                    >
                        AI Services
                        <span className={`px-1 rounded ${active_category === 'ai' ? 'bg-black/20 text-white' : 'bg-zinc-800 text-zinc-600'}`}>
                            {ai_services_count}
                        </span>
                    </button>
                </div>

                <div className="flex items-center gap-2">
                    <button
                        onClick={handle_import_click}
                        disabled={is_saving}
                        className="bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-300 px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-2 transition-all hover:border-indigo-500/50"
                    >
                        <Upload className="w-3.5 h-3.5" /> Import .md
                    </button>
                </div>

                <div className="relative">
                    <input
                        type="text"
                        placeholder={i18n.t('agent_manager.placeholder_search')}
                        value={search_query}
                        onChange={(e) => set_search_query(e.target.value)}
                        className="bg-zinc-900 border border-zinc-800 rounded-lg pl-8 pr-3 py-1.5 text-xs text-zinc-200 focus:outline-none focus:border-blue-500/50 w-48 transition-all"
                    />
                    <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 text-zinc-600" size={12} />
                </div>
            </div>
        </div>
    );
};
