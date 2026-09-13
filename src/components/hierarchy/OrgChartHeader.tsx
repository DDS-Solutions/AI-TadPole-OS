/**
 * @docs ARCHITECTURE:UI
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Hierarchy / OrgChartHeader
 * - **Primary Entrypoints**: `OrgChartHeader`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import React, { memo } from 'react';
import { i18n } from '../../i18n';

const JSON_LD = JSON.stringify({
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    "name": "Tadpole OS Organization Chart",
    "description": "Visual hierarchy and reporting structure for the autonomous agent swarm. Displays command-and-control relationships and cluster allocations.",
    "author": { "@type": "Organization", "name": "Sovereign Engineering" },
    "applicationCategory": "Organization Management",
    "operatingSystem": "Tadpole OS"
});

export const OrgChartHeader: React.FC = memo(() => {
    return (
        <>
            <script
                type="application/ld+json"
                dangerouslySetInnerHTML={{ __html: JSON_LD }} // security-scan:ignore
            />
            <h1 className="sr-only">Tadpole OS Neural Command Hierarchy & Swarm Organization</h1>

            {/* Fixed Overlay Indicator */}
            <div className="fixed bottom-8 right-8 flex flex-col gap-2 items-end z-40 pointer-events-none">
                <div className="px-3 py-1 bg-zinc-950/60 border border-[color:var(--color-border)] rounded-full backdrop-blur-md flex items-center gap-2 shadow-2xl">
                    <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]" />
                    <span className="text-[10px] font-mono text-zinc-400 uppercase tracking-widest">
                        {i18n.t('org_chart.label_swarm_active')}
                    </span>
                </div>
            </div>
        </>
    );
});

OrgChartHeader.displayName = 'OrgChartHeader';
