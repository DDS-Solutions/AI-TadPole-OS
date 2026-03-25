import { Key, Lock } from 'lucide-react';
import { Tooltip } from '../ui';
import { ConfirmDialog } from '../ui/ConfirmDialog';
import { i18n } from '../../i18n';

interface VaultLockScreenProps {
    passwordInput: string;
    onPasswordChange: (val: string) => void;
    onUnlock: () => void;
    error: string | null;
    isSecure: boolean;
    showResetConfirm: boolean;
    onSetShowResetConfirm: (show: boolean) => void;
    onResetVault: () => void;
}

export function VaultLockScreen({
    passwordInput,
    onPasswordChange,
    onUnlock,
    error,
    isSecure,
    showResetConfirm,
    onSetShowResetConfirm,
    onResetVault
}: VaultLockScreenProps) {
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
                            value={passwordInput}
                            onChange={(e) => onPasswordChange(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && onUnlock()}
                            placeholder={i18n.t('model_manager.vault.placeholder_passphrase')}
                            className="w-full bg-zinc-950 border border-zinc-800 rounded-xl pl-10 pr-4 py-3 text-sm text-zinc-200 focus:outline-none focus:border-emerald-500/50 transition-all font-mono"
                            autoFocus
                        />
                    </div>
                    {error && <p className="text-red-500 text-[9px] text-center font-bold uppercase tracking-[0.2em]">{error}</p>}
                    
                    {!isSecure && (
                        <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-xl text-[9px] text-red-500 font-bold uppercase tracking-widest text-center">
                            {i18n.t('model_manager.vault.secure_context_required')}
                        </div>
                    )}

                    <button
                        onClick={onUnlock}
                        disabled={!isSecure}
                        className="w-full bg-zinc-100 text-zinc-900 font-bold py-3 rounded-xl hover:bg-white transition-all shadow-[0_0_20px_rgba(255,255,255,0.1)] active:scale-[0.98] text-[10px] uppercase tracking-widest disabled:opacity-50"
                    >
                        {i18n.t('model_manager.vault.btn_unlock')}
                    </button>

                    <div className="pt-4 flex justify-center">
                        <button
                            onClick={() => onSetShowResetConfirm(true)}
                            className="text-zinc-600 hover:text-red-400 text-[9px] font-bold uppercase tracking-widest transition-colors"
                        >
                            {i18n.t('model_manager.vault.btn_reset')}
                        </button>
                    </div>
                </div>
            </div>

            <ConfirmDialog
                isOpen={showResetConfirm}
                title={i18n.t('model_manager.vault.reset_title')}
                message={i18n.t('model_manager.vault.reset_desc')}
                confirmLabel={i18n.t('model_manager.vault.btn_purge')}
                onConfirm={() => {
                    onResetVault();
                    onSetShowResetConfirm(false);
                }}
                onCancel={() => onSetShowResetConfirm(false)}
                variant="danger"
            />
        </div>
    );
}
