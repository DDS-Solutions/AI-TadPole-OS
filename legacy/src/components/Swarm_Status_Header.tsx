/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Centralized high-density telemetry bar for global swarm limits. 
 * Aggregates Agent Count, Token Throughput, and Budget Utilization into a shared header component with tooltip deep-dives.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Metric mismatch if `use_swarm_metrics` hook lags behind WebSocket updates, or tooltip occlusion in ultra-compact viewports.
 * - **Telemetry Link**: Search for `[Swarm_Status_Header]` or `swarm_pulse` in UI tracing.
 */

import React from 'react';
import { useSwarmMetrics, type SwarmMetric } from '../hooks/use_swarm_metrics';
import { Tooltip } from './ui';

export const Swarm_Status_Header: React.FC = () => {
    const metrics = useSwarmMetrics();

    return (
        <div className="flex items-center gap-6">
            {(metrics || []).map((m: SwarmMetric, i: number) => (
                <React.Fragment key={m.label}>
                    {i > 0 && <div className="h-3 w-px bg-zinc-800" />}
                    <Tooltip key={m.label} content={m.tooltip} position="bottom">
                        <div className="flex items-center gap-2 cursor-help group">
                            <m.icon size={13} className={`${m.color} opacity-70 group-hover:opacity-100 transition-opacity`} />
                            <span className="text-xs font-bold text-zinc-500 uppercase tracking-tighter whitespace-nowrap">
                                {m.label}: <span className={`${m.color} font-mono tracking-normal text-zinc-200`}>{m.value}</span>
                            </span>
                        </div>
                    </Tooltip>
                </React.Fragment>
            ))}
        </div>
    );
};

