import React from 'react';
import { Users, Check } from 'lucide-react';
import { i18n } from '../../i18n';
import type { Agent } from '../../types';

interface AssignmentModalProps {
    isOpen: boolean;
    onClose: () => void;
    assignTarget: { type: 'skill' | 'workflow' | 'mcp', name: string } | null;
    agents: Agent[];
    onToggleAssignment: (agentId: string) => void;
}

export const AssignmentModal: React.FC<AssignmentModalProps> = ({
    isOpen,
    onClose,
    assignTarget,
    agents,
    onToggleAssignment
}) => {
    if (!isOpen || !assignTarget) return null;

    return (
        <div className="fixed inset-0 z-[60] bg-black/80 backdrop-blur-md flex items-center justify-center p-4">
            <div className="bg-zinc-950 border border-zinc-800 w-full max-w-md rounded-2xl shadow-2xl flex flex-col relative overflow-hidden">
                <div className="neural-grid opacity-10" />
                <div className="p-5 border-b border-zinc-800 flex justify-between items-center shrink-0 relative z-10 bg-zinc-950/50">
                    <h2 className="text-sm font-bold text-zinc-100 flex items-center gap-2 uppercase tracking-wider">
                        <Users className="text-emerald-500" size={16} /> {i18n.t('agent_manager.modal_assign_title', { type: assignTarget.type.toUpperCase(), name: assignTarget.name })}
                    </h2>
                    <button onClick={onClose} className="text-zinc-500 hover:text-zinc-300 p-1">✕</button>
                </div>
                <div className="p-6 space-y-4 relative z-10 bg-zinc-950/80">
                    <p className="text-xs text-zinc-500 font-mono leading-relaxed">
                        {i18n.t('agent_manager.label_select_agents')}
                    </p>
                    <div className="space-y-2 max-h-64 overflow-y-auto custom-scrollbar pr-2">
                        {agents.map(agent => {
                            const isAssigned = 
                                (assignTarget.type === 'skill' && (agent.skills || []).includes(assignTarget.name)) ||
                                (assignTarget.type === 'workflow' && (agent.workflows || []).includes(assignTarget.name)) ||
                                (assignTarget.type === 'mcp' && (agent.mcpTools || []).includes(assignTarget.name));
                            
                            return (
                                <button
                                    key={agent.id}
                                    onClick={() => onToggleAssignment(agent.id)}
                                    className={`w-full flex items-center justify-between p-3 rounded-xl border transition-all ${
                                        isAssigned 
                                            ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400 shadow-[0_0_10px_rgba(16,185,129,0.1)]' 
                                            : 'bg-zinc-900/50 border-zinc-800 text-zinc-400 hover:border-zinc-700'
                                    }`}
                                >
                                    <div className="flex items-center gap-3">
                                        <div 
                                            className="w-8 h-8 rounded-lg flex items-center justify-center font-bold text-sm shrink-0"
                                            style={{ 
                                                backgroundColor: agent.themeColor ? `${agent.themeColor}20` : '#27272a',
                                                color: agent.themeColor || '#71717a',
                                                border: `1px solid ${agent.themeColor ? `${agent.themeColor}40` : '#3f3f46'}`
                                            }}
                                        >
                                            {agent.name[0]}
                                        </div>
                                        <div className="text-left">
                                            <div className="text-xs font-bold truncate max-w-[180px]">{agent.name}</div>
                                            <div className="text-[10px] opacity-50 font-mono uppercase">{agent.role}</div>
                                        </div>
                                    </div>
                                    {isAssigned && <Check size={14} className="text-emerald-500" />}
                                </button>
                            );
                        })}
                    </div>
                </div>
                <div className="p-6 border-t border-zinc-800 bg-zinc-900/50 flex justify-end">
                    <button 
                        onClick={onClose} 
                        className="px-6 py-2 bg-zinc-800 hover:bg-zinc-700 text-white rounded-lg text-xs font-bold uppercase tracking-widest transition-all"
                    >
                        Done
                    </button>
                </div>
            </div>
        </div>
    );
};
