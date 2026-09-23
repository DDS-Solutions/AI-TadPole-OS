//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Startup / CLI & Environment
//! - **Primary Entrypoints**: `BootstrapIntent`, `handle_admin_cli`, `detect_bootstrap_intent`, `load_environment`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

/// Configures the weight and scope of the engine boot sequence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BootstrapIntent {
    /// Full mission execution: Warm up all Code Graph, mDNS, and ingestion workers.
    Full,
    /// Fast Path: Skip heavy warm-up tasks for simple CLI status/version requests.
    Fast,
}

/// Loads environmental configuration and validates it against the schema.
pub fn load_environment() {
    check_sovereign_config();
}

/// Checks for critical AI provider API keys and issues a "Sovereign Warning" if missing.
fn check_sovereign_config() {
    let providers = [
        ("OPENAI_API_KEY", "OpenAI"),
        ("ANTHROPIC_API_KEY", "Anthropic"),
        ("GOOGLE_API_KEY", "Google Gemini"),
        ("GROQ_API_KEY", "Groq"),
    ];

    let mut missing = Vec::new();
    for (key, name) in providers {
        let is_missing = if let Ok(val) = std::env::var(key) {
            let trimmed = val.trim();
            trimmed.is_empty()
                || trimmed.eq_ignore_ascii_case("todo")
                || trimmed.contains("placeholder")
                || trimmed.contains("YOUR_KEY")
                || trimmed.len() < 10
        } else {
            true
        };
        if is_missing {
            missing.push(name);
        }
    }

    let privacy_mode = std::env::var("PRIVACY_MODE")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    if !missing.is_empty() && !privacy_mode {
        println!("\n\x1b[1;33m⚠️  [SOVEREIGN WARNING]\x1b[0m");
        println!("\x1b[1;33m--------------------------------------------------\x1b[0m");
        println!("The following AI providers are not configured:");
        for name in &missing {
            println!("  - {}", name);
        }
        println!("\nAI-Tadpole-OS will fall back to local models (Ollama) if available.");
        println!("To enable these providers, add your API keys to the \x1b[1m.env\x1b[0m file.");
        println!("See \x1b[1mdocs/GETTING_STARTED.md\x1b[0m for instructions.");
        println!("\x1b[1;33m--------------------------------------------------\x1b[0m\n");

        tracing::warn!(missing = ?missing, "Sovereign Warning: Some AI providers are not configured.");
    } else if privacy_mode {
        tracing::info!("🔒 [Privacy Guard] Running in strict local-only mode (Zero-Cloud).");
    }
}

/// Handles version/help administrative queries before full engine initialization.
pub fn handle_admin_cli(args: &[String]) -> anyhow::Result<Option<()>> {
    if args.iter().any(|arg| arg == "--version" || arg == "-v") {
        println!("Tadpole OS Engine v{}", env!("CARGO_PKG_VERSION"));
        return Ok(Some(()));
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("Tadpole OS - Sovereign AI Swarm Engine\n");
        println!("Usage: server-rs [OPTIONS]\n");
        println!("Options:");
        println!("  -v, --version    Show version and exit");
        println!("  -h, --help       Show this help and exit");
        println!("  --status         Show engine status and exit (Fast Path)");
        println!("  --port <PORT>    Set the port to listen on (Default: 8000)");
        return Ok(Some(()));
    }
    Ok(None)
}

/// Detects the bootstrap intent based on the command line arguments.
pub fn detect_bootstrap_intent(args: &[String]) -> BootstrapIntent {
    if args.iter().any(|arg| arg == "--status") {
        BootstrapIntent::Fast
    } else {
        BootstrapIntent::Full
    }
}
