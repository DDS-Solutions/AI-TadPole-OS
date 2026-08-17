/**
 * @docs ARCHITECTURE:UI
 * 
 * ### AI Assist Note
 * Dynamic SVG connector curves linking cluster and agent nodes in the Org Chart hierarchy.
 * Computes bezier branching lines from Nexus coordinator to departmental chains.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Broken SVG path calculation or layout offset mismatch.
 * - **Telemetry Link**: Search `[HierarchyConnectionLines]` in UI logs.
 */

import React, { memo } from 'react';

interface HierarchyConnectionLinesProps {
    branchPath: string;
    xCoords: number[];
    hasActiveChains: boolean;
}

export const HierarchyConnectionLines: React.FC<HierarchyConnectionLinesProps> = memo(({
    branchPath,
    xCoords,
    hasActiveChains,
}) => {
    if (!branchPath || xCoords.length === 0) return null;

    return (
        <svg
            aria-hidden="true"
            className="absolute top-[100%] left-1/2 -translate-x-1/2 w-[1000px] h-[52px] overflow-visible pointer-events-none"
        >
            <defs>
                <linearGradient id="neural-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" stopColor="rgba(113, 113, 122, 0.1)" />
                    <stop offset="50%" stopColor="rgba(16, 185, 129, 0.4)" />
                    <stop offset="100%" stopColor="rgba(113, 113, 122, 0.1)" />
                </linearGradient>
            </defs>
            <path
                d={branchPath}
                fill="none"
                stroke="rgba(113, 113, 122, 0.2)"
                strokeWidth="1.5"
                className="transition-all duration-1000"
            />
            {hasActiveChains && (
                <path
                    d={branchPath}
                    fill="none"
                    stroke="url(#neural-gradient)"
                    strokeWidth="2"
                    strokeDasharray="10 20"
                    className="animate-[dash_3s_linear_infinite]"
                />
            )}
            {xCoords.map(x => (
                <circle key={x} cx={x} cy={x === 500 ? 20 : 30} r="1.5" fill="rgba(113, 113, 122, 0.4)" />
            ))}
        </svg>
    );
});

HierarchyConnectionLines.displayName = 'HierarchyConnectionLines';

// Metadata: [HierarchyConnectionLines]
