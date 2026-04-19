/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Long-term memory repository and SME data source connector. 
 * Handles manual memory injection and monitors background sync sources for local knowledge ingestion.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Memory Save RPC timeout, connector path invalidation (FS error), or memory list flash during background refresh.
 * - **Telemetry Link**: Search for `[Memory_Section]` or `neural_injection` in service logs.
 */

import { useState } from 'react';
import { Brain, Trash2, RefreshCw } from 'lucide-react';
import { i18n } from '../../i18n';

interface Memory_Entry {
    id: string;
    content?: string;
    text?: string;
}

interface Connector_Config {
    type: string;
    uri: string;
}

interface Memory_Section_Props {
    memories: Memory_Entry[];
    connector_configs: Connector_Config[];
    is_loading: boolean;
    memory_input: string;
    theme_color: string;
    on_memory_input_change: (val: string) => void;
    on_save_memory: () => void;
    on_delete_memory: (id: string) => void;
    on_refresh: () => void;
    on_add_connector: (uri: string) => void;
    on_remove_connector: (uri: string) => void;
}

/**
 * Memory_Section
 * Handles agent internal memory management and external SME data connectors.
 * Supports neural injection and context synchronization.
 */
export function Memory_Section({
    memories,
    connector_configs,
    is_loading,
    memory_input,
    theme_color,
    on_memory_input_change,
    on_save_memory,
    on_delete_memory,
    on_refresh,
    on_add_connector,
    on_remove_connector
}: Memory_Section_Props) {
    const [connector_input, set_connector_input] = useState('');

    return (
        <div className="p-4 space-y-8 animate-in fade-in duration-300">
            {/* Phase 2: SME Data Connectors */}
            <div className="space-y-4">
                <div className="flex items-center justify-between px-2">
                    <label className="block text-[10px] font-bold text-zinc-600 uppercase tracking-[0.2em]">{i18n.t('memory_section.label_sme_sources')}</label>
                    <div className="px-2 py-0.5 rounded-full bg-blue-500/10 border border-blue-500/20 text-[8px] text-blue-400 font-bold uppercase tracking-wider">
                        Sync Active
                    </div>
                </div>

                <div className="p-4 bg-zinc-900 border border-zinc-800 rounded-2xl space-y-4">
                    <div className="flex gap-2">
                        <input
                            type="text"
                            value={connector_input}
                            onChange={(e) => set_connector_input(e.target.value)}
                            placeholder={i18n.t('memory_section.placeholder_local_path')}
                            className="flex-1 bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 font-mono transition-all"
                        />
                        <button
                            onClick={() => {
                                if (connector_input) {
                                    on_add_connector(connector_input);
                                    set_connector_input('');
                                }
                            }}
                            className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-[10px] font-bold uppercase rounded-xl transition-all border border-zinc-700"
                        >
                            Add Source
                        </button>
                    </div>

                    <div className="space-y-2">
                        {connector_configs.length === 0 ? (
                            <p className="text-[10px] text-zinc-600 italic px-2">{i18n.t('memory_section.msg_no_sync_sources')}</p>
                        ) : (
                            connector_configs.map((c: Connector_Config, i: number) => (
                                <div key={i} className="flex items-center justify-between p-2 bg-zinc-950/50 border border-zinc-800/50 rounded-lg group">
                                    <div className="flex items-center gap-2 min-w-0">
                                        <div className="w-1.5 h-1.5 rounded-full bg-green-500/40 animate-pulse" />
                                        <span className="text-[10px] text-zinc-400 font-mono truncate">{c.uri}</span>
                                    </div>
                                    <button 
                                        onClick={() => on_remove_connector(c.uri)}
                                        className="p-1 text-zinc-700 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-all"
                                    >
                                        <Trash2 size={12} />
                                    </button>
                                </div>
                            ))
                        )}
                    </div>
                </div>
            </div>

            {/* Neural Injection */}
            <div className="space-y-4">
                <div className="flex items-center justify-between px-2">
                    <label className="block text-[10px] font-bold text-zinc-600 uppercase tracking-[0.2em]">{i18n.t('agent_config.label_neural_injection')}</label>
                </div>
                <div className="p-4 bg-zinc-900 border border-zinc-800 rounded-2xl space-y-3">
                    <div className="flex gap-2">
                        <input
                            type="text"
                            value={memory_input}
                            onChange={(e) => on_memory_input_change(e.target.value)}
                            placeholder={i18n.t('agent_config.placeholder_memory_injection')}
                            className="flex-1 bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 font-mono transition-all"
                            style={{ borderLeft: `2px solid ${theme_color}40` }}
                        />
                        <button
                            onClick={on_save_memory}
                            className="p-2 text-black rounded-xl transition-all shadow-lg"
                            style={{ backgroundColor: theme_color }}
                        >
                            <Brain size={16} />
                        </button>
                    </div>
                </div>

                <div className="space-y-2">
                    <div className="flex items-center justify-between px-2">
                        <label className="block text-[10px] font-bold text-zinc-600 uppercase tracking-[0.2em]">{i18n.t('agent_config.label_persisted_memories')}</label>
                        <button onClick={on_refresh} disabled={is_loading} className="p-1 hover:bg-zinc-800 rounded-lg transition-colors">
                            <RefreshCw size={10} className={`text-zinc-500 ${is_loading ? 'animate-spin' : ''}`} />
                        </button>
                    </div>

                    <div className="space-y-2">
                        {memories.length === 0 ? (
                            <div className="p-8 text-center bg-zinc-900/40 border border-zinc-800/50 border-dashed rounded-2xl">
                                <Brain size={24} className="mx-auto text-zinc-800 mb-2 opacity-20" />
                                <p className="text-[10px] text-zinc-600 uppercase font-bold tracking-[0.2em]">{i18n.t('agent_config.no_memories')}</p>
                            </div>
                        ) : (
                            memories.map((m) => (
                                <div key={m.id} className="group p-3 bg-zinc-900/60 border border-zinc-800 hover:border-zinc-700 rounded-xl transition-all flex items-start justify-between gap-3">
                                    <p className="text-xs text-zinc-300 leading-relaxed font-mono line-clamp-3">{m.content || m.text}</p>
                                    <button
                                        onClick={() => on_delete_memory(m.id)}
                                        className="p-1.5 text-zinc-700 hover:text-red-500 hover:bg-red-500/10 rounded-lg transition-all opacity-0 group-hover:opacity-100"
                                    >
                                        <Trash2 size={12} />
                                    </button>
                                </div>
                            ))
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}

