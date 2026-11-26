/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Mcp_Store / Mcp_Blueprints_Tab
 * - **Primary Entrypoints**: `Mcp_Blueprints_Tab`
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[Mcp_Blueprints_Tab]`
 */

import { useCallback, useEffect, useMemo, useState } from 'react';
import { AlertCircle, RefreshCw, Zap } from 'lucide-react';
import { fetchMCPRegistry } from './mcp_store_api';
import { MCP_Card } from './MCP_Card';
import type { MCP_Connector } from './types';
import { APP_REFRESH_AGENTS_EVENT, REPO_URL } from '../template_store/constants';
import type { InstalledSwarmSummary } from '../template_store/types';
import { system_api_service } from '../../services/system_api_service';
import { use_notification_store } from '../../stores/notification_store';
import { event_bus } from '../../services/event_bus';
import { i18n } from '../../i18n';

interface McpBlueprintsTabProps {
    installedSwarms: InstalledSwarmSummary[];
    onInstalledChange: () => Promise<void>;
    onCountChange?: (count: number) => void;
}

export function Mcp_Blueprints_Tab({
    installedSwarms,
    onInstalledChange,
    onCountChange,
}: McpBlueprintsTabProps) {
    const [connectors, setConnectors] = useState<MCP_Connector[]>([]);
    const [sessionInstalledIds, setSessionInstalledIds] = useState<Set<string>>(() => new Set());
    const [installingId, setInstallingId] = useState<string | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const loadConnectors = useCallback(async () => {
        setIsLoading(true);
        setError(null);
        try {
            const data = await fetchMCPRegistry();
            setConnectors(data);
            onCountChange?.(data.length);
        } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            setError(msg);
        } finally {
            setIsLoading(false);
        }
    }, [onCountChange]);

    useEffect(() => {
        let mounted = true;
        void (async () => {
            if (mounted) {
                await loadConnectors();
            }
        })();
        return () => {
            mounted = false;
        };
    }, [loadConnectors]);

    const resolvedConnectors = useMemo(() => {
        return connectors.map((item) => {
            const isPersistedInstalled = installedSwarms.some(
                (s) => s.id === item.id || s.template_path === item.path
            );
            return {
                ...item,
                installed: Boolean(item.installed || isPersistedInstalled || sessionInstalledIds.has(item.id)),
            };
        });
    }, [connectors, installedSwarms, sessionInstalledIds]);

    const handleInstall = useCallback(async (connector: MCP_Connector) => {
        setInstallingId(connector.id);
        setError(null);
        try {
            await system_api_service.engine.install_template(
                REPO_URL,
                connector.path,
                undefined,
                true
            );
            setSessionInstalledIds((prev) => new Set(prev).add(connector.id));
            await onInstalledChange();
            window.dispatchEvent(new Event(APP_REFRESH_AGENTS_EVENT));
            use_notification_store.getState().add_notification?.({
                title: i18n.t('mcp_store.installed_title', { defaultValue: 'MCP Connector Installed' }),
                message: `${connector.name} (v${connector.version}) installed via template engine.`,
                severity: 'success',
                persistent: false,
            });
            event_bus.emit_log({
                source: 'System',
                text: `[Mcp_Blueprints_Tab] Installed MCP connector: ${connector.name} (v${connector.version})`,
                severity: 'success',
            });
        } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            console.warn('[Mcp_Blueprints_Tab] Failed to install MCP connector:', msg);
            setError(`Failed to install ${connector.name}: ${msg}`);
            use_notification_store.getState().add_notification?.({
                title: i18n.t('mcp_store.install_failed_title', { defaultValue: 'MCP Install Failed' }),
                message: msg,
                severity: 'error',
                persistent: false,
            });
        } finally {
            setInstallingId(null);
        }
    }, [onInstalledChange]);

    return (
        <div className="space-y-6 animate-in fade-in duration-200">
            <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3">
                <p className="text-sm text-zinc-400">
                    {i18n.t('mcp_store.description', {
                        defaultValue: 'Explore and install Model Context Protocol (MCP) connector blueprints directly into the engine.',
                    })}
                </p>
                <div className="flex items-center gap-2 px-3 py-1.5 bg-[color:var(--color-surface)] rounded-full border border-[color:var(--color-border)] text-xs font-mono text-zinc-400 shrink-0">
                    <Zap size={14} className="text-emerald-500 animate-pulse" />
                    <span>
                        {resolvedConnectors.length}{' '}
                        {i18n.t('mcp_store.available_count', { defaultValue: 'Connectors Available' })}
                    </span>
                </div>
            </div>

            {error && (
                <div className="p-4 rounded-xl bg-rose-500/10 border border-rose-500/30 flex items-center justify-between text-xs text-rose-300">
                    <div className="flex items-center gap-2">
                        <AlertCircle className="w-4 h-4 text-rose-400 shrink-0" />
                        <span>{error}</span>
                    </div>
                    <button
                        type="button"
                        onClick={loadConnectors}
                        className="px-3 py-1 bg-rose-600 hover:bg-rose-500 text-white font-mono rounded text-[10px] flex items-center gap-1 transition-colors cursor-pointer"
                    >
                        <RefreshCw className="w-3 h-3" />
                        {i18n.t('common.retry', { defaultValue: 'Retry' })}
                    </button>
                </div>
            )}

            {isLoading ? (
                <div className="flex flex-col items-center justify-center py-24 gap-3">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-emerald-500/50 border-t-emerald-500" />
                    <span className="text-xs text-zinc-500 font-mono animate-pulse">
                        {i18n.t('common.loading', { defaultValue: 'Loading registry...' })}
                    </span>
                </div>
            ) : resolvedConnectors.length === 0 && !error ? (
                <div className="text-center py-16 text-zinc-500 text-sm">
                    {i18n.t('mcp_store.empty', { defaultValue: 'No MCP connectors found in the registry.' })}
                </div>
            ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {resolvedConnectors.map((connector) => (
                        <MCP_Card
                            key={connector.id}
                            connector={connector}
                            isInstalling={installingId === connector.id}
                            onInstall={handleInstall}
                        />
                    ))}
                </div>
            )}
        </div>
    );
}
