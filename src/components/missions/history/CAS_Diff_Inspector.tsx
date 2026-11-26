/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / History / CAS_Diff_Inspector
 * - **Primary Entrypoints**: `CAS_Diff_Inspector`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Pre-mutation CAS BLOB vs active disk diff, line numbering, JetBrains Mono syntax view.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState, useMemo } from 'react';
import {
    FileCode2,
    Copy,
    Check,
    FileDiff,
    Columns,
    List,
} from 'lucide-react';
import type { CasDiffRecord, AlignedDiffLine } from '../../../types/mission_dossier';
import { Tooltip } from '../../ui';
import { compute_aligned_diff } from './cas_diff_align';

interface CasDiffInspectorProps {
    diffs: CasDiffRecord[];
}

const EmptyCasDiffState: React.FC = () => (
    <div className="flex flex-col items-center justify-center h-full p-12 text-center text-zinc-500 bg-zinc-950 border border-zinc-800/80 rounded-xl">
        <FileCode2 size={36} className="text-zinc-600 mb-3" />
        <h3 className="text-sm font-semibold text-zinc-300">No CAS File Mutations</h3>
        <p className="text-xs text-zinc-500 max-w-sm mt-1">
            This mission operated in read-only analysis mode and did not alter any workspace files.
        </p>
    </div>
);

interface MutatedFileItemProps {
    diff: CasDiffRecord;
    is_active: boolean;
    on_select: () => void;
}

const MutatedFileItem: React.FC<MutatedFileItemProps> = ({ diff, is_active, on_select }) => {
    const active_cls = is_active
        ? 'bg-purple-950/20 border-purple-500/50 shadow-sm'
        : 'bg-zinc-900/30 hover:bg-zinc-900/60 border-zinc-800/80 text-zinc-400 hover:text-zinc-200';
    return (
        <button
            onClick={on_select}
            className={`w-full text-left p-2.5 rounded-lg border text-xs transition-all flex flex-col gap-1.5 ${active_cls}`}
        >
            <div className="flex items-center justify-between">
                <span className="font-mono text-[10px] text-zinc-500 bg-zinc-950 px-1 rounded border border-zinc-800">rev v{diff.version_num}</span>
                <div className="flex items-center gap-1 font-mono text-[10px]">
                    <span className="text-emerald-400 font-bold">+{diff.diff_summary.additions}</span>
                    <span className="text-rose-400 font-bold">-{diff.diff_summary.deletions}</span>
                </div>
            </div>
            <span className="font-mono text-[11px] truncate text-zinc-200 font-medium">{diff.file_path}</span>
        </button>
    );
};

interface ViewModeToggleProps {
    view_mode: 'split' | 'unified';
    on_view_mode_change: (mode: 'split' | 'unified') => void;
}

const ViewModeToggle: React.FC<ViewModeToggleProps> = ({ view_mode, on_view_mode_change }) => {
    const split_cls = view_mode === 'split' ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300';
    const unified_cls = view_mode === 'unified' ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300';

    return (
        <div className="flex items-center bg-zinc-950 p-0.5 rounded-lg border border-zinc-800">
            <Tooltip content="Side-by-side comparative diff" position="top">
                <button
                    onClick={() => on_view_mode_change('split')}
                    className={`p-1 rounded text-[10px] font-sans flex items-center gap-1 ${split_cls}`}
                    aria-label="Side-by-side view"
                >
                    <Columns size={12} />
                    <span>Split</span>
                </button>
            </Tooltip>
            <Tooltip content="Unified patch diff" position="top">
                <button
                    onClick={() => on_view_mode_change('unified')}
                    className={`p-1 rounded text-[10px] font-sans flex items-center gap-1 ${unified_cls}`}
                    aria-label="Unified view"
                >
                    <List size={12} />
                    <span>Unified</span>
                </button>
            </Tooltip>
        </div>
    );
};

interface DiffHeaderBarProps {
    active_diff: CasDiffRecord;
    copied_hash: boolean;
    view_mode: 'split' | 'unified';
    on_copy_hash: (hash: string) => void;
    on_view_mode_change: (mode: 'split' | 'unified') => void;
}

