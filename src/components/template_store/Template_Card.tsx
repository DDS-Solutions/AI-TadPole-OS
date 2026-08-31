/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Template_Card
 * - **Primary Entrypoints**: `Template_Card`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { Star, Clock, Code } from 'lucide-react';
import type { Template } from './types';
import { i18n } from '../../i18n';

interface TemplateCardProps {
    template: Template;
    isInstalling: boolean;
    onPreview: (template: Template) => void;
}

export function Template_Card({ template, isInstalling, onPreview }: TemplateCardProps) {
    return (
        <div className="sovereign-card flex flex-col hover:border-green-500/30 transition-all duration-300 group hover:-translate-y-1">
            <div className="flex justify-between items-start mb-3">
                <div>
                    <div className="flex gap-2">
                        <span className="text-[10px] font-bold uppercase tracking-wider px-2 py-1 bg-green-500/10 text-green-400 rounded-md">
                            {template.industry}
                        </span>
                        {template.company_size && (
                            <span className="text-[10px] font-bold uppercase tracking-wider px-2 py-1 bg-success-bg text-success-text rounded-md">
                                {template.company_size} {i18n.t('template_store.seats')}
                            </span>
                        )}
                    </div>
                    <h3 className="text-lg font-bold text-zinc-100 mt-2">{template.name}</h3>
                </div>
                <div className="flex items-center gap-1 text-xs text-zinc-500 font-mono">
                    <Star size={12} className="text-amber-500" />
                    {template.stars}
                </div>
            </div>

            <p className="text-sm text-zinc-400 leading-relaxed mb-6 flex-1">
                {template.description}
            </p>

            <div className="mt-auto space-y-4">
                <div className="flex justify-between items-center text-xs text-zinc-500 font-mono border-t border-[color:var(--color-border)]/50 pt-4">
                    <span>By {template.author}</span>
                    <span className="flex items-center gap-1">
                        <Clock size={12} /> {template.updatedAt}
                    </span>
                </div>

                <button
                    disabled={template.installed || isInstalling}
                    onClick={() => onPreview(template)}
                    className={`w-full py-2 rounded-lg font-bold text-sm flex items-center justify-center gap-2 transition-all ${
                        template.installed
                            ? 'bg-zinc-800 text-zinc-500 cursor-not-allowed'
                            : isInstalling
                            ? 'bg-green-600/50 text-white cursor-wait'
                            : 'bg-zinc-100 text-zinc-900 hover:bg-white'
                    }`}
                >
                    {template.installed ? (
                        <>{i18n.t('template_store.btn_installed')}</>
                    ) : isInstalling ? (
                        <>{i18n.t('template_store.btn_deploying')}</>
                    ) : (
                        <>
                            <Code size={16} />
                            {i18n.t('template_store.btn_preview')}
                        </>
                    )}
                </button>
            </div>
        </div>
    );
}
