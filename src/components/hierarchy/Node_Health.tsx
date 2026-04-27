/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Health monitor and agent recovery portal. 
 * Facilitates the `reset_agent` RPC flow via `agent_api_service` and provides real-time failure count telemetry.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Reset RPC timeout/rejection (500), `failure_count` desync if `agent_store` update fails, or overlay mount collision with `Hierarchy_Node` base.
 * - **Telemetry Link**: Search for `[Node_Health]` or `reset_agent` in service logs.
 */

import React, { useState } from 'react';
import { Shield, RefreshCw, AlertTriangle, CheckCircle2, X } from 'lucide-react';
import { i18n } from '../../i18n';
import type { Agent } from '../../types';
import { agent_api_service } from '../../services/agent_api_service';
import { use_agent_store } from '../../stores/agent_store';

/**
 * Node_Health_Props
 */
interface Node_Health_Props {
    agent: Agent;
    on_close: () => void;
}

/**
 * Node_Health
 * Health monitor overlay for a specific agent node.
 * Refactored for strict snake_case compliance for backend parity.
 */
export const Node_Health: React.FC<Node_Health_Props> = ({ agent, on_close }) => {
    const [is_resetting, set_is_resetting] = useState(false);
    const update_agent = use_agent_store(s => s.update_agent);

    const failure_count = agent.failure_count || 0;
    const is_throttled = failure_count >= 3;
    const is_healthy = failure_count === 0;

    const handle_reset = async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (is_resetting) return;

        set_is_resetting(true);
        try {
            const result = await agent_api_service.reset_agent(agent.id);
            if (result.status === 'ok') {
                update_agent(agent.id, { 
                    failure_count: 0, 
                    last_failure_at: undefined,
                    status: 'idle'
                });
            }
        } catch (error) {
            console.error('Failed to reset agent:', error);
        } finally {
            set_is_resetting(false);
        }
    };

    return (
        <div className="absolute inset-0 bg-zinc-950/95 backdrop-blur-md z-[60] flex flex-col p-3 border border-zinc-800/50 rounded-lg animate-in fade-in zoom-in-95 duration-200">
            <div className="flex items-center justify-between mb-3 border-b border-zinc-800/50 pb-2">
                <div className="flex items-center gap-2">
                    <Shield size={14} className="text-green-400" />
                    <span className="text-[10px] uppercase font-bold tracking-widest text-zinc-400">
                        {i18n.t('swarm_health_monitor')}
                    </span>
                </div>
                <button 
                    onClick={(e) => { e.stopPropagation(); on_close(); }}
                    className="p-1 hover:bg-zinc-800 rounded-full text-zinc-500 hover:text-zinc-300 transition-colors"
                >
                    <X size={14} />
                </button>
            </div>

            <div className="flex-1 flex flex-col gap-4">
                <div className="flex items-center justify-between bg-zinc-900/40 p-2 rounded border border-zinc-800/30">
                    <div className="flex items-center gap-2">
                        {is_healthy ? (
                            <CheckCircle2 size={16} className="text-emerald-500" />
                        ) : is_throttled ? (
                            <AlertTriangle size={16} className="text-red-500 animate-pulse" />
                        ) : (
                            <AlertTriangle size={16} className="text-amber-500" />
                        )}
                        <span className={`text-xs font-bold uppercase tracking-wide ${
                            is_healthy ? 'text-emerald-400' : is_throttled ? 'text-red-400' : 'text-amber-400'
                        }`}>
                            {is_healthy ? i18n.t('healthy') : is_throttled ? i18n.t('throttled') : i18n.t('degraded')}
                        </span>
                    </div>
                    <div className="text-[10px] font-mono text-zinc-500">
                        {i18n.t('failures_label', { count: failure_count })}
                    </div>
                </div>

                {agent.last_failure_at && (() => {
                    const failure_date = new Date(agent.last_failure_at);
                    const is_valid = !isNaN(failure_date.getTime());
                    return (
                        <div className="px-1">
                            <div className="text-[9px] uppercase font-bold text-zinc-600 tracking-wider mb-1">
                                Last Failure
                            </div>
                            <div className="text-[10px] font-mono text-zinc-400 break-all">
                                {is_valid ? failure_date.toLocaleString() : i18n.t('agent_card.unknown_failure_time')}
                            </div>
                        </div>
                    );
                })()}

                <div className="mt-auto">
                    <button
                        onClick={handle_reset}
                        disabled={is_healthy || is_resetting}
                        className={`w-full py-2 px-3 rounded flex items-center justify-center gap-2 text-xs font-bold transition-all ${
                            is_healthy 
                                ? 'bg-zinc-800/50 text-zinc-600 cursor-not-allowed border border-zinc-800'
                                : 'bg-green-600/10 hover:bg-green-600/20 text-green-400 border border-green-500/30 active:scale-95'
                        }`}
                    >
                        <RefreshCw size={14} className={is_resetting ? 'animate-spin' : ''} />
                        {is_resetting ? 'Resetting...' : i18n.t('reset_agent')}
                    </button>
                </div>
            </div>
            
            <div className="neural-grid opacity-5 pointer-events-none absolute inset-0" />
        </div>
    );
};


// Metadata: [Node_Health]
