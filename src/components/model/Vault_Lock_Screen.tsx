/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Security gate for sensitive provider secrets. 
 * Orchestrates vault unlock flows, session persistence tracking, and provides a sovereign "Zero-Knowledge" aesthetic.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Unlock RPC timeout (500), local storage sync failure for session tokens, or animation freeze during high disk I/O.
 * - **Telemetry Link**: Search for `[Vault_Lock_Screen]` or `unlock_vault` in auth logs.
 */

import { Key, Lock } from 'lucide-react';
import { Tooltip } from '../ui';
import { Confirm_Dialog } from '../ui/Confirm_Dialog';
import { i18n } from '../../i18n';

interface Vault_Lock_Screen_Props {
    password_input: string;
    on_password_change: (val: string) => void;
    on_unlock: () => void;
    error: string | null;
    is_secure: boolean;
    show_reset_confirm: boolean;
    on_set_show_reset_confirm: (show: boolean) => void;
    on_reset_vault: () => void;
}

export function Vault_Lock_Screen({
    password_input,
    on_password_change,
    on_unlock,
    error,
    is_secure,
    show_reset_confirm,
    on_set_show_reset_confirm,
    on_reset_vault
}: Vault_Lock_Screen_Props) {
    return (
        <div className="h-full flex flex-col items-center justify-center space-y-8 animate-in fade-in zoom-in-95 duration-500 min-h-[500px] relative">
            <div className="neural-grid" />
            <Tooltip content={i18n.t('model_manager.vault.tooltip')} position="top">
                <div className="relative p-8 bg-zinc-900 border border-emerald-500/30 rounded-[2rem] shadow-2xl shadow-emerald-500/10 group-hover:shadow-emerald-500/20 transition-all cursor-help">
                    <Lock className="w-12 h-12 text-emerald-500/80" />
                </div>
            </Tooltip>
            <div className="text-center space-y-3 max-w-sm px-6 relative">
                <h2 className="text-3xl font-bold tracking-tight text-zinc-100 flex items-center justify-center gap-2 font-mono">
                    <span className="text-emerald-500">◈</span> {i18n.t('model_manager.vault.title')}
                </h2>
                <p className="text-zinc-500 text-[10px] font-bold tracking-[0.2em] leading-relaxed uppercase">
                    {i18n.t('model_manager.vault.desc')}
                </p>
            </div>
            <div className="w-full max-w-xs space-y-6 px-4 relative">
                <div className="space-y-3">
                    <div className="relative">
                        <Key className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-600" />
                        <label htmlFor="master-passphrase" title={i18n.t('model_manager.vault.placeholder_passphrase')} className="sr-only">{i18n.t('model_manager.vault.placeholder_passphrase')}</label>
                        <input
                            id="master-passphrase"
                            type="password"
                            value={password_input}
                            onChange={(e) => on_password_change(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && on_unlock()}
                            placeholder={i18n.t('model_manager.vault.placeholder_passphrase')}
                            className="w-full bg-zinc-950 border border-zinc-800 rounded-xl pl-10 pr-4 py-3 text-sm text-zinc-200 focus:outline-none focus:border-emerald-500/50 transition-all font-mono"
                            autoFocus
                        />
                    </div>
                    {error && <p className="text-red-500 text-[9px] text-center font-bold uppercase tracking-[0.2em]">{error}</p>}
                    
                    {!is_secure && (
                        <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-xl text-[9px] text-red-500 font-bold uppercase tracking-widest text-center">
                            {i18n.t('model_manager.vault.secure_context_required')}
                        </div>
                    )}

                    <button
                        onClick={on_unlock}
                        disabled={!is_secure}
                        className="w-full bg-zinc-100 text-zinc-900 font-bold py-3 rounded-xl hover:bg-white transition-all shadow-[0_0_20px_rgba(255,255,255,0.1)] active:scale-[0.98] text-[10px] uppercase tracking-widest disabled:opacity-50"
                    >
                        {i18n.t('model_manager.vault.btn_unlock')}
                    </button>

                    <div className="pt-4 flex justify-center">
                        <button
                            onClick={() => on_set_show_reset_confirm(true)}
                            className="text-zinc-600 hover:text-red-400 text-[9px] font-bold uppercase tracking-widest transition-colors"
                        >
                            {i18n.t('model_manager.vault.btn_reset')}
                        </button>
                    </div>
                </div>
            </div>

            <Confirm_Dialog
                is_open={show_reset_confirm}
                title={i18n.t('model_manager.vault.reset_title')}
                message={i18n.t('model_manager.vault.reset_desc')}
                confirm_label={i18n.t('model_manager.vault.btn_purge')}
                on_confirm={() => {
                    on_reset_vault();
                    on_set_show_reset_confirm(false);
                }}
                on_cancel={() => on_set_show_reset_confirm(false)}
                variant="danger"
            />
        </div>
    );
}

