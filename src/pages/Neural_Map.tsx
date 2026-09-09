/**
 * @docs ARCHITECTURE:UI-Pages
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / Neural_Map
 * - **Primary Entrypoints**: `Neural_Map`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import React from 'react';
import { KnowledgeGraph } from '../components/intelligence/KnowledgeGraph';
import { Network } from 'lucide-react';
import { i18n } from '../i18n';

export default function Neural_Map(): React.ReactElement {
    return (
        <div className="h-full flex flex-col gap-6 p-6 overflow-y-auto">
            {/* Header section */}
            <div className="flex flex-col gap-2 md:flex-row md:items-center md:justify-between shrink-0">
                <div>
                    <h1 className="text-2xl font-black text-white tracking-wider uppercase flex items-center gap-3">
                        <Network className="text-cyan-450 w-8 h-8" style={{ color: '#22d3ee' }} />
                        {i18n.t('common.neural_map_title')}
                    </h1>
                    <p className="text-xs text-zinc-400 mt-1 max-w-2xl leading-relaxed">
                        {i18n.t('common.neural_map_desc')}
                    </p>
                </div>
            </div>

            {/* Main graph panel */}
            <div className="flex-1 min-h-[600px] sovereign-panel p-0 relative overflow-hidden">
                <KnowledgeGraph />
            </div>
        </div>
    );
}
