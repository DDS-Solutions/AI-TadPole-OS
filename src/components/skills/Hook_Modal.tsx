import React from 'react';
import { Shield, X } from 'lucide-react';
import type { Hook_Definition } from '../../stores/skill_store';

interface Hook_Modal_Props {
    is_open: boolean;
    on_close: () => void;
    editing_hook: Partial<Hook_Definition>;
    hook_form: Hook_Definition;
    set_hook_form: (form: Hook_Definition) => void;
    hook_save_error: string | null;
    is_saving: boolean;
    on_save: () => void;
}

export const Hook_Modal: React.FC<Hook_Modal_Props> = ({
    is_open,
    on_close,
    editing_hook,
    hook_form,
    set_hook_form,
    hook_save_error,
    is_saving,
    on_save
}) => {
    if (!is_open) return null;

    return (
        <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/80 backdrop-blur-md">
            <div className="bg-zinc-950 border border-zinc-800 w-full max-w-2xl rounded-2xl overflow-hidden shadow-2xl neural-border">
                <div className="p-6 border-b border-zinc-800 flex items-center justify-between bg-zinc-900/50">
                    <div>
                        <h3 className="text-lg font-bold text-white flex items-center gap-2">
                            <Shield className="text-emerald-500" size={20} />
                            {editing_hook.name ? 'Edit Lifecycle Hook' : 'Register New Hook'}
                        </h3>
                        <p className="text-xs text-zinc-500">Configure interceptor behavior and validation logic</p>
                    </div>
                    <button onClick={on_close} className="text-zinc-500 hover:text-white">
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
                                value={hook_form.name}
                                onChange={(e) => set_hook_form({ ...hook_form, name: e.target.value })}
                                className="w-full bg-zinc-900 border border-zinc-800 rounded-xl px-4 py-3 text-sm text-zinc-200 focus:outline-none focus:border-emerald-500/50 focus:ring-1 focus:ring-emerald-500/20 transition-all placeholder:text-zinc-700"
                            />
                        </div>
                        <div className="space-y-2">
                            <label className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest pl-1">Interceptor Type</label>
                            <select
                                value={hook_form.hook_type}
                                onChange={(e) => set_hook_form({ ...hook_form, hook_type: e.target.value })}
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
                            value={hook_form.description}
                            onChange={(e) => set_hook_form({ ...hook_form, description: e.target.value })}
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
                            value={hook_form.content}
                            onChange={(e) => set_hook_form({ ...hook_form, content: e.target.value })}
                            className="w-full bg-zinc-950/50 border border-zinc-800 rounded-xl px-4 py-4 text-xs font-mono text-emerald-400 focus:outline-none focus:border-emerald-500/30 transition-all placeholder:text-emerald-900/30"
                        />
                    </div>

                    <div className="flex items-center gap-6 p-4 bg-zinc-900/30 rounded-xl border border-zinc-800/50">
                        <div className="flex items-center gap-3">
                            <button
                                onClick={() => set_hook_form({ ...hook_form, active: !hook_form.active })}
                                className={`w-10 h-5 rounded-full transition-all relative ${hook_form.active ? 'bg-emerald-600' : 'bg-zinc-800'}`}
                            >
                                <div className={`absolute top-1 w-3 h-3 rounded-full bg-white transition-all ${hook_form.active ? 'left-6' : 'left-1'}`} />
                            </button>
                            <span className="text-[11px] font-bold text-zinc-400 uppercase tracking-wider">Hook Status: {hook_form.active ? 'Active' : 'Bypassed'}</span>
                        </div>
                        <div className="h-4 w-px bg-zinc-800" />
                        <div className="flex items-center gap-2">
                            <Shield size={14} className={hook_form.category === 'ai' ? 'text-indigo-500' : 'text-blue-500'} />
                            <span className="text-[11px] font-bold text-zinc-400 uppercase tracking-wider">Registry: {hook_form.category === 'ai' ? 'AI Sector' : 'User Domain'}</span>
                        </div>
                        {hook_save_error && <div className="text-red-500 text-[10px] ml-auto">{hook_save_error}</div>}
                    </div>
                </div>

                <div className="p-6 border-t border-zinc-800 bg-zinc-900/50 flex justify-end gap-3">
                    <button onClick={on_close} className="px-5 py-2.5 text-xs text-zinc-500 hover:text-white font-bold transition-all uppercase tracking-widest">
                        Cancel
                    </button>
                    <button 
                        onClick={on_save} 
                        disabled={is_saving}
                        className="px-6 py-2.5 bg-emerald-600/20 text-emerald-400 border border-emerald-500/30 rounded-xl text-xs font-bold uppercase tracking-widest hover:bg-emerald-600/30 transition-all shadow-lg hover:shadow-emerald-500/20 disabled:opacity-50"
                    >
                        {is_saving ? 'Processing...' : editing_hook.name ? 'Update Registry' : 'Initialize Hook'}
                    </button>
                </div>
            </div>
        </div>
    );
};
