/**
 * @docs ARCHITECTURE:UI-Pages
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Neural_Map]` in observability traces.
 */

import React from 'react';
import { KnowledgeGraph } from '../components/intelligence/KnowledgeGraph';
import { Network } from 'lucide-react';

export default function Neural_Map(): React.ReactElement {
    return (
        <div className="h-full flex flex-col gap-6 p-6 overflow-y-auto">
            {/* Header section */}
            <div className="flex flex-col gap-2 md:flex-row md:items-center md:justify-between shrink-0">
                <div>
                    <h1 className="text-2xl font-black text-white tracking-wider uppercase flex items-center gap-3">
                        <Network className="text-cyan-450 w-8 h-8" style={{ color: '#22d3ee' }} />
                        Neural Map & Knowledge Graph
                    </h1>
                    <p className="text-xs text-zinc-400 mt-1 max-w-2xl leading-relaxed">
                        Interactive topology graph illustrating the functional hierarchy and references of the Tadpole OS codebase, or the semantic relations of the Open Knowledge Format (OKF) memory store.
                        Select nodes to trace downstream dependencies (Blast Radius Analysis) and inspect system anomalies.
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

// Metadata: [Neural_Map]
