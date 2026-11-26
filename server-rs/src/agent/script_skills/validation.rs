//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! - **Subsystem**: Script skills / Artifact security validation
//! - Security scans and rejection thresholds run before any repository mutation.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::SkillDefinition;
use crate::error::AppError;

pub(super) async fn validate_skill(
    mut skill: SkillDefinition,
) -> Result<SkillDefinition, AppError> {
    let mut content_to_scan = String::new();
    content_to_scan.push_str(&skill.name);
    content_to_scan.push('\n');
    content_to_scan.push_str(&skill.description);
    content_to_scan.push('\n');
    if let Some(ref inst) = skill.full_instructions {
        content_to_scan.push_str(inst);
        content_to_scan.push('\n');
    }
    content_to_scan.push_str(&skill.execution_command);
    content_to_scan.push('\n');
    if let Some(ref script) = skill.verification_script {
        content_to_scan.push_str(script);
        content_to_scan.push('\n');
    }
    if let Some(ref constraints) = skill.negative_constraints {
        for c in constraints {
            content_to_scan.push_str(c);
            content_to_scan.push('\n');
        }
    }
    content_to_scan.push_str(&skill.schema.to_string());

    // 1. Primary documentation & metadata scan as Markdown
    let mut scan_res =
        crate::security::skillspector::scan_content(&content_to_scan, "SKILL.md").await?;

    // 2. [F2] Dedicated executable code scan preserving script AST type (.py / .sh / .ps1)
    if !skill.execution_command.trim().is_empty() {
        let cmd_lower = skill.execution_command.to_lowercase();
        let script_ext = if cmd_lower.contains("python")
            || cmd_lower.contains("import ")
            || cmd_lower.contains("def ")
        {
            "script.py"
        } else if cmd_lower.contains("powershell") || cmd_lower.contains(".ps1") {
            "script.ps1"
        } else {
            "script.sh"
        };

        let code_scan =
            crate::security::skillspector::scan_content(&skill.execution_command, script_ext)
                .await?;
        if code_scan.risk_score > scan_res.risk_score {
            scan_res.risk_score = code_scan.risk_score;
            scan_res.risk_severity = code_scan.risk_severity;
            scan_res.risk_recommendation = code_scan.risk_recommendation;
        }
        scan_res
            .filtered_findings
            .extend(code_scan.filtered_findings);
    }

    // 3. [R-c] Dedicated executable code scan for verification_script if present
    if let Some(ref script) = skill.verification_script {
        if !script.trim().is_empty() {
            let script_lower = script.to_lowercase();
            let verify_ext = if script_lower.contains("python")
                || script_lower.contains("import ")
                || script_lower.contains("def ")
            {
                "verify.py"
            } else if script_lower.contains("powershell") || script_lower.contains(".ps1") {
                "verify.ps1"
            } else {
                "verify.sh"
            };

            let verify_scan =
                crate::security::skillspector::scan_content(script, verify_ext).await?;
            if verify_scan.risk_score > scan_res.risk_score {
                scan_res.risk_score = verify_scan.risk_score;
                scan_res.risk_severity = verify_scan.risk_severity;
                scan_res.risk_recommendation = verify_scan.risk_recommendation;
            }
            scan_res
                .filtered_findings
                .extend(verify_scan.filtered_findings);
        }
    }

    skill.security_score = Some(scan_res.risk_score);
    skill.security_severity = Some(scan_res.risk_severity.clone());
    if scan_res.risk_score >= crate::security::skillspector::RISK_REJECT_THRESHOLD {
        return Err(AppError::Forbidden(format!(
                "Security audit failed: skill has a high-risk security score of {} (Severity: {}). Registration rejected.",
                scan_res.risk_score, scan_res.risk_severity
            )));
    }
    skill.security_report = Some(
        serde_json::to_value(&scan_res).unwrap_or(serde_json::json!({ "filtered_findings": [] })),
    );

    Ok(skill)
}

pub(super) async fn validate_content(
    content: &str,
    filename: &str,
    kind: &str,
) -> Result<(), AppError> {
    let scan = crate::security::skillspector::scan_content(content, filename).await?;
    if scan.risk_score >= crate::security::skillspector::RISK_REJECT_THRESHOLD {
        return Err(AppError::Forbidden(format!(
            "Security audit failed: {} has a high-risk security score of {} (Severity: {}). Registration rejected.",
            kind, scan.risk_score, scan.risk_severity
        )));
    }
    Ok(())
}
