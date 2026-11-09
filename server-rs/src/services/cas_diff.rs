//! @docs ARCHITECTURE:Core:Storage:CAS
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Services / cas_diff
//! - **Primary Entrypoints**: `CasDiffSummary`, `AlignedDiffLine`, `CasFileDiff`, `compute_aligned_diff`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Deterministic LCS line alignment with prefix/suffix trimming and bounded fallback.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CasDiffSummary {
    pub additions: usize,
    pub deletions: usize,
    pub changes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlignedDiffLine {
    pub line_no: usize,
    pub pre_line_no: Option<usize>,
    pub cur_line_no: Option<usize>,
    pub pre: Option<String>,
    pub cur: Option<String>,
    pub is_modified: bool,
    pub is_added: bool,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasFileDiff {
    pub file_path: String,
    pub version_num: i64,
    pub hash: String,
    pub size_bytes: usize,
    pub created_at: String,
    pub diff_summary: CasDiffSummary,
    pub aligned_lines: Vec<AlignedDiffLine>,
}

#[derive(Clone, Copy)]
enum EditOp {
    Equal(usize, usize),
    Delete(usize),
    Add(usize),
}

fn split_normalized_lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return Vec::new();
    }
    text.split('\n')
        .map(|s| s.strip_suffix('\r').unwrap_or(s))
        .collect()
}

fn make_unchanged_row(row_idx: usize, p_idx: usize, c_idx: usize, text: &str) -> AlignedDiffLine {
    let line = text.to_string();
    AlignedDiffLine {
        line_no: row_idx,
        pre_line_no: Some(p_idx + 1),
        cur_line_no: Some(c_idx + 1),
        pre: Some(line.clone()),
        cur: Some(line),
        is_modified: false,
        is_added: false,
        is_deleted: false,
    }
}

fn make_pair_row(
    row_idx: usize,
    p_idx: Option<usize>,
    c_idx: Option<usize>,
    pre_lines: &[&str],
    cur_lines: &[&str],
) -> AlignedDiffLine {
    let pre_val = p_idx.map(|idx| pre_lines[idx]);
    let cur_val = c_idx.map(|idx| cur_lines[idx]);
    let has_pre = pre_val.is_some();
    let has_cur = cur_val.is_some();
    let is_equal = has_pre && pre_val == cur_val;
    AlignedDiffLine {
        line_no: row_idx,
        pre_line_no: p_idx.map(|idx| idx + 1),
        cur_line_no: c_idx.map(|idx| idx + 1),
        pre: pre_val.map(str::to_string),
        cur: cur_val.map(str::to_string),
        is_modified: has_pre && has_cur && !is_equal,
        is_added: !has_pre && has_cur,
        is_deleted: has_pre && !has_cur,
    }
}

fn build_lcs_table(mid_pre: &[&str], mid_cur: &[&str]) -> Vec<u32> {
    let m = mid_pre.len();
    let n = mid_cur.len();
    let stride = n + 1;
    let mut dp = vec![0u32; (m + 1) * stride];
    for i in 1..=m {
        fill_lcs_row(&mut dp, stride, i, mid_pre[i - 1], mid_cur);
    }
    dp
}

fn fill_lcs_row(dp: &mut [u32], stride: usize, i: usize, pre_line: &str, mid_cur: &[&str]) {
    let prev_row = (i - 1) * stride;
    let curr_row = i * stride;
    for (j_zero, &cur_line) in mid_cur.iter().enumerate() {
        let j = j_zero + 1;
        dp[curr_row + j] = if pre_line == cur_line {
            dp[prev_row + (j - 1)] + 1
        } else {
            dp[prev_row + j].max(dp[curr_row + (j - 1)])
        };
    }
}

fn backtrack_lcs_ops(
    dp: &[u32],
    mid_pre: &[&str],
    mid_cur: &[&str],
    prefix_len: usize,
) -> Vec<EditOp> {
    let m = mid_pre.len();
    let n = mid_cur.len();
    let stride = n + 1;
    let mut ops = Vec::with_capacity(m + n);
    let mut i = m;
    let mut j = n;
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && mid_pre[i - 1] == mid_cur[j - 1] {
            ops.push(EditOp::Equal(prefix_len + i - 1, prefix_len + j - 1));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i * stride + (j - 1)] >= dp[(i - 1) * stride + j]) {
            ops.push(EditOp::Add(prefix_len + j - 1));
            j -= 1;
        } else {
            ops.push(EditOp::Delete(prefix_len + i - 1));
            i -= 1;
        }
    }
    ops.reverse();
    ops
}

fn emit_linear_fallback(
    mid_pre: &[&str],
    mid_cur: &[&str],
    prefix_len: usize,
    pre_lines: &[&str],
    cur_lines: &[&str],
    result: &mut Vec<AlignedDiffLine>,
    counts: &mut (usize, usize),
) {
    let m = mid_pre.len();
    let n = mid_cur.len();
    let max_len = m.max(n);
    for idx in 0..max_len {
        let p_idx = (idx < m).then_some(prefix_len + idx);
        let c_idx = (idx < n).then_some(prefix_len + idx);
        let row = make_pair_row(result.len() + 1, p_idx, c_idx, pre_lines, cur_lines);
        counts.0 += usize::from(row.is_modified || row.is_added);
        counts.1 += usize::from(row.is_modified || row.is_deleted);
        result.push(row);
    }
}

