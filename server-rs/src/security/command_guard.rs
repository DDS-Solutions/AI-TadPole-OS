//! @docs ARCHITECTURE:Security:CommandGuard
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security / CommandGuard
//! - **Primary Entrypoints**: `validate_shell_command`, `parse_command_tokens`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Enforces zero-trust command whitelisting; blocks command separators, wildcards, expansions, and option escapes.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::AppError;

/// Helper tokenizer to safely extract command tokens while respecting single and double quotes.
pub fn parse_command_tokens(command: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    let mut chars = command.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '\\' => {
                if let Some(&next_c) = chars.peek() {
                    if next_c == '"' || next_c == '\'' || next_c == '\\' || next_c == ' ' {
                        current.push(next_c);
                        chars.next();
                    } else {
                        current.push('\\');
                    }
                } else {
                    current.push('\\');
                }
            }
            c if c.is_whitespace() && !in_double_quote && !in_single_quote => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            c => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Validates a shell command against a ZERO-TRUST whitelist, preventing separators (S-001) and subprocess RCEs.
pub fn validate_shell_command(command: &str) -> Result<(), AppError> {
    let lower = command.to_lowercase();

    // 1. Block Newline, Carriage Return, Null Byte (RCE separators - S-001)
    if lower.contains('\n') || lower.contains('\r') || lower.contains('\0') {
        return Err(AppError::Forbidden(
            "Newline, carriage return, or null bytes are prohibited in commands".to_string(),
        ));
    }

    // 2. Block Shell Comments: Reject '#' to prevent validation vs execution divergence
    if lower.contains('#') {
        return Err(AppError::Forbidden(
            "Shell comments ('#') are prohibited in commands to prevent execution divergence"
                .to_string(),
        ));
    }

    // 3. Block Command Substitution & Variable/Environment Expansion (Critical Vulnerability)
    if lower.contains("$(") || lower.contains('`') || lower.contains("${") {
        return Err(AppError::Forbidden(
            "Command substitution or variable expansion detected".to_string(),
        ));
    }

    // Disallow bare $ and ~ expansion (except $null in output redirection)
    if lower.contains('$') {
        let stripped = lower
            .replace("> $null", "")
            .replace(">$null", "")
            .replace("2>$null", "")
            .replace("1>$null", "");
        if stripped.contains('$') {
            return Err(AppError::Forbidden(
                "Environment variable expansion ('$') is prohibited in commands".to_string(),
            ));
        }
    }

    if lower
        .split_whitespace()
        .any(|t| t == "~" || t.starts_with("~/") || t.starts_with("~\\"))
    {
        return Err(AppError::Forbidden(
            "Tilde user directory expansion ('~') is prohibited in commands".to_string(),
        ));
    }

    // 4. Block Piping, Chaining, and Input Redirection (including Unicode variants)
    if lower.contains('|')
        || lower.contains('<')
        || lower.contains(';')
        || lower.contains('&')
        || lower.contains('\u{FF1B}')
        || lower.contains('\u{FF5C}')
    {
        return Err(AppError::Forbidden(
            "Piping, chaining, or input redirection prohibited".to_string(),
        ));
    }

    // 5. Harden Output Redirection: Only allow redirection to /dev/null, $null, or standard error/output descriptors
    if lower.contains('>') {
        let mut rest = lower.as_str();
        while let Some(idx) = rest.find('>') {
            let redirect_target = rest[idx + 1..].trim();
            let target_token = redirect_target.split_whitespace().next().unwrap_or("");
            if target_token != "/dev/null"
                && target_token != "$null"
                && !target_token.starts_with("&1")
                && !target_token.starts_with("&2")
            {
                return Err(AppError::Forbidden(
                    "Output redirection is only permitted to /dev/null or $null".to_string(),
                ));
            }
            rest = &rest[idx + 1..];
        }
    }

    // 6. Block wildcard character glob expansion to prevent sandbox blacklist bypasses
    if lower.contains('*') || lower.contains('?') || lower.contains('[') || lower.contains(']') {
        return Err(AppError::Forbidden(
            "Wildcard characters (*, ?, []) are prohibited in commands to prevent sandbox bypass"
                .to_string(),
        ));
    }

    // 7. Whitelist of Allowed Base Commands
    let allowed_commands = [
        "ls", "cd", "pwd", "cat", "echo", "grep", "find", "cargo", "npm", "pnpm", "git", "python",
        "node", "rustc", "mkdir", "cp", "mv", "touch", "test",
    ];

    let tokens = parse_command_tokens(command);
    let first_word = match tokens.first() {
        Some(w) => w.to_lowercase(),
        None => {
            return Err(AppError::Forbidden(
                "Empty command is prohibited".to_string(),
            ))
        }
    };

    if !allowed_commands.contains(&first_word.as_str()) {
        return Err(AppError::Forbidden(format!(
            "Command '{}' is not in the authorized whitelist",
            first_word
        )));
    }

    // 8. Command-specific option restrictions to prevent sub-shell escapes (S-002, S-003, S-004)
    if first_word == "find" {
        let dangerous_find = [
            "-exec", "-execdir", "-ok", "-okdir", "-fprint", "-fprintf", "-delete",
        ];
        for flag in dangerous_find {
            if tokens.iter().any(|t| {
                let tl = t.to_lowercase();
                tl == flag || tl.starts_with(&format!("{}=", flag))
            }) {
                return Err(AppError::Forbidden(format!(
                    "Unauthorized find parameter: '{}'",
                    flag
                )));
            }
        }
    } else if first_word == "node" {
        // Grouped options check (e.g. -pe)
        for token in &tokens {
            if token.starts_with('-') && !token.starts_with("--") {
                for c in token[1..].chars() {
                    if ['e', 'p', 'i', 'r'].contains(&c) {
                        return Err(AppError::Forbidden(format!(
                            "Unauthorized node option: '-{}'",
                            c
                        )));
                    }
                }
            } else {
                let tl = token.to_lowercase();
                if tl == "--eval"
                    || tl.starts_with("--eval=")
                    || tl == "--print"
                    || tl.starts_with("--print=")
                    || tl == "--interactive"
                    || tl.starts_with("--interactive=")
                    || tl == "--require"
                    || tl.starts_with("--require=")
                {
                    return Err(AppError::Forbidden(format!(
                        "Unauthorized node option: '{}'",
                        token
                    )));
                }
            }
        }
    } else if first_word == "python" {
        for token in &tokens {
            if token.starts_with('-') && !token.starts_with("--") {
                for c in token[1..].chars() {
                    if ['c', 'm', 'i'].contains(&c) {
                        return Err(AppError::Forbidden(format!(
                            "Unauthorized python option: '-{}'",
                            c
                        )));
                    }
                }
            }
        }
    } else if first_word == "git" {
        // Parse argv token boundaries (V2-005) to block config/exec/etc. and transport protocols.
        let dangerous_git_subcommands = ["config", "exec", "upload-pack", "receive-pack"];
        let dangerous_git_flags = ["-c", "--git-dir", "--work-tree", "core.pager", "--ext-diff"];

        for token in &tokens {
            let tl = token.to_lowercase();
            if dangerous_git_subcommands.contains(&tl.as_str()) {
                return Err(AppError::Forbidden(format!(
                    "Unauthorized git parameter or option detected: '{}'",
                    token
                )));
            }
            if dangerous_git_flags
                .iter()
                .any(|&flag| tl == flag || tl.starts_with(&format!("{}=", flag)))
            {
                return Err(AppError::Forbidden(format!(
                    "Unauthorized git parameter or option detected: '{}'",
                    token
                )));
            }
            // Check for transport protocols (e.g. ext::, ssh::, git::, etc. or url variants)
            if tl.contains("ext::")
                || tl.contains("ssh::")
                || tl.contains("git::")
                || tl.contains("file://")
                || tl.contains("ssh://")
                || tl.contains("git://")
            {
                return Err(AppError::Forbidden(
                    "Unauthorized git transport protocol prefix detected".to_string(),
                ));
            }
        }
    } else if first_word == "npm" {
        let dangerous_npm = [
            "run", "exec", "install", "i", "config", "ci", "update", "link",
        ];
        for token in &tokens {
            let tl = token.to_lowercase();
            if dangerous_npm.contains(&tl.as_str()) {
                return Err(AppError::Forbidden(format!(
                    "Unauthorized npm parameter or option detected: '{}'",
                    token
                )));
            }
        }
    } else if first_word == "cargo" {
        let dangerous_cargo = ["run", "test", "bench"];
        for token in &tokens {
            let tl = token.to_lowercase();
            if dangerous_cargo.contains(&tl.as_str()) {
                return Err(AppError::Forbidden(format!(
                    "Unauthorized cargo command detected: '{}'",
                    token
                )));
            }
        }
    }

    // 9. Blacklist specific dangerous flags/paths for allowed commands
    let dangerous_flags = [
        "--erase",
        "--delete",
        "-rf",
        "/etc",
        "/root",
        "/var",
        "/bin",
        "/usr",
        "tadpole.db",
        ".env",
        ".gemini",
        "knowledge",
    ];
    for flag in dangerous_flags {
        if tokens.iter().any(|t| t.to_lowercase().contains(flag)) {
            return Err(AppError::Forbidden(format!(
                "Dangerous flag or path detected: '{}'",
                flag
            )));
        }
    }

    Ok(())
}
