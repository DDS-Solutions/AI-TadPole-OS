/**
 * @docs ARCHITECTURE:UI-Components
 * @docs OPERATIONS_MANUAL:Tracing
 * 
 * ### AI Assist Note
 * **Time-Travel Replay Bar**: Interactive timeline slider allowing users to pause
 * live WebSocket telemetry streams and scrub backwards/forwards through mission execution history.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Empty timeline state when mission has no recorded telemetry.
 * - **Telemetry Link**: Search `[TimeTravelReplay]` in UI logs.
 */

import React, { useState, useEffect } from 'react';
import { Play, Pause, RotateCcw, Clock, FastForward } from 'lucide-react';
import { motion } from 'framer-motion';
import type { BufferedTelemetryEvent } from '../../services/telemetry_buffer';

interface TimeTravelReplayBarProps {
    events: BufferedTelemetryEvent[];
    onSeek?: (index: number, event: BufferedTelemetryEvent) => void;
    on_seek?: (index: number, event: BufferedTelemetryEvent) => void;
    onLiveToggle?: (isLive: boolean) => void;
    on_live_toggle?: (is_live: boolean) => void;
    isLive?: boolean;
    is_live?: boolean;
}

export const TimeTravelReplayBar: React.FC<TimeTravelReplayBarProps> = ({
    events,
    onSeek,
    on_seek,
    onLiveToggle,
    on_live_toggle,
    isLive,
    is_live
}) => {
    const [currentIdx, setCurrentIdx] = useState<number>(0);

    const active_is_live = isLive !== undefined ? isLive : (is_live !== undefined ? is_live : true);
    const seek_fn = onSeek || on_seek;
    const live_toggle_fn = onLiveToggle || on_live_toggle;

    // 🚨 Live Stream Index Synchronization:
    // Ensures currentIdx tracks the latest event as new telemetry streams arrive.
    useEffect(() => {
        if (active_is_live && events.length > 0) {
            const lastIdx = events.length - 1;
            queueMicrotask(() => setCurrentIdx(lastIdx));
            if (seek_fn && events[lastIdx]) {
                seek_fn(lastIdx, events[lastIdx]);
            }
        }
    }, [active_is_live, events, seek_fn]);

    if (events.length === 0) return null;

    const handleSliderChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const idx = parseInt(e.target.value, 10);
        // Telemetry event log: [TimeTravelReplay]
        setCurrentIdx(idx);
        if (active_is_live && live_toggle_fn) {
            live_toggle_fn(false);
        }
        if (seek_fn && events[idx]) {
            seek_fn(idx, events[idx]);
        }
    };

    const handleResetLive = () => {
        const lastIdx = events.length - 1;
        setCurrentIdx(lastIdx);
        if (live_toggle_fn) {
            live_toggle_fn(true);
        }
        if (seek_fn && events[lastIdx]) {
            seek_fn(lastIdx, events[lastIdx]);
        }
    };

    const currentEvent = events[currentIdx] || events[events.length - 1];
    const timestampStr = currentEvent ? new Date(currentEvent.timestamp).toLocaleTimeString() : '--:--:--';

    return (
        <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="w-full bg-zinc-900/90 border border-zinc-800 rounded-lg p-3 flex flex-col md:flex-row items-center gap-4 text-xs font-mono text-zinc-300 backdrop-blur-md shadow-xl"
        >
            {/* Control Buttons */}
            <div className="flex items-center gap-2 min-w-max">
                <button
                    onClick={() => live_toggle_fn && live_toggle_fn(!active_is_live)}
                    className="p-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-white transition-colors cursor-pointer"
                    title={active_is_live ? 'Pause Stream' : 'Resume Live'}
                >
                    {active_is_live ? <Pause className="w-4 h-4 text-amber-400" /> : <Play className="w-4 h-4 text-emerald-400" />}
                </button>

                <button
                    onClick={handleResetLive}
                    className="p-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-white transition-colors cursor-pointer"
                    title="Jump to Present"
                >
                    <RotateCcw className="w-4 h-4 text-sky-400" />
                </button>

                <span className="flex items-center gap-1 text-zinc-400 font-semibold">
                    <Clock className="w-3.5 h-3.5 text-zinc-500" />
                    {timestampStr}
                </span>
            </div>

            {/* Timeline Slider */}
            <div className="flex-1 w-full flex items-center gap-3">
                <input
                    type="range"
                    aria-label="Telemetry timeline seek"
                    min={0}
                    max={events.length - 1}
                    value={currentIdx}
                    onChange={handleSliderChange}
                    className="w-full accent-emerald-500 bg-zinc-800 h-1.5 rounded-lg appearance-none cursor-pointer"
                />

                <span className="text-[10px] text-zinc-400 font-mono whitespace-nowrap">
                    {currentIdx + 1} / {events.length} events
                </span>
            </div>

            {/* Live Indicator */}
            <div className="min-w-max flex items-center gap-2">
                {active_is_live ? (
                    <span className="flex items-center gap-1.5 text-emerald-400 font-bold px-2 py-0.5 rounded bg-emerald-950/80 border border-emerald-800/60 uppercase tracking-widest text-[10px]">
                        <span className="w-2 h-2 rounded-full bg-emerald-500 animate-ping" />
                        LIVE
                    </span>
                ) : (
                    <span className="flex items-center gap-1 text-amber-400 font-bold px-2 py-0.5 rounded bg-amber-950/80 border border-amber-800/60 uppercase tracking-widest text-[10px]">
                        <FastForward className="w-3 h-3" />
                        REPLAY MODE
                    </span>
                )}
            </div>
        </motion.div>
    );
};

export const Time_Travel_Replay_Bar = TimeTravelReplayBar;
// Metadata: [Time_Travel_Replay_Bar]