fn emit_edit_ops(
    ops: &[EditOp],
    pre_lines: &[&str],
    cur_lines: &[&str],
    result: &mut Vec<AlignedDiffLine>,
    counts: &mut (usize, usize),
) {
    let mut k = 0;
    while k < ops.len() {
        if let EditOp::Equal(p_idx, c_idx) = ops[k] {
            let next_no = result.len() + 1;
            result.push(make_unchanged_row(next_no, p_idx, c_idx, pre_lines[p_idx]));
            k += 1;
            continue;
        }
        let (deletes, adds, next_k) = collect_change_hunk(ops, k);
        k = next_k;
        counts.1 += deletes.len();
        counts.0 += adds.len();
        let max_block = deletes.len().max(adds.len());
        for b in 0..max_block {
            let next_no = result.len() + 1;
            let p_idx = deletes.get(b).copied();
            let c_idx = adds.get(b).copied();
            result.push(make_pair_row(next_no, p_idx, c_idx, pre_lines, cur_lines));
        }
    }
}

fn collect_change_hunk(ops: &[EditOp], start_k: usize) -> (Vec<usize>, Vec<usize>, usize) {
    let mut deletes = Vec::new();
    let mut adds = Vec::new();
    let mut k = start_k;
    while k < ops.len() {
        match ops[k] {
            EditOp::Delete(p_idx) => deletes.push(p_idx),
            EditOp::Add(c_idx) => adds.push(c_idx),
            EditOp::Equal(_, _) => break,
        }
        k += 1;
    }
    (deletes, adds, k)
}

fn trim_common_affixes(pre_lines: &[&str], cur_lines: &[&str]) -> (usize, usize) {
    let mut prefix_len = 0;
    while prefix_len < pre_lines.len()
        && prefix_len < cur_lines.len()
        && pre_lines[prefix_len] == cur_lines[prefix_len]
    {
        prefix_len += 1;
    }

    let mut suffix_len = 0;
    while suffix_len < (pre_lines.len() - prefix_len)
        && suffix_len < (cur_lines.len() - prefix_len)
        && pre_lines[pre_lines.len() - 1 - suffix_len]
            == cur_lines[cur_lines.len() - 1 - suffix_len]
    {
        suffix_len += 1;
    }
    (prefix_len, suffix_len)
}

/// Computes a sequence-aligned line diff and accurate summary between `pre_content` and `cur_content`.
pub fn compute_aligned_diff(
    pre_content: &str,
    cur_content: &str,
) -> (Vec<AlignedDiffLine>, CasDiffSummary) {
    let pre_lines = split_normalized_lines(pre_content);
    let cur_lines = split_normalized_lines(cur_content);
    if pre_lines.is_empty() && cur_lines.is_empty() {
        return (Vec::new(), CasDiffSummary::default());
    }

    let (prefix_len, suffix_len) = trim_common_affixes(&pre_lines, &cur_lines);
    let mid_pre = &pre_lines[prefix_len..pre_lines.len() - suffix_len];
    let mid_cur = &cur_lines[prefix_len..cur_lines.len() - suffix_len];
    let mut result = Vec::with_capacity(pre_lines.len().max(cur_lines.len()));
    let mut counts = (0usize, 0usize);

    for (idx, &line) in pre_lines.iter().enumerate().take(prefix_len) {
        let next_no = result.len() + 1;
        result.push(make_unchanged_row(next_no, idx, idx, line));
    }

    let has_mid = !mid_pre.is_empty() || !mid_cur.is_empty();
    let use_fallback =
        mid_pre.is_empty() || mid_cur.is_empty() || mid_pre.len() * mid_cur.len() > 4_000_000;
    if has_mid && use_fallback {
        emit_linear_fallback(
            mid_pre,
            mid_cur,
            prefix_len,
            &pre_lines,
            &cur_lines,
            &mut result,
            &mut counts,
        );
    } else if has_mid {
        let dp = build_lcs_table(mid_pre, mid_cur);
        let ops = backtrack_lcs_ops(&dp, mid_pre, mid_cur, prefix_len);
        emit_edit_ops(&ops, &pre_lines, &cur_lines, &mut result, &mut counts);
    }

    let pre_suf = pre_lines.len() - suffix_len;
    let cur_suf = cur_lines.len() - suffix_len;
    for idx in 0..suffix_len {
        let p_idx = pre_suf + idx;
        let c_idx = cur_suf + idx;
        let next_no = result.len() + 1;
        result.push(make_unchanged_row(next_no, p_idx, c_idx, pre_lines[p_idx]));
    }

    let summary = CasDiffSummary {
        additions: counts.0,
        deletions: counts.1,
        changes: counts.0.min(counts.1),
    };
    (result, summary)
}
