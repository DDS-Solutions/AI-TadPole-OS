import { useEffect, useState, useMemo } from 'react';
import {
    Shield,
    CheckCircle,
    XCircle,
    Clock,
    Target,
    AlertTriangle,
    Activity,
    Terminal as TerminalIcon,
    Search,
    Cpu,
    Plus,
    WifiOff,
    ShieldCheck
} from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { TwEmptyState, Tooltip } from '../components/ui';
import { useEngineStatus } from '../hooks/useEngineStatus';
import { useWorkspaceStore } from '../stores/workspaceStore';
import { TadpoleOSService } from '../services/tadpoleosService';
import { agents as allAgents } from '../data/mockAgents';
import { MOCK_PENDING, MOCK_LEDGER, type OversightEntry, type LedgerEntry } from '../data/mockOversight';
import { CommandTable } from '../components/CommandTable';
import { i18n } from '../i18n';

// Types mirrored from server/types.ts
// Types are now imported from ../data/mockOversight

export default function OversightDashboard() {
    const navigate = useNavigate();
    const { isOnline } = useEngineStatus();
    const [pending, setPending] = useState<OversightEntry[]>(() => {
        const saved = localStorage.getItem('tadpole_oversight_pending');
        if (saved) {
            try { return JSON.parse(saved); } catch { return []; }
        }
        return [];
    });
    const [ledger, setLedger] = useState<LedgerEntry[]>(() => {
        const saved = localStorage.getItem('tadpole_oversight_ledger');
        if (saved) {
            try { return JSON.parse(saved); } catch { return []; }
        }
        return [];
    });
    const [filter, setFilter] = useState('');
    const [isSimulated, setIsSimulated] = useState(false);
    const [hasAttemptedFetch, setHasAttemptedFetch] = useState(false);
    const [selectedClusterId, setSelectedClusterId] = useState<string>('all');
    const { clusters, activeProposals } = useWorkspaceStore();

    // Persistence: Save to localStorage on change
    useEffect(() => {
        if (pending.length > 0 || hasAttemptedFetch) {
            localStorage.setItem('tadpole_oversight_pending', JSON.stringify(pending));
        }
    }, [pending, hasAttemptedFetch]);

    useEffect(() => {
        if (ledger.length > 0 || hasAttemptedFetch) {
            localStorage.setItem('tadpole_oversight_ledger', JSON.stringify(ledger));
        }
    }, [ledger, hasAttemptedFetch]);

    // Poll for data (WebSocket would be better, but polling is simpler for Phase 3 start)
    useEffect(() => {
        const fetchData = async () => {
            if (isSimulated) {
                if (pending.length === 0 && !hasAttemptedFetch) {
                    setPending(MOCK_PENDING);
                    setLedger(MOCK_LEDGER);
                }
                return;
            }

            try {
                const [pendingData, ledgerData] = await Promise.all([
                    TadpoleOSService.getPendingOversight(),
                    TadpoleOSService.getOversightLedger()
                ]);

                setPending(pendingData as OversightEntry[]);
                setLedger(ledgerData as LedgerEntry[]);
                setIsSimulated(false); // We got real data (even if empty)

                // Use the data just fetched to update stats, rather than waiting for next render
                // This is slightly tricky without closure data, so we let the useEffect [ledger, pending] handle it
                setHasAttemptedFetch(true);
            } catch {
                if (!hasAttemptedFetch) {
                    setPending(MOCK_PENDING);
                    setLedger(MOCK_LEDGER);
                    setIsSimulated(true);
                    setHasAttemptedFetch(true);
                }
            }
        };

        // updateStats removed - handled by separate effect below

        fetchData();
        const interval = setInterval(() => {
            if (document.visibilityState === 'visible') fetchData();
        }, isSimulated ? 5000 : 2000);
        return () => clearInterval(interval);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- pending.length used inside but adding it would restart polling on each decision
    }, [isSimulated, hasAttemptedFetch]);

    // Use useMemo for stats to avoid cascading renders
    const stats = useMemo(() => ({
        pending: pending.length,
        approved: ledger.filter((entry) => entry.decision === 'approved').length,
        rejected: ledger.filter((entry) => entry.decision === 'rejected').length
    }), [pending, ledger]);

    const handleDecide = async (id: string, decision: 'approved' | 'rejected') => {
        if (isSimulated) {
            // Simulated local move
            const entry = pending.find(p => p.id === id);
            if (entry) {
                setLedger(prev => [{ ...entry, decision, timestamp: new Date().toISOString() } as LedgerEntry, ...prev]);
                setPending(prev => prev.filter(p => p.id !== id));
            }
            return;
        }

        try {
            await TadpoleOSService.decideOversight(id, decision);
            // Optimistic update
            setPending(prev => prev.filter(p => p.id !== id));
        } catch {
            // Silently fail, would be logged in a real environment
        }
    };

    const handleKillSwitch = async () => {
        if (!confirm(i18n.t('oversight.confirm_halt_agents'))) return;

        if (isSimulated) {
            setPending([]);
            return;
        }

        try {
            await TadpoleOSService.killAgents();
            alert(i18n.t('oversight.agents_halted'));
        } catch {
            alert(i18n.t('oversight.halt_agents_failed'));
        }
    };

    const handleKillEngine = async () => {
        if (!confirm(i18n.t('oversight.confirm_kill_engine'))) return;

        const userInput = prompt(i18n.t('oversight.shutdown_confirm_prompt'));
        if (userInput !== 'SHUTDOWN') return;

        try {
            await TadpoleOSService.shutdownEngine();
            alert(i18n.t('oversight.engine_shutting_down'));
        } catch (e: unknown) {
            alert(i18n.t('oversight.kill_engine_failed', { error: e instanceof Error ? e.message : 'Unknown error' }));
        }
    };

    const filteredLedger = ledger
        .filter(l => {
            const toolCall = l.toolCall || l; // Handle flat structures from older backend versions
            const matchesCluster = selectedClusterId === 'all' || toolCall.clusterId === selectedClusterId;
            const matchesSearch = (toolCall.agentId || '').toLowerCase().includes(filter.toLowerCase()) ||
                (toolCall.skill || '').toLowerCase().includes(filter.toLowerCase()) ||
                JSON.stringify(toolCall.params || {}).toLowerCase().includes(filter.toLowerCase());
            return matchesCluster && matchesSearch;
        })
        .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());

    const filteredPending = pending.filter(p => {
        const toolCall = p.toolCall || p;
        return selectedClusterId === 'all' || toolCall.clusterId === selectedClusterId;
    });

    return (
        <div className="p-6 space-y-6 max-w-7xl mx-auto">
            {/* Fallback / Simulation Banner */}
            {isSimulated && (
                <div className="bg-amber-500/10 border border-amber-500/30 p-3 rounded-xl flex items-center justify-between animate-in fade-in slide-in-from-top-4">
                    <div className="flex items-center gap-3">
                        <div className="p-1.5 bg-amber-500/20 rounded-lg">
                            <WifiOff size={16} className="text-amber-500" />
                        </div>
                        <div>
                            <p className="text-xs font-bold text-amber-200 uppercase tracking-widest">{i18n.t('oversight.disconnected_title')}</p>
                            <p className="text-[10px] text-amber-500/70 font-mono">{i18n.t('oversight.disconnected_subtitle')}</p>
                        </div>
                    </div>
                    <button
                        onClick={() => setIsSimulated(false)}
                        className="text-[10px] px-3 py-1 bg-amber-500/20 hover:bg-amber-500/30 text-amber-400 rounded-full border border-amber-500/30 transition-colors uppercase font-bold tracking-tighter"
                    >
                        {i18n.t('oversight.retry_connection')}
                    </button>
                </div>
            )}

            {/* Header / Stats */}
            <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
                <Tooltip content={i18n.t('oversight.pending_actions_tooltip')} position="top" className="w-full">
                    <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-lg flex items-center justify-between cursor-help w-full">
                        <div>
                            <p className="text-zinc-400 text-sm">{i18n.t('oversight.pending_actions_label')}</p>
                            <p className="text-2xl font-bold text-yellow-400">{stats.pending}</p>
                        </div>
                        <Clock className="w-8 h-8 text-yellow-500/20" />
                    </div>
                </Tooltip>
                <Tooltip content={i18n.t('oversight.approved_tooltip')} position="top" className="w-full">
                    <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-lg flex items-center justify-between cursor-help w-full">
                        <div>
                            <p className="text-zinc-400 text-sm">{i18n.t('oversight.approved_label')}</p>
                            <p className="text-2xl font-bold text-green-400">{stats.approved}</p>
                        </div>
                        <CheckCircle className="w-8 h-8 text-green-500/20" />
                    </div>
                </Tooltip>
                <Tooltip content={i18n.t('oversight.rejected_tooltip')} position="top" className="w-full">
                    <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-lg flex items-center justify-between cursor-help w-full">
                        <div>
                            <p className="text-zinc-400 text-sm">{i18n.t('oversight.rejected_label')}</p>
                            <p className="text-2xl font-bold text-red-400">{stats.rejected}</p>
                        </div>
                        <XCircle className="w-8 h-8 text-red-500/20" />
                    </div>
                </Tooltip>
                <Tooltip content={i18n.t('oversight.halt_agents_tooltip')} position="top" className="w-full">
                    <button
                        onClick={handleKillSwitch}
                        className={`p-4 rounded-lg flex items-center justify-center gap-2 font-bold transition-colors cursor-pointer group border w-full ${isOnline ? 'bg-amber-500/10 hover:bg-amber-500/20 border-amber-500/50 text-amber-400' : 'bg-zinc-800/10 border-zinc-700/50 text-zinc-600 opacity-50'}`}
                        disabled={!isOnline}
                    >
                        <Shield className="w-5 h-5 group-hover:scale-110 transition-transform" />
                        {isOnline ? i18n.t('oversight.halt_agents_button') : i18n.t('oversight.offline_label')}
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('oversight.kill_engine_tooltip')} position="top" className="w-full">
                    <button
                        onClick={handleKillEngine}
                        className={`p-4 rounded-lg flex items-center justify-center gap-2 font-bold transition-colors cursor-pointer group border w-full ${isOnline ? 'bg-red-600/10 hover:bg-red-600/20 border-red-600/50 text-red-500' : 'bg-zinc-800/10 border-zinc-700/50 text-zinc-600 opacity-50'}`}
                        disabled={!isOnline}
                    >
                        <WifiOff className="w-5 h-5 group-hover:scale-110 transition-transform" />
                        {isOnline ? i18n.t('oversight.kill_engine_button') : i18n.t('oversight.offline_label')}
                    </button>
                </Tooltip>

                {/* Row 2 (Partial) */}
                <div className="md:col-start-4 md:col-span-2">
                    <Tooltip content={i18n.t('oversight.security_dashboard_tooltip')} position="top" className="w-full">
                        <button
                            onClick={() => navigate('/security')}
                            className="w-full p-4 bg-zinc-950 border border-zinc-700 hover:border-blue-500/50 hover:bg-blue-500/5 text-zinc-300 hover:text-blue-400 rounded-lg flex items-center justify-center gap-2 font-bold transition-all group"
                        >
                            <ShieldCheck className="w-5 h-5 group-hover:scale-110 transition-transform text-blue-500" />
                            {i18n.t('oversight.security_dashboard_button')}
                        </button>
                    </Tooltip>
                </div>
            </div>

            {/* Pending Queue */}
            {pending.length > 0 && (
                <div className="bg-zinc-900 border border-yellow-500/30 rounded-lg overflow-hidden">
                    <div className="bg-yellow-500/10 p-3 border-b border-yellow-500/20 flex items-center gap-2">
                        <AlertTriangle className="w-4 h-4 text-yellow-500" />
                        <h2 className="font-semibold text-yellow-100">{i18n.t('oversight.awaiting_approval_title', { count: filteredPending.length })}</h2>
                    </div>
                    <div className="divide-y divide-zinc-800">
                        {filteredPending.map(entry => (
                            <div key={entry.id} className="p-4 flex flex-col md:flex-row gap-4 items-start md:items-center justify-between animate-in fade-in slide-in-from-top-2">
                                <div className="space-y-1">
                                    <div className="flex items-center gap-2">
                                        <span className="text-xs font-mono bg-zinc-800 text-zinc-300 px-2 py-0.5 rounded">
                                            {entry.toolCall?.agentId || entry.agentId || i18n.t('oversight.unknown_agent')}
                                        </span>
                                        <span className="text-sm font-medium text-blue-400 flex items-center gap-1">
                                            <TerminalIcon className="w-3 h-3" />
                                            {entry.toolCall?.skill || entry.skill || i18n.t('oversight.capability_proposal')}
                                        </span>
                                        <span className="text-xs text-zinc-500">
                                            {new Date(entry.createdAt).toLocaleTimeString()}
                                        </span>
                                    </div>
                                    <p className="text-zinc-300">{entry.toolCall?.description || entry.description || i18n.t('oversight.awaiting_authorization')}</p>
                                    <pre className="text-xs bg-black/50 p-2 rounded text-zinc-400 font-mono overflow-auto max-w-2xl">
                                        {JSON.stringify(entry.toolCall?.params || entry.params || {}, null, 2)}
                                    </pre>
                                </div>
                                <div className="flex gap-2 w-full md:w-auto">
                                    <Tooltip content={i18n.t('oversight.approve_action_tooltip')} position="top">
                                        <button
                                            onClick={() => handleDecide(entry.id, 'approved')}
                                            className="flex-1 md:flex-none bg-green-500/20 hover:bg-green-500/30 text-green-400 px-4 py-2 rounded border border-green-500/30 font-medium transition-colors"
                                        >
                                            {i18n.t('oversight.approve_button')}
                                        </button>
                                    </Tooltip>
                                    <Tooltip content={i18n.t('oversight.reject_action_tooltip')} position="top">
                                        <button
                                            onClick={() => handleDecide(entry.id, 'rejected')}
                                            className="flex-1 md:flex-none bg-red-500/20 hover:bg-red-500/30 text-red-400 px-4 py-2 rounded border border-red-500/30 font-medium transition-colors"
                                        >
                                            {i18n.t('oversight.reject_button')}
                                        </button>
                                    </Tooltip>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            )}

            {/* Action Ledger */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden flex flex-col h-[600px]">
                <div className="p-4 border-b border-zinc-800 flex items-center justify-between bg-zinc-900/50">
                    <div className="flex items-center gap-2">
                        <Tooltip content={i18n.t('oversight.ledger_tooltip')} position="right">
                            <Activity className="w-4 h-4 text-blue-400 cursor-help" />
                        </Tooltip>
                        <h2 className="font-semibold text-zinc-100">{i18n.t('oversight.ledger_title')}</h2>
                    </div>
                    <div className="flex items-center gap-4">
                        <div className="relative">
                            <Tooltip content="Filter logs by mission cluster" position="top">
                                <Target className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 cursor-help" />
                            </Tooltip>
                            <select
                                value={selectedClusterId}
                                onChange={(e) => setSelectedClusterId(e.target.value)}
                                className="bg-zinc-950 border border-zinc-700 rounded-full pl-9 pr-8 py-1.5 text-sm text-zinc-200 focus:outline-none focus:border-blue-500 appearance-none cursor-pointer"
                            >
                                <option value="all">{i18n.t('oversight.all_missions')}</option>
                                {clusters.map(c => (
                                    <option key={c.id} value={c.id}>{c.name}</option>
                                ))}
                            </select>
                        </div>
                        <div className="relative">
                            <Tooltip content={i18n.t('oversight.search_ledger_tooltip')} position="top">
                                <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 cursor-help" />
                            </Tooltip>
                            <input
                                type="text"
                                placeholder={i18n.t('oversight.filter_actions_placeholder')}
                                value={filter}
                                onChange={(e) => setFilter(e.target.value)}
                                className="bg-zinc-950 border border-zinc-700 rounded-full pl-9 pr-4 py-1.5 text-sm text-zinc-200 focus:outline-none focus:border-blue-500 w-48"
                            />
                        </div>
                    </div>
                </div>

                <div className="overflow-auto flex-1 p-0">
                    <table className="w-full text-left text-sm">
                        <thead className="bg-zinc-950 text-zinc-400 sticky top-0 z-10">
                            <tr>
                                <th className="p-3 font-medium border-b border-zinc-800">{i18n.t('oversight.table_time')}</th>
                                <th className="p-3 font-medium border-b border-zinc-800">{i18n.t('oversight.table_agent')}</th>
                                <th className="p-3 font-medium border-b border-zinc-800">{i18n.t('oversight.table_action')}</th>
                                <th className="p-3 font-medium border-b border-zinc-800">{i18n.t('oversight.table_params')}</th>
                                <th className="p-3 font-medium border-b border-zinc-800">{i18n.t('oversight.table_result')}</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-zinc-800/50">
                            {filteredLedger.map(entry => (
                                <tr key={entry.id} className="hover:bg-zinc-800/20 transition-colors">
                                    <td className="p-3 text-zinc-500 whitespace-nowrap font-mono text-xs">
                                        {new Date(entry.timestamp).toLocaleTimeString()}
                                    </td>
                                    <td className="p-3 text-zinc-300 font-medium">
                                        {entry.toolCall?.agentId || entry.agentId || i18n.t('oversight.unknown_agent')}
                                    </td>
                                    <td className="p-3">
                                        <div className="flex items-center gap-2">
                                            <span className={`w-2 h-2 rounded-full ${entry.decision === 'approved' ? 'bg-green-500' : 'bg-red-500'}`} />
                                            <span className="font-mono text-blue-400">{entry.toolCall?.skill || entry.skill || i18n.t('oversight.proposal_label')}</span>
                                        </div>
                                    </td>
                                    <td className="p-3 max-w-xs truncate text-zinc-400 font-mono text-xs" title={JSON.stringify(entry.toolCall?.params || entry.params || {}, null, 2)}>
                                        {JSON.stringify(entry.toolCall?.params || entry.params || {})}
                                    </td>
                                    <td className="p-3">
                                        {entry.decision === 'rejected' ? (
                                            <span className="text-red-400 text-xs uppercase font-bold tracking-wider">{i18n.t('oversight.blocked_label')}</span>
                                        ) : (
                                            <span className={`text-xs ${entry.result?.success ? 'text-green-400' : 'text-red-400'}`}>
                                                {entry.result?.success ? i18n.t('oversight.success_label') : i18n.t('oversight.failed_label')}
                                                <span className="text-zinc-600 ml-1">({entry.result?.durationMs}ms)</span>
                                            </span>
                                        )}
                                    </td>
                                </tr>
                            ))}
                            {filteredLedger.length === 0 && (
                                <tr>
                                    <td colSpan={5}>
                                        <TwEmptyState title={i18n.t('oversight.no_actions_title')} description={i18n.t('oversight.no_actions_description')} />
                                    </td>
                                </tr>
                            )}
                        </tbody>
                    </table>
                </div>
            </div>

            {/* Swarm Intelligence / Alpha Reasoning (Option A Enhancement) */}
            <div className="bg-zinc-900 border border-blue-500/30 rounded-lg overflow-hidden">
                <div className="bg-blue-500/10 p-3 border-b border-blue-500/20 flex items-center gap-2">
                    <Tooltip content={i18n.t('oversight.swarm_intel_tooltip')} position="right">
                        <Cpu className="w-4 h-4 text-blue-400 cursor-help" />
                    </Tooltip>
                    <h2 className="font-semibold text-blue-100">{i18n.t('oversight.swarm_intel_title')}</h2>
                </div>
                <div className="p-6">
                    {Object.values(activeProposals || {}).length > 0 ? (
                        <div className="grid grid-cols-1 gap-6">
                            {Object.values(activeProposals).map((proposal) => {
                                const cluster = clusters.find(c => c.id === proposal.clusterId);
                                return (
                                    <div key={proposal.clusterId} className="bg-zinc-950 border border-zinc-800 rounded-xl p-4 space-y-4">
                                        <div className="flex justify-between items-center">
                                            <div className="flex items-center gap-2">
                                                <div className="w-2 h-2 rounded-full bg-blue-500 animate-pulse" />
                                                <span className="text-xs font-bold text-zinc-300 uppercase tracking-widest">{cluster?.name || i18n.t('oversight.unknown_cluster')}</span>
                                            </div>
                                            <span className="text-[10px] text-zinc-600 font-mono">
                                                {i18n.t('oversight.alpha_node_prefix', { alphaId: cluster?.alphaId ?? '?' })} • {new Date(proposal.timestamp).toLocaleTimeString()}
                                            </span>
                                        </div>

                                        <div className="bg-black/40 p-3 rounded-lg border border-zinc-800/50">
                                            <div className="text-[9px] font-bold text-zinc-500 mb-2 uppercase tracking-wide flex items-center gap-2">
                                                <Activity size={10} /> {i18n.t('oversight.neural_trace_label')}
                                            </div>
                                            <p className="text-xs text-zinc-400 leading-relaxed font-mono whitespace-pre-wrap">
                                                {proposal.reasoning}
                                            </p>
                                        </div>

                                        <div className="space-y-2">
                                            <div className="text-[9px] font-bold text-zinc-500 uppercase tracking-wide">{i18n.t('oversight.proposed_reallocations_label')}</div>
                                            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
                                                {proposal.changes.map(change => {
                                                    const agent = allAgents.find(a => a.id === change.agentId);
                                                    return (
                                                        <div key={change.agentId} className="p-2.5 bg-zinc-900 border border-zinc-800 rounded-lg flex flex-col gap-1.5">
                                                            <div className="flex justify-between items-start">
                                                                <span className="text-[10px] font-bold text-zinc-200">{agent?.name}</span>
                                                                <span className="text-[8px] px-1 rounded bg-zinc-800 text-zinc-500 font-mono uppercase">{i18n.t('oversight.mod_req_label')}</span>
                                                            </div>
                                                            <div className="space-y-1">
                                                                {change.proposedRole && (
                                                                    <div className="flex items-center gap-2 text-[9px]">
                                                                        <span className="text-zinc-600 uppercase">{i18n.t('oversight.role_label')}</span>
                                                                        <span className="text-blue-400">{change.proposedRole}</span>
                                                                    </div>
                                                                )}
                                                                {change.proposedModel && (
                                                                    <div className="flex items-center gap-2 text-[9px]">
                                                                        <span className="text-zinc-600 uppercase">{i18n.t('oversight.model_label')}</span>
                                                                        <span className="text-blue-400">{change.proposedModel}</span>
                                                                    </div>
                                                                )}
                                                                {change.addedSkills && (
                                                                    <div className="flex items-center gap-2 text-[9px]">
                                                                        <span className="text-zinc-600 uppercase">{i18n.t('oversight.skills_label')}</span>
                                                                        <span className="text-emerald-400">+{change.addedSkills.length}</span>
                                                                    </div>
                                                                )}
                                                            </div>
                                                        </div>
                                                    );
                                                })}
                                            </div>
                                        </div>
                                    </div>
                                );
                            })}
                        </div>
                    ) : (
                        <TwEmptyState
                            icon={<Plus size={32} />}
                            title={i18n.t('oversight.no_optimization_traces')}
                        />
                    )}
                </div>
            </div>

            {/* Neural Footprint Monitoring (Moved from OpsDashboard) */}
            <CommandTable />
        </div>
    );
}

