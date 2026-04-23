/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Public catalog of available AI models and intelligence templates. 
 * Orchestrates discovery and deployment of new model nodes into the local swarm.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Catalog retrieval error (external network), or deployment script rejection due to permission scope.
 * - **Telemetry Link**: Search for `[Model_Store]` or `CATALOG_FETCH` in service logs.
 */

import { useState, useEffect, useCallback } from 'react';
import { 
    Download, 
    Check, 
    Cpu, 
    Info, 
    Search, 
    HardDrive,
    Zap,
    RefreshCw,
    Server
} from 'lucide-react';
import { tadpole_os_service, type Store_Model, type Swarm_Node } from '../services/tadpoleos_service';
import { Tw_Empty_State, Tooltip } from '../components/ui';
import { i18n } from '../i18n';

export default function Model_Store() {
    const [catalog, set_catalog] = useState<Store_Model[]>([]);
    const [nodes, set_nodes] = useState<Swarm_Node[]>([]);
    const [filter, set_filter] = useState('');
    const [category, set_category] = useState<Store_Model['tags'][number] | 'all'>('all');
    const [is_loading, set_is_loading] = useState(true);
    const [pull_statuses, set_pull_statuses] = useState<Record<string, 'idle' | 'pulling' | 'completed' | 'error'>>({});
    const [error_msg, set_error_msg] = useState<string | null>(null);

    const categories: (Store_Model['tags'][number] | 'all')[] = ['all', 'vision', 'code', 'general'];

    const fetch_data = useCallback(async () => {
        set_is_loading(true);
        try {
            const [catalog_data, nodes_data] = await Promise.all([
                tadpole_os_service.get_model_catalog(),
                tadpole_os_service.get_nodes()
            ]);
            set_catalog(catalog_data);
            set_nodes(nodes_data.filter(n => n.status !== 'offline'));
            set_error_msg(null);
        } catch (e) {
            set_error_msg(i18n.t('model_store.fetch_failed'));
            console.error(e);
        } finally {
            set_is_loading(false);
        }
    }, []);

    useEffect(() => {
        void (async () => {
            await Promise.resolve();
            fetch_data();
        })();
    }, [fetch_data]);

    const handle_pull = async (model_id: string, node_id: string) => {
        const status_key = `${model_id}-${node_id}`;
        set_pull_statuses(prev => ({ ...prev, [status_key]: 'pulling' }));

        try {
            await tadpole_os_service.pull_model(model_id, node_id);
            set_pull_statuses(prev => ({ ...prev, [status_key]: 'completed' }));
            
            // Keep completed status for 3 seconds
            setTimeout(() => {
                set_pull_statuses(prev => ({ ...prev, [status_key]: 'idle' }));
            }, 3000);
        } catch (e) {
            set_pull_statuses(prev => ({ ...prev, [status_key]: 'error' }));
            console.error(e);
        }
    };

    const filtered_catalog = catalog.filter(model => {
        const matches_search = model.name.toLowerCase().includes(filter.toLowerCase()) || 
                             model.id.toLowerCase().includes(filter.toLowerCase());
        const matches_category = category === 'all' || model.tags.includes(category);
        return matches_search && matches_category;
    });

    if (is_loading) {
        return (
            <div className="p-6 flex flex-col items-center justify-center min-h-[400px] space-y-4">
                <RefreshCw className="w-8 h-8 text-green-500 animate-spin" />
                <p className="text-zinc-400 font-mono text-sm animate-pulse">{i18n.t('model_store.loading_catalog')}</p>
            </div>
        );
    }

    return (
        <div className="p-6 space-y-6 max-w-7xl mx-auto">
            {/* GEO Optimization: Structured Data & Semantic Header */}
            <script type="application/ld+json">
            {JSON.stringify({
              "@context": "https://schema.org",
              "@type": "SoftwareApplication",
              "name": "Tadpole OS Model Store",
              "description": "Public catalog of qualified AI models and intelligence templates for local swarm deployment. VRAM-aware and privacy-first AI orchestration.",
              "author": { "@type": "Organization", "name": "Sovereign Engineering" },
              "applicationCategory": "AI Model Catalog",
              "operatingSystem": "Tadpole OS"
            })}
            </script>
            <h1 className="sr-only">{i18n.t('model_store.title') || 'Model Catalog & Swarm Deployment'}</h1>
            <h2 className="sr-only">Qualified Intelligence Distribution</h2>
            <h2 className="sr-only">Neural Token Optimization</h2>
            <div className="flex flex-col md:flex-row justify-between gap-6">
                <div className="space-y-4 flex-1">
                    <div className="relative">
                        <Search className="w-5 h-5 absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" />
                        <input
                            type="text"
                            placeholder={i18n.t('model_store.search_placeholder')}
                            aria-label={i18n.t('model_store.search_placeholder')}
                            className="w-full bg-zinc-900/50 border border-zinc-800 rounded-xl pl-10 pr-4 py-3 text-zinc-100 focus:outline-none focus:border-green-500 transition-colors"
                            value={filter}
                            onChange={(e) => set_filter(e.target.value)}
                        />
                    </div>
                    <div className="flex flex-wrap gap-2">
                        {categories.map(c => (
                            <button
                                key={c}
                                onClick={() => set_category(c)}
                                className={`px-4 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider transition-all border ${
                                    category === c 
                                    ? 'bg-green-600 border-green-500 text-white shadow-lg shadow-green-500/20' 
                                    : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:border-zinc-700'
                                }`}
                            >
                                {c}
                            </button>
                        ))}
                    </div>
                </div>

                <div className="flex items-start gap-4">
                    <div className="bg-zinc-900/50 border border-zinc-800 rounded-xl p-3 flex items-center gap-3">
                        <div className="p-2 bg-emerald-500/10 rounded-lg">
                            <Server className="w-4 h-4 text-emerald-500" />
                        </div>
                        <div>
                            <p className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest">{i18n.t('model_store.active_nodes')}</p>
                            <p className="text-xl font-bold text-zinc-100">{nodes.length}</p>
                        </div>
                    </div>
                    <button 
                        onClick={fetch_data}
                        aria-label={i18n.t('common.refresh')}
                        className="p-3 bg-zinc-900 border border-zinc-800 rounded-xl text-zinc-400 hover:text-zinc-100 hover:border-zinc-700 transition-all group"
                    >
                        <RefreshCw className="w-5 h-5 group-active:rotate-180 transition-transform duration-500" />
                    </button>
                </div>
            </div>

            {error_msg && (
                <div className="bg-red-500/10 border border-red-500/20 p-4 rounded-xl flex items-center gap-3 text-red-500 text-sm">
                    <Info size={16} />
                    {error_msg}
                </div>
            )}

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {filtered_catalog.map(model => (
                    <div key={model.id} className="bg-zinc-900/40 border border-zinc-800/50 hover:border-green-500/30 rounded-2xl overflow-hidden flex flex-col transition-all group">
                        <div className="p-6 space-y-4 flex-1">
                            <div className="flex justify-between items-start">
                                <div className="space-y-1">
                                    <h3 className="text-lg font-bold text-zinc-100 group-hover:text-green-400 transition-colors">{model.name}</h3>
                                    <p className="text-xs font-mono text-zinc-500">{model.id}</p>
                                </div>
                                <div className="flex items-center gap-2">
                                    {model.tags.includes('vision') && (
                                        <Tooltip content={i18n.t('model_store.vision_support')}>
                                            <Zap className="w-4 h-4 text-amber-500" />
                                        </Tooltip>
                                    )}
                                    <div className="px-2 py-0.5 rounded-md bg-zinc-800 border border-zinc-700 text-[10px] font-bold text-zinc-400 uppercase">
                                        {model.provider}
                                    </div>
                                </div>
                            </div>

                            <p className="text-sm text-zinc-400 line-clamp-2 leading-relaxed">
                                {model.description || i18n.t('model_store.no_description')}
                            </p>

                            <div className="grid grid-cols-2 gap-4">
                                <div className="space-y-1">
                                    <p className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest">{i18n.t('model_store.disk_size')}</p>
                                    <div className="flex items-center gap-2 text-zinc-300">
                                        <HardDrive size={14} className="text-zinc-500" />
                                        <span className="text-sm font-medium">{model.size}</span>
                                    </div>
                                </div>
                                <div className="space-y-1">
                                    <p className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest">{i18n.t('model_store.vram_req')}</p>
                                    <div className="flex items-center gap-2 text-zinc-300">
                                        <Cpu size={14} className="text-zinc-500" />
                                        <span className="text-sm font-medium">{model.vram}</span>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div className="p-4 bg-zinc-950/50 border-t border-zinc-800/50 space-y-3">
                            <p className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-2 px-1">{i18n.t('model_store.deploy_to_node')}</p>
                            <div className="space-y-2">
                                {nodes.map(node => {
                                    const status_key = `${model.id}-${node.id}`;
                                    const status = pull_statuses[status_key] || 'idle';
                                    
                                    return (
                                        <button
                                            key={node.id}
                                            disabled={status !== 'idle'}
                                            onClick={() => handle_pull(model.id, node.id)}
                                            className={`w-full p-2.5 rounded-xl border flex items-center justify-between transition-all group/btn ${
                                                status === 'completed' 
                                                ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-500'
                                                : status === 'pulling'
                                                ? 'bg-green-500/10 border-green-500/30 text-green-500'
                                                : 'bg-zinc-900/50 border-zinc-800 text-zinc-400 hover:border-green-500/40 hover:text-zinc-100'
                                            }`}
                                        >
                                            <div className="flex items-center gap-3">
                                                <Server size={14} className={status === 'idle' ? 'text-zinc-600 group-hover/btn:text-green-500' : ''} />
                                                <span className="text-xs font-medium">{node.id}</span>
                                            </div>
                                            
                                            {status === 'pulling' ? (
                                                <RefreshCw size={14} className="animate-spin" />
                                            ) : status === 'completed' ? (
                                                <Check size={14} />
                                            ) : (
                                                <Download size={14} className="opacity-0 group-hover/btn:opacity-100 transition-opacity" />
                                            )}
                                        </button>
                                    );
                                })}
                                {nodes.length === 0 && (
                                    <p className="text-[10px] text-zinc-600 text-center py-2 italic">
                                        {i18n.t('model_store.no_active_nodes')}
                                    </p>
                                )}
                            </div>
                        </div>
                    </div>
                ))}
            </div>

            {filtered_catalog.length === 0 && (
                <Tw_Empty_State 
                    title={i18n.t('model_store.no_models_found')}
                    description={i18n.t('model_store.filter_no_results')}
                />
            )}
        </div>
    );
}


// Metadata: [Model_Store]
