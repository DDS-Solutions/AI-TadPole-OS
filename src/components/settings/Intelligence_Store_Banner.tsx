/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Intelligence_Store_Banner]` in observability traces.
 */

import React from 'react';
import { Store } from 'lucide-react';
import { i18n } from '../../i18n';

interface IntelligenceStoreBannerProps {
    on_browse: () => void;
    is_scanning?: boolean;
}

export const Intelligence_Store_Banner: React.FC<IntelligenceStoreBannerProps> = ({ on_browse, is_scanning }) => {
    return (
        <div className="space-y-4">
            <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                {i18n.t('settings.header_model_store', { defaultValue: 'Model Marketplace' })}
            </h2>
            <div className="bg-[color:var(--color-surface)]/50 border border-[color:var(--color-border)] rounded-3xl p-6 relative overflow-hidden group">
                <div className="absolute top-0 right-0 p-8 opacity-[0.03] group-hover:opacity-[0.1] transition-opacity">
                    <Store size={120} />
                </div>
                <div className="flex items-start justify-between relative z-10">
                    <div className="space-y-2">
                        <div className="flex items-center gap-2">
                            <h3 className="text-xl font-black text-zinc-100 uppercase tracking-tight">{i18n.t('settings.title_intelligence_store')}</h3>
                            <span className="bg-green-500 text-[9px] font-black text-white px-2 py-0.5 rounded-full uppercase tracking-widest animate-pulse">{i18n.t('settings.badge_alpha')}</span>
                        </div>
                        <p className="text-zinc-500 text-xs max-w-md italic leading-relaxed">
                            Deploy high-fidelity LLMs and vision models directly to your local Swarm nodes with one-click orchestration. VRAM-aware and privacy-first.
                        </p>
                        <div className="flex items-center gap-4 mt-6">
                            <button
                                onClick={on_browse}
                                className="flex items-center gap-2 px-6 py-2.5 bg-green-600 hover:bg-green-500 text-white rounded-2xl text-[10px] font-bold uppercase tracking-widest transition-all shadow-lg shadow-green-500/20 active:scale-95"
                            >
                                <Store size={14} /> {i18n.t('settings.btn_browse_models', { defaultValue: 'Browse Intelligence' })}
                            </button>
                            <div className="h-4 w-px bg-zinc-800" />
                            <div className="text-[10px] text-zinc-500 font-mono uppercase tracking-tighter">
                                {is_scanning ? 'Scanning Cluster...' : 'Swarm Discovery Active'}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

// Metadata: [Intelligence_Store_Banner]
