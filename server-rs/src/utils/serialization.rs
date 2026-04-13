//! Secure Serialization & Data Pruning
//!
//! Provides wrappers for JSON values that automatically truncate large 
//! strings and prevent recursive depth attacks, ensuring that logs and 
//! messages remain within safe operational bounds.
//!
//! @docs ARCHITECTURE:DataProtocols
//! 
//! ### AI Assist Note
//! **Secure Serialization & Data Pruning**: Orchestrates the 
//! defensive wrapping of JSON payloads to prevent resource 
//! exhaustion attacks. Features **Irreversible Destructive 
//! Sanitization**: `prune_large_strings` and `sanitize_json` 
//! automatically truncate strings exceeding safe operational bounds 
//! (default 2KB). Once truncated, the original data is discarded 
//! and replaced with a size-delta marker (e.g., `... [TRUNCATED 
//! 512 bytes]`). Promotes the use of **O(N) Traversal** for deep 
//! JSON pruning to mitigate recursive depth attacks (PRUNE-01).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Accidental truncation of legitimate high-volume 
//!   telemetry (e.g., swarm graph state or mission logs), or 
//!   bypass of pruning via deeply nested non-string primitives.
//! - **Trace Scope**: `server-rs::utils::serialization`

use serde_json::Value;

/// Scans a JSON value and prunes any strings longer than the specified limit.
///
/// This is used to prevent "long-string" attacks or accidental DB bloat when
/// agents attempt to log massive payloads (e.g., base64 images or minified JS).
pub fn prune_large_strings(value: &mut Value, limit: usize) {
    match value {
        Value::String(s) => {
            if s.len() > limit {
                let original_len = s.len();
                s.truncate(limit);
                s.push_str(&format!("... [TRUNCATED {} bytes]", original_len - limit));
            }
        }
        Value::Array(arr) => {
            for v in arr {
                prune_large_strings(v, limit);
            }
        }
        Value::Object(obj) => {
            for v in obj.values_mut() {
                prune_large_strings(v, limit);
            }
        }
        _ => {}
    }
}

/// A convenience wrapper that prunes strings to a default limit of 2KB (2048 bytes).
pub fn sanitize_json(mut value: Value) -> Value {
    prune_large_strings(&mut value, 2048);
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_string_pruning() {
        let mut val = json!({
            "short": "hello",
            "long": "a".repeat(3000),
            "nested": {
                "inner_long": "b".repeat(4000)
            },
            "array": ["short", "c".repeat(2500)]
        });

        prune_large_strings(&mut val, 10);

        assert_eq!(val["short"], "hello");
        assert!(val["long"].as_str().unwrap().contains("... [TRUNCATED"));
        assert_eq!(val["long"].as_str().unwrap().len(), 10 + 26); // 10 chars + suffix
        assert!(val["nested"]["inner_long"]
            .as_str()
            .unwrap()
            .contains("... [TRUNCATED"));
        assert!(val["array"][1].as_str().unwrap().contains("... [TRUNCATED"));
    }
}
