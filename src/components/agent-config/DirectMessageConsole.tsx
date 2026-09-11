/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Agent-Config / DirectMessageConsole
 * - **Primary Entrypoints**: `DirectMessageConsole`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { Send } from 'lucide-react';
import { i18n } from '../../i18n';

interface DirectMessageConsoleProps {
    value: string;
    onUpdateValue: (val: string) => void;
    onSend: () => void;
    agentName: string;
    themeColor: string;
}

/**
 * Direct_Message_Console
 * Provides a dedicated input for direct neural instructions to a specific agent node.
 */
export function DirectMessageConsole({
    value,
    onUpdateValue,
    onSend,
    agentName,
    themeColor
}: DirectMessageConsoleProps) {
    return (
        <div className="p-4 bg-[color:var(--color-surface)] border-t border-[color:var(--color-border)] shrink-0">
            <div 
                className="bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-xl px-4 py-2 flex items-center gap-3 group transition-all shadow-inner"
                style={{ borderColor: value.trim() ? `${themeColor}40` : undefined }}
            >
                <div 
                    className="text-[10px] font-bold text-zinc-600 uppercase tracking-[0.2em] shrink-0 opacity-40 group-focus-within:opacity-100 transition-opacity"
                    style={{ color: value.trim() ? themeColor : undefined }}
                >
                    {i18n.t('agent_config.label_dm_to', { name: agentName })}
                </div>
                <input
                    type="text"
                    value={value}
                    onChange={(e) => onUpdateValue(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && onSend()}
                    placeholder={i18n.t('agent_config.placeholder_neural_instruction')}
                    aria-label={i18n.t('agent_config.placeholder_neural_instruction')}
                    className="flex-1 bg-transparent border-none p-0 text-sm text-zinc-300 focus:ring-0 placeholder:text-zinc-700 font-mono"
                />
                <button
                    onClick={onSend}
                    disabled={!value.trim()}
                    aria-label={i18n.t('agent_config.btn_send')}
                    className="p-1.5 transition-colors"
                    style={{ color: value.trim() ? themeColor : undefined }}
                >
                    <Send size={18} />
                </button>
            </div>
        </div>
    );
}
