//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / encoder
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

static AAAK_REPLACEMENTS: &[(&str, &str)] = &[
    ("Finding for mission", "FND|"),
    ("Weather for zip", "WTR|"),
    ("Strategic Intent:", "INT:"),
    ("Task in progress", "*busy*"),
    ("Mission Complete", "*done*"),
    ("STATUS: completed", "*done*"),
    ("STATUS: success", "*ok*"),
    ("STATUS: failed", "*err*"),
    ("STATUS: error", "*err*"),
    ("STATUS: ok", "*ok*"),
    ("Primary Goal:", "GOAL:"),
    ("TOOL_CALL:", "TC:"),
    ("OBJECTIVE:", "OBJ:"),
    ("FINDING:", "FND:"),
    ("RESULT:", "RES:"),
    ("SOURCE:", "SRC:"),
    ("Location:", "LOC:"),
    // Trailing word-boundary checks for isolated tokens
    (" degrees ", " deg "),
    (" temperature ", " temp "),
];

/// 📟 [AAAK Encoder]
/// Compresses mission context using a deterministic, ordered replacement table
/// to reduce token consumption while preserving structured JSON and code integrity.
pub fn aaak_encode(text: &str) -> String {
    let mut encoded = text.to_string();
    for &(pattern, replacement) in AAAK_REPLACEMENTS {
        encoded = encoded.replace(pattern, replacement);
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aaak_encode_deterministic() {
        let input =
            "Finding for mission: STATUS: success with Strategic Intent: test. Location: US";
        let out1 = aaak_encode(input);
        let out2 = aaak_encode(input);
        assert_eq!(out1, out2);
        assert!(out1.contains("FND|: *ok* with INT: test. LOC: US"));
    }

    #[test]
    fn test_aaak_preserves_json_keys() {
        let json_input = r#"{"temperature": 0.7, "degrees_of_freedom": 4}"#;
        let encoded = aaak_encode(json_input);
        // "temperature" without surrounding spaces should NOT be rewritten inside json key
        assert_eq!(json_input, encoded);
    }

    #[test]
    fn test_status_completed_mapping() {
        assert_eq!(aaak_encode("STATUS: completed"), "*done*");
        assert_eq!(aaak_encode("Mission Complete"), "*done*");
    }
}
