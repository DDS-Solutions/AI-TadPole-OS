/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Playbook_List
 * - **Primary Entrypoints**: `Playbook_List`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { BookOpen } from 'lucide-react';
import type { PlaybookPreview } from './types';
import { Playbook_Card } from './Playbook_Card';
import { i18n } from '../../i18n';

interface PlaybookListProps {
    playbooks: PlaybookPreview[];
}

export function Playbook_List({ playbooks }: PlaybookListProps) {
    if (!playbooks || playbooks.length === 0) return null;

    return (
        <div className="mt-8 space-y-4">
            <div className="flex items-center gap-2 text-xs font-bold text-zinc-500 uppercase tracking-widest">
                <BookOpen size={14} className="text-green-500" />
                {i18n.t('template_store.header_playbooks')}
            </div>
            <div className="grid grid-cols-1 gap-4">
                {playbooks.map((item, idx) => (
                    <Playbook_Card key={idx} playbook={item} />
                ))}
            </div>
        </div>
    );
}
