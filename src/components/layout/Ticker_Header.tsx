/**
 * @docs ARCHITECTURE:Interface
 * @docs OPERATIONS_MANUAL:Navigation
 * 
 * ### AI Assist Note
 * **UI Shell Component**: Unified telemetry ticker header. 
 * Replaces the legacy Swarm_Status_Header with a high-density, infinite scrolling marquee.
 * Implements strict horizontal overflow containment using framer-motion.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Null metrics (useSwarmMetrics safety), animation stall (check motion initialization), or wrapping (check whitespace-nowrap).
 * - **Telemetry Link**: Search for `[Ticker_Header]` or `motion.div` in UI tracing.
 */

import React from 'react';
import { motion } from 'framer-motion';
import { useSwarmMetrics, type SwarmMetric } from '../../hooks/use_swarm_metrics';
import { Tooltip } from '../ui';

/**
 * Ticker_Header
 * Primary telemetry scrolling ticker for the OS header.
 * 
 * Features:
 * - Infinite horizontal scroll (0% -> -33.33%)
 * - Seamless looping with triple-buffered content
 * - Strict single-row layout without wrapping
 */
export const Ticker_Header: React.FC = () => {
    const raw_metrics = useSwarmMetrics();
    const metrics = Array.isArray(raw_metrics) ? raw_metrics : [];

    const content = (
        <div className="inline-flex items-center gap-12 whitespace-nowrap flex-nowrap pr-12 select-none h-full">
            {metrics.map((m: SwarmMetric, i: number) => (
                <div key={m.label} className="inline-flex items-center gap-4 whitespace-nowrap flex-nowrap">
                    {i > 0 && <div className="h-6 w-px bg-zinc-800/80" />}
                    <Tooltip content={m.tooltip} position="bottom">
                        <div className="inline-flex items-center gap-2 cursor-help group whitespace-nowrap flex-nowrap">
                            <m.icon size={12} className={`${m.color} opacity-60 group-hover:opacity-100 transition-opacity`} />
                            <span className="text-[9px] font-bold text-zinc-500 uppercase tracking-tighter whitespace-nowrap">
                                {m.label}
                            </span>
                            <span className="text-[10px] font-mono text-zinc-300 font-black tabular-nums whitespace-nowrap">
                                {m.value}
                            </span>
                        </div>
                    </Tooltip>
                </div>
            ))}
        </div>
    );

    return (
        <div className="w-full relative flex items-center overflow-hidden pointer-events-auto border-b border-zinc-800/80 bg-zinc-950 h-8 shrink-0 z-20">
            {/* Edge Fading for high-fidelity look */}
            <div className="absolute inset-y-0 left-0 w-16 bg-gradient-to-r from-zinc-950 via-zinc-950/90 to-transparent z-10 pointer-events-none" />
            <div className="absolute inset-y-0 right-0 w-16 bg-gradient-to-l from-zinc-950 via-zinc-950/90 to-transparent z-10 pointer-events-none" />

            <motion.div
                className="inline-flex items-center whitespace-nowrap flex-nowrap h-full"
                initial={{ x: "0%" }}
                animate={{ x: "-33.333%" }}
                transition={{
                    duration: 45,
                    ease: "linear",
                    repeat: Infinity
                }}
            >
                {/* Triple Loop for seamless continuity */}
                {content}
                {content}
                {content}
            </motion.div>
        </div>
    );
};
