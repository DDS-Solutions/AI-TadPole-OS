/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / AgentConfig / Footer
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { i18n } from '../../i18n';

interface AgentConfigFooterProps {
  showPromote: boolean;
  newRoleName: string;
  isSaving: boolean;
  isNew: boolean;
  safeThemeColor: string;
  onTogglePromote: () => void;
  onNewRoleNameChange: (val: string) => void;
  onPromote: () => void;
  onSave: () => void;
}

export function AgentConfigFooter({
  showPromote,
  newRoleName,
  isSaving,
  isNew,
  safeThemeColor,
  onTogglePromote,
  onNewRoleNameChange,
  onPromote,
  onSave,
}: AgentConfigFooterProps) {
  return (
    <div className="p-6 bg-[color:var(--color-surface)]/50 border-t border-[color:var(--color-border)] flex items-center justify-between gap-4 shrink-0 z-10">
      <button
        onClick={onTogglePromote}
        className="px-4 py-2 rounded-xl border border-[color:var(--color-border)] text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] hover:border-zinc-700 hover:text-zinc-300 transition-all"
      >
        {showPromote
          ? i18n.t('agent_config.btn_cancel')
          : i18n.t('agent_config.btn_save_as_role')}
      </button>

      <div className="flex items-center gap-3">
        {showPromote && (
          <div className="flex items-center gap-2 animate-in slide-in-from-right-4 duration-300">
            <input
              placeholder={i18n.t('agent_config.placeholder_role_name')}
              aria-label={i18n.t('agent_config.placeholder_role_name')}
              value={newRoleName}
              onChange={(e) => onNewRoleNameChange(e.target.value)}
              className="bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg px-3 py-1.5 text-xs text-zinc-300 focus:outline-none focus:border-emerald-500/50 w-40"
            />
            <button
              onClick={onPromote}
              className="px-4 py-1.5 bg-emerald-500/10 text-emerald-500 border border-emerald-500/20 rounded-lg text-[10px] font-bold uppercase tracking-[0.2em] hover:bg-emerald-500/20 transition-all"
            >
              {i18n.t('agent_config.btn_confirm')}
            </button>
          </div>
        )}

        <button
          onClick={onSave}
          disabled={isSaving}
          className="px-8 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-[0.2em] transition-all disabled:opacity-50 disabled:grayscale flex items-center gap-2 shadow-lg"
          style={{
            backgroundColor: safeThemeColor,
            color: 'black',
            boxShadow: `0 0 20px ${safeThemeColor}40`,
          }}
        >
          {isSaving ? (
            <>
              <div className="w-3 h-3 border-2 border-black/30 border-t-black rounded-full animate-spin" />
              {i18n.t('agent_config.btn_saving')}
            </>
          ) : isNew ? (
            i18n.t('agent_config.btn_create_agent')
          ) : (
            i18n.t('agent_config.btn_save_config')
          )}
        </button>
      </div>
    </div>
  );
}

