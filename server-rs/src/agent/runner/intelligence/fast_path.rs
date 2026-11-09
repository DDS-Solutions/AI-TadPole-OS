//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / fast_path
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

static INQUIRY_PREFIXES: &[&str] = &[
    "what is",
    "what are",
    "why",
    "how does",
    "how do",
    "how can",
    "explain",
    "tell me about",
    "where is",
    "can you explain",
    "could you explain",
];

static SIMPLE_TRIGGERS: &[&str] = &[
    "what is",
    "get status",
    "read file",
    "list files",
    "check budget",
    "show metrics",
    "get metrics",
    "health",
    "system details",
    "who are you",
];

static COMPLEX_WORDS: &[&str] = &[
    "implement",
    "build",
    "refactor",
    "debug",
    "compile",
    "migrate",
    "deploy",
    "benchmark",
    "integrate",
    "solve",
    "fix",
    "add",
    "create",
    "delete",
    "remove",
    "update",
    "patch",
    "write",
    "modify",
    "change",
];

static COMPLEX_PHRASES: &[&str] = &["run tests", "stress test"];

static INTERROGATIVE_PREFIXES: &[&str] = &[
    "why", "how", "when", "where", "who", "explain", "tell me", "show",
];

fn is_inquiry_query(lower: &str) -> bool {
    INQUIRY_PREFIXES
        .iter()
        .any(|prefix| lower.starts_with(prefix))
}

fn has_simple_trigger(lower: &str) -> bool {
    SIMPLE_TRIGGERS
        .iter()
        .any(|trigger| lower.contains(trigger))
}

fn has_complex_action_word(lower: &str) -> bool {
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|s| !s.is_empty())
        .collect();

    COMPLEX_WORDS.iter().any(|word| words.contains(word))
        || COMPLEX_PHRASES.iter().any(|phrase| lower.contains(phrase))
}

fn is_interrogative_query(lower: &str) -> bool {
    INTERROGATIVE_PREFIXES
        .iter()
        .any(|prefix| lower.starts_with(prefix))
        || lower.ends_with('?')
}

pub fn is_fast_path_query(message: &str) -> bool {
    let lower = message.to_lowercase();
    let is_short = message.len() < 150;

    if is_short && is_inquiry_query(&lower) {
        return true;
    }
    if has_simple_trigger(&lower) {
        return true;
    }
    if has_complex_action_word(&lower) {
        return false;
    }
    is_short && is_interrogative_query(&lower)
}
