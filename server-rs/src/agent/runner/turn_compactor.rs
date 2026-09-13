//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / turn_compactor
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[compactor]`
//! - **Witness Tests**: none declared

pub fn build_sandboxed_transcript(role: &str, raw_history: &[String]) -> Vec<String> {
    tracing::debug!("[compactor] Compacting transcript turns for role={}", role);
    let mut clean = Vec::new();
    for msg in raw_history {
        // Event Isolation Scope: restrict visibility of internal/private events to supervisor nodes
        if (msg.contains("[SystemOnly]") || msg.contains("[Private]"))
            && !role.eq_ignore_ascii_case("CEO")
            && !role.eq_ignore_ascii_case("Alpha")
        {
            continue;
        }

        let mut clean_msg = msg
            .replace("<halting_signal/>", "")
            .replace("<halt/>", "")
            .replace("<thinking>", "")
            .replace("</thinking>", "")
            .trim()
            .to_string();

        if (clean_msg.starts_with("OBSERVATION:") || clean_msg.starts_with("TOOL OUTPUT:"))
            && clean_msg.len() > 300
        {
            clean_msg = format!(
                "{}... [TRUNCATED TOOL OUTPUT]",
                super::safe_truncate_str(&clean_msg, 300)
            );
        }
        clean.push(clean_msg);
    }
    clean
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_sandboxed_transcript_truncation() {
        let history = vec![
            "USER: Hello".to_string(),
            format!("OBSERVATION: {}", "A".repeat(500)),
        ];
        let result = build_sandboxed_transcript("Specialist", &history);
        assert_eq!(result.len(), 2);
        assert!(result[1].contains("[TRUNCATED TOOL OUTPUT]"));
    }

    #[test]
    fn test_build_sandboxed_transcript_privacy_filter() {
        let history = vec![
            "[Private] Secret data".to_string(),
            "Public message".to_string(),
        ];
        // Non-supervisor role
        let result = build_sandboxed_transcript("Worker", &history);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Public message");

        // Supervisor role (Alpha)
        let result_alpha = build_sandboxed_transcript("Alpha", &history);
        assert_eq!(result_alpha.len(), 2);
    }
}
