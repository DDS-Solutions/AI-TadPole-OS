/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / MCP_Store
 * - **Primary Entrypoints**: `MCP_Store`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `MCP_Store.test.tsx`
 */

import { useEffect, useState, useCallback } from 'react';
import { Box, Zap, AlertCircle, RefreshCw } from 'lucide-react';
import { fetchMCPRegistry } from '../components/mcp_store/mcp_store_api';
import { MCP_Card } from '../components/mcp_store/MCP_Card';
import type { MCP_Connector } from '../components/mcp_store/types';
import { i18n } from '../i18n';

function MCP_Store() {
    const [connectors, setConnectors] = useState<MCP_Connector[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const loadConnectors = useCallback(async () => {
        setIsLoading(true);
        setError(null);
        try {
            const data = await fetchMCPRegistry();
            setConnectors(data);
        } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            setError(msg);
        } finally {
            setIsLoading(false);
        }
    }, []);

    useEffect(() => {
        let mounted = true;
        void (async () => {
            await Promise.resolve();
            if (mounted) {
                try {
                    await loadConnectors();
                } catch (err) {
                    if (mounted) {
                        setError(err instanceof Error ? err.message : String(err));
                        setIsLoading(false);
                    }
                }
            }
        })();
        return () => { mounted = false; };
    }, [loadConnectors]);

    return (
        <div className="p-6 space-y-6 max-w-7xl mx-auto w-full">
            {/* GEO Optimization: Structured Data & Semantic Header */}
            <script type="application/ld+json">
                {JSON.stringify({
                    "@context": "https://schema.org",
                    "@type": "SoftwareApplication",
                    "name": "Tadpole OS MCP Connector Store",
                    "description": "Public catalog of Model Context Protocol (MCP) servers and tools for sovereign agent integration.",
                    "author": { "@type": "Organization", "name": "Sovereign Engineering" },
                    "applicationCategory": "Developer Tool",
                    "operatingSystem": "Tadpole OS"
                })}
            </script>

            <header className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 border-b border-[color:var(--color-border)]/50 pb-6 mb-4">
                <div>
                    <h1 className="text-2xl font-bold text-zinc-100 flex items-center gap-3 tracking-tight">
                        <Box className="text-emerald-500" size={28} />
                        {i18n.t('mcp_store.title', { defaultValue: 'MCP Connector Store' })}
                    </h1>
                    <p className="text-sm text-zinc-400 mt-1">
                        {i18n.t('mcp_store.description', { defaultValue: 'Explore and install Model Context Protocol (MCP) connectors for your sovereign swarm.' })}
                    </p>
                </div>
                <div className="flex items-center gap-2 px-3 py-1.5 bg-[color:var(--color-surface)] rounded-full border border-[color:var(--color-border)] text-xs font-mono text-zinc-400">
                    <Zap size={14} className="text-emerald-500 animate-pulse" />
                    <span>{connectors.length} {i18n.t('mcp_store.available_count', { defaultValue: 'Connectors Available' })}</span>
                </div>
            </header>

            {error && (
                <div className="p-4 rounded-xl bg-rose-500/10 border border-rose-500/30 flex items-center justify-between text-xs text-rose-300">
                    <div className="flex items-center gap-2">
                        <AlertCircle className="w-4 h-4 text-rose-400" />
                        <span>{error}</span>
                    </div>
                    <button
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
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-emerald-500/50 border-t-emerald-500"></div>
                    <span className="text-xs text-zinc-500 font-mono animate-pulse">{i18n.t('common.loading', { defaultValue: 'Loading registry...' })}</span>
                </div>
            ) : connectors.length === 0 && !error ? (
                <div className="text-center py-16 text-zinc-500 text-sm">
                    {i18n.t('mcp_store.empty', { defaultValue: 'No MCP connectors found in the registry.' })}
                </div>
            ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {connectors.map(c => (
                        <MCP_Card key={c.id} connector={c} />
                    ))}
                </div>
            )}
        </div>
    );
}

export default MCP_Store;
