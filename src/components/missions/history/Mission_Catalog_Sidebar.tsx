/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / History / Catalog_Sidebar
 * - **Primary Entrypoints**: `Mission_Catalog_Sidebar`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Compliant with DESIGN.md color tokens, zero nested card inception, JetBrains Mono metrics.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { 
    Search, 
    Pin, 
    PinOff, 
    CheckCircle2, 
    AlertCircle, 
    Clock, 
    Layers, 
    FileCode2, 
    ChevronLeft, 
    ChevronRight,
    RefreshCw
} from 'lucide-react';
import type { MissionSummaryItem } from '../../../types/mission_dossier';
import { Tooltip } from '../../ui';

interface MissionCatalogSidebarProps {
    missions: MissionSummaryItem[];
    selected_id: string | null;
    is_loading: boolean;
    error?: string | null;
    total: number;
    page: number;
    per_page: number;
    search_query: string;
    status_filter: string;
    on_select_mission: (id: string) => void;
    on_search_change: (q: string) => void;
    on_status_change: (status: string) => void;
    on_page_change: (page: number) => void;
    on_toggle_pin: (id: string) => void;
    on_refresh: () => void;
}

const STATUS_FILTERS = [
    { key: 'all', label: 'All' },
    { key: 'completed', label: 'Done' },
    { key: 'failed', label: 'Failed' },
    { key: 'active', label: 'Active' },
    { key: 'paused', label: 'Paused' },
];

