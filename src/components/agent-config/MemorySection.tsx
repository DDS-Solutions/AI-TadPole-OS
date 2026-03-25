import { Brain, Trash2, RefreshCw } from 'lucide-react';
import { i18n } from '../../i18n';

interface MemoryEntry {
    id: string;
    content?: string;
    text?: string;
}

interface MemorySectionProps {
    memories: MemoryEntry[];
    isLoading: boolean;
    memoryInput: string;
    themeColor: string;
    onMemoryInputChange: (val: string) => void;
    onSaveMemory: () => void;
    onDeleteMemory: (id: string) => void;
    onRefresh: () => void;
}

export function MemorySection({
    memories,
    isLoading,
    memoryInput,
    themeColor,
    onMemoryInputChange,
    onSaveMemory,
    onDeleteMemory,
    onRefresh
}: MemorySectionProps) {
    return (
        <div className="p-4 space-y-6 animate-in fade-in duration-300">
            <div className="space-y-4">
                <div className="p-4 bg-zinc-900 border border-zinc-800 rounded-2xl space-y-3">
                    <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em]">{i18n.t('agent_config.label_neural_injection')}</label>
                    <div className="flex gap-2">
                        <input
                            type="text"
                            value={memoryInput}
                            onChange={(e) => onMemoryInputChange(e.target.value)}
                            placeholder={i18n.t('agent_config.placeholder_memory_injection')}
                            className="flex-1 bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 font-mono transition-all"
                            style={{ borderLeft: `2px solid ${themeColor}40` }}
                        />
                        <button
                            onClick={onSaveMemory}
                            className="p-2 text-black rounded-xl transition-all shadow-lg"
                            style={{ backgroundColor: themeColor }}
                        >
                            <Brain size={16} />
                        </button>
                    </div>
                </div>

                <div className="space-y-2">
                    <div className="flex items-center justify-between px-2">
                        <label className="block text-[10px] font-bold text-zinc-600 uppercase tracking-[0.2em]">{i18n.t('agent_config.label_persisted_memories')}</label>
                        <button onClick={onRefresh} disabled={isLoading} className="p-1 hover:bg-zinc-800 rounded-lg transition-colors">
                            <RefreshCw size={10} className={`text-zinc-500 ${isLoading ? 'animate-spin' : ''}`} />
                        </button>
                    </div>

                    <div className="space-y-2">
                        {memories.length === 0 ? (
                            <div className="p-8 text-center bg-zinc-900/40 border border-zinc-800/50 border-dashed rounded-2xl">
                                <Brain size={24} className="mx-auto text-zinc-800 mb-2 opacity-20" />
                                <p className="text-[10px] text-zinc-600 uppercase font-bold tracking-[0.2em]">{i18n.t('agent_config.no_memories')}</p>
                            </div>
                        ) : (
                            memories.map((m) => (
                                <div key={m.id} className="group p-3 bg-zinc-900/60 border border-zinc-800 hover:border-zinc-700 rounded-xl transition-all flex items-start justify-between gap-3">
                                    <p className="text-xs text-zinc-300 leading-relaxed font-mono line-clamp-3">{m.content || m.text}</p>
                                    <button
                                        onClick={() => onDeleteMemory(m.id)}
                                        className="p-1.5 text-zinc-700 hover:text-red-500 hover:bg-red-500/10 rounded-lg transition-all opacity-0 group-hover:opacity-100"
                                    >
                                        <Trash2 size={12} />
                                    </button>
                                </div>
                            ))
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
