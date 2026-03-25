import React, { useState } from 'react';
import { Shield, RefreshCw, AlertTriangle, CheckCircle2, X } from 'lucide-react';
import { i18n } from '../../i18n';
import type { Agent } from '../../types';
import { AgentApiService } from '../../services/AgentApiService';
import { useAgentStore } from '../../stores/agentStore';

interface NodeHealthProps {
    agent: Agent;
    onClose: () => void;
}

export const NodeHealth: React.FC<NodeHealthProps> = ({ agent, onClose }) => {
    const [isResetting, setIsResetting] = useState(false);
    const updateAgent = useAgentStore(s => s.updateAgent);

    const failureCount = agent.failureCount || 0;
    const isThrottled = failureCount >= 3;
    const isHealthy = failureCount === 0;

    const handleReset = async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (isResetting) return;

        setIsResetting(true);
        try {
            const result = await AgentApiService.resetAgent(agent.id);
            if (result.status === 'ok') {
                // The store update should come via WebSocket, 
                // but we can also update locally for immediate feedback
                updateAgent(agent.id, { 
                    failureCount: 0, 
                    lastFailureAt: undefined,
                    status: 'idle'
                });
            }
        } catch (error) {
            console.error('Failed to reset agent:', error);
        } finally {
            setIsResetting(false);
        }
    };

    return (
        <div className="absolute inset-0 bg-zinc-950/95 backdrop-blur-md z-[60] flex flex-col p-3 border border-zinc-800/50 rounded-lg animate-in fade-in zoom-in-95 duration-200">
            <div className="flex items-center justify-between mb-3 border-b border-zinc-800/50 pb-2">
                <div className="flex items-center gap-2">
                    <Shield size={14} className="text-blue-400" />
                    <span className="text-[10px] uppercase font-bold tracking-widest text-zinc-400">
                        {i18n.t('swarm_health_monitor')}
                    </span>
                </div>
                <button 
                    onClick={(e) => { e.stopPropagation(); onClose(); }}
                    className="p-1 hover:bg-zinc-800 rounded-full text-zinc-500 hover:text-zinc-300 transition-colors"
                >
                    <X size={14} />
                </button>
            </div>

            <div className="flex-1 flex flex-col gap-4">
                <div className="flex items-center justify-between bg-zinc-900/40 p-2 rounded border border-zinc-800/30">
                    <div className="flex items-center gap-2">
                        {isHealthy ? (
                            <CheckCircle2 size={16} className="text-emerald-500" />
                        ) : isThrottled ? (
                            <AlertTriangle size={16} className="text-red-500 animate-pulse" />
                        ) : (
                            <AlertTriangle size={16} className="text-amber-500" />
                        )}
                        <span className={`text-xs font-bold uppercase tracking-wide ${
                            isHealthy ? 'text-emerald-400' : isThrottled ? 'text-red-400' : 'text-amber-400'
                        }`}>
                            {isHealthy ? i18n.t('healthy') : isThrottled ? i18n.t('throttled') : i18n.t('degraded')}
                        </span>
                    </div>
                    <div className="text-[10px] font-mono text-zinc-500">
                        {i18n.t('failures_label', { count: failureCount })}
                    </div>
                </div>

                {agent.lastFailureAt && (
                    <div className="px-1">
                        <div className="text-[9px] uppercase font-bold text-zinc-600 tracking-wider mb-1">
                            Last Failure
                        </div>
                        <div className="text-[10px] font-mono text-zinc-400 break-all">
                            {new Date(agent.lastFailureAt).toLocaleString()}
                        </div>
                    </div>
                )}

                <div className="mt-auto">
                    <button
                        onClick={handleReset}
                        disabled={isHealthy || isResetting}
                        className={`w-full py-2 px-3 rounded flex items-center justify-center gap-2 text-xs font-bold transition-all ${
                            isHealthy 
                                ? 'bg-zinc-800/50 text-zinc-600 cursor-not-allowed border border-zinc-800'
                                : 'bg-blue-600/10 hover:bg-blue-600/20 text-blue-400 border border-blue-500/30 active:scale-95'
                        }`}
                    >
                        <RefreshCw size={14} className={isResetting ? 'animate-spin' : ''} />
                        {isResetting ? 'Resetting...' : i18n.t('reset_agent')}
                    </button>
                </div>
            </div>
            
            <div className="neural-grid opacity-5 pointer-events-none absolute inset-0" />
        </div>
    );
};
