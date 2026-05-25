/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Fiscal and throughput audit table for the active swarm. 
 * Provides a high-density "Neural Footprint" view, tracking token consumption, accreted costs, and per-node allocation ratios.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Utility bar overflow (util > 100%), cost precision rounding errors in the USD column, or agent name truncation on ultra-wide clusters.
 * - **Telemetry Link**: Search for `[Command_Table]` or `accreted_usd` in UI logs.
 */

import React from 'react';
import { use_agent_store } from '../stores/agent_store';
import { TrendingUp, DollarSign, BarChart2, MessageSquare } from 'lucide-react';
import { use_sovereign_store } from '../stores/sovereign_store';
import { useEngineStatus } from '../hooks/use_engine_status';
import clsx from 'clsx';
import { motion } from 'framer-motion';
import { i18n } from '../i18n';

/**
 * Command_Table
 * High-density tabular view of the swarm's neural footprint.
 * Refined with glassmorphism, neural-grid overlays, and premium transitions.
 */
export const Command_Table: React.FC = (): React.ReactElement => {
    const { agents } = use_agent_store();
    const engine = useEngineStatus();
    const { set_scope, set_detached, set_selected_agent_id } = use_sovereign_store();

    const [sort_key, set_sort_key] = React.useState<'cost' | 'throughput' | 'util' | 'name'>('cost');
    const [sort_dir, set_sort_dir] = React.useState<'asc' | 'desc'>('desc');

    const join_chat = (agent_id: string): void => {
        set_selected_agent_id(agent_id);
        set_scope('agent');
        set_detached(false);
    };

    const sorted_agents = React.useMemo(() => {
        return [...(agents || [])].sort((a, b) => {
            let val_a: string | number;
            let val_b: string | number;

            if (sort_key === 'cost') {
                val_a = a.cost_usd || 0;
                val_b = b.cost_usd || 0;
            } else if (sort_key === 'throughput') {
                val_a = a.tokens_used || 0;
                val_b = b.tokens_used || 0;
            } else if (sort_key === 'util') {
                val_a = (a.cost_usd || 0) / (a.budget_usd || 1);
                val_b = (b.cost_usd || 0) / (b.budget_usd || 1);
            } else {
                val_a = a.name;
                val_b = b.name;
            }

            if (typeof val_a === 'string' && typeof val_b === 'string') {
                return sort_dir === 'asc' ? val_a.localeCompare(val_b) : val_b.localeCompare(val_a);
            }
            if (typeof val_a === 'number' && typeof val_b === 'number') {
                return sort_dir === 'asc' ? val_a - val_b : val_b - val_a;
            }
            return 0;
        });
    }, [agents, sort_key, sort_dir]);

    const toggle_sort = (key: 'cost' | 'throughput' | 'util' | 'name') => {
        if (sort_key === key) {
            set_sort_dir(prev => prev === 'asc' ? 'desc' : 'asc');
        } else {
            set_sort_key(key);
            set_sort_dir('desc');
        }
    };

    return (
        <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="bg-[color:var(--color-surface)]/40 backdrop-blur-xl border border-[color:var(--color-border)]/50 rounded-2xl overflow-hidden shadow-[0_20px_50px_-12px_rgba(0,0,0,0.8)] relative"
        >
            {/* Neural Grid Overlay */}
            <div className="neural-grid opacity-[0.03] absolute inset-0 pointer-events-none" />

            {/* Header Area */}
            <div className="p-5 border-b border-[color:var(--color-border)]/50 bg-[color:var(--color-background)]/40 backdrop-blur-md flex items-center justify-between relative z-10">
                <div className="flex items-center gap-3">
                    <div className="p-2 rounded-xl bg-zinc-800/80 border border-zinc-700/50 text-green-400 shadow-lg shadow-green-500/5">
                        <BarChart2 size={20} />
                    </div>
                    <div>
                        <h2 className="font-bold text-sm tracking-[0.15em] text-zinc-100 uppercase">
                            {i18n.t('command.neural_footprint')}
                        </h2>
                        <div className="flex items-center gap-2 mt-0.5">
                            <span className="text-[9px] font-mono text-zinc-600 uppercase tracking-tighter">
                                {i18n.t('command.cluster_ver')}
                            </span>
                            <div className="h-1 w-1 rounded-full bg-green-500/50" />
                            <span className="text-[9px] font-mono text-zinc-600 uppercase tracking-tighter">
                                {i18n.t('command.live_feed')}
                            </span>
                        </div>
                    </div>
                </div>
                <div className="flex items-center gap-6">
                    <div className="hidden md:flex flex-col items-end">
                        <div className="flex items-center gap-1.5 text-zinc-400 text-[10px] font-bold uppercase tracking-wider">
                            <TrendingUp size={12} className="text-emerald-500" />
                            <span>{i18n.t('command.system_load')}</span>
                        </div>
                        <span className="text-[10px] text-zinc-400 font-mono">
                        {i18n.t('command.system_load')}: <span className="text-green-400 ml-1">{i18n.t('command.nominal')} ({(engine.latency ?? 0).toFixed(2)}{i18n.t('common_units.ms')})</span>
                    </span>
                    </div>
                </div>
            </div>

            {/* Table Container */}
            <div className="overflow-x-auto relative z-10 custom-scrollbar">
                <table className="w-full text-left border-collapse">
                    <thead>
                        <tr className="border-b border-[color:var(--color-border)]/30 bg-[color:var(--color-background)]/20">
                            <th 
                                onClick={() => toggle_sort('name')}
                                className="p-5 text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] cursor-pointer hover:text-zinc-300 transition-colors"
                            >
                                {i18n.t('command.col_node')} {sort_key === 'name' && (sort_dir === 'asc' ? '↑' : '↓')}
                            </th>
                            <th className="p-5 text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em]">{i18n.t('command.col_sector')}</th>
                            <th 
                                onClick={() => toggle_sort('throughput')}
                                className="p-5 text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] text-right cursor-pointer hover:text-zinc-300 transition-colors"
                            >
                                {i18n.t('command.col_throughput')} {sort_key === 'throughput' && (sort_dir === 'asc' ? '↑' : '↓')}
                            </th>
                            <th 
                                onClick={() => toggle_sort('cost')}
                                className="p-5 text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] text-right cursor-pointer hover:text-zinc-300 transition-colors"
                            >
                                {i18n.t('command.col_cost')} {sort_key === 'cost' && (sort_dir === 'asc' ? '↑' : '↓')}
                            </th>
                            <th 
                                onClick={() => toggle_sort('util')}
                                className="p-5 text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] text-right cursor-pointer hover:text-zinc-300 transition-colors"
                            >
                                {i18n.t('command.col_allocation')} {sort_key === 'util' && (sort_dir === 'asc' ? '↑' : '↓')}
                            </th>
                            <th className="p-5 text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] text-center">{i18n.t('command.col_protocol')}</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-zinc-800/20">
                        {(sorted_agents || []).map((agent): React.ReactElement => {
                            const cost_usd = agent.cost_usd || 0;
                            const budget_usd = agent.budget_usd ?? 1;
                            const util = budget_usd === 0 ? (cost_usd > 0 ? 100 : 0) : (cost_usd / budget_usd) * 100;
                            const is_active = agent.status !== 'offline';

                            return (
                                <tr key={agent.id} className="hover:bg-zinc-800/20 transition-all duration-300 group/row">
                                    <td className="p-5">
                                        <div className="flex items-center gap-4">
                                            <div className="relative">
                                                <div
                                                    className="w-10 h-10 rounded-xl flex items-center justify-center font-bold text-sm shadow-xl transition-transform group-hover/row:scale-110"
                                                    style={{ backgroundColor: agent.theme_color || '#52525b', color: '#fff' }}
                                                >
                                                    {agent.name.charAt(0)}
                                                </div>
                                                {is_active && (
                                                    <div className="absolute -bottom-1 -right-1 w-3.5 h-3.5 bg-emerald-500 rounded-full border-4 border-[color:var(--color-surface)] shadow-lg" />
                                                )}
                                            </div>
                                            <div>
                                                <div className="font-bold text-sm text-zinc-100 group-hover/row:text-white transition-colors">
                                                    {agent.name}
                                                </div>
                                                <div className="text-[10px] text-zinc-600 font-bold uppercase tracking-wider font-mono">
                                                    ID_{agent.id.split('-')[0]}
                                                </div>
                                            </div>
                                        </div>
                                    </td>
                                    <td className="p-5">
                                        <div className="flex items-center gap-2">
                                            <div className="h-1 w-1 rounded-full bg-zinc-700" />
                                            <span className="text-xs font-mono text-zinc-400 uppercase tracking-tighter">
                                                {agent.department}
                                            </span>
                                        </div>
                                    </td>
                                    <td className="p-5 text-right font-mono">
                                        <div className="flex flex-col items-end gap-1">
                                            <span className="text-sm text-zinc-300 font-bold">
                                                {(agent.tokens_used || 0).toLocaleString()}
                                            </span>
                                            <span className="text-[9px] text-zinc-600 uppercase">{i18n.t('command.tkn_count')}</span>
                                            {/* I/O Token Breakdown */}
                                            {((agent.input_tokens || 0) + (agent.output_tokens || 0)) > 0 && (
                                                <div className="flex gap-1.5 text-[9px] font-mono text-zinc-700">
                                                    <span className="text-sky-700">↑{(agent.input_tokens || 0).toLocaleString()}</span>
                                                    <span className="text-emerald-700">↓{(agent.output_tokens || 0).toLocaleString()}</span>
                                                </div>
                                            )}
                                        </div>
                                    </td>
                                    <td className="p-5 text-right font-mono">
                                        <div className="flex items-center justify-end gap-1.5 text-zinc-100 text-sm font-bold">
                                            <DollarSign size={13} className="text-zinc-500" />
                                            {cost_usd.toFixed(4)}
                                        </div>
                                        <span className="text-[9px] text-zinc-600 uppercase tracking-tighter font-mono">{i18n.t('command.accreted_usd')}</span>
                                    </td>
                                    <td className="p-5">
                                        <div className="flex flex-col gap-2 min-w-[120px]">
                                            <div className="flex justify-between text-[10px] font-mono leading-none">
                                                <span className={clsx(
                                                    "font-bold",
                                                    util > 90 ? 'text-red-500' : util > 70 ? 'text-amber-500' : 'text-emerald-500'
                                                )}>
                                                    {util.toFixed(1)}%
                                                </span>
                                                <span className="text-zinc-600">{i18n.t('agent_config.fiscal_symbol')}{budget_usd.toFixed(0)} {i18n.t('common.max')}</span>
                                            </div>
                                            <div className="h-1.5 w-full bg-zinc-800/50 rounded-full overflow-hidden border border-zinc-700/20 p-[1px]">
                                                <motion.div
                                                    initial={{ width: 0 }}
                                                    animate={{ width: `${Math.min(util, 100)}%` }}
                                                    transition={{ duration: 1, ease: "easeOut" }}
                                                    className={clsx(
                                                        "h-full rounded-full transition-colors shadow-[0_0_8px_rgba(0,0,0,0.5)]",
                                                        util > 90 ? "bg-red-500 shadow-red-500/20" :
                                                            util > 70 ? "bg-amber-500 shadow-amber-500/20" :
                                                                "bg-emerald-500 shadow-emerald-500/20"
                                                    )}
                                                />
                                            </div>
                                        </div>
                                    </td>
                                    <td className="p-5">
                                        <div className="flex justify-center">
                                            <button
                                                onClick={() => join_chat(agent.id)}
                                                className="group/btn relative p-2.5 bg-zinc-800/50 border border-zinc-700/50 text-zinc-500 hover:text-white hover:bg-zinc-700/50 hover:border-zinc-500 rounded-xl transition-all active:scale-95 shadow-lg"
                                                title={i18n.t('command.establish_link')}
                                            >
                                                <MessageSquare size={18} className="group-hover/btn:scale-110 transition-transform" />
                                                <div className="absolute -inset-1 bg-white blur-md opacity-0 group-hover/btn:opacity-10 transition-opacity" />
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            );
                        })}
                    </tbody>
                </table>
            </div>

            {/* Footer Telemetry */}
            <div className="bg-[color:color-mix(in_srgb,var(--color-background)_80%,transparent)] px-4 py-2 border-t border-[color:var(--color-border)]/50 flex items-center justify-between text-[9px] font-mono text-zinc-500 uppercase tracking-widest backdrop-blur-sm">
                <div className="flex gap-4">
                    <span>{i18n.t('command.nodes_online', { count: engine.active_agents })}</span>
                    <span>{i18n.t('command.sync_freq', { freq: '60HZ' })}</span>
                </div>
                <div className="flex gap-4">
                    <span>{i18n.t('command.version', { ver: '1.1.70' })}</span>
                    <span className="text-zinc-600">{i18n.t('command.latency', { val: (engine.latency ?? 0).toFixed(2) })}{i18n.t('common_units.ms')}</span>
                </div>
            </div>
        </motion.div>
    );
};


// Metadata: [Command_Table]
