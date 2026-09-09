/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Playbook_Card
 * - **Primary Entrypoints**: `Playbook_Card`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { ExternalLink } from 'lucide-react';
import type { PlaybookPreview } from './types';

interface PlaybookCardProps {
    playbook: PlaybookPreview;
}

export function Playbook_Card({ playbook }: PlaybookCardProps) {
    return (
        <div className="p-4 bg-[color:var(--color-surface)]/50 border border-[color:var(--color-border)]/50 rounded-xl hover:border-green-500/20 transition-all duration-300">
            <div className="flex justify-between items-start mb-2">
                <div className="flex items-center gap-2 flex-wrap">
                    <h4 className="text-sm font-bold text-zinc-100">
                        {playbook.title || 'Untitled Playbook'}
                    </h4>
                    {playbook.concept_type && (
                        <span className="text-[9px] uppercase tracking-wider font-bold text-emerald-400 bg-emerald-500/10 px-1.5 py-0.5 rounded">
                            {playbook.concept_type}
                        </span>
                    )}
                    {playbook.topic && (
                        <span className="text-[9px] uppercase tracking-wider font-bold text-blue-400 bg-blue-500/10 px-1.5 py-0.5 rounded">
                            {playbook.topic}
                        </span>
                    )}
                </div>
                {playbook.resource_uri && (
                    <a
                        href={playbook.resource_uri}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-zinc-500 hover:text-green-400 transition-colors text-xs font-mono flex items-center gap-1 flex-shrink-0"
                    >
                        <span>SOP</span>
                        <ExternalLink size={12} />
                    </a>
                )}
            </div>
            {playbook.description && (
                <p className="text-xs text-zinc-400 leading-relaxed">{playbook.description}</p>
            )}
            {playbook.tags && (
                <div className="flex flex-wrap gap-1.5 mt-2">
                    {playbook.tags.split(',').map((tag, tIdx) => (
                        <span
                            key={tIdx}
                            className="text-[9px] font-mono text-zinc-500 bg-zinc-800/50 px-1.5 py-0.5 rounded"
                        >
                            #{tag.trim()}
                        </span>
                    ))}
                </div>
            )}
        </div>
    );
}
