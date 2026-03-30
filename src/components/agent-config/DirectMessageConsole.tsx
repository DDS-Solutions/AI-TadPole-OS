import { Send } from 'lucide-react';
import { i18n } from '../../i18n';

interface DirectMessageConsoleProps {
    value: string;
    onUpdateValue: (val: string) => void;
    onSend: () => void;
    agentName: string;
    themeColor: string;
}

export function DirectMessageConsole({
    value,
    onUpdateValue,
    onSend,
    agentName,
    themeColor
}: DirectMessageConsoleProps) {
    return (
        <div className="p-4 bg-zinc-900 border-t border-zinc-800 shrink-0">
            <div 
                className="bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2 flex items-center gap-3 group transition-all shadow-inner"
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
                    className="flex-1 bg-transparent border-none p-0 text-sm text-zinc-300 focus:ring-0 placeholder:text-zinc-700 font-mono"
                />
                <button
                    onClick={onSend}
                    disabled={!value.trim()}
                    className="p-1.5 transition-colors"
                    style={{ color: value.trim() ? themeColor : undefined }}
                >
                    <Send size={18} />
                </button>
            </div>
        </div>
    );
}
