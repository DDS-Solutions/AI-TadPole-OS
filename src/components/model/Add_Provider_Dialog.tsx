/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Initial provider onboarding interface. 
 * Facilitates the creation of new provider configurations (OpenAI, Anthropic, etc.) and propagates initial API settings.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Duplicate provider ID collision, or initial form reset failure during modal re-mount.
 * - **Telemetry Link**: Search for `[Add_Provider_Dialog]` or `add_provider` in browser logs.
 */

import { i18n } from '../../i18n';

interface Add_Provider_Dialog_Props {
    name: string;
    icon: string;
    on_name_change: (val: string) => void;
    on_icon_change: (val: string) => void;
    on_confirm: () => void;
    on_cancel: () => void;
    error: string | null;
}

export function Add_Provider_Dialog({
    name,
    icon,
    on_name_change,
    on_icon_change,
    on_confirm,
    on_cancel,
    error
}: Add_Provider_Dialog_Props) {
    return (
        <div className="p-5 bg-zinc-900 border border-emerald-500/30 rounded-2xl flex flex-col space-y-4 animate-in fade-in zoom-in-95 shadow-xl shadow-emerald-500/5">
            <label htmlFor="new-provider-name" className="sr-only">{i18n.t('model_manager.add_provider.placeholder_name')}</label>
            <input
                id="new-provider-name"
                className="bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2 text-[10px] text-zinc-200 focus:outline-none focus:border-emerald-500/50 transition-colors font-mono"
                placeholder={i18n.t('model_manager.add_provider.placeholder_name')}
                value={name}
                onChange={e => on_name_change(e.target.value)}
                autoFocus
            />
            <label htmlFor="new-provider-icon" className="sr-only">{i18n.t('model_manager.add_node.aria_provider_icon')}</label>
            <input
                id="new-provider-icon"
                className="bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2 text-center text-sm"
                placeholder={i18n.t('model_manager.add_provider.placeholder_icon')}
                value={icon}
                onChange={e => on_icon_change(e.target.value)}
            />
            {error && <p className="text-red-500 text-[9px] text-center font-bold uppercase tracking-[0.2em]">{error}</p>}
            <div className="flex gap-2">
                <button
                    onClick={on_confirm}
                    className="flex-1 py-2 bg-emerald-500/20 text-emerald-400 border border-emerald-500/20 hover:bg-emerald-500/30 rounded-xl transition-all text-[9px] font-bold uppercase tracking-widest"
                >
                    {i18n.t('model_manager.add_provider.btn_init')}
                </button>
                <button
                    onClick={on_cancel}
                    className="flex-1 py-2 bg-zinc-800/50 text-zinc-500 hover:bg-zinc-800 rounded-xl transition-all text-[9px] font-bold uppercase tracking-widest"
                >
                    {i18n.t('model_manager.add_provider.btn_cancel')}
                </button>
            </div>
        </div>
    );
}


// Metadata: [Add_Provider_Dialog]
