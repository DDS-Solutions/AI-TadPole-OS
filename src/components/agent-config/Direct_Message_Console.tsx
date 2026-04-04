import { Send } from 'lucide-react';
import { i18n } from '../../i18n';

interface Direct_Message_Console_Props {
    value: string;
    on_update_value: (val: string) => void;
    on_send: () => void;
    agent_name: string;
    theme_color: string;
}

/**
 * Direct_Message_Console
 * Provides a dedicated input for direct neural instructions to a specific agent node.
 */
export function Direct_Message_Console({
    value,
    on_update_value,
    on_send,
    agent_name,
    theme_color
}: Direct_Message_Console_Props) {
    return (
        <div className="p-4 bg-zinc-900 border-t border-zinc-800 shrink-0">
            <div 
                className="bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2 flex items-center gap-3 group transition-all shadow-inner"
                style={{ border_color: value.trim() ? `${theme_color}40` : undefined } as React.CSSProperties}
            >
                <div 
                    className="text-[10px] font-bold text-zinc-600 uppercase tracking-[0.2em] shrink-0 opacity-40 group-focus-within:opacity-100 transition-opacity"
                    style={{ color: value.trim() ? theme_color : undefined }}
                >
                    {i18n.t('agent_config.label_dm_to', { name: agent_name })}
                </div>
                <input
                    type="text"
                    value={value}
                    onChange={(e) => on_update_value(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && on_send()}
                    placeholder={i18n.t('agent_config.placeholder_neural_instruction')}
                    className="flex-1 bg-transparent border-none p-0 text-sm text-zinc-300 focus:ring-0 placeholder:text-zinc-700 font-mono"
                />
                <button
                    onClick={on_send}
                    disabled={!value.trim()}
                    className="p-1.5 transition-colors"
                    style={{ color: value.trim() ? theme_color : undefined }}
                >
                    <Send size={18} />
                </button>
            </div>
        </div>
    );
}
