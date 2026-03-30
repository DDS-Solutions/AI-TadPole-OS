import React from 'react';
import { FlaskConical, X, AlertTriangle, Gauge, Shield, Activity, Play } from 'lucide-react';
import { i18n } from '../../i18n';
import type { McpToolHubDefinition } from '../../stores/skillStore';

interface McpLabModalProps {
    isOpen: boolean;
    onClose: () => void;
    labTool: McpToolHubDefinition | null;
    labInput: string;
    setLabInput: (input: string) => void;
    labSchemaError: string | null;
    isLabRunning: boolean;
    handleRunTool: () => void;
    labResult: Record<string, unknown> | null;
}

export const McpLabModal: React.FC<McpLabModalProps> = ({
    isOpen,
    onClose,
    labTool,
    labInput,
    setLabInput,
    labSchemaError,
    isLabRunning,
    handleRunTool,
    labResult
}) => {
    if (!isOpen || !labTool) return null;

    return (
        <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="bg-zinc-950 border border-zinc-800 w-full max-w-2xl rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
                <div className="bg-zinc-900/50 p-6 border-b border-zinc-800 flex items-center justify-between">
                    <div className="flex items-center gap-3">
                        <div className="w-10 h-10 rounded-xl bg-cyan-500/10 flex items-center justify-center border border-cyan-500/20 text-cyan-400">
                            <FlaskConical size={20} />
                        </div>
                        <div>
                            <h2 className="text-lg font-bold text-zinc-100 font-mono">{labTool.name}</h2>
                            <p className="text-xs text-zinc-500 uppercase tracking-widest font-bold">{i18n.t('skills.label_manual_execution_lab')}</p>
                        </div>
                    </div>
                    <button onClick={onClose} className="text-zinc-500 hover:text-zinc-300 transition-colors bg-zinc-800/50 p-2 rounded-lg"><X size={18} /></button>
                </div>

                <div className="flex-1 overflow-y-auto p-6 custom-scrollbar space-y-6">
                    <div>
                        <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-1.5">{i18n.t('skills.label_input_args')}</label>
                        <textarea
                            value={labInput}
                            onChange={(e) => setLabInput(e.target.value)}
                            className="w-full h-40 bg-black/50 border border-zinc-800 rounded-xl p-4 font-mono text-xs text-cyan-300 focus:outline-none focus:border-cyan-500/50 transition-colors custom-scrollbar"
                            placeholder='{ "arg": "value" }'
                        />
                        {labSchemaError && (
                            <div className="mt-2 text-[10px] text-red-400 font-mono bg-red-400/5 p-2 rounded border border-red-400/10 flex items-center gap-2">
                                <AlertTriangle size={12} /> {labSchemaError}
                            </div>
                        )}
                    </div>

                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-4 text-[10px] font-bold text-zinc-600 uppercase">
                            <span className="flex items-center gap-1.5"><Gauge size={12} /> {i18n.t('skills.label_live_telemetry')}</span>
                            <span className="flex items-center gap-1.5"><Shield size={12} /> {i18n.t('skills.label_governance_audit')}</span>
                        </div>
                        <button
                            onClick={handleRunTool}
                            disabled={isLabRunning}
                            className={`flex items-center gap-2 px-6 py-2.5 bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500 text-white rounded-xl text-sm font-bold shadow-lg shadow-cyan-500/20 transition-all active:scale-95 disabled:opacity-50 disabled:grayscale`}
                        >
                            {isLabRunning ? <Activity size={18} className="animate-spin" /> : <Play size={18} fill="currentColor" />}
                            {isLabRunning ? i18n.t('skills.btn_executing') : i18n.t('skills.btn_run_tool')}
                        </button>
                    </div>

                    {labResult && (
                        <div className="animate-in slide-in-from-bottom-4 duration-300">
                            <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-1.5">{i18n.t('skills.label_execution_result')}</label>
                            <div className={`p-4 rounded-xl border font-mono text-xs overflow-x-auto custom-scrollbar ${labResult.status === 'success' ? 'bg-emerald-500/5 border-emerald-500/20 text-emerald-400' : 'bg-red-500/5 border-red-500/20 text-red-400'}`}>
                                <pre>{JSON.stringify(labResult, null, 2)}</pre>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};