export const Mission_Catalog_Sidebar: React.FC<MissionCatalogSidebarProps> = ({
    missions,
    selected_id,
    is_loading,
    error,
    total,
    page,
    per_page,
    search_query,
    status_filter,
    on_select_mission,
    on_search_change,
    on_status_change,
    on_page_change,
    on_toggle_pin,
    on_refresh,
}) => {
    const total_pages = Math.max(1, Math.ceil(total / per_page));

    const render_status_pill = (status: string) => {
        const s = status.toLowerCase();
        if (s === 'completed') {
            return (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
                    <CheckCircle2 size={10} />
                    Done
                </span>
            );
        }
        if (s === 'failed') {
            return (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-rose-500/10 text-rose-400 border border-rose-500/30">
                    <AlertCircle size={10} />
                    Failed
                </span>
            );
        }
        return (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-amber-500/10 text-amber-400 border border-amber-500/30">
                <Clock size={10} />
                {status}
            </span>
        );
    };

    const format_date = (iso_str: string) => {
        try {
            const d = new Date(iso_str);
            return d.toLocaleDateString(undefined, { 
                month: 'short', 
                day: 'numeric', 
                hour: '2-digit', 
                minute: '2-digit' 
            });
        } catch {
            return iso_str;
        }
    };

    return (
        <aside className="w-80 md:w-96 flex flex-col h-full bg-zinc-950 border-r border-[color:var(--color-border)] select-none">
            {/* Header / Search Area */}
            <div className="p-4 border-b border-[color:var(--color-border)] flex flex-col gap-3 bg-[color:var(--color-surface)]/40 backdrop-blur-md">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <Layers size={16} className="text-cyan-400" />
                        <h2 className="text-xs font-bold text-zinc-100 uppercase tracking-widest">
                            Run Catalog
                        </h2>
                    </div>
                    <div className="flex items-center gap-1.5">
                        <span className="font-mono text-[11px] text-zinc-400 px-2 py-0.5 rounded bg-zinc-800/80 border border-zinc-700/50">
                            {total} runs
                        </span>
                        <Tooltip content="Refresh mission catalog from database" position="bottom">
                            <button
                                onClick={on_refresh}
                                aria-label="Refresh catalog"
                                className="p-1.5 text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-lg transition-colors"
                            >
                                <RefreshCw size={12} className={is_loading ? 'animate-spin' : ''} />
                            </button>
                        </Tooltip>
                    </div>
                </div>

                {/* Search Bar */}
                <div className="relative">
                    <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 pointer-events-none" />
                    <input
                        type="text"
                        placeholder="Search runs by ID, title, or agent..."
                        value={search_query}
                        onChange={(e) => on_search_change(e.target.value)}
                        className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg pl-9 pr-3 py-1.5 text-xs text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-emerald-500/50 focus:ring-1 focus:ring-emerald-500/30 transition-all font-sans"
                    />
                </div>

                {/* Status Filter Tabs */}
                <div className="flex items-center gap-1 overflow-x-auto custom-scrollbar pb-0.5">
                    {STATUS_FILTERS.map((f) => {
                        const active = status_filter === f.key;
                        return (
                            <button
                                key={f.key}
                                onClick={() => on_status_change(f.key)}
                                className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-all whitespace-nowrap ${
                                    active
                                        ? 'bg-cyan-500/15 text-cyan-300 border border-cyan-500/40 shadow-sm'
                                        : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/50'
                                }`}
                            >
                                {f.label}
                            </button>
                        );
                    })}
                </div>
            </div>

            {/* Mission List */}
            <div className="flex-1 overflow-y-auto custom-scrollbar p-3 flex flex-col gap-2">
                {error ? (
                    <div className="flex flex-col items-center justify-center p-6 text-center text-rose-400 gap-2.5 border border-rose-500/20 bg-rose-500/5 rounded-xl m-1">
                        <AlertCircle size={24} className="text-rose-400" />
                        <span className="text-xs font-semibold">Catalog Sync Failed</span>
                        <span className="text-[11px] text-zinc-400 max-w-[220px] break-words">{error}</span>
                        <button
                            onClick={on_refresh}
                            className="mt-1 px-3 py-1.5 rounded-lg bg-zinc-900 border border-zinc-800 text-xs text-zinc-300 hover:text-zinc-100 hover:border-zinc-700 transition-colors"
                        >
                            Retry Sync
                        </button>
                    </div>
                ) : is_loading && missions.length === 0 ? (
                    <div className="flex flex-col gap-2 p-2">
                        {[1, 2, 3, 4, 5].map((i) => (
                            <div key={i} className="h-20 bg-zinc-900/40 rounded-xl animate-pulse border border-zinc-800/40" />
                        ))}
                    </div>
                ) : missions.length === 0 ? (
                    <div className="flex flex-col items-center justify-center p-8 text-center text-zinc-500">
                        <Layers size={28} className="text-zinc-600 mb-2" />
                        <span className="text-xs font-semibold text-zinc-400">No missions found</span>
                        <span className="text-[11px] text-zinc-600 mt-1">Adjust search filter or run a new mission</span>
                    </div>
                ) : (
                    missions.map((m) => {
                        const is_selected = selected_id === m.id;
                        return (
                            <div
                                key={m.id}
                                role="button"
                                tabIndex={0}
                                aria-pressed={is_selected}
                                aria-label={`Select mission ${m.title || m.id}`}
                                onClick={() => on_select_mission(m.id)}
                                onKeyDown={(e) => {
                                    if (e.key === 'Enter' || e.key === ' ') {
                                        e.preventDefault();
                                        on_select_mission(m.id);
                                    }
                                }}
                                className={`group p-3 rounded-xl cursor-pointer transition-all border text-left flex flex-col gap-2 focus:outline-none focus:ring-1 focus:ring-cyan-500/50 ${
                                    is_selected
                                        ? 'bg-cyan-950/20 border-cyan-500/50 shadow-lg shadow-cyan-950/20'
                                        : 'bg-zinc-900/30 hover:bg-zinc-800/40 border-zinc-800/80 hover:border-zinc-700'
                                }`}
                            >
                                <div className="flex items-start justify-between gap-2">
                                    <div className="flex-1 min-w-0">
                                        <div className="flex items-center gap-2 mb-1">
                                            {render_status_pill(m.status)}
                                            <span className="font-mono text-[10px] text-zinc-400 truncate">
                                                {m.id}
                                            </span>
                                        </div>
                                        <h3 className="text-xs font-semibold text-zinc-200 group-hover:text-cyan-200 transition-colors line-clamp-1">
                                            {m.title || 'Untitled Mission'}
                                        </h3>
                                    </div>
                                    <Tooltip content={m.is_pinned ? "Exempt from Reaper: Retention pin active" : "Pin mission: Protect from Reaper deletion"} position="left">
                                        <button
                                            onClick={(e) => {
                                                e.stopPropagation();
                                                on_toggle_pin(m.id);
                                            }}
                                            aria-label={m.is_pinned ? 'Unpin mission' : 'Pin mission (exempt from Reaper)'}
                                            className={`p-1 rounded-md transition-colors ${
                                                m.is_pinned
                                                    ? 'text-amber-400 bg-amber-500/10 hover:bg-amber-500/20'
                                                    : 'text-zinc-600 hover:text-zinc-300 hover:bg-zinc-800'
                                            }`}
                                        >
                                            {m.is_pinned ? <Pin size={13} className="fill-amber-400" /> : <PinOff size={13} />}
                                        </button>
                                    </Tooltip>
                                </div>

                                <div className="flex items-center justify-between text-[11px] text-zinc-400 pt-1 border-t border-zinc-800/40 font-mono">
                                    <span className="text-zinc-500">{format_date(m.created_at)}</span>
                                    <div className="flex items-center gap-3">
                                        <Tooltip content="Discrete turn steps in Merkle audit trail" position="top">
                                            <span className="flex items-center gap-1 text-zinc-400">
                                                <Layers size={11} className="text-zinc-500" />
                                                {m.log_count}
                                            </span>
                                        </Tooltip>
                                        {m.diff_count !== undefined && (
                                            <Tooltip content="Unique files with pre-mutation CAS snapshots" position="top">
                                                <span className={`flex items-center gap-1 ${m.diff_count > 0 ? 'text-cyan-400' : 'text-zinc-500'}`}>
                                                    <FileCode2 size={11} />
                                                    {m.diff_count}
                                                </span>
                                            </Tooltip>
                                        )}
                                        <Tooltip content="Total LLM execution cost in USD" position="top">
                                            <span className="font-bold text-zinc-300">
                                                ${m.cost_usd.toFixed(4)}
                                            </span>
                                        </Tooltip>
                                    </div>
                                </div>
                            </div>
                        );
                    })
                )}
            </div>

            {/* Pagination Controls */}
            <div className="p-3 border-t border-[color:var(--color-border)] flex items-center justify-between bg-[color:var(--color-surface)]/20 text-xs">
                <span className="text-[11px] text-zinc-400 font-mono">
                    Page {page} of {total_pages}
                </span>
                <div className="flex items-center gap-1">
                    <Tooltip content="Previous page" position="top">
                        <button
                            disabled={page <= 1 || is_loading}
                            onClick={() => on_page_change(page - 1)}
                            className="p-1 rounded-md border border-zinc-800 bg-zinc-900/60 text-zinc-400 hover:text-zinc-100 disabled:opacity-30 disabled:pointer-events-none transition-colors"
                            aria-label="Previous page"
                        >
                            <ChevronLeft size={14} />
                        </button>
                    </Tooltip>
                    <Tooltip content="Next page" position="top">
                        <button
                            disabled={page >= total_pages || is_loading}
                            onClick={() => on_page_change(page + 1)}
                            className="p-1 rounded-md border border-zinc-800 bg-zinc-900/60 text-zinc-400 hover:text-zinc-100 disabled:opacity-30 disabled:pointer-events-none transition-colors"
                            aria-label="Next page"
                        >
                            <ChevronRight size={14} />
                        </button>
                    </Tooltip>
                </div>
            </div>
        </aside>
    );
};
