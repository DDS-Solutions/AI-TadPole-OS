/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Financial "Burn Rate" visualization for the Ops Dashboard. 
 * Renders temporal cost trajectories using high-fidelity `recharts` for fiscal governance and resource allocation monitoring.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Null trajectory on fresh sessions (missing log data), or scale collision with large cost spikes.
 * - **Telemetry Link**: Search for `[Burn_Rate_Chart]` or `get_cost_history` in browser logs.
 */

import { useMemo, useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { useSafeLifecycle } from '../../hooks/use_safe_lifecycle';
import { i18n } from '../../i18n';

interface BurnRateChartProps {
    currentBurn: number; // USD per minute or similar metric
    height?: number;
}

/**
 * Burn_Rate_Chart
 * 
 * A cinematic "Neural Pulse" visualization of cost-velocity.
 * Renders a smooth SVG area chart representing the burn rate over time.
 */
export const Burn_Rate_Chart = ({ currentBurn, height = 120 }: BurnRateChartProps) => {
    const { isMounted } = useSafeLifecycle();
    const [history, setHistory] = useState<number[]>(() => new Array(30).fill(0));

    // Update history buffer
    useEffect(() => {
        if (!isMounted()) return;
        // Use requestAnimationFrame to decouple state update from the render effect cycle
        // to avoid "cascading renders" warnings in high-frequency pulse charts.
        const frame = requestAnimationFrame(() => {
            setHistory(prev => {
                const next = [...prev.slice(1), currentBurn];
                return next;
            });
        });
        return () => cancelAnimationFrame(frame);
    }, [currentBurn, isMounted]); // Include isMounted in dependencies as it is a stable ref-getter

    const maxBurn = useMemo(() => Math.max(...history, 0.01), [history]);

    // Generate SVG path for the line and area
    const { linePath, areaPath } = useMemo(() => {
        const points = history.map((value, i) => {
            const x = (i / (history.length - 1)) * 100;
            const y = 100 - (value / maxBurn) * 100;
            return { x, y };
        });

        if (points.length === 0) return { linePath: '', areaPath: '' };

        // Simple linear path for "Neural Core" aesthetic
        // Could be made cubic-bezier later
        const L = points.map(p => `${p.x},${p.y}`).join(' L ');
        const line = `M ${L}`;
        const area = `${line} L 100,100 L 0,100 Z`;

        return { linePath: line, areaPath: area };
    }, [history, maxBurn]);

    return (
        <div 
            className="relative w-full sovereign-card overflow-hidden bg-zinc-900/50 border-zinc-800/50 group"
            style={{ height }}
        >
            {/* Background Grid */}
            <div className="neural-grid opacity-[0.03]" />
            
            {/* Legend / Info */}
            <div className="absolute top-4 left-4 z-10 flex flex-col">
                <span className="sovereign-header-text !text-zinc-500">
                    {i18n.t('dashboard_burn.title')}
                </span>
                <span className="text-lg font-mono font-bold text-green-400">
                    {i18n.t('agent_config.label_currency_symbol')}{currentBurn.toFixed(4)}<span className="text-[10px] text-zinc-600 ml-1">{i18n.t('agent_config.label_per_min')}</span>
                </span>
            </div>

            {/* SVG Chart */}
            <svg 
                viewBox="0 0 100 100" 
                preserveAspectRatio="none" 
                className="absolute inset-0 w-full h-full"
            >
                <defs>
                    <linearGradient id="burnGradient" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor="#3b82f6" stopOpacity="0.2" />
                        <stop offset="100%" stopColor="#3b82f6" stopOpacity="0" />
                    </linearGradient>
                </defs>

                {/* Area Fill */}
                <motion.path
                    d={areaPath}
                    fill="url(#burnGradient)"
                    initial={false}
                    animate={{ d: areaPath }}
                    transition={{ type: 'spring', bounce: 0, duration: 0.5 }}
                />

                {/* The Line */}
                <motion.path
                    d={linePath}
                    fill="none"
                    stroke="#3b82f6"
                    strokeWidth="0.5"
                    strokeLinecap="round"
                    initial={false}
                    animate={{ d: linePath }}
                    transition={{ type: 'spring', bounce: 0, duration: 0.5 }}
                />

                {/* Scanning Pulse Effect */}
                <motion.div
                    className="absolute inset-y-0 w-1 bg-green-500/20 blur-sm"
                    animate={{
                        left: ['0%', '100%'],
                    }}
                    transition={{
                        duration: 3,
                        repeat: Infinity,
                        ease: "linear"
                    }}
                />
            </svg>

            {/* Right Side Info */}
            <div className="absolute top-4 right-4 z-10 text-right">
                <div className="flex items-center gap-2 justify-end">
                    <div className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
                    <span className="text-[10px] font-mono text-zinc-500 uppercase">{i18n.t('dashboard_burn.live_feed')}</span>
                </div>
                <div className="text-[10px] text-zinc-700 font-mono mt-1">
                    {i18n.t('dashboard_burn.window_depth', { count: 30 })}
                </div>
            </div>
        </div>
    );
};

// Metadata: [Burn_Rate_Chart]
