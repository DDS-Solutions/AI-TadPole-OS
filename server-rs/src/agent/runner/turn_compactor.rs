//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / turn_compactor
//!
//! ### AI Assist Note
//! Keep bounded tool evidence visible to the next turn; offload oversized results.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[compactor]`

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
            // If the observation points to an offloaded file, preserve the pointer and guidance intact
            if !clean_msg.contains("tool_overflow") && !clean_msg.contains("Content too large") {
                clean_msg = format!(
                    "{}... [TRUNCATED TOOL OUTPUT: {} chars total]",
                    super::safe_truncate_str(&clean_msg, 300),
                    clean_msg.len()
                );
            }
        }
        clean.push(clean_msg);
    }
    clean
}

use std::path::{Path, PathBuf};

const MAX_INLINE_OBSERVATION_BYTES: usize = 24_000;

/// Retains bounded execution evidence inline. Oversized output is saved to
/// `.tmp/tool_overflow/` with a compact reference for later retrieval.
pub async fn compact_and_offload_observation(
    tool_name: &str,
    raw_output: &str,
    workspace_root: &Path,
) -> (String, Option<PathBuf>) {
    if raw_output.len() <= MAX_INLINE_OBSERVATION_BYTES {
        return (format!("OBSERVATION: {}", raw_output), None);
    }

    let overflow_dir = workspace_root.join(".tmp").join("tool_overflow");
    if let Err(e) = tokio::fs::create_dir_all(&overflow_dir).await {
        tracing::warn!("[compactor] Failed to create overflow directory: {}", e);
        let preview = super::safe_truncate_str(raw_output, 300);
        return (
            format!(
                "OBSERVATION: {}... [TRUNCATED TOOL OUTPUT: {} chars total]",
                preview,
                raw_output.len()
            ),
            None,
        );
    }

    let timestamp = chrono::Utc::now().timestamp_millis();
    let short_id = uuid::Uuid::new_v4()
        .to_string()
        .chars()
        .take(8)
        .collect::<String>();
    let clean_tool = tool_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "");
    let filename = format!("tool_output_{}_{}_{}.txt", clean_tool, timestamp, short_id);
    let target_path = overflow_dir.join(&filename);

    if let Err(e) = tokio::fs::write(&target_path, raw_output).await {
        tracing::warn!("[compactor] Failed to write overflow file: {}", e);
        let preview = super::safe_truncate_str(raw_output, 300);
        return (
            format!(
                "OBSERVATION: {}... [TRUNCATED TOOL OUTPUT: {} chars total]",
                preview,
                raw_output.len()
            ),
            None,
        );
    }

    let rel_path = format!(".tmp/tool_overflow/{}", filename);
    let preview = super::safe_truncate_str(raw_output, 300);
    let compacted = format!(
        "OBSERVATION: Content too large. Full output ({} bytes) saved to: {}\n\nPreview:\n{}\n\nGuidance: Use file read tools on {} with specific line ranges if exact details are needed.",
        raw_output.len(),
        rel_path,
        preview,
        rel_path
    );

    (compacted, Some(target_path))
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
        assert!(result[1].contains("[TRUNCATED TOOL OUTPUT"));
    }

    #[test]
    fn test_build_sandboxed_transcript_preserves_offload_pointer() {
        let history = vec![
            "USER: Scan repo".to_string(),
            format!(
                "OBSERVATION: Content too large. Full output (50000 bytes) saved to: .tmp/tool_overflow/tool_output_read_file_123.txt\n\nPreview:\n{}",
                "A".repeat(500)
            ),
        ];
        let result = build_sandboxed_transcript("Specialist", &history);
        assert_eq!(result.len(), 2);
        assert!(result[1].contains(".tmp/tool_overflow/tool_output_read_file_123.txt"));
        assert!(!result[1].contains("[TRUNCATED TOOL OUTPUT"));
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

    #[tokio::test]
    async fn test_compact_and_offload_observation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let ws_root = temp_dir.path();

        // Short output stays inline.
        let short_out = "Command succeeded in 12ms.";
        let (obs, path_opt) =
            compact_and_offload_observation("test_tool", short_out, ws_root).await;
        assert_eq!(obs, format!("OBSERVATION: {}", short_out));
        assert!(path_opt.is_none());

        // Source evidence beyond the old 300-character cutoff stays available.
        let source = "pub fn inspect() { /* bounded source evidence */ }\n".repeat(80);
        let (obs, path_opt) =
            compact_and_offload_observation("read_codebase_file", &source, ws_root).await;
        assert_eq!(obs, format!("OBSERVATION: {source}"));
        assert!(path_opt.is_none());

        let unicode = "日".repeat(MAX_INLINE_OBSERVATION_BYTES / 3);
        let (obs, path_opt) =
            compact_and_offload_observation("read_codebase_file", &unicode, ws_root).await;
        assert!(obs.ends_with(&unicode));
        assert!(path_opt.is_none());

        // The byte limit still bounds multibyte content and preserves overflow.
        let large_out = format!("{unicode}日");
        let (obs, path_opt) =
            compact_and_offload_observation("test_tool", &large_out, ws_root).await;
        assert!(obs.contains("Content too large"));
        assert!(obs.contains(".tmp/tool_overflow/"));
        assert!(path_opt.is_some());

        let written_path = path_opt.unwrap();
        assert!(written_path.exists());
        let saved_content = tokio::fs::read_to_string(&written_path).await.unwrap();
        assert_eq!(saved_content, large_out);
    }
}
