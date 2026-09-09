/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Template_Preview_Modal
 * - **Primary Entrypoints**: `Template_Preview_Modal`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `Template_Store.test.tsx`
 */

import { useState } from 'react';
import { X, Code, AlertTriangle, ShieldCheck, Download, ArrowLeft, Layers, Sliders } from 'lucide-react';
import type { Template, PlaybookPreview, ModelMappingSelection, InstallOptions } from './types';
import { Playbook_List } from './Playbook_List';
import { Model_Resolver_Select } from './Model_Resolver_Select';
import { i18n } from '../../i18n';

interface TemplatePreviewModalProps {
    template: Template;
    config: Record<string, unknown> | null;
    knowledge: PlaybookPreview[] | null;
    isLoading: boolean;
    error: string | null;
    isInstalling: boolean;
    onClose: () => void;
    onInstall: (options: InstallOptions) => void;
}

export function Template_Preview_Modal({
    template,
    config,
    knowledge,
    isLoading,
    error,
    isInstalling,
    onClose,
    onInstall
}: TemplatePreviewModalProps) {
    const [modelMapping, setModelMapping] = useState<ModelMappingSelection>({
        strategy: 'system'
    });

    const defaultNamespace = template.id.replace(/[^a-zA-Z0-9_]/g, '_').toLowerCase();
    const [useNamespace, setUseNamespace] = useState(false);
    const [namespace, setNamespace] = useState(defaultNamespace);
    const [overwrite, setOverwrite] = useState(false);
    const [showAdvanced, setShowAdvanced] = useState(false);

    // Detect authored models from config if available
    let templateOriginalModel = 'gemini-pro-latest, gpt-4o-mini';
    if (config) {
        if (typeof config.model === 'string') {
            templateOriginalModel = config.model;
        } else if (Array.isArray(config.agents) && config.agents.length > 0) {
            const agentModels = (config.agents as Array<Record<string, unknown>>)
                .map(a => {
                    if (typeof a.model === 'string') return a.model;
                    const modelsObj = typeof a.models === 'object' && a.models !== null ? (a.models as Record<string, unknown>) : undefined;
                    const modelObj = modelsObj && typeof modelsObj.model === 'object' && modelsObj.model !== null ? (modelsObj.model as Record<string, unknown>) : undefined;
                    const modelId = typeof modelObj?.modelId === 'string' ? modelObj.modelId : (typeof modelsObj?.model_id === 'string' ? modelsObj.model_id : undefined);
                    return modelId;
                })
                .filter((m): m is string => Boolean(m));
            if (agentModels.length > 0) {
                templateOriginalModel = Array.from(new Set(agentModels)).join(', ');
            }
        }
    }

    const handleDeploy = () => {
        onInstall({
            modelMapping,
            overwrite,
            namespace: useNamespace ? namespace.trim() || undefined : undefined
        });
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-zinc-950/80 backdrop-blur-sm p-4 animate-in fade-in duration-200">
            <div className="bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-2xl w-full max-w-4xl max-h-[90vh] flex flex-col shadow-2xl overflow-hidden relative">
                <button
                    onClick={onClose}
                    aria-label="Close template preview"
                    className="absolute top-4 right-4 text-zinc-500 hover:text-zinc-100 transition-colors p-2 hover:bg-[color:var(--color-surface)] rounded-full z-10"
                >
                    <X size={20} />
                </button>

                {/* Modal Header */}
                <div className="p-8 border-b border-[color:var(--color-border)]/50">
                    <div className="flex items-center gap-3 mb-4">
                        <span className="text-[10px] uppercase tracking-wider font-bold text-green-400 bg-green-500/10 px-2 py-0.5 rounded">
                            {template.industry}
                        </span>
                        {template.company_size && (
                            <span className="text-[10px] uppercase tracking-wider font-bold text-success-text bg-success-bg px-2 py-0.5 rounded">
                                {template.company_size} {i18n.t('template_store.seats')}
                            </span>
                        )}
                    </div>
                    <h2 className="text-3xl font-bold text-white tracking-tight mb-2">
                        {template.name}
                    </h2>
                    <p className="text-zinc-400 max-w-2xl leading-relaxed">
                        {template.description}
                    </p>
                </div>

                {/* Modal Content - Scrollable */}
                <div className="flex-1 overflow-y-auto p-8 space-y-6 custom-scrollbar">
                    <div className="flex items-center gap-2 mb-2 text-xs font-bold text-zinc-500 uppercase tracking-widest">
                        <Code size={14} className="text-green-500" />
                        {i18n.t('template_store.header_swarm_config') || 'Swarm Configuration (swarm.json)'}
                    </div>

                    <div className="bg-zinc-950/40 border border-[color:var(--color-border)] rounded-xl overflow-hidden relative">
                        {isLoading ? (
                            <div className="py-20 flex flex-col items-center justify-center text-zinc-500">
                                <div className="w-6 h-6 rounded-full border-2 border-green-500 border-t-transparent animate-spin mb-3"></div>
                                <p className="text-xs font-mono">
                                    {i18n.t('template_store.modal_fetching')}
                                </p>
                            </div>
                        ) : error ? (
                            <div className="py-20 flex flex-col items-center justify-center text-red-400">
                                <AlertTriangle size={32} className="mb-3 opacity-50" />
                                <p className="text-xs font-mono">{error}</p>
                            </div>
                        ) : config ? (
                            <pre className="p-6 text-sm font-mono text-blue-300 overflow-x-auto whitespace-pre-wrap leading-relaxed">
                                {JSON.stringify(config, null, 2)}
                            </pre>
                        ) : (
                            <div className="py-20 flex flex-col items-center justify-center text-red-400">
                                <AlertTriangle size={32} className="mb-3 opacity-50" />
                                <p className="text-xs font-mono">
                                    {i18n.t('template_store.modal_fail_resolve')}
                                </p>
                            </div>
                        )}
                    </div>

                    {/* Model Provider Resolver Selector */}
                    <Model_Resolver_Select
                        value={modelMapping}
                        onChange={setModelMapping}
                        templateOriginalModel={templateOriginalModel}
                    />

                    {/* Advanced Collision & Namespacing Settings */}
                    <div className="border border-[color:var(--color-border)]/60 rounded-xl overflow-hidden bg-[color:var(--color-surface)]/30">
                        <button
                            type="button"
                            onClick={() => setShowAdvanced(prev => !prev)}
                            className="w-full px-5 py-3.5 flex items-center justify-between text-xs font-bold text-zinc-300 hover:text-white transition-colors"
                        >
                            <span className="flex items-center gap-2">
                                <Sliders size={14} className="text-emerald-400" />
                                Collision Resolution & Swarm Namespacing
                            </span>
                            <span className="text-[11px] font-mono text-zinc-500">
                                {showAdvanced ? 'Hide Settings' : 'Configure Prefix / Overwrite'}
                            </span>
                        </button>

                        {showAdvanced && (
                            <div className="p-5 border-t border-[color:var(--color-border)]/40 space-y-4 animate-in fade-in duration-150">
                                <label className="flex items-start gap-3 cursor-pointer">
                                    <input
                                        type="checkbox"
                                        checked={useNamespace}
                                        onChange={e => setUseNamespace(e.target.checked)}
                                        className="mt-0.5 rounded border-zinc-700 text-emerald-500 focus:ring-emerald-500 bg-zinc-900"
                                    />
                                    <div className="space-y-1">
                                        <span className="text-xs font-bold text-zinc-200 flex items-center gap-1.5">
                                            <Layers size={13} className="text-emerald-400" />
                                            Enable Swarm Namespacing Prefix
                                        </span>
                                        <p className="text-[11px] text-zinc-400 leading-normal">
                                            Prefixes agent IDs and workflows with a namespace to prevent collisions with other swarms.
                                        </p>
                                    </div>
                                </label>

                                {useNamespace && (
                                    <div className="pl-6 space-y-1.5">
                                        <label htmlFor="custom-namespace-input" className="block text-[11px] font-mono text-zinc-400">
                                            Namespace Prefix:
                                        </label>
                                        <input
                                            id="custom-namespace-input"
                                            type="text"
                                            value={namespace}
                                            onChange={e => setNamespace(e.target.value)}
                                            placeholder="e.g. field_services"
                                            className="w-full max-w-sm bg-zinc-950/70 border border-zinc-700/60 rounded-lg px-3 py-1.5 text-xs font-mono text-white placeholder-zinc-600 focus:outline-none focus:border-emerald-500"
                                        />
                                        <p className="text-[10px] text-zinc-500 font-mono">
                                            Preview: <span className="text-emerald-400">{namespace || 'prefix'}__agent.json</span>
                                        </p>
                                    </div>
                                )}

                                <label className="flex items-start gap-3 cursor-pointer pt-2 border-t border-[color:var(--color-border)]/30">
                                    <input
                                        type="checkbox"
                                        checked={overwrite}
                                        onChange={e => setOverwrite(e.target.checked)}
                                        className="mt-0.5 rounded border-zinc-700 text-emerald-500 focus:ring-emerald-500 bg-zinc-900"
                                    />
                                    <div className="space-y-0.5">
                                        <span className="text-xs font-bold text-zinc-200">
                                            Overwrite existing agents from previous installation
                                        </span>
                                        <p className="text-[11px] text-zinc-400 leading-normal">
                                            Allows updating and replacing existing agent profiles and directives cleanly.
                                        </p>
                                    </div>
                                </label>
                            </div>
                        )}
                    </div>

                    {knowledge && knowledge.length > 0 && (
                        <Playbook_List playbooks={knowledge} />
                    )}

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div className="p-4 bg-[color:var(--color-surface)]/50 border border-[color:var(--color-border)]/50 rounded-xl">
                            <h4 className="text-xs font-bold text-zinc-100 mb-2 flex items-center gap-2">
                                <ShieldCheck size={14} className="text-emerald-500" />
                                {i18n.t('template_store.security_verified_title')}
                            </h4>
                            <p className="text-[11px] text-zinc-500 leading-normal">
                                {i18n.t('template_store.security_verified_desc')}
                            </p>
                        </div>
                        <div className="p-4 bg-[color:var(--color-surface)]/50 border border-[color:var(--color-border)]/50 rounded-xl">
                            <h4 className="text-xs font-bold text-zinc-100 mb-2 flex items-center gap-2">
                                <Download size={14} className="text-green-500" />
                                {i18n.t('template_store.hot_loading_title')}
                            </h4>
                            <p className="text-[11px] text-zinc-500 leading-normal">
                                {i18n.t('template_store.hot_loading_desc')}
                            </p>
                        </div>
                    </div>
                </div>

                {/* Modal Footer */}
                <div className="p-6 bg-[color:var(--color-surface)]/30 border-t border-[color:var(--color-border)]/50 flex items-center justify-between gap-4">
                    <button
                        onClick={onClose}
                        aria-label="Close template preview"
                        className="px-6 py-2.5 rounded-lg font-bold text-sm text-zinc-400 hover:text-white hover:bg-[color:var(--color-surface)] transition-all flex items-center gap-2"
                    >
                        <ArrowLeft size={16} />
                        {i18n.t('template_store.btn_back')}
                    </button>

                    <button
                        disabled={isInstalling}
                        onClick={handleDeploy}
                        className={`px-8 py-2.5 rounded-lg font-bold text-sm flex items-center gap-2 shadow-lg transition-all ${
                            isInstalling
                                ? 'bg-emerald-600/50 text-white cursor-wait opacity-50'
                                : 'bg-emerald-600 text-white hover:bg-emerald-500 shadow-emerald-500/20'
                        }`}
                    >
                        {isInstalling ? (
                            <>
                                <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                                {i18n.t('template_store.btn_deploying')}
                            </>
                        ) : (
                            <>
                                <Download size={18} />
                                {i18n.t('template_store.btn_install')}
                            </>
                        )}
                    </button>
                </div>
            </div>
        </div>
    );
}

// Metadata: [Template_Store]
