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
import { Target, Zap, RefreshCw, ArrowLeft, ArrowRight, Download, Route, ZoomIn, ZoomOut, Maximize, ShieldAlert, AlertTriangle } from 'lucide-react';
import { GraphProvider } from './knowledge_graph/GraphContext';
import { 
    useGraphDataContext, 
    useSelectionContext, 
    useNavigationContext, 
    useViewportContext, 
    useUIStateContext 
} from './knowledge_graph/graph_context_hooks';
import { i18n } from '../../i18n';
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
            <h3 className="text-xs font-black text-white uppercase tracking-[0.4em] mb-2">{i18n.t('knowledge_graph.error_degraded_title')}</h3>
            <p className="text-[10px] text-zinc-500 uppercase tracking-wider mb-6 text-center max-w-md">
                {i18n.t('knowledge_graph.error_degraded_desc')}
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
                {i18n.t('knowledge_graph.btn_reload_context')}
            </button>
        </div>
    );
};

// ==========================================
// 2. Specialized Presenter HUD Components
// ==========================================

const HeaderHUD: React.FC = () => {
    const { data } = useGraphDataContext();
    const { viewMode } = useUIStateContext();

    return (
        <div className="absolute top-6 left-6 pointer-events-none select-none z-30">
            <div className="flex flex-col gap-2">
                <div className="flex items-center gap-3">
                    <div className="w-2.5 h-2.5 rounded-full bg-cyan-500 shadow-[0_0_15px_#22d3ee]" />
                    <h2 className="text-xs font-black text-white uppercase tracking-[0.4em]">
                        {viewMode === 'okf' ? i18n.t('knowledge_graph.title_okf') : i18n.t('knowledge_graph.title_symbols')}
                    </h2>
                </div>
                <div className="flex items-center gap-4 ml-6">
                    <div className="flex items-center gap-2">
                        <Target size={10} className="text-zinc-500" />
                        <span className="text-[9px] font-bold text-zinc-500 uppercase tracking-widest">
                            {data?.nodes.length || 0} {viewMode === 'okf' ? i18n.t('knowledge_graph.label_concepts') : i18n.t('knowledge_graph.label_symbols')}
                        </span>
                    </div>
                    <div className="w-px h-2 bg-zinc-800" />
                    <div className="flex items-center gap-2">
                        <Zap size={10} className="text-zinc-500" />
                        <span className="text-[9px] font-bold text-zinc-500 uppercase tracking-widest">{data?.links.length || 0} {i18n.t('knowledge_graph.label_edges')}</span>
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
                title={i18n.t('knowledge_graph.tooltip_back')}
            >
                <ArrowLeft size={12} />
            </button>
            <button
                disabled={historyIndex >= nodeHistory.length - 1}
                onClick={goForward}
                className="p-2 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-zinc-700 disabled:hover:border-zinc-900 rounded-lg text-zinc-400 hover:text-white disabled:text-zinc-650 disabled:border-zinc-900 transition-all cursor-pointer disabled:cursor-not-allowed"
                title={i18n.t('knowledge_graph.tooltip_forward')}
            >
                <ArrowRight size={12} />
            </button>
        </div>
    );
};

