/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Installed_Swarms_List
 * - **Primary Entrypoints**: `Installed_Swarms_List`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `Installed_Swarms_List.test.tsx`
 */

import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import {
    Search,
    Bot,
    FileText,
    Server,
    Trash2,
    RefreshCw,
    Layers,
    Calendar,
    ArrowRight,
    ShieldCheck,
    AlertCircle,
    CheckCircle2,
    Wrench,
    X
} from 'lucide-react';
import { system_api_service } from '../../services/system_api_service';
import type { InstalledSwarmSummary, UninstallTemplateResponse } from './types';
import { Uninstall_Swarm_Modal } from './Uninstall_Swarm_Modal';
import { APP_REFRESH_AGENTS_EVENT } from './constants';
import { i18n } from '../../i18n';

function t_or(key: string, fallback: string): string {
    const val = i18n.t(key);
    return val && val.trim() !== '' ? val : fallback;
}

interface InstalledSwarmsListProps {
    swarms?: InstalledSwarmSummary[];
    isLoading?: boolean;
    error?: string | null;
    onUninstallClick?: (swarm: InstalledSwarmSummary) => void;
    onRefresh?: () => void;
    onBrowseMarketplace?: () => void;
    onRefreshCatalog?: () => void;
}

interface SwarmCardProps {
    swarm: InstalledSwarmSummary;
    onUninstallClick: (swarm: InstalledSwarmSummary) => void;
}

export const Swarm_Card = React.memo(function Swarm_Card({
    swarm,
    onUninstallClick
}: SwarmCardProps) {
    const formattedDate = useMemo(() => {
        if (!swarm.installed_at) return '—';
        const parsed = new Date(swarm.installed_at);
        if (isNaN(parsed.getTime())) return '—';
        return parsed.toLocaleDateString(undefined, {
            year: 'numeric',
            month: 'short',
            day: 'numeric'
        });
    }, [swarm.installed_at]);

    const hasSkills = (swarm.skills?.length ?? 0) > 0;
    const gridColsClass = hasSkills ? 'grid-cols-4' : 'grid-cols-3';

    return (
        <div className="bg-[color:var(--color-surface)]/40 border border-[color:var(--color-border)]/60 hover:border-emerald-500/40 rounded-2xl p-6 flex flex-col justify-between space-y-5 transition-all shadow-lg hover:shadow-emerald-950/20 group">
            <div className="space-y-3">
                <div className="flex items-center justify-between gap-2">
                    <div className="flex items-center gap-2">
                        <span className="p-1.5 bg-emerald-500/10 text-emerald-400 rounded-lg border border-emerald-500/20">
                            <ShieldCheck size={14} />
                        </span>
                        {swarm.industry && (
                            <span className="text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded bg-zinc-800 text-zinc-300">
                                {swarm.industry}
                            </span>
                        )}
                    </div>

                    <span className="text-[10px] text-zinc-500 font-mono flex items-center gap-1">
                        <Calendar size={11} />
                        {formattedDate}
                    </span>
                </div>

                <div>
                    <h3 className="text-base font-bold text-white tracking-tight group-hover:text-emerald-300 transition-colors">
                        {swarm.name}
                    </h3>
                    <p className="text-xs text-zinc-400 line-clamp-2 mt-1 leading-relaxed">
                        {swarm.description || 'Installed sovereign swarm cluster.'}
                    </p>
                </div>

                {/* Asset Counts */}
                <div className={`grid ${gridColsClass} gap-2 pt-2 border-t border-[color:var(--color-border)]/40`}>
                    <div className="p-2 bg-zinc-950/40 rounded-lg border border-[color:var(--color-border)]/40 flex flex-col items-center">
                        <div className="flex items-center gap-1 text-emerald-400 mb-0.5">
                            <Bot size={12} />
                            <span className="text-xs font-bold">{swarm.agents?.length || 0}</span>
                        </div>
                        <span className="text-[9px] uppercase tracking-wider text-zinc-500 font-mono">Agents</span>
                    </div>

                    <div className="p-2 bg-zinc-950/40 rounded-lg border border-[color:var(--color-border)]/40 flex flex-col items-center">
                        <div className="flex items-center gap-1 text-blue-400 mb-0.5">
                            <FileText size={12} />
                            <span className="text-xs font-bold">{swarm.workflows?.length || 0}</span>
                        </div>
                        <span className="text-[9px] uppercase tracking-wider text-zinc-500 font-mono">Workflows</span>
                    </div>

                    {hasSkills && (
                        <div className="p-2 bg-zinc-950/40 rounded-lg border border-[color:var(--color-border)]/40 flex flex-col items-center">
                            <div className="flex items-center gap-1 text-amber-400 mb-0.5">
                                <Wrench size={12} />
                                <span className="text-xs font-bold">{swarm.skills?.length || 0}</span>
                            </div>
                            <span className="text-[9px] uppercase tracking-wider text-zinc-500 font-mono">Skills</span>
                        </div>
                    )}

                    <div className="p-2 bg-zinc-950/40 rounded-lg border border-[color:var(--color-border)]/40 flex flex-col items-center">
                        <div className="flex items-center gap-1 text-purple-400 mb-0.5">
                            <Server size={12} />
                            <span className="text-xs font-bold">{swarm.mcp_servers?.length || 0}</span>
                        </div>
                        <span className="text-[9px] uppercase tracking-wider text-zinc-500 font-mono">MCP</span>
                    </div>
                </div>
            </div>

            {/* Actions */}
            <div className="pt-3 border-t border-[color:var(--color-border)]/40 flex items-center justify-between gap-3">
                <span className="text-[10px] font-mono text-zinc-500 truncate">
                    ID: {swarm.id}
                </span>

                <button
                    type="button"
                    onClick={() => onUninstallClick(swarm)}
                    aria-label={`Uninstall ${swarm.name}`}
                    className="px-3 py-1.5 rounded-lg text-xs font-bold text-red-400 hover:text-red-300 hover:bg-red-500/10 border border-red-500/20 flex items-center gap-1.5 transition-all"
                >
                    <Trash2 size={13} />
                    <span>{t_or('template_store.btn_uninstall', 'Uninstall')}</span>
                </button>
            </div>
        </div>
    );
});

