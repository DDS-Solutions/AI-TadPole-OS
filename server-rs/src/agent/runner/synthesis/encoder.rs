use std::collections::HashMap;

/// 📟 [AAAK Encoder]
/// Compresses mission context using the MemPalace-inspired AAAK dialect
/// to increase context fidelity while reducing token load.
pub fn aaak_encode(text: &str) -> String {
    let mut replacements = HashMap::new();
    replacements.insert("STATUS: ok", "*ok*");
    replacements.insert("STATUS: success", "*ok*");
    replacements.insert("STATUS: failed", "*err*");
    replacements.insert("STATUS: error", "*err*");
    replacements.insert("RESULT:", "RES:");
    replacements.insert("FINDING:", "FND:");
    replacements.insert("SOURCE:", "SRC:");
    replacements.insert("Location:", "LOC:");
    replacements.insert("Primary Goal:", "GOAL:");
    replacements.insert("Weather for zip", "WTR|");
    replacements.insert("degrees", "deg");
    replacements.insert("temperature", "temp");
    replacements.insert("Finding for mission", "FND|");
    replacements.insert("Strategic Intent:", "INT:");
    replacements.insert("Mission Complete", "*done*");
    replacements.insert("Task in progress", "*busy*");

    let mut encoded = text.to_string();
    for (pattern, replacement) in replacements {
        encoded = encoded.replace(pattern, replacement);
    }
    encoded
}
