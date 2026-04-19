/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Sub-module**: Orchestration bridge for local machine intelligence. 
 * Manages life-cycle signals (Start/Stop) for local LLM drivers (Ollama, LM Studio) and synchronizes file-path metadata.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Binary path not found (404), permission denied (EACCES), or driver crash during boot.
 * - **Telemetry Link**: Search for `Manual boot sequence` or `System termination signal` in UI logs.
 */

import React from 'react';
import { Server, Play, Square } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

/**
 * Local_Server_Module_Props
 * Defines the props for the Local_Server_Module component.
 */
interface Local_Server_Module_Props {
    /** Current local server path or URL */
    path: string;
    /** Callback for path changes */
    on_path_change: (path: string) => void;
    /** Callback to start the local server */
    on_start: () => void;
    /** Callback to stop the local server */
    on_stop: () => void;
}

/**
 * Local_Server_Module
 * Specialized orchestration controls for local model engines (e.g., Ollama, LM Studio).
 */
export function Local_Server_Module({ 
    path, 
    on_path_change, 
    on_start, 
    on_stop 
}: Local_Server_Module_Props): React.ReactElement {
    return (
        <div className="space-y-3 p-4 bg-blue-500/5 border border-blue-500/10 rounded-2xl animate-in slide-in-from-top-2 border-dashed">
            <div className="flex items-center justify-between">
                <label htmlFor="provider-local-path" className="text-[10px] font-bold text-blue-400 uppercase tracking-[0.2em] flex items-center gap-2">
                    <Server size={12} />
                    {i18n.t('provider.local_server_orchestration', { defaultValue: 'Local Server Orchestration' })}
                </label>
                <div className="flex gap-2">
                    <Tooltip content="Launch Local Engine" position="top">
                        <button 
                            onClick={on_start}
                            className="p-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg transition-all shadow-lg shadow-blue-500/20 active:scale-95"
                            aria-label={i18n.t('provider.aria_start_server')}
                        >
                            <Play size={12} fill="currentColor" />
                        </button>
                    </Tooltip>
                    <Tooltip content="Stop Engine" position="top">
                        <button 
                            onClick={on_stop}
                            className="p-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-400 rounded-lg transition-all"
                            aria-label={i18n.t('provider.aria_stop_server')}
                        >
                            <Square size={12} fill="currentColor" />
                        </button>
                    </Tooltip>
                </div>
            </div>
            <div className="relative">
                <input
                    id="provider-local-path"
                    type="text"
                    placeholder={i18n.t('provider.placeholder_local_path', { defaultValue: 'Paste local server link or path...' })}
                    value={path}
                    onChange={(e) => on_path_change(e.target.value)}
                    className="w-full bg-zinc-950/50 border border-zinc-800 rounded-lg px-3 py-2 text-[10px] text-zinc-400 focus:outline-none focus:border-blue-500/40 font-mono"
                    aria-label={i18n.t('provider.local_server_path_label')}
                />
            </div>
            <p className="text-[9px] text-zinc-500 leading-tight italic px-1">
                {i18n.t('provider.local_server_desc', { defaultValue: 'Automate model driver lifecycle. Paste the executable path or launch URL above.' })}
            </p>
        </div>
    );
}