const DiffHeaderBar: React.FC<DiffHeaderBarProps> = ({
    active_diff,
    copied_hash,
    view_mode,
    on_copy_hash,
    on_view_mode_change,
}) => {
    const hash_tip = copied_hash ? 'Copied CAS hash to clipboard!' : `Pre-mutation CAS SHA-256: ${active_diff.hash}`;

    return (
        <div className="p-3 border-b border-zinc-800/80 bg-zinc-900/40 flex flex-wrap items-center justify-between gap-3 text-xs">
            <div className="flex items-center gap-2 min-w-0">
                <span className="font-mono font-bold text-zinc-100 truncate text-xs">{active_diff.file_path}</span>
                <span className="font-mono text-[10px] text-purple-400 px-2 py-0.5 rounded bg-purple-500/10 border border-purple-500/20">version {active_diff.version_num}</span>
                <span className="font-mono text-[10px] text-zinc-500">{(active_diff.size_bytes / 1024).toFixed(1)} KB</span>
            </div>

            <div className="flex items-center gap-2 font-mono">
                <Tooltip content={hash_tip} position="top">
                    <button
                        onClick={() => on_copy_hash(active_diff.hash)}
                        className="flex items-center gap-1 text-[10px] text-zinc-400 hover:text-zinc-200 bg-zinc-950 px-2 py-1 rounded border border-zinc-800 transition-colors"
                        aria-label="Copy pre-mutation SHA-256 CAS hash"
                    >
                        <span className="text-zinc-500">sha256:</span>
                        <span>{active_diff.hash.slice(0, 12)}…</span>
                        {copied_hash ? <Check size={11} className="text-emerald-400" /> : <Copy size={11} />}
                    </button>
                </Tooltip>

                <ViewModeToggle view_mode={view_mode} on_view_mode_change={on_view_mode_change} />
            </div>
        </div>
    );
};

const SplitDiffView: React.FC<{ diff_lines: AlignedDiffLine[] }> = ({ diff_lines }) => (
    <div className="grid grid-cols-2 divide-x divide-zinc-800 min-w-[700px]">
        <div className="flex flex-col">
            <div className="sticky top-0 bg-zinc-950/90 backdrop-blur px-3 py-1.5 text-[10px] uppercase font-bold text-zinc-500 border-b border-zinc-800/80 z-10 flex items-center justify-between">
                <span>Pre-Mutation (CAS Storage)</span>
                <span className="text-zinc-600">Initial State</span>
            </div>
            <div className="divide-y divide-zinc-900/40">
                {diff_lines.map((l) => (
                    <div
                        key={`pre-${l.line_no}`}
                        className={`flex items-start px-2 py-0.5 ${l.is_modified || l.is_deleted ? 'bg-rose-950/20 text-rose-300' : 'text-zinc-400'}`}
                    >
                        <span className="w-8 text-zinc-600 select-none text-right pr-3 font-mono text-[10px]">
                            {l.pre !== null ? l.pre_line_no : ''}
                        </span>
                        <span className="whitespace-pre overflow-x-auto custom-scrollbar flex-1">{l.pre ?? ' '}</span>
                    </div>
                ))}
            </div>
        </div>

        <div className="flex flex-col">
            <div className="sticky top-0 bg-zinc-950/90 backdrop-blur px-3 py-1.5 text-[10px] uppercase font-bold text-zinc-500 border-b border-zinc-800/80 z-10 flex items-center justify-between">
                <span>Current File (Disk)</span>
                <span className="text-emerald-400 font-bold">Modified State</span>
            </div>
            <div className="divide-y divide-zinc-900/40">
                {diff_lines.map((l) => (
                    <div
                        key={`cur-${l.line_no}`}
                        className={`flex items-start px-2 py-0.5 ${l.is_modified || l.is_added ? 'bg-emerald-950/20 text-emerald-300' : 'text-zinc-400'}`}
                    >
                        <span className="w-8 text-zinc-600 select-none text-right pr-3 font-mono text-[10px]">
                            {l.cur !== null ? l.cur_line_no : ''}
                        </span>
                        <span className="whitespace-pre overflow-x-auto custom-scrollbar flex-1">{l.cur ?? ' '}</span>
                    </div>
                ))}
            </div>
        </div>
    </div>
);

