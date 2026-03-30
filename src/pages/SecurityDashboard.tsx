import { useEffect, useState } from 'react';
import { 
    ShieldCheck, 
    Lock, 
    Activity, 
    AlertCircle, 
    Heart, 
    Zap, 
    History,
    DollarSign,
    CheckCircle2,
    ArrowUpDown,
    Plus,
    Minus
} from 'lucide-react';
import { TadpoleOSService, type Quotas, type AuditEntry, type AgentHealth } from '../services/tadpoleosService';
import { Tooltip } from '../components/ui';
import { i18n } from '../i18n';

export default function SecurityDashboard() {
    const [quotas, setQuotas] = useState<Quotas | null>(null);
    const [auditTrail, setAuditTrail] = useState<AuditEntry[]>([]);
    const [agentHealth, setAgentHealth] = useState<AgentHealth[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [sortConfig, setSortConfig] = useState<{ key: 'name' | 'status' | 'quota', direction: 'asc' | 'desc' }>({ key: 'name', direction: 'asc' });
    const [isUpdatingQuota, setIsUpdatingQuota] = useState<string | null>(null);

    const fetchData = async () => {
        try {
            const [q, a, h] = await Promise.all([
                TadpoleOSService.getSecurityQuotas(),
                TadpoleOSService.getAuditTrail(1, 10),
                TadpoleOSService.getAgentHealth()
            ]);
            setQuotas(q);
            setAuditTrail(a.data || []);
            setAgentHealth(h.agents || []);
        } catch (error: unknown) {
            console.error("Failed to fetch security data", error);
        } finally {
            setIsLoading(false);
        }
    };

    const handleUpdateQuota = async (entityId: string, currentBudget: number, increment: number) => {
        setIsUpdatingQuota(entityId);
        try {
            await TadpoleOSService.updateSecurityQuota(entityId, currentBudget + increment);
            await fetchData();
        } catch (error) {
            console.error("Failed to update quota", error);
        } finally {
            setIsUpdatingQuota(null);
        }
    };

    const getSortedQuotas = () => {
        if (!quotas?.agentQuotas) return [];
        return [...quotas.agentQuotas].sort((a, b) => {
            const healthA = agentHealth.find(h => h.agentId === a.entity_id);
            const healthB = agentHealth.find(h => h.agentId === b.entity_id);

            if (sortConfig.key === 'name') {
                return sortConfig.direction === 'asc' 
                    ? a.entity_id.localeCompare(b.entity_id)
                    : b.entity_id.localeCompare(a.entity_id);
            }
            if (sortConfig.key === 'status') {
                const statusA = healthA?.status || 'offline';
                const statusB = healthB?.status || 'offline';
                return sortConfig.direction === 'asc'
                    ? statusA.localeCompare(statusB)
                    : statusB.localeCompare(statusA);
            }
            if (sortConfig.key === 'quota') {
                const ratioA = a.used_usd / a.budget_usd;
                const ratioB = b.used_usd / b.budget_usd;
                return sortConfig.direction === 'asc' ? ratioA - ratioB : ratioB - ratioA;
            }
            return 0;
        });
    };

    useEffect(() => {
        fetchData();
        const interval = setInterval(fetchData, 5000);
        return () => clearInterval(interval);
    }, []);

    if (isLoading && !quotas) {
        return (
            <div className="flex items-center justify-center p-20">
                <Activity className="animate-spin text-blue-500 mr-2" />
                <span className="text-zinc-400 font-mono">{i18n.t('security.loading')}</span>
            </div>
        );
    }

    return (
        <div className="p-6 space-y-6 max-w-7xl mx-auto" aria-label="Security Dashboard">
            <h1 className="sr-only">Tadpole OS Security & Governance Management</h1>
            <header className="flex justify-between items-center bg-zinc-900 border border-zinc-800 p-6 rounded-2xl">
                <div>
                    <h1 className="text-2xl font-bold text-zinc-100 flex items-center gap-2">
                        <ShieldCheck className="text-emerald-500" />
                        {i18n.t('security.title')}
                    </h1>
                    <p className="text-zinc-500 text-sm mt-1">
                        {i18n.t('security.subtitle')}
                    </p>
                </div>
                <div className="flex items-center gap-3">
                    <div className="grid grid-rows-4 grid-flow-col gap-1">
                        {agentHealth.map(a => (
                            <Tooltip key={a.agentId} content={`${a.name}: ${a.isHealthy ? i18n.t('security.status_healthy') : i18n.t('security.status_degraded')}`}>
                                <div className={`w-8 h-8 rounded-full border-2 border-zinc-900 flex items-center justify-center text-[10px] font-bold ${a.isHealthy ? 'bg-emerald-500/20 text-emerald-400' : 'bg-red-500/20 text-red-400'}`}>
                                    {a.name.charAt(0)}
                                </div>
                            </Tooltip>
                        ))}
                    </div>
                    <div className="h-8 w-px bg-zinc-800 mx-2" />
                    <div className={`px-3 py-1 border rounded-full flex items-center gap-2 ${quotas?.systemDefense.merkleIntegrity === 1.0 ? 'bg-emerald-500/10 border-emerald-500/30' : 'bg-red-500/10 border-red-500/30'}`}>
                        <div className={`w-2 h-2 rounded-full animate-pulse ${quotas?.systemDefense.merkleIntegrity === 1.0 ? 'bg-emerald-500' : 'bg-red-500'}`} />
                        <span className={`text-[10px] font-bold uppercase tracking-widest ${quotas?.systemDefense.merkleIntegrity === 1.0 ? 'text-emerald-400' : 'text-red-400'}`}>
                            {quotas?.systemDefense.merkleIntegrity === 1.0 ? i18n.t('security.system_secured') : i18n.t('security.integrity_compromised')}
                        </span>
                    </div>
                </div>
            </header>

            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-xl">
                    <div className="flex items-center justify-between mb-2">
                        <p className="text-zinc-400 text-xs font-mono uppercase">{i18n.t('security.budget_consumption')}</p>
                        <DollarSign size={14} className="text-zinc-500" />
                    </div>
                    <p className="text-2xl font-bold text-zinc-100">${quotas?.totalSpent.toFixed(2)}</p>
                    <div className="mt-3 h-1.5 bg-zinc-800 rounded-full overflow-hidden">
                        <div 
                            className="h-full bg-blue-500" 
                            style={{ width: `${Math.min(quotas?.efficiency || 0, 100)}%` }}
                        />
                    </div>
                    <p className="text-[10px] text-zinc-500 mt-2">
                        {i18n.t('security.efficiency_label', { percentage: (quotas?.efficiency ?? 0).toFixed(1) })}
                    </p>
                </div>

                <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-xl">
                    <div className="flex items-center justify-between mb-2">
                        <p className="text-zinc-400 text-xs font-mono uppercase">{i18n.t('security.active_agents')}</p>
                        <Zap size={14} className="text-amber-500" />
                    </div>
                    <p className="text-2xl font-bold text-zinc-100">{agentHealth.length}</p>
                    <p className="text-[10px] text-zinc-500 mt-2 font-mono">
                        {i18n.t('security.executing_missions', { count: agentHealth.filter(a => a.status === 'active').length })}
                    </p>
                </div>

                <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-xl">
                    <div className="flex items-center justify-between mb-2">
                        <p className="text-zinc-400 text-xs font-mono uppercase">{i18n.t('security.system_health')}</p>
                        <Heart size={14} className="text-emerald-500" />
                    </div>
                    <p className="text-2xl font-bold text-zinc-100">
                        {i18n.t('security.health_ratio', { healthy: agentHealth.filter(a => a.isHealthy).length, total: agentHealth.length })}
                    </p>
                    <p className="text-[10px] text-zinc-500 mt-2 font-mono uppercase tracking-tighter">
                        {agentHealth.every(a => a.isHealthy) ? i18n.t('security.optimal_ops') : i18n.t('security.degraded_ops')}
                    </p>
                </div>

                <div className="bg-zinc-900 border border-zinc-800 p-4 rounded-xl">
                    <div className="flex items-center justify-between mb-2">
                        <p className="text-zinc-400 text-xs font-mono uppercase">{i18n.t('security.verified_decisions')}</p>
                        <Lock size={14} className={quotas?.systemDefense.merkleIntegrity === 1.0 ? "text-emerald-500" : "text-red-500"} />
                    </div>
                    <p className="text-2xl font-bold text-zinc-100">{auditTrail.length}</p>
                    <p className={`text-[10px] mt-2 font-mono uppercase tracking-tighter ${quotas?.systemDefense.merkleIntegrity === 1.0 ? "text-emerald-500" : "text-red-500"}`}>
                        {i18n.t('security.crypto_integrity', { percentage: ((quotas?.systemDefense.merkleIntegrity || 0) * 100).toFixed(0) })}
                    </p>
                </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {/* Audit Trail */}
                <div className="bg-zinc-900 border border-zinc-800 rounded-2xl flex flex-col h-[400px]">
                    <div className="p-4 border-b border-zinc-800 flex items-center justify-between bg-zinc-900/50">
                        <div className="flex items-center gap-2">
                            <History className="w-4 h-4 text-emerald-400" />
                            <h2 className="font-semibold text-zinc-100">{i18n.t('security.audit_trail_title')}</h2>
                        </div>
                        <span className="text-[10px] bg-emerald-500/10 text-emerald-400 px-2 py-0.5 rounded border border-emerald-500/20 font-mono uppercase tracking-widest">
                            {i18n.t('security.merkle_chain_active')}
                        </span>
                    </div>
                    <div className="overflow-auto flex-1">
                        <table className="w-full text-left text-xs">
                            <thead className="bg-black/20 text-zinc-500 sticky top-0 z-10">
                                <tr>
                                    <th className="p-3 font-medium border-b border-zinc-800">{i18n.t('security.th_decided')}</th>
                                    <th className="p-3 font-medium border-b border-zinc-800">{i18n.t('security.th_agent')}</th>
                                    <th className="p-3 font-medium border-b border-zinc-800">{i18n.t('security.th_skill')}</th>
                                    <th className="p-3 font-medium border-b border-zinc-800">{i18n.t('security.th_status')}</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-zinc-800/50">
                                {auditTrail.map(entry => (
                                    <tr key={entry.id} className="hover:bg-zinc-800/20 transition-colors group">
                                        <td className="p-3 text-zinc-500 font-mono">
                                            {entry.decidedAt ? new Date(entry.decidedAt).toLocaleTimeString() : i18n.t('security.status_pending')}
                                        </td>
                                        <td className="p-3 text-zinc-300 font-medium">
                                            {entry.agentId}
                                        </td>
                                        <td className="p-3 text-blue-400 font-mono">
                                            {entry.skill || '—'}
                                        </td>
                                        <td className="p-3">
                                            <div className="flex items-center gap-2">
                                                {entry.isVerified ? (
                                                    <CheckCircle2 size={12} className="text-emerald-500" />
                                                ) : (
                                                    <AlertCircle size={12} className="text-red-500" />
                                                )}
                                                <span className={`uppercase font-bold tracking-tighter ${entry.isVerified ? 'text-emerald-500' : 'text-red-500'}`}>
                                                    {entry.isVerified ? (entry.decision || i18n.t('security.status_recorded')) : i18n.t('security.status_recorded')}
                                                </span>
                                            </div>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </div>

                {/* Agent Health Monitoring */}
                <div className="bg-zinc-900 border border-zinc-800 rounded-2xl flex flex-col h-[400px]">
                    <div className="p-4 border-b border-zinc-800 bg-zinc-900/50 flex items-center justify-between">
                        <div className="flex items-center gap-2">
                            <Activity className="w-4 h-4 text-amber-400" />
                            <h2 className="font-semibold text-zinc-100">{i18n.t('security.swarm_health_monitor')}</h2>
                        </div>
                    </div>
                    <div className="p-4 grid grid-cols-1 gap-3 overflow-auto">
                        {agentHealth.map(a => (
                            <div key={a.agentId} className={`p-4 rounded-xl border flex items-center justify-between ${a.isHealthy ? 'bg-zinc-950/50 border-zinc-800' : 'bg-red-500/5 border-red-500/20 ring-1 ring-red-500/10'}`}>
                                <div className="flex items-center gap-3">
                                    <div className={`w-10 h-10 rounded-xl border flex items-center justify-center font-bold ${a.isHealthy ? 'bg-zinc-900 border-zinc-800 text-zinc-300' : 'bg-red-500/10 border-red-500/20 text-red-400'}`}>
                                        {a.name.charAt(0)}
                                    </div>
                                    <div>
                                        <h3 className="text-sm font-bold text-zinc-200">{a.name}</h3>
                                        <p className="text-[10px] text-zinc-500 font-mono">{a.agentId}</p>
                                    </div>
                                </div>
                                <div className="flex flex-col items-end gap-1">
                                    <div className="flex items-center gap-1.5">
                                        <span className={`w-2 h-2 rounded-full ${a.isHealthy ? 'bg-emerald-500' : 'bg-red-500'}`} />
                                        <span className={`text-[10px] font-bold uppercase ${a.isHealthy ? 'text-zinc-500' : 'text-red-400'}`}>
                                            {i18n.t('security.failures_label', { count: a.failureCount })}
                                        </span>
                                    </div>
                                    {a.isThrottled && (
                                        <span className="text-[9px] bg-red-500/20 text-red-500 px-2 py-0.5 rounded-full font-bold uppercase tracking-widest">
                                            {i18n.t('security.throttled')}
                                        </span>
                                    )}
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            </div>

            {/* Periodic Resource Quotas (BudgetGuard) */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-2xl flex flex-col overflow-hidden">
                <div className="p-4 border-b border-zinc-800 bg-zinc-900/50 flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <DollarSign className="w-4 h-4 text-blue-400" />
                        <h2 className="font-semibold text-zinc-100">{i18n.t('security.resource_quotas')}</h2>
                    </div>
                    <div className="flex items-center gap-2">
                        <button 
                            onClick={() => setSortConfig({ key: 'name', direction: sortConfig.direction === 'asc' ? 'desc' : 'asc' })}
                            className={`px-3 py-1 rounded text-[10px] font-bold uppercase transition-colors flex items-center gap-1 ${sortConfig.key === 'name' ? 'bg-blue-500/20 text-blue-400' : 'bg-zinc-800 text-zinc-500 hover:bg-zinc-700'}`}
                        >
                            <ArrowUpDown size={10} /> {i18n.t('security.sort_name')}
                        </button>
                        <button 
                            onClick={() => setSortConfig({ key: 'status', direction: sortConfig.direction === 'asc' ? 'desc' : 'asc' })}
                            className={`px-3 py-1 rounded text-[10px] font-bold uppercase transition-colors flex items-center gap-1 ${sortConfig.key === 'status' ? 'bg-blue-500/20 text-blue-400' : 'bg-zinc-800 text-zinc-500 hover:bg-zinc-700'}`}
                        >
                            <ArrowUpDown size={10} /> {i18n.t('security.sort_status')}
                        </button>
                        <button 
                            onClick={() => setSortConfig({ key: 'quota', direction: sortConfig.direction === 'asc' ? 'desc' : 'asc' })}
                            className={`px-3 py-1 rounded text-[10px] font-bold uppercase transition-colors flex items-center gap-1 ${sortConfig.key === 'quota' ? 'bg-blue-500/20 text-blue-400' : 'bg-zinc-800 text-zinc-500 hover:bg-zinc-700'}`}
                        >
                            <ArrowUpDown size={10} /> {i18n.t('security.sort_quota')}
                        </button>
                    </div>
                </div>
                <div className="p-4 grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
                    {getSortedQuotas().map(q => {
                        const health = agentHealth.find(h => h.agentId === q.entity_id);
                        const isExceeded = q.used_usd >= q.budget_usd;
                        return (
                            <div key={q.entity_id} className={`p-4 rounded-xl border flex flex-col gap-4 ${isExceeded ? 'bg-red-500/5 border-red-500/30' : 'bg-zinc-950/50 border-zinc-800'}`}>
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center gap-3">
                                        <div className={`w-8 h-8 rounded-lg flex items-center justify-center font-bold text-xs ${health?.isHealthy ? 'bg-emerald-500/10 text-emerald-500 border border-emerald-500/20' : 'bg-zinc-800 text-zinc-500 border border-zinc-700'}`}>
                                            {q.entity_id.charAt(0).toUpperCase()}
                                        </div>
                                        <div>
                                            <h3 className="text-sm font-bold text-zinc-200">{health?.name || q.entity_id}</h3>
                                            <div className="flex items-center gap-2">
                                                <span className={`w-1.5 h-1.5 rounded-full ${health?.status === 'active' ? 'bg-emerald-500 animate-pulse' : 'bg-zinc-600'}`} />
                                                <p className="text-[10px] text-zinc-500 font-mono uppercase">{health?.status || 'offline'}</p>
                                            </div>
                                        </div>
                                    </div>
                                    <div className="text-right">
                                        <p className={`text-xs font-bold font-mono ${isExceeded ? 'text-red-500' : 'text-zinc-300'}`}>
                                            ${q.used_usd.toFixed(3)} / ${q.budget_usd.toFixed(2)}
                                        </p>
                                        <p className="text-[9px] text-zinc-500 uppercase font-bold tracking-tighter">
                                            {i18n.t('security.reset_label', { period: q.reset_period })}
                                        </p>
                                    </div>
                                </div>
                                
                                <div className="space-y-2">
                                    <div className="h-1.5 bg-zinc-900 rounded-full overflow-hidden border border-zinc-800">
                                        <div 
                                            className={`h-full transition-all duration-500 ${isExceeded ? 'bg-red-500' : (q.used_usd/q.budget_usd > 0.8 ? 'bg-amber-500' : 'bg-blue-500')}`}
                                            style={{ width: `${Math.min((q.used_usd / q.budget_usd) * 100, 100)}%` }}
                                        />
                                    </div>
                                    <div className="flex justify-between items-center">
                                        <span className="text-[10px] text-zinc-500 font-mono">{i18n.t('security.usage_label', { percentage: ((q.used_usd / q.budget_usd) * 100).toFixed(1) })}</span>
                                        <div className="flex items-center gap-2">
                                            <button 
                                                disabled={isUpdatingQuota === q.entity_id || q.budget_usd <= 0.1}
                                                onClick={() => handleUpdateQuota(q.entity_id, q.budget_usd, -0.5)}
                                                className="p-1.5 rounded-lg bg-zinc-900 border border-zinc-800 text-zinc-400 hover:text-red-400 hover:border-red-500/30 disabled:opacity-50 transition-all shadow-sm"
                                            >
                                                <Minus size={12} />
                                            </button>
                                            <button 
                                                disabled={isUpdatingQuota === q.entity_id}
                                                onClick={() => handleUpdateQuota(q.entity_id, q.budget_usd, 0.5)}
                                                className="p-1.5 rounded-lg bg-zinc-900 border border-zinc-800 text-zinc-400 hover:text-emerald-400 hover:border-emerald-500/30 disabled:opacity-50 transition-all shadow-sm flex items-center gap-1 pr-2"
                                            >
                                                <Plus size={12} />
                                                <span className="text-[10px] font-bold">+$0.50</span>
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        );
                    })}
                </div>
            </div>

            {/* Proactive Defense Matrix */}
            <div className="bg-zinc-900 border border-blue-500/30 rounded-2xl overflow-hidden shadow-[0_0_30px_rgba(59,130,246,0.1)]">
                <div className="bg-blue-500/10 p-4 border-b border-blue-500/20 flex items-center gap-2">
                    <ShieldCheck className="w-4 h-4 text-blue-400" />
                    <h2 className="font-semibold text-blue-100 uppercase tracking-widest text-xs">{i18n.t('security.defense_matrix')}</h2>
                </div>
                <div className="p-6">
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        <div className="space-y-4">
                            <h3 className="text-xs font-bold text-zinc-500 uppercase flex items-center gap-2">
                                <Lock size={12} /> {i18n.t('security.resource_guard')}
                            </h3>
                            <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800">
                                <div className="flex justify-between items-center mb-1">
                                    <span className="text-xs text-zinc-400">{i18n.t('security.memory_pressure')}</span>
                                    <span className={`text-[10px] font-mono ${ (quotas?.systemDefense.memoryPressure || 0) > 0.8 ? 'text-red-500' : 'text-emerald-500'}`}>
                                        {((quotas?.systemDefense.memoryPressure || 0) * 100).toFixed(1)}%
                                    </span>
                                </div>
                                <div className="h-1 bg-zinc-800 rounded-full">
                                    <div 
                                        className={`h-full transition-all duration-1000 ${ (quotas?.systemDefense.memoryPressure || 0) > 0.8 ? 'bg-red-500' : 'bg-emerald-500'}`} 
                                        style={{ width: `${(quotas?.systemDefense.memoryPressure || 0) * 100}%` }}
                                    />
                                </div>
                            </div>
                        </div>

                        <div className="space-y-4">
                            <h3 className="text-xs font-bold text-zinc-500 uppercase flex items-center gap-2">
                                <AlertCircle size={12} /> {i18n.t('security.capability_bounds')}
                            </h3>
                            <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800">
                                <div className="flex justify-between items-center mb-1">
                                    <span className="text-xs text-zinc-400">{i18n.t('security.environment')}</span>
                                    <span className={`text-[10px] font-mono ${quotas?.systemDefense.sandboxStatus === 'ACTIVE' ? 'text-blue-500' : 'text-amber-500'}`}>
                                        {quotas?.systemDefense.sandboxType || 'Unknown'}
                                    </span>
                                </div>
                                <div className="h-1 bg-zinc-800 rounded-full">
                                    <div 
                                        className={`h-full ${quotas?.systemDefense.sandboxStatus === 'ACTIVE' ? 'bg-blue-500/30' : 'bg-amber-500/30'}`} 
                                        style={{ width: quotas?.systemDefense.sandboxStatus === 'ACTIVE' ? '100%' : '50%' }}
                                    />
                                </div>
                            </div>
                        </div>

                        <div className="space-y-4">
                            <h3 className="text-xs font-bold text-zinc-500 uppercase flex items-center gap-2">
                                <ShieldCheck size={12} /> {i18n.t('security.shell_safety')}
                            </h3>
                            <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800">
                                <div className="flex justify-between items-center mb-1">
                                    <span className="text-xs text-zinc-400">{i18n.t('security.secret_leak_prevention')}</span>
                                    <span className="text-[10px] text-emerald-500 font-mono">{i18n.t('security.enabled')}</span>
                                </div>
                                <div className="h-1 bg-zinc-800 rounded-full">
                                    <div className="w-[45%] h-full bg-emerald-500" />
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
