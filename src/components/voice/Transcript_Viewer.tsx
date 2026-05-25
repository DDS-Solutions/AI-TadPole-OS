/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Transcript_Viewer]` in observability traces.
 */

import { useRef, useEffect } from 'react';
import { BarChart3 } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Transcript_Entry } from '../../hooks/useStandups';

interface TranscriptViewerProps {
    is_live: boolean;
    transcript_history: Transcript_Entry[];
    active_speaker: string | null;
}

export function Transcript_Viewer({
    is_live,
    transcript_history,
    active_speaker
}: TranscriptViewerProps) {
    const transcript_end_ref = useRef<HTMLDivElement>(null);

    useEffect(() => {
        transcript_end_ref.current?.scrollIntoView({ behavior: 'smooth' });
    }, [transcript_history]);

    return (
        <div className="bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-xl flex flex-col overflow-hidden">
            <Tooltip content={i18n.t('standups.tooltip_transcript')} position="left">
                <div className="p-4 border-b border-[color:var(--color-border)] bg-[color:var(--color-surface)] flex items-center justify-between cursor-help">
                    <h3 className="font-bold text-zinc-400 text-sm flex items-center gap-2">
                        <BarChart3 size={16} /> {i18n.t('standups.header_transcript')}
                    </h3>
                    <div className="flex items-center gap-2">
                        {is_live && <span className="w-2 h-2 bg-red-500 rounded-full animate-pulse"></span>}
                        <span className="text-xs text-zinc-500 font-mono">{is_live ? i18n.t('standups.status_rec') : i18n.t('standups.status_idle')}</span>
                    </div>
                </div>
            </Tooltip>

            <div className="flex-1 overflow-y-auto p-4 space-y-4 scroll-smooth">
                {transcript_history.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center text-zinc-600 italic text-sm">
                        {i18n.t('standups.empty_transcript')}
                    </div>
                ) : (
                    transcript_history.map((entry, i) => {
                        return (
                            <div key={i} className="flex gap-3 animate-in fade-in slide-in-from-bottom-2 duration-300">
                                <div className={`w-8 h-8 rounded shrink-0 flex items-center justify-center text-xs font-bold transition-colors ${
                                    entry.source === 'User' ? 'bg-emerald-900/50 text-emerald-400' :
                                    entry.source === 'Agent' ? 'bg-blue-900/50 text-green-400' :
                                    entry.source === 'System' ? 'bg-zinc-800 text-zinc-500' : 'bg-zinc-800 text-zinc-400'
                                }`}>
                                    {entry.speaker.substring(0, 1)}
                                </div>
                                <div className="flex-1 min-w-0">
                                    <div className="text-[10px] font-bold text-zinc-500 mb-0.5 uppercase tracking-wider">{entry.speaker}</div>
                                    <p className="text-sm text-zinc-200 leading-relaxed break-words">{entry.text}</p>
                                </div>
                            </div>
                        )
                    })
                )}
                {active_speaker && (
                    <div className="flex gap-2 items-center text-zinc-500 text-xs pl-11 animate-pulse">
                        {i18n.t('standups.label_speaking', { name: active_speaker })}
                    </div>
                )}
                <div ref={transcript_end_ref} />
            </div>
        </div>
    );
}

// Metadata: [Transcript_Viewer]