const UnifiedDiffView: React.FC<{ diff_lines: AlignedDiffLine[] }> = ({ diff_lines }) => (
    <div className="flex flex-col divide-y divide-zinc-900/40 min-w-[500px]">
        <div className="sticky top-0 bg-zinc-950/90 backdrop-blur px-3 py-1.5 text-[10px] uppercase font-bold text-zinc-500 border-b border-zinc-800/80 z-10">
            Unified Line View
        </div>
        {diff_lines.map((l) => (
            <React.Fragment key={`uni-${l.line_no}`}>
                {l.pre !== null && (l.is_modified || l.is_deleted) && (
                    <div className="flex items-start px-3 py-0.5 bg-rose-950/25 text-rose-300">
                        <span className="w-10 text-rose-500 select-none text-right pr-3 font-mono text-[10px]">-{l.pre_line_no}</span>
                        <span className="whitespace-pre flex-1">{l.pre}</span>
                    </div>
                )}
                {l.cur !== null && (l.is_modified || l.is_added) && (
                    <div className="flex items-start px-3 py-0.5 bg-emerald-950/25 text-emerald-300">
                        <span className="w-10 text-emerald-500 select-none text-right pr-3 font-mono text-[10px]">+{l.cur_line_no}</span>
                        <span className="whitespace-pre flex-1">{l.cur}</span>
                    </div>
                )}
                {!l.is_modified && !l.is_added && !l.is_deleted && (
                    <div className="flex items-start px-3 py-0.5 text-zinc-400">
                        <span className="w-10 text-zinc-600 select-none text-right pr-3 font-mono text-[10px]">{l.cur_line_no ?? l.pre_line_no}</span>
                        <span className="whitespace-pre flex-1">{l.cur ?? l.pre}</span>
                    </div>
                )}
            </React.Fragment>
        ))}
    </div>
);

interface MutatedFilesSidebarProps {
    diffs: CasDiffRecord[];
    safe_index: number;
    on_select: (idx: number) => void;
}

const MutatedFilesSidebar: React.FC<MutatedFilesSidebarProps> = ({ diffs, safe_index, on_select }) => (
    <div className="w-full md:w-72 border-b md:border-b-0 md:border-r border-zinc-800/80 bg-zinc-950/40 flex flex-col flex-shrink-0">
        <div className="p-3 border-b border-zinc-800/80 flex items-center justify-between text-xs font-bold text-zinc-300 uppercase tracking-wider">
            <span className="flex items-center gap-1.5"><FileDiff size={14} className="text-purple-400" />Mutated Files ({diffs.length})</span>
        </div>
        <div className="flex-1 overflow-y-auto custom-scrollbar p-2 flex flex-col gap-1">
            {diffs.map((d, idx) => (
                <MutatedFileItem
                    key={d.id}
                    diff={d}
                    is_active={idx === safe_index}
                    on_select={() => on_select(idx)}
                />
            ))}
        </div>
    </div>
);

export const CAS_Diff_Inspector: React.FC<CasDiffInspectorProps> = ({ diffs }) => {
    const [selected_index, set_selected_index] = useState<number>(0);
    const [view_mode, set_view_mode] = useState<'split' | 'unified'>('split');
    const [copied_hash, set_copied_hash] = useState(false);

    const safe_index = diffs.length > 0 ? Math.min(Math.max(0, selected_index), diffs.length - 1) : 0;
    const active_diff = diffs[safe_index] ?? null;

    const handle_copy_hash = async (hash: string) => {
        try {
            await navigator.clipboard.writeText(hash);
            set_copied_hash(true);
            setTimeout(() => set_copied_hash(false), 2000);
        } catch (err) {
            console.warn('Failed to copy CAS hash to clipboard:', err);
        }
    };

    const diff_lines = useMemo(() => {
        if (!active_diff) return [];
        if (active_diff.aligned_lines && active_diff.aligned_lines.length > 0) {
            return active_diff.aligned_lines;
        }
        return compute_aligned_diff(active_diff.pre_mutation_content, active_diff.current_content);
    }, [active_diff]);

    if (diffs.length === 0) {
        return <EmptyCasDiffState />;
    }

    return (
        <div className="flex flex-col md:flex-row h-full bg-zinc-950 border border-zinc-800/80 rounded-xl overflow-hidden">
            <MutatedFilesSidebar diffs={diffs} safe_index={safe_index} on_select={set_selected_index} />
            {active_diff && (
                <div className="flex-1 flex flex-col overflow-hidden bg-slate-950/30">
                    <DiffHeaderBar
                        active_diff={active_diff}
                        copied_hash={copied_hash}
                        view_mode={view_mode}
                        on_copy_hash={handle_copy_hash}
                        on_view_mode_change={set_view_mode}
                    />
                    <div className="flex-1 overflow-auto custom-scrollbar font-mono text-[11px] leading-relaxed select-text">
                        {view_mode === 'split' ? <SplitDiffView diff_lines={diff_lines} /> : <UnifiedDiffView diff_lines={diff_lines} />}
                    </div>
                </div>
            )}
        </div>
    );
};
