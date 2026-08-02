/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Template_Store_Banner]` in observability traces.
 */

import React from 'react';
import { Store, ExternalLink } from 'lucide-react';
import { i18n } from '../../i18n';

interface TemplateStoreBannerProps {
    on_open_store: () => void;
}

export const Template_Store_Banner: React.FC<TemplateStoreBannerProps> = ({ on_open_store }) => {
    return (
        <div className="space-y-4 pb-12">
            <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                {i18n.t('settings.header_templates')}
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8 bg-[color:var(--color-surface)] py-8 pl-8 pr-32 rounded-xl border border-[color:var(--color-border)] shadow-sm relative overflow-hidden group">
                <div className="absolute top-0 right-0 p-6 opacity-5 group-hover:opacity-10 transition-opacity pointer-events-none">
                    <Store size={80} />
                </div>

                <div className="space-y-4 z-10 relative md:col-span-2">
                    <div className="flex items-center gap-4">
                        <div className="p-3 bg-green-500/10 rounded-lg text-green-400">
                            <Store size={24} />
                        </div>
                        <div>
                            <h3 className="text-sm font-bold text-zinc-300">{i18n.t('settings.title_template_store')}</h3>
                            <p className="text-xs text-zinc-500 leading-relaxed max-w-xl">
                                {i18n.t('settings.desc_template_store')}
                            </p>
                        </div>
                        <button
                            onClick={on_open_store}
                            className="ml-auto flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-500 text-white rounded-lg font-bold text-sm transition-colors shadow-lg"
                        >
                            <ExternalLink size={16} />
                            {i18n.t('settings.btn_open_store')}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
};

// Metadata: [Template_Store_Banner]
