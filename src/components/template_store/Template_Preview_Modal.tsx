/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Template_Preview_Modal]` in observability traces.
 */

import { X, Code, AlertTriangle, ShieldCheck, Download, ArrowLeft } from 'lucide-react';
import type { Template, PlaybookPreview } from './types';
import { Playbook_List } from './Playbook_List';
import { i18n } from '../../i18n';

interface TemplatePreviewModalProps {
    template: Template;
    config: Record<string, unknown> | null;
    knowledge: PlaybookPreview[] | null;
    isLoading: boolean;
    error: string | null;
    isInstalling: boolean;
    onClose: () => void;
    onInstall: () => void;
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
    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4 animate-in fade-in duration-200">
            <div className="bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-2xl w-full max-w-4xl max-h-[90vh] flex flex-col shadow-2xl overflow-hidden relative">
                <button
                    onClick={onClose}
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
                <div className="flex-1 overflow-y-auto p-8 custom-scrollbar">
                    <div className="flex items-center gap-2 mb-4 text-xs font-bold text-zinc-500 uppercase tracking-widest">
                        <Code size={14} className="text-green-500" />
                        {i18n.t('template_store.header_swarm_config') || 'Swarm Configuration (swarm.json)'}
                    </div>

                    <div className="bg-black/40 border border-[color:var(--color-border)] rounded-xl overflow-hidden relative">
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

                    {knowledge && knowledge.length > 0 && (
                        <Playbook_List playbooks={knowledge} />
                    )}

                    <div className="mt-8 grid grid-cols-1 md:grid-cols-2 gap-6">
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
                        className="px-6 py-2.5 rounded-lg font-bold text-sm text-zinc-400 hover:text-white hover:bg-[color:var(--color-surface)] transition-all flex items-center gap-2"
                    >
                        <ArrowLeft size={16} />
                        {i18n.t('template_store.btn_back')}
                    </button>

                    <button
                        disabled={isInstalling}
                        onClick={onInstall}
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

// Metadata: [Template_Preview_Modal]
