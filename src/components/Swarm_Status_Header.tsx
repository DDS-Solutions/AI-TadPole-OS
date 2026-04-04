import React from 'react';
import { use_swarm_metrics } from '../hooks/use_swarm_metrics';
import { Tooltip } from './ui';

/**
 * Swarm_Status_Header
 * A centralized, single-line telemetry bar that displays swarm limits and metrics.
 * Integrated across all major OS pages for consistent oversight.
 * Refactored for strict snake_case compliance for backend parity.
 */
export const Swarm_Status_Header: React.FC = () => {
    const metrics = use_swarm_metrics();

    return (
        <div className="flex items-center gap-6">
            {metrics.map((m, i) => (
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
