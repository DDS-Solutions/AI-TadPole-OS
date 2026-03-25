import React from 'react';
import { Shield, X } from 'lucide-react';
import type { HookDefinition } from '../../stores/skillStore';

interface HookModalProps {
    isOpen: boolean;
    onClose: () => void;
    editingHook: Partial<HookDefinition>;
    hookForm: HookDefinition;
    setHookForm: (form: HookDefinition) => void;
    hookSaveError: string | null;
    isSaving: boolean;
    onSave: () => void;
}

export const HookModal: React.FC<HookModalProps> = ({
    isOpen,
    onClose,
    editingHook,
    hookForm,
    setHookForm,
    hookSaveError,
    isSaving,
    onSave
}) => {
    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/80 backdrop-blur-md">
            <div className="bg-zinc-950 border border-zinc-800 w-full max-w-2xl rounded-2xl overflow-hidden shadow-2xl neural-border">
                <div className="p-6 border-b border-zinc-800 flex items-center justify-between bg-zinc-900/50">
                    <div>
                        <h3 className="text-lg font-bold text-white flex items-center gap-2">
                            <Shield className="text-emerald-500" size={20} />
                            {editingHook.name ? 'Edit Lifecycle Hook' : 'Register New Hook'}
                        </h3>
                        <p className="text-xs text-zinc-500">Configure interceptor behavior and validation logic</p>
                    </div>
                    <button onClick={onClose} className="text-zinc-500 hover:text-white">
                        <X size={20} />
                    </button>
                </div>

                <div className="p-8 space-y-6 max-h-[70vh] overflow-y-auto custom-scrollbar">
                    <div className="grid grid-cols-2 gap-6">
                        <div className="space-y-2">
                            <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest pl-1">Hook Identifier</label>
                            <input
                                type="text"
                                placeholder="e.g. BudgetGuard_L1"
                                value={hookForm.name}
                                onChange={(e) => setHookForm({ ...hookForm, name: e.target.value })}
                                className="w-full bg-zinc-900 border border-zinc-800 rounded-xl px-4 py-3 text-sm text-zinc-200 focus:outline-none focus:border-emerald-500/50 focus:ring-1 focus:ring-emerald-500/20 transition-all placeholder:text-zinc-700"
                            />
                        </div>
                        <div className="space-y-2">
                            <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest pl-1">Interceptor Type</label>
                            <select
                                value={hookForm.hook_type}
                                onChange={(e) => setHookForm({ ...hookForm, hook_type: e.target.value })}
                                className="w-full bg-zinc-900 border border-zinc-800 rounded-xl px-4 py-3 text-sm text-zinc-200 focus:outline-none focus:border-emerald-500/50 transition-all cursor-pointer"
                            >
                                <option value="pre_validation">Pre-Validation</option>
                                <option value="post_analysis">Post-Analysis</option>
                                <option value="mission_start">Mission Start</option>
                                <option value="mission_complete">Mission Complete</option>
                            </select>
                        </div>
                    </div>

                    <div className="space-y-2">
                        <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest pl-1">Abstract Functional Description</label>
                        <textarea
                            rows={3}
                            placeholder="Briefly describe what this hook intercepts and validates..."
                            value={hookForm.description}
                            onChange={(e) => setHookForm({ ...hookForm, description: e.target.value })}
                            className="w-full bg-zinc-900 border border-zinc-800 rounded-xl px-4 py-3 text-sm text-zinc-200 focus:outline-none focus:border-emerald-500/50 transition-all resize-none placeholder:text-zinc-700"
                        />
                    </div>

                    <div className="space-y-2">
                        <div className="flex items-center justify-between mb-1">
                            <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest pl-1">Validation Payload (JSON/Script)</label>
                            <div className="text-[9px] text-emerald-500/50 font-mono tracking-tighter">Interception Logic</div>
                        </div>
                        <textarea
                            rows={8}
                            placeholder='{ "logic": "block_if_exceeds_budget", "threshold": 10 }'
                            value={hookForm.content}
                            onChange={(e) => setHookForm({ ...hookForm, content: e.target.value })}
                            className="w-full bg-zinc-950/50 border border-zinc-800 rounded-xl px-4 py-4 text-xs font-mono text-emerald-400 focus:outline-none focus:border-emerald-500/30 transition-all placeholder:text-emerald-900/30"
                        />
                    </div>

                    <div className="flex items-center gap-6 p-4 bg-zinc-900/30 rounded-xl border border-zinc-800/50">
                        <div className="flex items-center gap-3">
                            <button
                                onClick={() => setHookForm({ ...hookForm, active: !hookForm.active })}
                                className={`w-10 h-5 rounded-full transition-all relative ${hookForm.active ? 'bg-emerald-600' : 'bg-zinc-800'}`}
                            >
                                <div className={`absolute top-1 w-3 h-3 rounded-full bg-white transition-all ${hookForm.active ? 'left-6' : 'left-1'}`} />
                            </button>
                            <span className="text-[11px] font-bold text-zinc-400 uppercase tracking-wider">Hook Status: {hookForm.active ? 'Active' : 'Bypassed'}</span>
                        </div>
                        <div className="h-4 w-px bg-zinc-800" />
                        <div className="flex items-center gap-2">
                            <Shield size={14} className={hookForm.category === 'ai' ? 'text-indigo-500' : 'text-blue-500'} />
                            <span className="text-[11px] font-bold text-zinc-400 uppercase tracking-wider">Registry: {hookForm.category === 'ai' ? 'AI Sector' : 'User Domain'}</span>
                        </div>
                        {hookSaveError && <div className="text-red-500 text-[10px] ml-auto">{hookSaveError}</div>}
                    </div>
                </div>

                <div className="p-6 border-t border-zinc-800 bg-zinc-900/50 flex justify-end gap-3">
                    <button onClick={onClose} className="px-5 py-2.5 text-xs text-zinc-500 hover:text-white font-bold transition-all uppercase tracking-widest">
                        Cancel
                    </button>
                    <button 
                        onClick={onSave} 
                        disabled={isSaving}
                        className="px-6 py-2.5 bg-emerald-600/20 text-emerald-400 border border-emerald-500/30 rounded-xl text-xs font-bold uppercase tracking-widest hover:bg-emerald-600/30 transition-all shadow-lg hover:shadow-emerald-500/20 disabled:opacity-50"
                    >
                        {isSaving ? 'Processing...' : editingHook.name ? 'Update Registry' : 'Initialize Hook'}
                    </button>
                </div>
            </div>
        </div>
    );
};
