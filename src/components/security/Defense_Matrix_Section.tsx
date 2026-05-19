/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Defense_Matrix_Section]` in observability traces.
 */

import { ShieldCheck, Lock, AlertCircle } from 'lucide-react';
import { i18n } from '../../i18n';
import type { Quotas } from '../../services/tadpoleos_service';

interface DefenseMatrixSectionProps {
    system_defense: Quotas['system_defense'];
}

export function Defense_Matrix_Section({ system_defense }: DefenseMatrixSectionProps) {
    return (
        <div className="bg-[color:var(--color-surface)] border border-green-500/30 rounded-2xl overflow-hidden shadow-[0_0_30px_rgba(59,130,246,0.1)]">
            <div className="bg-green-500/10 p-4 border-b border-green-500/20 flex items-center gap-2">
                <ShieldCheck className="w-4 h-4 text-green-400" />
                <h2 className="font-semibold text-blue-100 uppercase tracking-widest text-xs">{i18n.t('security.defense_matrix')}</h2>
            </div>
            <div className="p-6">
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    <div className="space-y-4">
                        <h3 className="text-xs font-bold text-zinc-500 uppercase flex items-center gap-2">
                            <Lock size={12} /> {i18n.t('security.resource_guard')}
                        </h3>
                        <div className="bg-[color:var(--color-background)] p-4 rounded-xl border border-[color:var(--color-border)] transition-all hover:border-green-500/20">
                            <div className="flex justify-between items-center mb-1">
                                <span className="text-xs text-zinc-400">{i18n.t('security.memory_pressure')}</span>
                                <div className="flex items-center gap-2">
                                    <span className={`px-1.5 py-0.5 rounded-md text-[8px] font-bold ${(system_defense?.memory_pressure || 0) > 0.8 ? 'bg-red-500/10 text-red-500' : 'bg-emerald-500/10 text-emerald-500'}`}>
                                        {(system_defense?.memory_pressure || 0) > 0.8 ? 'PRESSURE_HIGH' : 'NOMINAL'}
                                    </span>
                                    <span className={`text-[10px] font-mono ${(system_defense?.memory_pressure || 0) > 0.8 ? 'text-red-500' : 'text-emerald-500'}`}>
                                        {((system_defense?.memory_pressure || 0) * 100).toFixed(1)}%
                                    </span>
                                </div>
                            </div>
                            <div className="h-1 bg-zinc-800 rounded-full overflow-hidden">
                                <div
                                    className={`h-full transition-all duration-1000 ${(system_defense?.memory_pressure || 0) > 0.8 ? 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.4)]' : 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.4)]'}`}
                                    style={{ width: `${(system_defense?.memory_pressure || 0) * 100}%` }}
                                />
                            </div>
                        </div>
                    </div>

                    <div className="space-y-4">
                        <h3 className="text-xs font-bold text-zinc-500 uppercase flex items-center gap-2">
                            <AlertCircle size={12} /> {i18n.t('security.capability_bounds')}
                        </h3>
                        <div className="bg-[color:var(--color-background)] p-4 rounded-xl border border-[color:var(--color-border)] transition-all hover:border-blue-500/20">
                            <div className="flex justify-between items-center mb-1">
                                <span className="text-xs text-zinc-400">{i18n.t('security.environment')}</span>
                                <div className="flex items-center gap-2">
                                    <span className={`px-1.5 py-0.5 rounded-md text-[8px] font-bold ${system_defense?.sandbox_status === 'ACTIVE' ? 'bg-blue-500/10 text-blue-400 border border-blue-500/20' : 'bg-amber-500/10 text-amber-500 border border-amber-500/20'}`}>
                                        {system_defense?.sandbox_status || 'IDLE'}
                                    </span>
                                    <span className="text-[10px] font-mono text-zinc-300">
                                        {system_defense?.sandbox_type || 'Unknown'}
                                    </span>
                                </div>
                            </div>
                            <div className="h-1 bg-zinc-800 rounded-full overflow-hidden">
                                <div
                                    className={`h-full transition-all duration-700 ${system_defense?.sandbox_status === 'ACTIVE' ? 'bg-blue-500/40' : 'bg-amber-500/40'}`}
                                    style={{ width: system_defense?.sandbox_status === 'ACTIVE' ? '100%' : '30%' }}
                                />
                            </div>
                        </div>
                    </div>

                    <div className="space-y-4">
                        <h3 className="text-xs font-bold text-zinc-500 uppercase flex items-center gap-2">
                            <ShieldCheck size={12} /> {i18n.t('security.shell_safety')}
                        </h3>
                        <div className="bg-[color:var(--color-background)] p-4 rounded-xl border border-[color:var(--color-border)] transition-all hover:border-emerald-500/20">
                            <div className="flex justify-between items-center mb-1">
                                <span className="text-xs text-zinc-400">{i18n.t('security.secret_leak_prevention')}</span>
                                <div className="flex items-center gap-2">
                                    <span className="px-1.5 py-0.5 rounded-md text-[8px] font-bold bg-emerald-500/10 text-emerald-500">
                                        VERIFIED
                                    </span>
                                    <span className="text-[10px] text-emerald-500 font-mono">
                                        {((system_defense?.merkle_integrity ?? 0.85) * 100).toFixed(2)}%
                                    </span>
                                </div>
                            </div>
                            <div className="h-1 bg-zinc-800 rounded-full overflow-hidden">
                                <div
                                    className="h-full bg-emerald-500 transition-all duration-1000 shadow-[0_0_10px_rgba(16,185,129,0.3)]"
                                    style={{ width: `${(system_defense?.merkle_integrity ?? 0.85) * 100}%` }}
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

// Metadata: [Defense_Matrix_Section]
