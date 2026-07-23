/**
 * @docs ARCHITECTURE:UI-Components
 * @docs OPERATIONS_MANUAL:Tracing
 * 
 * ### AI Assist Note
 * **Dual-Trace Playback Visualizer**: Side-by-side interactive trace playback component
 * inspired by Schema Harness. Displays dual timeline event streams (e.g. Model A vs Model B)
 * with Schema-style event badges ([Hypothesis], [Probe], [Surprise], [World Model Revision], [Certified]).
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Empty trace array or unsynchronized timestamp events.
 * - **Telemetry Link**: Search `[DualTracePlayback]` in UI logs.
 */

import React, { useState, useEffect, useRef, memo } from 'react';
import { Play, Pause, RotateCcw, ChevronLeft, ChevronRight, Activity, Terminal, ShieldCheck, HelpCircle, AlertTriangle, Code2 } from 'lucide-react';
import { motion } from 'framer-motion';

export interface TraceStep {
    id: string;
    step_index: number;
    role: 'hypothesis' | 'probe' | 'surprise' | 'model_revision' | 'certified' | 'log';
    title: string;
    description: string;
    code_snippet?: string;
    action_count?: number;
    timestamp: number;
}

interface DualTracePlaybackProps {
    titleA?: string;
    titleB?: string;
    traceA: TraceStep[];
    traceB: TraceStep[];
    onStepChange?: (indexA: number, indexB: number) => void;
}

// Static badge styles mapping allocated outside component definition
const BADGE_STYLES: Record<TraceStep['role'], { bg: string; icon: React.ReactNode; label: string }> = {
    hypothesis: {
        bg: 'bg-indigo-950/80 border-indigo-700 text-indigo-300',
        icon: <HelpCircle className="w-3.5 h-3.5 text-indigo-400" />,
        label: 'HYPOTHESIS'
    },
    probe: {
        bg: 'bg-amber-950/80 border-amber-700 text-amber-300',
        icon: <Activity className="w-3.5 h-3.5 text-amber-400" />,
        label: 'PROBE'
    },
    surprise: {
        bg: 'bg-rose-950/80 border-rose-700 text-rose-300',
        icon: <AlertTriangle className="w-3.5 h-3.5 text-rose-400" />,
        label: 'SURPRISE'
    },
    model_revision: {
        bg: 'bg-purple-950/80 border-purple-700 text-purple-300',
        icon: <Code2 className="w-3.5 h-3.5 text-purple-400" />,
        label: 'WORLD MODEL REVISION'
    },
    certified: {
        bg: 'bg-emerald-950/80 border-emerald-700 text-emerald-300',
        icon: <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />,
        label: 'CERTIFIED (100% PARITY)'
    },
    log: {
        bg: 'bg-zinc-800 border-zinc-700 text-zinc-300',
        icon: <Terminal className="w-3.5 h-3.5 text-zinc-400" />,
        label: 'LOG'
    }
};

interface TracePanelProps {
    title: string;
    steps: TraceStep[];
    currentIdx: number;
    onSetIndex: (idx: number) => void;
}

