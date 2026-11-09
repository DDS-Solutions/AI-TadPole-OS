//! @docs ARCHITECTURE:CodeBaseIntelligence
//! ### AI Assist Note
//! Structured report writer and terminal summary formatting for symbol validation.
//! ### 🔍 Debugging & Observability
//! Prints audit summaries and serializes machine-readable JSON reports.

use crate::GraphQueryError;
use std::fs;
use std::path::Path;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FileValidationFailure {
    pub file_path: String,
    pub missing_symbols: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ValidationReport {
    pub schema_version: u32,
    pub strict: bool,
    pub has_errors: bool,
    pub total_files_audited: usize,
    pub total_symbols_checked: usize,
    pub failed_files_count: usize,
    pub failures: Vec<FileValidationFailure>,
    pub anomalies: Vec<String>,
    pub anomaly_count: usize,
}

#[derive(Debug, Clone)]
pub struct DocGuardOptions<'a> {
    pub strict: bool,
    pub out: Option<&'a Path>,
    pub diff_only: bool,
    pub fix_enabled: bool,
}

pub struct AuditStats {
    pub total_files: usize,
    pub total_symbols: usize,
    pub failed_files: usize,
}

pub struct ReportPayload {
    pub strict: bool,
    pub has_errors: bool,
    pub failures: Vec<FileValidationFailure>,
    pub anomalies: Vec<String>,
}

pub fn print_audit_summary(stats: &AuditStats, anomalies: &[String], strict: bool) -> bool {
    const RED: &str = "\x1b[91m";
    const GREEN: &str = "\x1b[92m";
    const YELLOW: &str = "\x1b[93m";
    const RESET: &str = "\x1b[0m";

    println!("\n============================================================");
    println!("  ADG-SG AUDIT SUMMARY");
    println!("============================================================");
    println!("Total Files Audited: {}", stats.total_files);
    println!("Total Symbols Checked: {}", stats.total_symbols);
    println!("Failed Files: {}", stats.failed_files);

    if !anomalies.is_empty() {
        println!("\n{RED}[FAIL] Found {} structural codebase anomalies (unused symbols/dead code):{RESET}", anomalies.len());
        for anomaly in anomalies {
            println!("  - {anomaly}");
        }
    } else {
        println!("\n{GREEN}[OK] No structural codebase anomalies (dead code) detected.{RESET}");
    }

    let mut has_errors = false;
    if stats.failed_files > 0 {
        println!("\n{RED}[FAIL] Symbol-to-Header Parity check failed. Please align header docstrings with code.{RESET}");
        has_errors = true;
    }
    if !anomalies.is_empty() {
        if strict {
            println!("\n{RED}[FAIL] Unused/dead code check failed.{RESET}");
            has_errors = true;
        } else {
            println!("\n{YELLOW}[WARN] Unused/dead code check failed (Warning Only). Run with --strict to enforce.{RESET}");
        }
    }
    has_errors
}

pub fn save_report(
    out_path: &Path,
    stats: &AuditStats,
    payload: ReportPayload,
) -> Result<(), GraphQueryError> {
    let report = ValidationReport {
        schema_version: 1,
        strict: payload.strict,
        has_errors: payload.has_errors,
        total_files_audited: stats.total_files,
        total_symbols_checked: stats.total_symbols,
        failed_files_count: stats.failed_files,
        failures: payload.failures,
        anomaly_count: payload.anomalies.len(),
        anomalies: payload.anomalies,
    };
    let content = serde_json::to_string_pretty(&report).map_err(GraphQueryError::Serialization)?;
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(GraphQueryError::Io)?;
    }
    fs::write(out_path, content).map_err(GraphQueryError::Io)?;
    println!(
        "Saved structured validation report to: {}",
        out_path.display()
    );
    Ok(())
}