export function Installed_Swarms_List({
    swarms: controlledSwarms,
    isLoading: controlledLoading,
    error: controlledError,
    onUninstallClick,
    onRefresh,
    onBrowseMarketplace,
    onRefreshCatalog
}: InstalledSwarmsListProps) {
    const isControlled = controlledSwarms !== undefined;

    // Stable ref for callbacks to prevent infinite re-render cycles in dependencies
    const onRefreshRef = useRef(onRefresh);
    useEffect(() => {
        onRefreshRef.current = onRefresh;
    }, [onRefresh]);

    const onRefreshCatalogRef = useRef(onRefreshCatalog);
    useEffect(() => {
        onRefreshCatalogRef.current = onRefreshCatalog;
    }, [onRefreshCatalog]);

    // Request-id tracking to prevent race conditions and out-of-order responses
    const requestIdRef = useRef(0);

    // Internal state when used in uncontrolled mode
    const [internalSwarms, setInternalSwarms] = useState<InstalledSwarmSummary[]>([]);
    const [internalLoading, setInternalLoading] = useState(true);
    const [internalError, setInternalError] = useState<string | null>(null);

    // Uninstall modal & feedback states
    const [selectedForUninstall, setSelectedForUninstall] = useState<InstalledSwarmSummary | null>(null);
    const [isUninstalling, setIsUninstalling] = useState(false);
    const [uninstallError, setUninstallError] = useState<string | null>(null);
    const [uninstallSuccess, setUninstallSuccess] = useState<{ swarmName: string; receipt: UninstallTemplateResponse } | null>(null);

    const [searchQuery, setSearchQuery] = useState('');

    const fetchInstalledSwarms = useCallback(async () => {
        if (isControlled) {
            onRefreshRef.current?.();
            return;
        }

        const currentRequestId = ++requestIdRef.current;
        try {
            setInternalLoading(true);
            setInternalError(null);
            const response = await system_api_service.engine.get_installed_templates();

            // Ignore stale responses
            if (currentRequestId !== requestIdRef.current) {
                return;
            }
            setInternalSwarms(response?.swarms || []);
        } catch (err) {
            if (currentRequestId !== requestIdRef.current) {
                return;
            }
            const msg = err instanceof Error ? err.message : String(err);
            setInternalError(msg);
        } finally {
            if (currentRequestId === requestIdRef.current) {
                setInternalLoading(false);
            }
        }
    }, [isControlled]);

    useEffect(() => {
        let isMounted = true;
        if (!isControlled) {
            void (async () => {
                if (isMounted) {
                    await fetchInstalledSwarms();
                }
            })();
        }
        return () => {
            isMounted = false;
            // Invalidate in-flight requests on unmount
            requestIdRef.current += 1;
        };
    }, [isControlled, fetchInstalledSwarms]);

    const activeSwarms = isControlled ? controlledSwarms : internalSwarms;
    const activeLoading = isControlled ? (controlledLoading ?? false) : internalLoading;
    const activeError = isControlled ? (controlledError ?? null) : internalError;

    const handleInternalConfirmUninstall = useCallback(async (swarmId: string, archive: boolean) => {
        const targetSwarm = selectedForUninstall;
        try {
            setIsUninstalling(true);
            setUninstallError(null);
            const response = await system_api_service.engine.uninstall_template(swarmId, archive);

            setSelectedForUninstall(null);
            setUninstallSuccess({
                swarmName: targetSwarm?.name || swarmId,
                receipt: response
            });

            // Refetch or notify parent
            if (isControlled) {
                onRefreshRef.current?.();
            } else {
                await fetchInstalledSwarms();
            }

            window.dispatchEvent(new Event(APP_REFRESH_AGENTS_EVENT));
            onRefreshCatalogRef.current?.();
        } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            setUninstallError(msg);
        } finally {
            setIsUninstalling(false);
        }
    }, [selectedForUninstall, isControlled, fetchInstalledSwarms]);

    const handleUninstallClick = useCallback((swarm: InstalledSwarmSummary) => {
        if (onUninstallClick) {
            onUninstallClick(swarm);
        } else {
            setUninstallError(null);
            setSelectedForUninstall(swarm);
        }
    }, [onUninstallClick]);

    const handleBrowseClick = () => {
        if (onBrowseMarketplace) {
            onBrowseMarketplace();
        } else if (!isControlled) {
            fetchInstalledSwarms();
        }
    };

    const filtered = useMemo(() => {
        const query = searchQuery.toLowerCase().trim();
        const swarms = activeSwarms || [];
        if (!query) return swarms;

        return swarms.filter((s) => (
            s.name.toLowerCase().includes(query) ||
            s.id.toLowerCase().includes(query) ||
            s.description.toLowerCase().includes(query) ||
            (s.industry && s.industry.toLowerCase().includes(query))
        ));
    }, [activeSwarms, searchQuery]);

    return (
        <div className="space-y-6 animate-in fade-in duration-200">
            {/* Success Feedback Toast / Banner */}
            {uninstallSuccess && (
                <div className="p-4 bg-emerald-500/10 border border-emerald-500/30 rounded-2xl flex items-center justify-between gap-3 text-emerald-300 animate-in fade-in">
                    <div className="flex items-center gap-2.5 text-xs">
                        <CheckCircle2 size={18} className="text-emerald-400 shrink-0" />
                        <div>
                            <span className="font-bold">Deactivated {uninstallSuccess.swarmName}:</span>{' '}
                            <span>
                                {uninstallSuccess.receipt.uninstalled_agents?.length || 0} agents,{' '}
                                {uninstallSuccess.receipt.uninstalled_workflows?.length || 0} workflows,{' '}
                                {uninstallSuccess.receipt.uninstalled_skills?.length || 0} skills removed.
                            </span>
                            {uninstallSuccess.receipt.archived_path && (
                                <span className="block text-[11px] text-zinc-400 font-mono mt-0.5">
                                    Archived to: {uninstallSuccess.receipt.archived_path}
                                </span>
                            )}
                        </div>
                    </div>
                    <button
                        onClick={() => setUninstallSuccess(null)}
                        aria-label="Dismiss success message"
                        className="text-zinc-500 hover:text-zinc-300 p-1 rounded-lg hover:bg-emerald-500/10 transition-colors"
                    >
                        <X size={16} />
                    </button>
                </div>
            )}

            {/* Top Filter and Actions Bar */}
            <div className="flex flex-col sm:flex-row justify-between items-stretch sm:items-center gap-4 bg-[color:var(--color-surface)]/30 border border-[color:var(--color-border)]/50 rounded-2xl p-4">
                <div className="relative flex-1 max-w-md">
                    <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" />
                    <input
                        type="text"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        aria-label="Search installed swarms"
                        placeholder={t_or('template_store.installed_search_placeholder', 'Search installed swarms...')}
                        className="w-full bg-zinc-950/60 border border-[color:var(--color-border)] rounded-xl pl-9 pr-4 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none focus:border-emerald-500 transition-colors"
                    />
                </div>

                <div className="flex items-center gap-3">
                    <button
                        onClick={isControlled ? onRefresh : fetchInstalledSwarms}
                        disabled={activeLoading}
                        aria-label="Refresh installed swarms"
                        className="px-4 py-2 rounded-xl text-xs font-bold bg-[color:var(--color-surface)] hover:bg-[color:var(--color-surface)]/80 text-zinc-300 hover:text-white border border-[color:var(--color-border)] flex items-center gap-2 transition-all disabled:opacity-50"
                    >
                        <RefreshCw size={14} className={activeLoading ? 'animate-spin' : ''} />
                        {t_or('template_store.btn_refresh', 'Refresh')}
                    </button>

                    {onBrowseMarketplace && (
                        <button
                            onClick={handleBrowseClick}
                            className="px-4 py-2 rounded-xl text-xs font-bold bg-emerald-600 hover:bg-emerald-500 text-white flex items-center gap-1.5 shadow-lg shadow-emerald-500/20 transition-all"
                        >
                            <span>{t_or('template_store.browse_marketplace', 'Browse Marketplace')}</span>
                            <ArrowRight size={14} />
                        </button>
                    )}
                </div>
            </div>

            {/* Content Area */}
            {activeLoading ? (
                <div className="py-24 flex flex-col items-center justify-center text-zinc-500">
                    <div className="w-8 h-8 rounded-full border-2 border-emerald-500 border-t-transparent animate-spin mb-4"></div>
                    <p className="text-xs font-mono">{t_or('template_store.loading_installed', 'Loading installed swarms...')}</p>
                </div>
            ) : activeError ? (
                <div className="py-16 p-6 bg-red-500/10 border border-red-500/20 rounded-2xl flex flex-col items-center justify-center text-center text-red-400">
                    <AlertCircle size={32} className="mb-3 opacity-60" />
                    <h3 className="text-sm font-bold">{t_or('template_store.installed_error_title', 'Failed to load installed swarms')}</h3>
                    <p className="text-xs font-mono mt-1 text-red-300">{activeError}</p>
                </div>
            ) : (activeSwarms || []).length === 0 ? (
                <div className="py-24 border border-dashed border-[color:var(--color-border)] rounded-2xl flex flex-col items-center justify-center text-center p-8 bg-[color:var(--color-surface)]/20">
                    <div className="w-14 h-14 rounded-2xl bg-zinc-900 border border-zinc-800 flex items-center justify-center text-zinc-500 mb-4">
                        <Layers size={28} />
                    </div>
                    <h3 className="text-base font-bold text-zinc-100 mb-1">
                        {t_or('template_store.no_installed_title', 'No Swarms Installed Yet')}
                    </h3>
                    <p className="text-xs text-zinc-400 max-w-sm mb-6 leading-relaxed">
                        {t_or(
                            'template_store.no_installed_desc',
                            'Explore our industry template catalog to deploy turnkey multi-agent swarms with zero configuration.'
                        )}
                    </p>
                    <button
                        onClick={handleBrowseClick}
                        className="px-6 py-2.5 rounded-xl text-xs font-bold bg-emerald-600 hover:bg-emerald-500 text-white flex items-center gap-2 shadow-lg shadow-emerald-500/20 transition-all"
                    >
                        <span>
                            {onBrowseMarketplace
                                ? t_or('template_store.explore_templates', 'Explore Industry Templates')
                                : t_or('template_store.btn_refresh', 'Refresh Swarms')}
                        </span>
                        <ArrowRight size={14} />
                    </button>
                </div>
            ) : filtered.length === 0 ? (
                <div className="py-16 text-center text-zinc-500 text-xs">
                    {t_or('template_store.no_search_results', 'No installed swarms matching your search.')}
                </div>
            ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {filtered.map((swarm) => (
                        <Swarm_Card
                            key={swarm.id}
                            swarm={swarm}
                            onUninstallClick={handleUninstallClick}
                        />
                    ))}
                </div>
            )}

            {/* Render internal modal if selected and parent did not provide onUninstallClick */}
            {!onUninstallClick && selectedForUninstall && (
                <Uninstall_Swarm_Modal
                    swarm={selectedForUninstall}
                    isOpen={true}
                    isUninstalling={isUninstalling}
                    error={uninstallError}
                    onClose={() => {
                        setSelectedForUninstall(null);
                        setUninstallError(null);
                    }}
                    onConfirm={handleInternalConfirmUninstall}
                />
            )}
        </div>
    );
}

// Metadata: [Template_Store]
