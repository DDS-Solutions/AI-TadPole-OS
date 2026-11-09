//! @docs ARCHITECTURE:CodeBaseIntelligence
//! ### AI Assist Note
//! Atomic auto-fixer for header docstrings against symbol suggestions.
//! ### 🔍 Debugging & Observability
//! Enforces SHA256 concurrency bounds and atomic file renaming to prevent partial writes.

use crate::parser::SymbolRange;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct AutoFixContext<'a> {
    pub full_path: &'a Path,
    pub real_path: &'a str,
    pub initial_hash: Option<&'a [u8]>,
    pub content: &'a str,
}

fn verify_content_hash(full_path: &Path, expected_hash: Option<&[u8]>) -> bool {
    let Some(expected) = expected_hash else {
        return true;
    };
    let current_bytes = fs::read(full_path).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(&current_bytes);
    hasher.finalize().as_slice() == expected
}

fn apply_single_edit(
    content: &mut String,
    range: &SymbolRange,
    sym: &str,
    suggested: &str,
) -> bool {
    if range.start_byte >= content.len()
        || range.end_byte > content.len()
        || !content.is_char_boundary(range.start_byte)
        || !content.is_char_boundary(range.end_byte)
    {
        return false;
    }

    let orig_doc = &content[range.start_byte..range.end_byte];
    let target_backtick = format!("`{sym}`");
    let rep_backtick = format!("`{suggested}`");

    let mut new_doc = orig_doc.replace(&target_backtick, &rep_backtick);
    new_doc = new_doc.replace(&format!("[[{sym}"), &format!("[[{suggested}"));

    if new_doc != orig_doc {
        content.replace_range(range.start_byte..range.end_byte, &new_doc);
        return true;
    }
    false
}

#[allow(clippy::needless_borrows_for_generic_args)]
fn write_atomic_replacement(full_path: &Path, real_path: &str, content: &str) -> bool {
    let tmp_path = full_path.with_extension("docguard.tmp");
    if let Err(e) = fs::write(&tmp_path, content) {
        println!(
            "⚠️ [ERROR] Failed to write auto-fixes to temporary file {}: {e}",
            tmp_path.display()
        );
        return false;
    }
    if let Err(e) = fs::rename(&tmp_path, &full_path) {
        println!("⚠️ [ERROR] Failed to atomically rename auto-fixes to {real_path}: {e}");
        let _ = fs::remove_file(&tmp_path);
        return false;
    }
    true
}

fn filter_fixed_symbols(
    missing_symbols: Vec<String>,
    mut applied: HashMap<String, usize>,
) -> Vec<String> {
    let mut remaining = Vec::new();
    for s in missing_symbols {
        if let Some(count) = applied.get_mut(&s) {
            if *count > 0 {
                *count -= 1;
                continue;
            }
        }
        remaining.push(s);
    }
    remaining
}

pub fn apply_autofixes(
    ctx: AutoFixContext<'_>,
    mut edits: Vec<(SymbolRange, String, String)>,
    missing_symbols: Vec<String>,
) -> Vec<String> {
    const YELLOW: &str = "\x1b[93m";
    const RESET: &str = "\x1b[0m";

    if !verify_content_hash(ctx.full_path, ctx.initial_hash) {
        println!("{YELLOW}[WARN] [ADG-SG] {}: content changed on disk after scan; refusing to auto-fix.{RESET}", ctx.real_path);
        return missing_symbols;
    }

    edits.sort_by_key(|e| std::cmp::Reverse(e.0.start_byte));
    let mut applied_occurrences: HashMap<String, usize> = HashMap::new();
    let mut new_content = ctx.content.to_string();
    let mut dirty = false;

    for (range, sym, suggested) in &edits {
        if apply_single_edit(&mut new_content, range, sym, suggested) {
            dirty = true;
            *applied_occurrences.entry(sym.to_string()).or_insert(0) += 1;
            println!(
                "   🔧 [FIXED] Replaced `{sym}` with `{suggested}` in {}",
                ctx.real_path
            );
        }
    }

    if dirty && write_atomic_replacement(ctx.full_path, ctx.real_path, &new_content) {
        return filter_fixed_symbols(missing_symbols, applied_occurrences);
    }

    missing_symbols
}
