/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / History / cas_diff_align
 * - **Primary Entrypoints**: `compute_aligned_diff`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic LCS line alignment with prefix/suffix trimming.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import type { AlignedDiffLine } from '../../../types/mission_dossier';

type EditOp =
    | { type: 'equal'; pre_idx: number; cur_idx: number }
    | { type: 'delete'; pre_idx: number }
    | { type: 'add'; cur_idx: number };

function makeUnchangedLine(line_no: number, p_idx: number, c_idx: number, text: string): AlignedDiffLine {
    return {
        line_no,
        pre_line_no: p_idx + 1,
        cur_line_no: c_idx + 1,
        pre: text,
        cur: text,
        is_modified: false,
        is_added: false,
        is_deleted: false,
    };
}

function makePairLine(
    line_no: number,
    p_idx: number | null,
    c_idx: number | null,
    pre_lines: string[],
    cur_lines: string[],
): AlignedDiffLine {
    const pre = p_idx !== null ? pre_lines[p_idx] : null;
    const cur = c_idx !== null ? cur_lines[c_idx] : null;
    const has_pre = p_idx !== null;
    const has_cur = c_idx !== null;
    return {
        line_no,
        pre_line_no: p_idx !== null ? p_idx + 1 : null,
        cur_line_no: c_idx !== null ? c_idx + 1 : null,
        pre,
        cur,
        is_modified: has_pre && has_cur && pre !== cur,
        is_added: !has_pre && has_cur,
        is_deleted: has_pre && !has_cur,
    };
}

function fillLcsRow(dp: Int32Array, stride: number, i: number, pre_line: string, mid_cur: string[]): void {
    const prev_row = (i - 1) * stride;
    const curr_row = i * stride;
    for (let j = 1; j <= mid_cur.length; j++) {
        if (pre_line === mid_cur[j - 1]) {
            dp[curr_row + j] = dp[prev_row + (j - 1)] + 1;
        } else {
            const up = dp[prev_row + j];
            const left = dp[curr_row + (j - 1)];
            dp[curr_row + j] = up >= left ? up : left;
        }
    }
}

function buildLcsOps(mid_pre: string[], mid_cur: string[], prefix_len: number): EditOp[] {
    const m = mid_pre.length;
    const n = mid_cur.length;
    const stride = n + 1;
    const dp = new Int32Array((m + 1) * stride);
    for (let i = 1; i <= m; i++) {
        fillLcsRow(dp, stride, i, mid_pre[i - 1], mid_cur);
    }

    const ops: EditOp[] = [];
    let i = m;
    let j = n;
    while (i > 0 || j > 0) {
        if (i > 0 && j > 0 && mid_pre[i - 1] === mid_cur[j - 1]) {
            ops.push({ type: 'equal', pre_idx: prefix_len + i - 1, cur_idx: prefix_len + j - 1 });
            i--;
            j--;
        } else if (j > 0 && (i === 0 || dp[i * stride + (j - 1)] >= dp[(i - 1) * stride + j])) {
            ops.push({ type: 'add', cur_idx: prefix_len + j - 1 });
            j--;
        } else {
            ops.push({ type: 'delete', pre_idx: prefix_len + i - 1 });
            i--;
        }
    }
    ops.reverse();
    return ops;
}

function emitFallbackWindow(
    mid_pre: string[],
    mid_cur: string[],
    prefix_len: number,
    pre_lines: string[],
    cur_lines: string[],
    result: AlignedDiffLine[],
): void {
    const m = mid_pre.length;
    const n = mid_cur.length;
    const max_len = Math.max(m, n);
    for (let idx = 0; idx < max_len; idx++) {
        const p_idx = idx < m ? prefix_len + idx : null;
        const c_idx = idx < n ? prefix_len + idx : null;
        result.push(makePairLine(result.length + 1, p_idx, c_idx, pre_lines, cur_lines));
    }
}

function collectHunk(ops: EditOp[], start_k: number): { deletes: number[]; adds: number[]; next_k: number } {
    const deletes: number[] = [];
    const adds: number[] = [];
    let k = start_k;
    while (k < ops.length && ops[k].type !== 'equal') {
        const cur_op = ops[k];
        if (cur_op.type === 'delete') {
            deletes.push(cur_op.pre_idx);
        } else if (cur_op.type === 'add') {
            adds.push(cur_op.cur_idx);
        }
        k++;
    }
    return { deletes, adds, next_k: k };
}

function emitEditOps(ops: EditOp[], pre_lines: string[], cur_lines: string[], result: AlignedDiffLine[]): void {
    let k = 0;
    while (k < ops.length) {
        const op = ops[k];
        if (op.type === 'equal') {
            result.push(makeUnchangedLine(result.length + 1, op.pre_idx, op.cur_idx, pre_lines[op.pre_idx]));
            k++;
            continue;
        }
        const { deletes, adds, next_k } = collectHunk(ops, k);
        k = next_k;
        const max_block = Math.max(deletes.length, adds.length);
        for (let b = 0; b < max_block; b++) {
            const p_idx = b < deletes.length ? deletes[b] : null;
            const c_idx = b < adds.length ? adds[b] : null;
            result.push(makePairLine(result.length + 1, p_idx, c_idx, pre_lines, cur_lines));
        }
    }
}

export function compute_aligned_diff(pre_content: string, cur_content: string): AlignedDiffLine[] {
    const pre_lines = pre_content.split('\n');
    const cur_lines = cur_content.split('\n');

    let prefix_len = 0;
    while (
        prefix_len < pre_lines.length &&
        prefix_len < cur_lines.length &&
        pre_lines[prefix_len] === cur_lines[prefix_len]
    ) {
        prefix_len++;
    }

    let suffix_len = 0;
    while (
        suffix_len < pre_lines.length - prefix_len &&
        suffix_len < cur_lines.length - prefix_len &&
        pre_lines[pre_lines.length - 1 - suffix_len] === cur_lines[cur_lines.length - 1 - suffix_len]
    ) {
        suffix_len++;
    }

    const mid_pre = pre_lines.slice(prefix_len, pre_lines.length - suffix_len);
    const mid_cur = cur_lines.slice(prefix_len, cur_lines.length - suffix_len);
    const result: AlignedDiffLine[] = [];

    for (let idx = 0; idx < prefix_len; idx++) {
        result.push(makeUnchangedLine(result.length + 1, idx, idx, pre_lines[idx]));
    }

    const has_mid = mid_pre.length > 0 || mid_cur.length > 0;
    const use_fallback = mid_pre.length * mid_cur.length > 2_500_000;
    if (has_mid && use_fallback) {
        emitFallbackWindow(mid_pre, mid_cur, prefix_len, pre_lines, cur_lines, result);
    } else if (has_mid) {
        const ops = buildLcsOps(mid_pre, mid_cur, prefix_len);
        emitEditOps(ops, pre_lines, cur_lines, result);
    }

    const pre_suf = pre_lines.length - suffix_len;
    const cur_suf = cur_lines.length - suffix_len;
    for (let idx = 0; idx < suffix_len; idx++) {
        result.push(makeUnchangedLine(result.length + 1, pre_suf + idx, cur_suf + idx, pre_lines[pre_suf + idx]));
    }

    return result;
}