const ViewportHUD: React.FC = () => {
    const { data } = useGraphDataContext();
    const { zoomIn, zoomOut, zoomFit, exportPNG } = useViewportContext();
    const { setIsPathFinderOpen, viewMode, setViewMode, isAnomalyPanelOpen, setIsAnomalyPanelOpen } = useUIStateContext();

    const rawAnomalies = data?.anomalies;
    const anomalyCount = Array.isArray(rawAnomalies)
        ? rawAnomalies.filter((a): a is string => typeof a === 'string' && a.trim().length > 0).length
        : 0;

    return (
        <div className="absolute top-20 left-6 flex items-center gap-2 z-35 pointer-events-auto select-none">
            <NavigationHUD />
            <div className="w-px h-4 bg-zinc-800 mx-1" />
            
            {/* View Mode Toggle */}
            <div className="flex bg-zinc-900/80 border border-zinc-800 rounded-lg p-0.5 backdrop-blur-md">
                <button
                    onClick={() => setViewMode('symbols')}
                    className={`px-2.5 py-1 text-[9px] font-black uppercase tracking-wider rounded-md transition-all cursor-pointer ${
                        viewMode === 'symbols'
                            ? 'bg-zinc-850 text-cyan-400 shadow-[0_0_10px_rgba(34,211,238,0.15)] border border-zinc-750'
                            : 'text-zinc-500 hover:text-zinc-300'
                    }`}
                >
                    {i18n.t('knowledge_graph.btn_symbols_mode')}
                </button>
                <button
                    onClick={() => setViewMode('okf')}
                    className={`px-2.5 py-1 text-[9px] font-black uppercase tracking-wider rounded-md transition-all cursor-pointer ${
                        viewMode === 'okf'
                            ? 'bg-zinc-850 text-cyan-400 shadow-[0_0_10px_rgba(34,211,238,0.15)] border border-zinc-750'
                            : 'text-zinc-500 hover:text-zinc-300'
                    }`}
                >
                    {i18n.t('knowledge_graph.btn_knowledge_mode')}
                </button>
            </div>
            
            <div className="w-px h-4 bg-zinc-800 mx-1" />
            <button
                onClick={zoomIn}
                className="p-2 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-zinc-700 rounded-lg text-zinc-400 hover:text-white transition-all cursor-pointer"
                title={i18n.t('knowledge_graph.tooltip_zoom_in')}
            >
                <ZoomIn size={12} />
            </button>
            <button
                onClick={zoomOut}
                className="p-2 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-zinc-700 rounded-lg text-zinc-400 hover:text-white transition-all cursor-pointer"
                title={i18n.t('knowledge_graph.tooltip_zoom_out')}
            >
                <ZoomOut size={12} />
            </button>
            <button
                onClick={zoomFit}
                className="p-2 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-zinc-700 rounded-lg text-zinc-400 hover:text-white transition-all cursor-pointer"
                title={i18n.t('knowledge_graph.tooltip_fit')}
            >
                <Maximize size={12} />
            </button>
            <div className="w-px h-4 bg-zinc-800 mx-1" />
            <button
                onClick={() => setIsPathFinderOpen(true)}
                className="flex items-center gap-1.5 px-3 py-1.5 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-cyan-500/50 hover:bg-zinc-900 rounded-lg text-[10px] font-bold text-zinc-300 hover:text-cyan-400 transition-all cursor-pointer font-mono"
                title={i18n.t('knowledge_graph.tooltip_pathfinder')}
            >
                <Route size={12} />
                <span>{i18n.t('knowledge_graph.btn_find_path')}</span>
            </button>
            <button
                onClick={exportPNG}
                className="flex items-center gap-1.5 px-3 py-1.5 bg-zinc-900/80 backdrop-blur-md border border-zinc-800 hover:border-emerald-500/50 hover:bg-zinc-900 rounded-lg text-[10px] font-bold text-zinc-300 hover:text-emerald-400 transition-all cursor-pointer font-mono"
                title={i18n.t('knowledge_graph.tooltip_export_png')}
            >
                <Download size={12} />
                <span>{i18n.t('knowledge_graph.btn_export_png')}</span>
            </button>
            <button
                onClick={() => setIsAnomalyPanelOpen((prev) => !prev)}
                className={`flex items-center gap-1.5 px-3 py-1.5 backdrop-blur-md border rounded-lg text-[10px] font-bold transition-all cursor-pointer font-mono ${
                    isAnomalyPanelOpen
                        ? 'bg-amber-500/20 border-amber-500/80 text-amber-400 shadow-[0_0_12px_rgba(245,158,11,0.2)]'
                        : 'bg-zinc-900/80 border-zinc-800 hover:border-amber-500/50 hover:bg-zinc-900 text-zinc-300 hover:text-amber-400'
                }`}
                title={i18n.t('knowledge_graph.tooltip_anomalies', { defaultValue: 'Toggle Anomalies Panel' })}
            >
                <AlertTriangle size={12} className="text-amber-500 shrink-0" />
                <span>{`Anomalies (${anomalyCount})`}</span>
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
    const { viewMode } = useUIStateContext();

    return (
        <div className="w-full h-full relative bg-zinc-950 rounded-2xl border border-zinc-900 overflow-hidden flex flex-col items-stretch min-h-[500px]">
            {loading ? (
                <div className="absolute inset-0 flex items-center justify-center bg-zinc-950/80 backdrop-blur-sm z-50">
                    <div className="flex flex-col items-center gap-4">
                        <RefreshCw className="w-8 h-8 text-cyan-500 animate-spin" />
                        <p className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.3em]">{i18n.t('knowledge_graph.label_synthesizing')}</p>
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
                {viewMode === 'okf' ? (
                    <>
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: '#22c55e' }} />
                            <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">{i18n.t('knowledge_graph.legend_confirmed')}</span>
                        </div>
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: '#f59e0b' }} />
                            <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">{i18n.t('knowledge_graph.legend_expiring')}</span>
                        </div>
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: '#ef4444' }} />
                            <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">{i18n.t('knowledge_graph.legend_broken')}</span>
                        </div>
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: '#71717a' }} />
                            <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">{i18n.t('knowledge_graph.legend_base')}</span>
                        </div>
                    </>
                ) : (
                    <>
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: '#22d3ee' }} />
                            <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">{i18n.t('knowledge_graph.legend_function')}</span>
                        </div>
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
                            <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">{i18n.t('knowledge_graph.legend_struct')}</span>
                        </div>
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full bg-amber-400" />
                            <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">{i18n.t('knowledge_graph.legend_trait')}</span>
                        </div>
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: '#06b6d4' }} />
                            <span className="text-[8px] font-bold text-zinc-400 uppercase tracking-widest">{i18n.t('knowledge_graph.legend_enum')}</span>
                        </div>
                    </>
                )}
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
