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
import type { CasDiffRecord } from '../../../types/mission_dossier';
import { Tooltip } from '../../ui';

interface CasDiffInspectorProps {
    diffs: CasDiffRecord[];
}

interface AlignedDiffLine {
    line_no: number;
    pre_line_no: number | null;
    cur_line_no: number | null;
    pre: string | null;
    cur: string | null;
    is_modified: boolean;
    is_added: boolean;
    is_deleted: boolean;
}

// eslint-disable-next-line react-refresh/only-export-components
export function compute_aligned_diff(pre_content: string, cur_content: string): AlignedDiffLine[] {
    const pre_lines = pre_content.split('\n');
    const cur_lines = cur_content.split('\n');

    const m = pre_lines.length;
    const n = cur_lines.length;

    if (m * n > 2_500_000) {
        const max_len = Math.max(m, n);
        const result: AlignedDiffLine[] = [];
        for (let idx = 0; idx < max_len; idx++) {
            const pre = idx < m ? pre_lines[idx] : null;
            const cur = idx < n ? cur_lines[idx] : null;
            result.push({
                line_no: idx + 1,
                pre_line_no: pre !== null ? idx + 1 : null,
                cur_line_no: cur !== null ? idx + 1 : null,
                pre,
                cur,
                is_modified: pre !== cur && pre !== null && cur !== null,
                is_added: pre === null && cur !== null,
                is_deleted: pre !== null && cur === null,
            });
        }
        return result;
    }

    const dp: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));

    for (let i = 1; i <= m; i++) {
        for (let j = 1; j <= n; j++) {
            if (pre_lines[i - 1] === cur_lines[j - 1]) {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
            }
        }
    }

    type EditOp = 
        | { type: 'equal'; pre_idx: number; cur_idx: number }
        | { type: 'delete'; pre_idx: number }
        | { type: 'add'; cur_idx: number };

    const ops: EditOp[] = [];
    let i = m;
    let j = n;

    while (i > 0 || j > 0) {
        if (i > 0 && j > 0 && pre_lines[i - 1] === cur_lines[j - 1]) {
            ops.push({ type: 'equal', pre_idx: i - 1, cur_idx: j - 1 });
            i--;
            j--;
        } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
            ops.push({ type: 'add', cur_idx: j - 1 });
            j--;
        } else if (i > 0 && (j === 0 || dp[i][j - 1] < dp[i - 1][j])) {
            ops.push({ type: 'delete', pre_idx: i - 1 });
            i--;
        }
    }

    ops.reverse();

    const result: AlignedDiffLine[] = [];
    let row_idx = 1;
    let k = 0;

    while (k < ops.length) {
        const op = ops[k];
        if (op.type === 'equal') {
            result.push({
                line_no: row_idx++,
                pre_line_no: op.pre_idx + 1,
                cur_line_no: op.cur_idx + 1,
                pre: pre_lines[op.pre_idx],
                cur: cur_lines[op.cur_idx],
                is_modified: false,
                is_added: false,
                is_deleted: false,
            });
            k++;
        } else {
            const deletes: number[] = [];
            const adds: number[] = [];

            while (k < ops.length && ops[k].type !== 'equal') {
                const cur_op = ops[k];
                if (cur_op.type === 'delete') {
                    deletes.push(cur_op.pre_idx);
                } else if (cur_op.type === 'add') {
                    adds.push(cur_op.cur_idx);
                }
                k++;
            }

            const max_block = Math.max(deletes.length, adds.length);
            for (let b = 0; b < max_block; b++) {
                const p_idx = b < deletes.length ? deletes[b] : null;
                const c_idx = b < adds.length ? adds[b] : null;

                const has_pre = p_idx !== null;
                const has_cur = c_idx !== null;
                const is_modified = has_pre && has_cur;
                const is_deleted = has_pre && !has_cur;
                const is_added = !has_pre && has_cur;

                result.push({
                    line_no: row_idx++,
                    pre_line_no: has_pre ? p_idx + 1 : null,
                    cur_line_no: has_cur ? c_idx + 1 : null,
                    pre: has_pre ? pre_lines[p_idx] : null,
                    cur: has_cur ? cur_lines[c_idx] : null,
                    is_modified,
                    is_added,
                    is_deleted,
                });
            }
        }
    }

    return result;
}

