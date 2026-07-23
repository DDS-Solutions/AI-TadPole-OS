//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Loop Detector**: Prevents infinite tool call cycles using signature hashes.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: JSON normalization depth limit exceedance or hash collisions.
//!

use sha2::{Digest, Sha256};

pub(crate) fn normalize_json(val: &serde_json::Value) -> String {
    normalize_json_inner(val, 0)
}

fn normalize_json_inner(val: &serde_json::Value, depth: usize) -> String {
    if depth > 32 {
        return "\"[DEPTH_EXCEEDED]\"".to_string();
    }
    match val {
        serde_json::Value::Object(map) => {
            let mut sorted_keys: Vec<_> = map.keys().collect();
            sorted_keys.sort();
            let mut parts = Vec::new();
            for key in sorted_keys {
                let normalized_val = normalize_json_inner(&map[key], depth + 1);
                parts.push(format!("\"{}\":{}", key, normalized_val));
            }
            format!("{{{}}}", parts.join(","))
        }
        serde_json::Value::Array(arr) => {
            let parts: Vec<_> = arr
                .iter()
                .map(|v| normalize_json_inner(v, depth + 1))
                .collect();
            format!("[{}]", parts.join(","))
        }
        _ => val.to_string(),
    }
}

#[derive(Debug)]
pub struct DoomLoopDetector {
    signatures: std::collections::VecDeque<String>,
}

impl DoomLoopDetector {
    pub fn new() -> Self {
        Self {
            signatures: std::collections::VecDeque::new(),
        }
    }

    pub fn check(&mut self, agent_id: &str, tool_name: &str, arguments: &str, output: &str) -> bool {
        let normalized_args = match serde_json::from_str::<serde_json::Value>(arguments) {
            Ok(val) => normalize_json(&val),
            Err(_) => arguments.trim().to_string(),
        };
 
        let mut hasher = Sha256::new();
        hasher.update(output.as_bytes());
        let output_hash = hex::encode(hasher.finalize());
 
        let sig = format!("{}:{}:{}:{}", agent_id, tool_name, normalized_args, &output_hash[..16]);
        self.signatures.push_back(sig);
 
        if self.signatures.len() > 12 {
            self.signatures.pop_front();
        }
 
        self.has_loop()
    }

    fn has_loop(&self) -> bool {
        let n = self.signatures.len();
        for period in 1..=4 {
            let reps = if period == 1 { 3 } else { 2 };
            let needed = period * reps;
            if n < needed {
                continue;
            }

            let start_idx = n - needed;
            let mut is_loop = true;
            for i in 0..needed {
                if self.signatures[start_idx + i] != self.signatures[start_idx + (i % period)] {
                    is_loop = false;
                    break;
                }
            }
            if is_loop {
                return true;
            }
        }
        false
    }
}
