/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Interface**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Skill_Header]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Contextual header for the Capability Forge. 
 */

import React from 'react';
import { Settings, Search, Upload } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface Skill_Header_Props {
    stats: {
        user_registry_count: number;
        ai_services_count: number;
    };
    handlers: {
        set_active_category: (category: 'user' | 'ai') => void;
        set_search_query: (query: string) => void;
        handle_import_click: () => void;
        on_create_skill: () => void;
        on_create_workflow: () => void;
    };
    state: {
        active_category: 'user' | 'ai';
        search_query: string;
        is_saving: boolean;
    };
}

export const Skill_Header: React.FC<Skill_Header_Props> = ({ stats, handlers, state }) => {
    return (
        <div className="flex items-center justify-between border-b border-zinc-900 pb-2 px-1 shrink-0">
            <div>
                <h1 className="text-xl font-bold flex items-center gap-2 text-zinc-100">
                    <Tooltip content={i18n.t('skills.tooltip_main')} position="right">
                        <Settings className="text-green-500 cursor-help" />
                    </Tooltip>
                    {i18n.t('skills.title')}
                </h1>
            </div>

            <div className="flex items-center gap-4">
                <div className="flex items-center p-1 bg-zinc-900 rounded-lg border border-zinc-800 self-center">
                    <button
                        onClick={() => handlers.set_active_category('user')}
                        className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all flex items-center gap-2 ${state.active_category === 'user'
                            ? 'bg-green-600 text-white shadow-lg'
                            : 'text-zinc-500 hover:text-zinc-300'
                            }`}
                    >
                        {i18n.t('skills.user_registry')}
                        <span className={`px-1 rounded ${state.active_category === 'user' ? 'bg-black/20 text-white' : 'bg-zinc-800 text-zinc-600'}`}>
                            {stats.user_registry_count}
                        </span>
                    </button>
                    <button
                        onClick={() => handlers.set_active_category('ai')}
                        className={`px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-md transition-all flex items-center gap-2 ${state.active_category === 'ai'
                            ? 'bg-green-600 text-white shadow-lg'
                            : 'text-zinc-500 hover:text-zinc-300'
                            }`}
                    >
                        {i18n.t('skills.ai_services')}
                        <span className={`px-1 rounded ${state.active_category === 'ai' ? 'bg-black/20 text-white' : 'bg-zinc-800 text-zinc-600'}`}>
                            {stats.ai_services_count}
                        </span>
                    </button>
                </div>

                <div className="flex items-center gap-2">
                    <Tooltip content={i18n.t('skills.tooltip_import')} position="bottom">
                        <button
                            onClick={handlers.handle_import_click}
                            disabled={state.is_saving}
                            className="bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-400 hover:text-zinc-100 px-3 py-1.5 rounded-lg text-[10px] uppercase tracking-widest font-bold flex items-center gap-2 transition-all hover:border-green-500/50"
                        >
                            <Upload className="w-3.5 h-3.5" /> {i18n.t('skills.btn_import')}
                        </button>
                    </Tooltip>
                    
                    <Tooltip content={i18n.t('skills.tooltip_new_skill')} position="bottom">
                        <button
                            onClick={handlers.on_create_skill}
                            className="bg-green-600 hover:bg-green-500 text-white px-3 py-1.5 rounded-lg text-[10px] uppercase tracking-widest font-bold transition-all shadow-lg shadow-green-500/20 active:scale-95"
                        >
                            + {i18n.t('skills.btn_new_skill')}
                        </button>
                    </Tooltip>
 
                    <Tooltip content={i18n.t('skills.tooltip_new_workflow')} position="bottom">
                        <button
                            onClick={handlers.on_create_workflow}
                            className="bg-zinc-800 hover:bg-zinc-700 text-white px-3 py-1.5 rounded-lg text-[10px] uppercase tracking-widest font-bold transition-all shadow-lg shadow-zinc-500/10 active:scale-95 border border-zinc-700"
                        >
                            + {i18n.t('skills.btn_new_workflow')}
                        </button>
                    </Tooltip>
                </div>

                <div className="relative">
                    <input
                        type="text"
                        placeholder={i18n.t('agent_manager.placeholder_search')}
                        value={state.search_query}
                        onChange={(e) => handlers.set_search_query(e.target.value)}
                        className="bg-zinc-900 border border-zinc-800 rounded-lg pl-8 pr-3 py-1.5 text-xs text-zinc-200 focus:outline-none focus:border-green-500/50 w-48 transition-all"
                    />
                    <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 text-zinc-600" size={12} />
                </div>
            </div>
        </div>
    );
};

// Metadata: [Skill_Header]
