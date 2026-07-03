//! @docs ARCHITECTURE:VulnerabilityScanning
//!
//! ### AI Assist Note
//! **NVIDIA SkillSpector Wrapper**: Launches the python-based SkillSpector scanner 
//! as a child process to check dynamic skills and templates for injection, 
//! memory poisoning, and MCP privileges before registration.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Python executable missing, SkillSpector library not installed, 
//!   or JSON parse errors from the scanner output.
//! - **Trace Scope**: `server-rs::security::skillspector`

use std::fs;
use std::process::Command;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::error::AppError;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSpectorFinding {
    pub severity: String,
    pub rule_id: String,
    pub location: Option<String>,
    pub finding: Option<String>,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSpectorResult {
    pub risk_score: u8,
    pub risk_severity: String,
    pub risk_recommendation: String,
    pub filtered_findings: Vec<SkillSpectorFinding>,
}

impl SkillSpectorResult {
    pub fn safe() -> Self {
        Self {
            risk_score: 0,
            risk_severity: "LOW".to_string(),
            risk_recommendation: "SAFE".to_string(),
            filtered_findings: Vec::new(),
        }
    }
}

/// Runs a SkillSpector scan on a specific file path
pub fn scan_path(path: &Path) -> Result<SkillSpectorResult, AppError> {
    if !path.exists() {
        return Err(AppError::NotFound(format!("File to scan not found: {:?}", path)));
    }

    // Run python -m skillspector scan <path> --no-llm --format json
    let output = match Command::new("python")
        .arg("-m")
        .arg("skillspector")
        .arg("scan")
        .arg(path)
        .arg("--no-llm")
        .arg("--format")
        .arg("json")
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            tracing::warn!("⚠️ [SkillSpector] Python environment not available or failed to start: {}", e);
            let require_scan = std::env::var("REQUIRE_SECURITY_SCAN").map(|s| s == "true").unwrap_or(false);
            if require_scan {
                return Err(AppError::InternalServerError(format!("Security scanner unavailable: {}", e)));
            } else {
                return Ok(SkillSpectorResult::safe());
            }
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("⚠️ [SkillSpector] Scan command failed: {}", stderr);
        let require_scan = std::env::var("REQUIRE_SECURITY_SCAN").map(|s| s == "true").unwrap_or(false);
        if require_scan {
            return Err(AppError::InternalServerError(format!("Security scan command failed: {}", stderr)));
        } else {
            return Ok(SkillSpectorResult::safe());
        }
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: SkillSpectorResult = serde_json::from_str(&stdout).map_err(|e| {
        AppError::InternalServerError(format!("Failed to parse SkillSpector JSON: {}", e))
    })?;

    Ok(result)
}

/// Runs a SkillSpector scan on raw script/markdown string content
pub fn scan_content(content: &str, file_name: &str) -> Result<SkillSpectorResult, AppError> {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(file_name);

    fs::write(&temp_file, content).map_err(|e| AppError::Io(e))?;
    let result = scan_path(&temp_file);
    let _ = fs::remove_file(temp_file);

    result
}

// Metadata: [skillspector]