// Memoized individual panel to eliminate unnecessary re-renders
const TracePanel: React.FC<TracePanelProps> = memo(({ title, steps, currentIdx, onSetIndex }) => {
    const step = steps[currentIdx];
    const badge = step ? BADGE_STYLES[step.role] || BADGE_STYLES.log : null;

    return (
        <div className="flex-1 bg-zinc-950 border border-zinc-800 rounded-lg p-4 flex flex-col justify-between space-y-4 shadow-xl">
            {/* Panel Header */}
            <div className="flex justify-between items-center border-b border-zinc-800 pb-2">
                <span className="font-semibold text-sm text-zinc-200">{title}</span>
                <span className="text-xs font-mono text-zinc-500">
                    Step {steps.length > 0 ? currentIdx + 1 : 0} / {steps.length}
                </span>
            </div>

            {/* Event Card Content */}
            {step && badge ? (
                <motion.div
                    key={step.id || currentIdx}
                    initial={{ opacity: 0, y: 6 }}
                    animate={{ opacity: 1, y: 0 }}
                    className="space-y-3 min-h-[220px]"
                >
                    {/* Event Role Badge */}
                    <div className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded border text-[11px] font-mono font-medium ${badge.bg}`}>
                        {badge.icon}
                        <span>{badge.label}</span>
                    </div>

                    {/* Title & Description */}
                    <div>
                        <h4 className="font-medium text-sm text-zinc-100">{step.title}</h4>
                        <p className="text-xs text-zinc-400 mt-1 leading-relaxed">{step.description}</p>
                    </div>

                    {/* Code Snippet if present */}
                    {step.code_snippet && (
                        <div className="bg-zinc-900 border border-zinc-800 rounded p-2.5 font-mono text-xs text-emerald-400 overflow-x-auto">
                            <pre>{step.code_snippet}</pre>
                        </div>
                    )}
                </motion.div>
            ) : (
                <div className="min-h-[220px] flex items-center justify-center text-xs text-zinc-600 font-mono">
                    No telemetry event at current index
                </div>
            )}

            {/* Mini Step Navigator */}
            <div className="flex items-center justify-between pt-2 border-t border-zinc-900">
                <button
                    onClick={() => onSetIndex(Math.max(0, currentIdx - 1))}
                    disabled={currentIdx === 0}
                    className="p-1 rounded hover:bg-zinc-800 text-zinc-400 disabled:opacity-30 cursor-pointer"
                    aria-label={`Previous step for ${title}`}
                >
                    <ChevronLeft className="w-4 h-4" />
                </button>
                <input
                    type="range"
                    min={0}
                    max={Math.max(0, steps.length - 1)}
                    value={currentIdx}
                    onChange={(e) => onSetIndex(parseInt(e.target.value, 10))}
                    className="w-full mx-3 accent-emerald-500 cursor-pointer"
                    aria-label={`Timeline slider for ${title}`}
                />
                <button
                    onClick={() => onSetIndex(Math.min(steps.length - 1, currentIdx + 1))}
                    disabled={currentIdx >= steps.length - 1}
                    className="p-1 rounded hover:bg-zinc-800 text-zinc-400 disabled:opacity-30 cursor-pointer"
                    aria-label={`Next step for ${title}`}
                >
                    <ChevronRight className="w-4 h-4" />
                </button>
            </div>
        </div>
    );
});

TracePanel.displayName = 'TracePanel';

export const DualTracePlayback: React.FC<DualTracePlaybackProps> = ({
    titleA = "Model A (Baseline)",
    titleB = "Model B (World Model + Harness)",
    traceA,
    traceB,
    onStepChange
}) => {
    // Single consolidated atomic state for dual indices
    const [indices, setIndices] = useState<{ a: number; b: number }>({ a: 0, b: 0 });
    const [isPlaying, setIsPlaying] = useState<boolean>(false);
    const [playbackSpeed, setPlaybackSpeed] = useState<number>(1000); // ms per step

    // Use ref for callback to avoid resetting playback interval when callback reference changes
    const onStepChangeRef = useRef(onStepChange);
    useEffect(() => {
        onStepChangeRef.current = onStepChange;
    }, [onStepChange]);

    const maxSteps = Math.max(traceA.length, traceB.length);

    useEffect(() => {
        if (!isPlaying || maxSteps === 0) return;

        const interval = setInterval(() => {
            setIndices(prev => {
                const nextA = prev.a < traceA.length - 1 ? prev.a + 1 : prev.a;
                const nextB = prev.b < traceB.length - 1 ? prev.b + 1 : prev.b;

                if (nextA === traceA.length - 1 && nextB === traceB.length - 1) {
                    setIsPlaying(false);
                }

                // UI Telemetry trace event: [DualTracePlayback]
                if (onStepChangeRef.current) {
                    onStepChangeRef.current(nextA, nextB);
                }

                return { a: nextA, b: nextB };
            });
        }, playbackSpeed);

        return () => clearInterval(interval);
    }, [isPlaying, maxSteps, traceA.length, traceB.length, playbackSpeed]);

    const handleSetIndexA = (idx: number) => {
        setIndices(prev => ({ ...prev, a: idx }));
        if (onStepChangeRef.current) onStepChangeRef.current(idx, indices.b);
    };

    const handleSetIndexB = (idx: number) => {
        setIndices(prev => ({ ...prev, b: idx }));
        if (onStepChangeRef.current) onStepChangeRef.current(indices.a, idx);
    };

    return (
        <div className="w-full bg-zinc-900/90 border border-zinc-800 rounded-xl p-5 space-y-4 backdrop-blur-md">
            {/* Global Controls Bar */}
            <div className="flex flex-wrap justify-between items-center gap-3 border-b border-zinc-800 pb-3 font-mono text-xs text-zinc-300">
                <div className="flex items-center gap-2">
                    <button
                        onClick={() => setIsPlaying(!isPlaying)}
                        className="px-3 py-1.5 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-medium flex items-center gap-1.5 transition-colors cursor-pointer"
                        aria-label={isPlaying ? 'Pause Playback' : 'Play Synchronized Trace'}
                    >
                        {isPlaying ? <Pause className="w-3.5 h-3.5" /> : <Play className="w-3.5 h-3.5" />}
                        <span>{isPlaying ? 'Pause Playback' : 'Play Synchronized Trace'}</span>
                    </button>

                    <button
                        onClick={() => {
                            setIndices({ a: 0, b: 0 });
                            setIsPlaying(false);
                            if (onStepChangeRef.current) onStepChangeRef.current(0, 0);
                        }}
                        className="p-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 cursor-pointer"
                        title="Reset Timeline"
                        aria-label="Reset Timeline"
                    >
                        <RotateCcw className="w-3.5 h-3.5" />
                    </button>
                </div>

                <div className="flex items-center gap-2">
                    <span className="text-zinc-500">Speed:</span>
                    <button
                        onClick={() => setPlaybackSpeed(1500)}
                        className={`px-2 py-1 rounded cursor-pointer ${playbackSpeed === 1500 ? 'bg-zinc-700 text-white' : 'text-zinc-400 hover:bg-zinc-800'}`}
                        aria-label="Set playback speed to 1x"
                    >
                        1x
                    </button>
                    <button
                        onClick={() => setPlaybackSpeed(750)}
                        className={`px-2 py-1 rounded cursor-pointer ${playbackSpeed === 750 ? 'bg-zinc-700 text-white' : 'text-zinc-400 hover:bg-zinc-800'}`}
                        aria-label="Set playback speed to 2x"
                    >
                        2x
                    </button>
                    <button
                        onClick={() => setPlaybackSpeed(300)}
                        className={`px-2 py-1 rounded cursor-pointer ${playbackSpeed === 300 ? 'bg-zinc-700 text-white' : 'text-zinc-400 hover:bg-zinc-800'}`}
                        aria-label="Set playback speed to 4x"
                    >
                        4x
                    </button>
                </div>
            </div>

            {/* Side-by-Side Dual Panels */}
            <div className="flex flex-col md:flex-row gap-4">
                <TracePanel title={titleA} steps={traceA} currentIdx={indices.a} onSetIndex={handleSetIndexA} />
                <TracePanel title={titleB} steps={traceB} currentIdx={indices.b} onSetIndex={handleSetIndexB} />
            </div>
        </div>
    );
};
