/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:UI-Components**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[KnowledgeGraph]` in observability traces.
 */

import React, { Component, type ReactNode } from 'react';
import { Target, Zap, RefreshCw, ArrowLeft, ArrowRight, Download, Route, ZoomIn, ZoomOut, Maximize, ShieldAlert } from 'lucide-react';
import { GraphProvider } from './knowledge_graph/GraphContext';
import { 
    useGraphDataContext, 
    useSelectionContext, 
    useNavigationContext, 
    useViewportContext, 
    useUIStateContext 
} from './knowledge_graph/graph_context_hooks';
import { GraphView } from './knowledge_graph/GraphView';
import { CognitionSidebar } from './knowledge_graph/CognitionSidebar';
import { AnomalyPanel } from './knowledge_graph/AnomalyPanel';
import { PathFinderModal } from './knowledge_graph/PathFinderModal';

// ==========================================
// 1. Error Boundary Component
// ==========================================

interface ErrorBoundaryProps {
    children: ReactNode;
    fallback: ReactNode;
}

interface ErrorBoundaryState {
    hasError: boolean;
    error: Error | null;
}

class GraphErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
    public state: ErrorBoundaryState = {
        hasError: false,
        error: null
    };

    public static getDerivedStateFromError(error: Error): ErrorBoundaryState {
        return { hasError: true, error };
    }

    public componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
        console.error('[GraphErrorBoundary] Caught symbol graph crash:', error, errorInfo);
    }

    public render() {
        if (this.state.hasError) {
            return this.props.fallback;
        }
        return this.props.children;
    }
}

const GraphCrashFallback: React.FC<{ error?: Error | null }> = ({ error }) => {
    return (
        <div className="w-full h-full min-h-[500px] flex flex-col items-center justify-center bg-zinc-950 border border-zinc-900 rounded-2xl p-8 font-mono">
            <ShieldAlert className="w-12 h-12 text-rose-500 animate-pulse mb-4" />
            <h3 className="text-xs font-black text-white uppercase tracking-[0.4em] mb-2">Symbol Graph Degraded</h3>
            <p className="text-[10px] text-zinc-500 uppercase tracking-wider mb-6 text-center max-w-md">
                An exception occurred during graph layout parsing or rendering.
            </p>
            {error && (
                <div className="text-[10px] text-rose-400 bg-rose-950/20 border border-rose-900/30 p-4 rounded-xl max-w-xl overflow-x-auto whitespace-pre-wrap">
                    {error.message || String(error)}
                </div>
            )}
            <button 
                onClick={() => window.location.reload()}
                className="mt-6 px-4 py-2 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-[10px] font-bold text-zinc-300 hover:text-white rounded-lg transition-all cursor-pointer font-mono"
            >
                Reload Context
            </button>
        </div>
    );
};

// ==========================================
// 2. Specialized Presenter HUD Components
// ==========================================

const HeaderHUD: React.FC = () => {
    const { data } = useGraphDataContext();

    return (
        <div className="absolute top-6 left-6 pointer-events-none select-none z-30">
            <div className="flex flex-col gap-2">
                <div className="flex items-center gap-3">
                    <div className="w-2.5 h-2.5 rounded-full bg-cyan-500 shadow-[0_0_15px_#22d3ee]" />
                    <h2 className="text-xs font-black text-white uppercase tracking-[0.4em]">Knowledge Graph</h2>
                </div>
                <div className="flex items-center gap-4 ml-6">
                    <div className="flex items-center gap-2">
                        <Target size={10} className="text-zinc-500" />
                        <span className="text-[9px] font-bold text-zinc-500 uppercase tracking-widest">{data?.nodes.length || 0} Symbols</span>
                    </div>
                    <div className="w-px h-2 bg-zinc-800" />
                    <div className="flex items-center gap-2">
                        <Zap size={10} className="text-zinc-500" />
                        <span className="text-[9px] font-bold text-zinc-500 uppercase tracking-widest">{data?.links.length || 0} Edges</span>
                    </div>
                </div>
            </div>
        </div>
    );
};

const NavigationHUD: React.FC = () => {
    const { historyIndex, nodeHistory, goBack, goForward } = useNavigationContext();

    return (
        <div className="flex items-center gap-2">
            <button
                disabled={historyIndex <= 0}
                onClick={goBack}
                className="p-2 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-zinc-700 disabled:hover:border-zinc-900 rounded-lg text-zinc-400 hover:text-white disabled:text-zinc-650 disabled:border-zinc-900 transition-all cursor-pointer disabled:cursor-not-allowed"
                title="Go Back (History)"
            >
                <ArrowLeft size={12} />
            </button>
            <button
                disabled={historyIndex >= nodeHistory.length - 1}
                onClick={goForward}
                className="p-2 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-zinc-700 disabled:hover:border-zinc-900 rounded-lg text-zinc-400 hover:text-white disabled:text-zinc-650 disabled:border-zinc-900 transition-all cursor-pointer disabled:cursor-not-allowed"
                title="Go Forward (History)"
            >
                <ArrowRight size={12} />
            </button>
        </div>
    );
};

const ViewportHUD: React.FC = () => {
    const { zoomIn, zoomOut, zoomFit, exportPNG } = useViewportContext();
    const { setIsPathFinderOpen } = useUIStateContext();

    return (
        <div className="absolute top-20 left-6 flex items-center gap-2 z-35 pointer-events-auto select-none">
            <NavigationHUD />
            <div className="w-px h-4 bg-zinc-800 mx-1" />
            <button
                onClick={zoomIn}
                className="p-2 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-zinc-700 rounded-lg text-zinc-400 hover:text-white transition-all cursor-pointer"
                title="Zoom In"
            >
                <ZoomIn size={12} />
            </button>
            <button
                onClick={zoomOut}
                className="p-2 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-zinc-700 rounded-lg text-zinc-400 hover:text-white transition-all cursor-pointer"
                title="Zoom Out"
            >
                <ZoomOut size={12} />
            </button>
            <button
                onClick={zoomFit}
                className="p-2 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-zinc-700 rounded-lg text-zinc-400 hover:text-white transition-all cursor-pointer"
                title="Fit to Screen"
            >
                <Maximize size={12} />
            </button>
            <div className="w-px h-4 bg-zinc-800 mx-1" />
            <button
                onClick={() => setIsPathFinderOpen(true)}
                className="flex items-center gap-1.5 px-3 py-1.5 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-cyan-500/50 hover:bg-zinc-900 rounded-lg text-[10px] font-bold text-zinc-300 hover:text-cyan-400 transition-all cursor-pointer font-mono"
                title="Open Dependency Pathfinder"
            >
                <Route size={12} />
                <span>Find Path</span>
            </button>
            <button
                onClick={exportPNG}
                className="flex items-center gap-1.5 px-3 py-1.5 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-emerald-500/50 hover:bg-zinc-900 rounded-lg text-[10px] font-bold text-zinc-300 hover:text-emerald-400 transition-all cursor-pointer font-mono"
                title="Export Canvas to PNG"
            >
                <Download size={12} />
                <span>Export PNG</span>
            </button>
        </div>
    );
};

// ==========================================
// 3. Layout Coordinator Component
// ==========================================

const KnowledgeGraphContent: React.FC = () => {
    const { loading } = useGraphDataContext();
    const { selectedNode } = useSelectionContext();

    return (
        <div className="w-full h-full relative bg-zinc-950 rounded-2xl border border-zinc-900 overflow-hidden flex flex-col items-stretch min-h-[500px]">
            {loading ? (
                <div className="absolute inset-0 flex items-center justify-center bg-zinc-950/80 backdrop-blur-sm z-50">
                    <div className="flex flex-col items-center gap-4">
                        <RefreshCw className="w-8 h-8 text-cyan-500 animate-spin" />
                        <p className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.3em]">Synthesizing Symbol Graph...</p>
                    </div>
                </div>
            ) : null}

            {!loading && (
                <div className="flex-1 min-h-0 w-full relative">
                    <GraphView />
                </div>
            )}

            {/* Header HUD */}
            <HeaderHUD />

            {/* Navigation & Utilities HUD panel */}
            {!loading && (
                <ViewportHUD />
            )}

            {/* Floating Info Panel */}
            {selectedNode && (
                <CognitionSidebar />
            )}

            {/* Legend */}
            <div className="absolute top-6 right-6 flex flex-col gap-2 bg-zinc-950/40 backdrop-blur-md p-3 rounded-xl border border-zinc-900/50 select-none pointer-events-none z-30">
                <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: '#22d3ee' }} />
                    <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">Function / Method</span>
                </div>
                <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
                    <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">Struct / Class</span>
                </div>
                <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 rounded-full bg-amber-400" />
                    <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">Trait / Interface</span>
                </div>
                <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: '#06b6d4' }} />
                    <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">Enum</span>
                </div>
            </div>

            {/* Code Anomalies Panel */}
            <AnomalyPanel />

            {/* Pathfinder Modal Dialog Overlay */}
            <PathFinderModal />
        </div>
    );
};

export const KnowledgeGraph: React.FC = () => {
    return (
        <GraphErrorBoundary fallback={<GraphCrashFallback />}>
            <GraphProvider>
                <KnowledgeGraphContent />
            </GraphProvider>
        </GraphErrorBoundary>
    );
};

// Metadata: [KnowledgeGraph]
