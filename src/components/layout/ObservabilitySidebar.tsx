import React from 'react';
import { clsx } from 'clsx';
import { useTabStore } from '../../stores/tabStore';
import { LineageStream } from '../LineageStream';
import { NeuralWaterfall } from '../NeuralWaterfall';
import { SystemLog } from '../dashboard/SystemLog';

interface ObservabilitySidebarProps {
    isDetachedContext?: boolean;
    className?: string;
}

/**
 * ObservabilitySidebar
 * 
 * A unified sidebar containing the core observability sectors of Tadpole OS.
 * It intelligently hides sectors that are globally detached into their own windows.
 */
export const ObservabilitySidebar: React.FC<ObservabilitySidebarProps> = ({ 
    isDetachedContext = false,
    className 
}) => {
    const { 
        isSystemLogDetached, 
        isTraceStreamDetached, 
        isLineageStreamDetached 
    } = useTabStore();

    // If all sectors are detached globally, we don't need to render the sidebar container
    const isAllDetached = isSystemLogDetached && isTraceStreamDetached && isLineageStreamDetached;
    if (isAllDetached && !isDetachedContext) return null;

    return (
        <div className={clsx(
            "flex flex-col gap-4 relative pb-10",
            isDetachedContext ? "w-[400px] border-l border-zinc-900 bg-black/20" : "w-[400px]",
            className
        )}>
            {/* Sector Stack: Lineage -> Waterfall -> Log */}
            {!isLineageStreamDetached && <LineageStream />}
            {!isTraceStreamDetached && <NeuralWaterfall />}
            {!isSystemLogDetached && <SystemLog />}

            {/* Empty State / Legend if nothing is here but context is forced */}
            {!(!isLineageStreamDetached || !isTraceStreamDetached || !isSystemLogDetached) && (
                <div className="flex-1 flex flex-col items-center justify-center text-zinc-800 p-8 text-center">
                    <div className="w-12 h-12 rounded-full border border-zinc-900 flex items-center justify-center mb-4">
                        <div className="w-2 h-2 rounded-full bg-zinc-900 animate-pulse" />
                    </div>
                    <p className="text-[10px] uppercase tracking-[0.2em] font-bold">Monitor Channels Detached</p>
                </div>
            )}
        </div>
    );
};