export const CAS_Diff_Inspector: React.FC<CasDiffInspectorProps> = ({ diffs }) => {
    const [selected_index, set_selected_index] = useState<number>(0);
    const [view_mode, set_view_mode] = useState<'split' | 'unified'>('split');
    const [copied_hash, set_copied_hash] = useState(false);

    // Defensive clamping to prevent index-out-of-bounds blank view
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

    // Calculate sequence-aligned diff structures using LCS algorithm
    const diff_lines = useMemo(() => {
        if (!active_diff) return [];
        return compute_aligned_diff(active_diff.pre_mutation_content, active_diff.current_content);
    }, [active_diff]);

    if (diffs.length === 0) {
        return (
            <div className="flex flex-col items-center justify-center h-full p-12 text-center text-zinc-500 bg-zinc-950 border border-zinc-800/80 rounded-xl">
                <FileCode2 size={36} className="text-zinc-600 mb-3" />
                <h3 className="text-sm font-semibold text-zinc-300">No CAS File Mutations</h3>
                <p className="text-xs text-zinc-500 max-w-sm mt-1">
                    This mission operated in read-only analysis mode and did not alter any workspace files.
                </p>
            </div>
        );
    }

    return (
        <div className="flex flex-col md:flex-row h-full bg-zinc-950 border border-zinc-800/80 rounded-xl overflow-hidden">
            {/* Left File Selector List */}
            <div className="w-full md:w-72 border-b md:border-b-0 md:border-r border-zinc-800/80 bg-zinc-950/40 flex flex-col flex-shrink-0">
                <div className="p-3 border-b border-zinc-800/80 flex items-center justify-between text-xs font-bold text-zinc-300 uppercase tracking-wider">
                    <span className="flex items-center gap-1.5">
                        <FileDiff size={14} className="text-purple-400" />
                        Mutated Files ({diffs.length})
                    </span>
                </div>

                <div className="flex-1 overflow-y-auto custom-scrollbar p-2 flex flex-col gap-1">
                    {diffs.map((d, idx) => {
                        const is_active = idx === safe_index;
                        return (
                            <button
                                key={d.id}
                                onClick={() => set_selected_index(idx)}
                                className={`w-full text-left p-2.5 rounded-lg border text-xs transition-all flex flex-col gap-1.5 ${
                                    is_active
                                        ? 'bg-purple-950/20 border-purple-500/50 shadow-sm'
                                        : 'bg-zinc-900/30 hover:bg-zinc-900/60 border-zinc-800/80 text-zinc-400 hover:text-zinc-200'
                                }`}
                            >
                                <div className="flex items-center justify-between">
                                    <span className="font-mono text-[10px] text-zinc-500 bg-zinc-950 px-1 rounded border border-zinc-800">
                                        rev v{d.version_num}
                                    </span>
                                    <div className="flex items-center gap-1 font-mono text-[10px]">
                                        <span className="text-emerald-400 font-bold">+{d.diff_summary.additions}</span>
                                        <span className="text-rose-400 font-bold">-{d.diff_summary.deletions}</span>
                                    </div>
                                </div>
                                <span className="font-mono text-[11px] truncate text-zinc-200 font-medium">
                                    {d.file_path}
                                </span>
                            </button>
                        );
                    })}
                </div>
            </div>

            {/* Right Diff Viewer */}
            {active_diff && (
                <div className="flex-1 flex flex-col overflow-hidden bg-slate-950/30">
                    {/* Diff Header Bar */}
                    <div className="p-3 border-b border-zinc-800/80 bg-zinc-900/40 flex flex-wrap items-center justify-between gap-3 text-xs">
                        <div className="flex items-center gap-2 min-w-0">
                            <span className="font-mono font-bold text-zinc-100 truncate text-xs">
                                {active_diff.file_path}
                            </span>
                            <span className="font-mono text-[10px] text-purple-400 px-2 py-0.5 rounded bg-purple-500/10 border border-purple-500/20">
                                version {active_diff.version_num}
                            </span>
                            <span className="font-mono text-[10px] text-zinc-500">
                                {(active_diff.size_bytes / 1024).toFixed(1)} KB
                            </span>
                        </div>

                        <div className="flex items-center gap-2 font-mono">
                            {/* Hash button */}
                            <Tooltip content={copied_hash ? "Copied CAS hash to clipboard!" : `Pre-mutation CAS SHA-256: ${active_diff.hash}`} position="top">
                                <button
                                    onClick={() => handle_copy_hash(active_diff.hash)}
                                    className="flex items-center gap-1 text-[10px] text-zinc-400 hover:text-zinc-200 bg-zinc-950 px-2 py-1 rounded border border-zinc-800 transition-colors"
                                    aria-label="Copy pre-mutation SHA-256 CAS hash"
                                >
                                    <span className="text-zinc-500">sha256:</span>
                                    <span>{active_diff.hash.slice(0, 12)}…</span>
                                    {copied_hash ? <Check size={11} className="text-emerald-400" /> : <Copy size={11} />}
                                </button>
                            </Tooltip>

                            {/* View Mode Toggle */}
                            <div className="flex items-center bg-zinc-950 p-0.5 rounded-lg border border-zinc-800">
                                <Tooltip content="Side-by-side comparative diff" position="top">
                                    <button
                                        onClick={() => set_view_mode('split')}
                                        className={`p-1 rounded text-[10px] font-sans flex items-center gap-1 ${
                                            view_mode === 'split' ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300'
                                        }`}
                                        aria-label="Side-by-side view"
                                    >
                                        <Columns size={12} />
                                        <span>Split</span>
                                    </button>
                                </Tooltip>
                                <Tooltip content="Unified patch diff" position="top">
                                    <button
                                        onClick={() => set_view_mode('unified')}
                                        className={`p-1 rounded text-[10px] font-sans flex items-center gap-1 ${
                                            view_mode === 'unified' ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300'
                                        }`}
                                        aria-label="Unified view"
                                    >
                                        <List size={12} />
                                        <span>Unified</span>
                                    </button>
                                </Tooltip>
                            </div>
                        </div>
                    </div>

                    {/* Diff Editor Box */}
                    <div className="flex-1 overflow-auto custom-scrollbar font-mono text-[11px] leading-relaxed select-text">
                        {view_mode === 'split' ? (
                            <div className="grid grid-cols-2 divide-x divide-zinc-800 min-w-[700px]">
                                {/* Left: Pre-Mutation CAS Blob */}
                                <div className="flex flex-col">
                                    <div className="sticky top-0 bg-zinc-950/90 backdrop-blur px-3 py-1.5 text-[10px] uppercase font-bold text-zinc-500 border-b border-zinc-800/80 z-10 flex items-center justify-between">
                                        <span>Pre-Mutation (CAS Storage)</span>
                                        <span className="text-zinc-600">Initial State</span>
                                    </div>
                                    <div className="divide-y divide-zinc-900/40">
                                        {diff_lines.map((l) => (
                                            <div
                                                key={`pre-${l.line_no}`}
                                                className={`flex items-start px-2 py-0.5 ${
                                                    l.is_modified || l.is_deleted
                                                        ? 'bg-rose-950/20 text-rose-300'
                                                        : 'text-zinc-400'
                                                }`}
                                            >
                                                <span className="w-8 text-zinc-600 select-none text-right pr-3 font-mono text-[10px]">
                                                    {l.pre !== null ? l.pre_line_no : ''}
                                                </span>
                                                <span className="whitespace-pre overflow-x-auto custom-scrollbar flex-1">
                                                    {l.pre ?? ' '}
                                                </span>
                                            </div>
                                        ))}
                                    </div>
                                </div>

                                {/* Right: Current Active Disk Content */}
                                <div className="flex flex-col">
                                    <div className="sticky top-0 bg-zinc-950/90 backdrop-blur px-3 py-1.5 text-[10px] uppercase font-bold text-zinc-500 border-b border-zinc-800/80 z-10 flex items-center justify-between">
                                        <span>Current File (Disk)</span>
                                        <span className="text-emerald-400 font-bold">Modified State</span>
                                    </div>
                                    <div className="divide-y divide-zinc-900/40">
                                        {diff_lines.map((l) => (
                                            <div
                                                key={`cur-${l.line_no}`}
                                                className={`flex items-start px-2 py-0.5 ${
                                                    l.is_modified || l.is_added
                                                        ? 'bg-emerald-950/20 text-emerald-300'
                                                        : 'text-zinc-400'
                                                }`}
                                            >
                                                <span className="w-8 text-zinc-600 select-none text-right pr-3 font-mono text-[10px]">
                                                    {l.cur !== null ? l.cur_line_no : ''}
                                                </span>
                                                <span className="whitespace-pre overflow-x-auto custom-scrollbar flex-1">
                                                    {l.cur ?? ' '}
                                                </span>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            </div>
                        ) : (
                            /* Unified Diff View */
                            <div className="flex flex-col divide-y divide-zinc-900/40 min-w-[500px]">
                                <div className="sticky top-0 bg-zinc-950/90 backdrop-blur px-3 py-1.5 text-[10px] uppercase font-bold text-zinc-500 border-b border-zinc-800/80 z-10">
                                    Unified Line View
                                </div>
                                {diff_lines.map((l) => (
                                    <React.Fragment key={`uni-${l.line_no}`}>
                                        {l.pre !== null && (l.is_modified || l.is_deleted) && (
                                            <div className="flex items-start px-3 py-0.5 bg-rose-950/25 text-rose-300">
                                                <span className="w-10 text-rose-500 select-none text-right pr-3 font-mono text-[10px]">
                                                    -{l.pre_line_no}
                                                </span>
                                                <span className="whitespace-pre flex-1">{l.pre}</span>
                                            </div>
                                        )}
                                        {l.cur !== null && (l.is_modified || l.is_added) && (
                                            <div className="flex items-start px-3 py-0.5 bg-emerald-950/25 text-emerald-300">
                                                <span className="w-10 text-emerald-500 select-none text-right pr-3 font-mono text-[10px]">
                                                    +{l.cur_line_no}
                                                </span>
                                                <span className="whitespace-pre flex-1">{l.cur}</span>
                                            </div>
                                        )}
                                        {!l.is_modified && !l.is_added && !l.is_deleted && (
                                            <div className="flex items-start px-3 py-0.5 text-zinc-400">
                                                <span className="w-10 text-zinc-600 select-none text-right pr-3 font-mono text-[10px]">
                                                    {l.cur_line_no ?? l.pre_line_no}
                                                </span>
                                                <span className="whitespace-pre flex-1">{l.cur ?? l.pre}</span>
                                            </div>
                                        )}
                                    </React.Fragment>
                                ))}
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
};
