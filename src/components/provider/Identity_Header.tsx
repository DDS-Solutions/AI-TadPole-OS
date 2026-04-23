/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Sub-module**: Handles provider branding and identity resolution. 
 * Allows for naming overrides and icon management within the administrative panel.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Provider ID collision, or naming sync delay with the `provider_store`.
 * - **Telemetry Link**: Search for `[Identity_Header]` in UI tracing.
 */

import React from 'react';
import { X } from 'lucide-react';
import { i18n } from '../../i18n';

/**
 * Identity_Header_Props
 * Defines the props for the Identity_Header component.
 */
interface Identity_Header_Props {
    /** Current provider name */
    name: string;
    /** Current provider icon */
    icon: string;
    /** Immutable provider ID */
    provider_id: string;
    /** Callback for name changes */
    on_name_change: (name: string) => void;
    /** Callback to close the panel */
    on_close: () => void;
}

/**
 * Identity_Header
 * Handles the high-fidelity header for provider configuration, including name and icon editing.
 */
export function Identity_Header({ name, icon, provider_id, on_name_change, on_close }: Identity_Header_Props): React.ReactElement {
    return (
        <div className="p-6 border-b border-zinc-800 bg-zinc-900/80 backdrop-blur-md flex items-start justify-between shrink-0 relative overflow-hidden">
            <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-emerald-500/20 to-transparent" />

            <div className="flex items-start gap-4 z-10">
                <div className="p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-xl flex items-center justify-center italic">
                    {icon}
                </div>
                <div className="space-y-1">
                    <h2 className="text-xs font-bold text-emerald-500 tracking-[0.2em] uppercase opacity-80">
                        {i18n.t('provider.panel_title')}
                    </h2>
                    <div className="flex items-center gap-2">
                        <input
                            value={name}
                            onChange={(e) => on_name_change(e.target.value)}
                            className="bg-transparent border-b-2 border-transparent p-0 font-bold text-zinc-100 text-2xl leading-tight focus:ring-0 w-full hover:bg-white/5 focus:bg-white/5 focus:border-emerald-500/50 rounded-t px-1 -ml-1 transition-all"
                            aria-label={i18n.t('provider.field_name')}
                        />
                        <span className="text-[11px] text-zinc-500 font-mono tracking-tighter bg-zinc-900 border border-white/5 px-2 py-0.5 rounded">
                            {provider_id.toUpperCase()}
                        </span>
                    </div>
                </div>
            </div>
            <button onClick={on_close} className="p-2 rounded-lg hover:bg-zinc-800 transition-colors text-zinc-500 hover:text-white z-10">
                <X size={20} />
            </button>
        </div>
    );
}

// Metadata: [Identity_Header]
