//! @docs ARCHITECTURE:Registry
//!
//! ### AI Assist Note
//! - **Subsystem**: Sovereign Engine / Agent Runner / codebase
//! Codebase reads resolve their supplied target against the central project root.
//! Runner normalization supplies the absolute target authorized by the mission workspace.
//! Sensitivity checks use the verified mission-relative path for its own workspace.
//! Rejected reads propagate tool errors before successful audit or cache updates.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use super::require_str;
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{AgentRunner, RunContext};

/// Resolves the file path argument from a tool call, accepting both `path` and
/// `filename` as valid keys. Agents occasionally emit `filename` instead of
/// `path`; this alias prevents a hard `[FAILURE]` on an otherwise valid call.
///
/// Priority: `path` is checked first; `filename` is used as a fallback.
pub(super) fn require_path(
    ctx: &RunContext,
    args: &serde_json::Value,
    tool_name: &str,
) -> Result<String, ToolExecutionError> {
    // Priority: canonical "path" > "filename" > "file_name" > "file"
    for alias in &["path", "filename", "file_name", "file"] {
        if let Some(val) = args.get(*alias).and_then(|v| v.as_str()) {
            if !val.trim().is_empty() {
                if *alias != "path" {
                    tracing::debug!(
                        "[codebase] Agent {} used '{}' alias instead of 'path' in '{}'. Accepting.",
                        ctx.identity.agent_id,
                        alias,
                        tool_name
                    );
                }
                return require_str(ctx, args, alias, tool_name);
            }
        }
    }
    // Neither key present — delegate to require_str for a canonical error message
    require_str(ctx, args, "path", tool_name)
}

const CODEBASE_READ_MAX_CHARS: usize = 30_000;
const BREADCRUMB_CAP: usize = 30;
const MAX_TEMPLATE_LITERAL_DEPTH: usize = 32;

mod graph_query;
mod path_access;
mod symbols;

#[cfg(test)]
mod tests {
    use super::symbols::{count_braces_robust, BraceCounterState};
    use super::*;
    use crate::agent::runner::RunContext;
    use crate::state::AppState;
    use std::sync::Arc;

    async fn setup_test_runner() -> (AgentRunner, Arc<AppState>) {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        (runner, state)
    }

    #[tokio::test]
    async fn test_codebase_dispatch_preserves_canonical_mission_target() {
        let _trace =
            tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());
        use crate::agent::runner::service_traits::ToolExecutor;
        use crate::agent::types::{EngineAgent, ToolCall};
        use crate::security::permissions::PermissionMode;

        let repository = tempfile::tempdir().unwrap();
        let repository_root = repository.path().canonicalize().unwrap();
        let mission_root = repository_root.join("data/workspaces/audit");
        std::fs::create_dir_all(repository_root.join("src")).unwrap();
        std::fs::create_dir_all(mission_root.join("src")).unwrap();
        let repository_target = repository_root.join("src/target.rs");
        let mission_target = mission_root.join("src/target.rs");
        std::fs::write(&repository_target, "pub fn repository_only() {}\n").unwrap();
        std::fs::write(&mission_target, "pub fn mission_only() {}\n").unwrap();
        let denied_target = mission_root.join(".env");
        std::fs::write(&denied_target, "denied_payload").unwrap();
        let mission_target = mission_target.canonicalize().unwrap();

        let mut state = AppState::new_minimal_mock().await;
        state.base_dir = repository.path().to_path_buf();
        state
            .governance
            .auto_approve_safe_skills
            .store(false, std::sync::atomic::Ordering::SeqCst);
        let state = Arc::new(state);
        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.identity.agent_id = "codebase-target-agent".into();
        ctx.env.workspace_root = mission_root.canonicalize().unwrap();
        ctx.env.fs_adapter =
            crate::adapter::filesystem::FilesystemAdapter::new(ctx.env.workspace_root.clone());
        ctx.skills = ["read_codebase_file", "list_file_symbols", "get_symbol_body"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.identity.agent_id.clone();
        agent.capabilities.skills = ctx.skills.clone();
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .unwrap();
        sqlx::query("INSERT INTO mission_history (id, agent_id, title, status) VALUES (?, ?, 'Codebase scope', 'active')")
            .bind(&ctx.mission_state.mission_id)
            .bind(&ctx.identity.agent_id)
            .execute(&state.resources.pool)
            .await
            .unwrap();
        state
            .registry
            .agents
            .insert(ctx.identity.agent_id.clone(), agent);

        for name in &ctx.skills {
            state
                .security
                .permission_policy
                .set_mode(name, PermissionMode::Allow)
                .await
                .unwrap();
            let call = ToolCall {
                name: name.clone(),
                args: serde_json::json!({
                    "path": &mission_target,
                    "symbol": "mission_only"
                }),
            };
            // Execute remains subject to the mandatory Prompt floor even with
            // a legacy Allow row. Exercise the real approval path for this read.
            let mut events = state.comms.event_tx.subscribe();
            let approve = async {
                loop {
                    let event = events.recv().await.unwrap();
                    if event["type"] == "oversight:new" {
                        let id = event["entry"]["id"].as_str().unwrap();
                        let (_, resolver) = state.comms.oversight_resolvers.remove(id).unwrap();
                        resolver
                            .send(crate::agent::types::OversightResolution {
                                approved: true,
                                override_slot: None,
                                user_answer: None,
                            })
                            .unwrap();
                        break;
                    }
                }
            };
            let (result, ()) = tokio::time::timeout(std::time::Duration::from_secs(5), async {
                tokio::join!(runner.execute_tool(&ctx, &call, ""), approve)
            })
            .await
            .expect("Codebase read should request approval and finish");
            let (output, _) = result.unwrap();
            let expected_content = if name == "list_file_symbols" {
                "[fn] mission_only"
            } else {
                "pub fn mission_only() {}"
            };
            assert!(output.contains(expected_content), "{name}: {output}");
            assert!(!output.contains("repository_only"), "{name}: {output}");
            assert!(!output.contains("TOOL FAILURE"), "{name}: {output}");
            assert!(ctx
                .last_accessed_files
                .lock()
                .iter()
                .any(|path| path.replace('\\', "/") == "data/workspaces/audit/src/target.rs"));

            let denied_call = ToolCall {
                name: name.clone(),
                args: serde_json::json!({"path": &denied_target, "symbol": "mission_only"}),
            };
            let mut events = state.comms.event_tx.subscribe();
            let approve = async {
                loop {
                    let event = events.recv().await.unwrap();
                    if event["type"] == "oversight:new" {
                        let id = event["entry"]["id"].as_str().unwrap();
                        let (_, resolver) = state.comms.oversight_resolvers.remove(id).unwrap();
                        resolver
                            .send(crate::agent::types::OversightResolution {
                                approved: true,
                                override_slot: None,
                                user_answer: None,
                            })
                            .unwrap();
                        break;
                    }
                }
            };
            let (result, ()) = tokio::time::timeout(std::time::Duration::from_secs(5), async {
                tokio::join!(runner.execute_tool(&ctx, &denied_call, ""), approve)
            })
            .await
            .expect("An approved request must still reject sensitive content");
            let (failure, _) = result.unwrap();
            assert!(failure.contains("TOOL FAILURE"), "{name}: {failure}");
            assert!(!failure.contains("denied_payload"));
            let actions: Vec<String> = sqlx::query_scalar(
                "SELECT action FROM audit_trail WHERE mission_id = ? AND action IN (?, ?)",
            )
            .bind(&ctx.mission_state.mission_id)
            .bind(format!("[SUCCESS] {name}"))
            .bind(format!("[FAILURE] {name}"))
            .fetch_all(&state.resources.pool)
            .await
            .unwrap();
            assert_eq!(
                actions
                    .iter()
                    .filter(|a| a.starts_with("[SUCCESS]"))
                    .count(),
                1
            );
            assert_eq!(
                actions
                    .iter()
                    .filter(|a| a.starts_with("[FAILURE]"))
                    .count(),
                1
            );

            // The central project boundary does not grant reads outside the mission.
            let outside_mission = ToolCall {
                name: name.clone(),
                args: serde_json::json!({
                    "path": &repository_target,
                    "symbol": "repository_only"
                }),
            };
            let (output, _) = runner
                .execute_tool(&ctx, &outside_mission, "")
                .await
                .unwrap();
            assert!(output.contains("TOOL FAILURE"), "{name}: {output}");
            assert!(
                !output.contains("pub fn repository_only"),
                "{name}: {output}"
            );
        }
    }

    #[tokio::test]
    async fn test_workspace_read_sensitivity_and_typed_failures() {
        use crate::agent::types::ToolCall;

        let repository = tempfile::tempdir().unwrap();
        let root = repository.path().canonicalize().unwrap();
        let workspace = root.join("data/workspaces/audit");
        std::fs::create_dir_all(workspace.join("src")).unwrap();
        std::fs::write(workspace.join("src/benign.rs"), "pub fn benign() {}\n").unwrap();
        let mut state = AppState::new_minimal_mock().await;
        state.base_dir = root.clone();
        let runner = AgentRunner::new(Arc::new(state));
        let mut ctx = RunContext::default();
        ctx.env.workspace_root = workspace.canonicalize().unwrap();

        for relative in [
            "data/workspaces/audit/.env",
            "data/workspaces/audit/.git/config",
            "data/workspaces/audit/src/credentials.json",
            "data/workspaces/audit/data/state/tadpole.db",
            "data/state/tadpole.db",
            "data/workspaces/other/src/benign.rs",
        ] {
            let target = root.join(relative);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::write(&target, "denied_payload").unwrap();
            for name in ["read_codebase_file", "list_file_symbols", "get_symbol_body"] {
                let call = ToolCall {
                    name: name.into(),
                    args: serde_json::json!({"path": target, "symbol": "benign"}),
                };
                let result = match name {
                    "list_file_symbols" => runner.handle_list_file_symbols(&ctx, &call).await,
                    "get_symbol_body" => runner.handle_get_symbol_body(&ctx, &call).await,
                    _ => runner.handle_read_codebase_file(&ctx, &call).await,
                };
                assert!(
                    matches!(result, Err(ToolExecutionError::SecurityBlocked(_))),
                    "{name}: {relative}"
                );
            }
        }
        for name in ["read_codebase_file", "list_file_symbols", "get_symbol_body"] {
            let call = ToolCall {
                name: name.into(),
                args: serde_json::json!({
                    "path": workspace.join("src/missing.rs"), "symbol": "benign"
                }),
            };
            let result = match name {
                "list_file_symbols" => runner.handle_list_file_symbols(&ctx, &call).await,
                "get_symbol_body" => runner.handle_get_symbol_body(&ctx, &call).await,
                _ => runner.handle_read_codebase_file(&ctx, &call).await,
            };
            assert!(
                matches!(result, Err(ToolExecutionError::ExecutionFailed(_))),
                "{name}"
            );
        }
        assert!(ctx.last_accessed_files.lock().is_empty());

        // Exercise a real symlink when the host permits creating one.
        let link = workspace.join("src/linked.rs");
        let outside = root.join("data/state/tadpole.db");
        #[cfg(windows)]
        let linked = std::os::windows::fs::symlink_file(&outside, &link);
        #[cfg(unix)]
        let linked = std::os::unix::fs::symlink(&outside, &link);
        #[cfg(any(windows, unix))]
        if linked.is_ok() {
            assert!(runner
                .require_path_safety(&ctx, link.to_str().unwrap())
                .await
                .is_err());
        }
    }

    #[test]
    fn test_count_braces_robust() {
        let mut state = BraceCounterState::default();
        count_braces_robust("fn hello() {}", &mut state);
        assert_eq!(state.current_depth, 0);

        let mut state = BraceCounterState::default();
        count_braces_robust("fn hello() {", &mut state);
        assert_eq!(state.current_depth, 1);

        let mut state = BraceCounterState::default();
        count_braces_robust("}", &mut state);
        assert_eq!(state.current_depth, -1);

        // String literals
        let mut state = BraceCounterState::default();
        count_braces_robust("let x = \"{ escaped brace }\";", &mut state);
        assert_eq!(state.current_depth, 0);

        // Char literals
        let mut state = BraceCounterState::default();
        count_braces_robust("let x = '{';", &mut state);
        assert_eq!(state.current_depth, 0);

        let mut state = BraceCounterState::default();
        count_braces_robust("let x = '}';", &mut state);
        assert_eq!(state.current_depth, 0);

        // Nested braces
        let mut state = BraceCounterState::default();
        count_braces_robust("let x = '{'; fn nested() {", &mut state);
        assert_eq!(state.current_depth, 1);

        // Comments
        let mut state = BraceCounterState::default();
        count_braces_robust("let x = \"}\"; // comment with {", &mut state);
        assert_eq!(state.current_depth, 0);

        // Block comments spanning lines
        let mut state = BraceCounterState::default();
        count_braces_robust("/* { brace in block comment */", &mut state);
        assert_eq!(state.current_depth, 0);
        assert!(!state.in_block_comment);

        let mut state = BraceCounterState::default();
        count_braces_robust("/* start comment", &mut state);
        assert_eq!(state.current_depth, 0);
        assert!(state.in_block_comment);
        count_braces_robust("brace { still in comment */ }", &mut state);
        assert_eq!(state.current_depth, -1); // only the trailing `}` should count
        assert!(!state.in_block_comment);

        // Rust Raw strings
        let mut state = BraceCounterState::default();
        count_braces_robust("let raw = r#\" { raw content } \"#;", &mut state);
        assert_eq!(state.current_depth, 0);

        // JS template literals with interpolation
        let mut state = BraceCounterState::default();
        count_braces_robust("let msg = `hello ${name}`;", &mut state);
        assert_eq!(state.current_depth, 0);

        let mut state = BraceCounterState::default();
        count_braces_robust("let msg = `nested ${ {a: '{'} }`;", &mut state);
        assert_eq!(state.current_depth, 0);

        // Multi-level template literals nesting
        let mut state = BraceCounterState::default();
        count_braces_robust("let s = `${ a ? `${b}` : c }`;", &mut state);
        assert_eq!(state.current_depth, 0);

        let mut state = BraceCounterState::default();
        count_braces_robust("let s = `a ${ `${'}'}` } b`;", &mut state);
        assert_eq!(state.current_depth, 0);
    }

    #[tokio::test]
    async fn test_extract_symbols_rust() {
        let (runner, _) = setup_test_runner().await;
        let content = r#"
            pub fn run() {}
            struct Data {}
            enum Kind {}
        "#;
        let symbols = runner.extract_symbols(content, "src/main.rs");
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0], "[fn] run");
        assert_eq!(symbols[1], "[struct] Data");
        assert_eq!(symbols[2], "[enum] Kind");
    }

    #[tokio::test]
    async fn test_extract_symbol_body_rust() {
        let (runner, _) = setup_test_runner().await;
        let content = r#"
            fn add(a: i32, b: i32) -> i32 {
                let sum = a + b;
                sum
            }
        "#;
        let body = runner.extract_symbol_body(content, "add", "src/main.rs");
        assert!(body.is_some());
        let body_str = body.unwrap();
        assert!(body_str.contains("fn add"));
        assert!(body_str.contains("let sum = a + b;"));
        assert!(body_str.contains("}"));
    }

    #[tokio::test]
    async fn test_extract_symbol_body_python_indentation() {
        let (runner, _) = setup_test_runner().await;
        let content = r#"
def calc(a, b):
    # This is a tab expanded indent
	val = a + b
	return val

def another_func():
    pass
"#;
        let body = runner.extract_symbol_body(content, "calc", "main.py");
        assert!(body.is_some());
        let body_str = body.unwrap();
        assert!(body_str.contains("def calc"));
        assert!(body_str.contains("val = a + b"));
        assert!(body_str.contains("return val"));
        assert!(!body_str.contains("def another_func"));
    }

    #[tokio::test]
    async fn test_extract_symbol_body_matches_full_identifier() {
        let (runner, _) = setup_test_runner().await;
        let content = "def calculator():\n    return 1\ndef calc():\n    return 2\n";

        let body = runner
            .extract_symbol_body(content, "calc", "main.py")
            .expect("expected exact symbol definition");

        assert!(body.contains("return 2"));
        assert!(!body.contains("return 1"));
    }

    #[tokio::test]
    async fn test_extract_symbol_body_ignores_comment_braces() {
        let (runner, _) = setup_test_runner().await;
        let content = "fn calc()\n/* comment with { */\n{\n    2\n}\n";

        let body = runner
            .extract_symbol_body(content, "calc", "src/lib.rs")
            .expect("expected exact symbol definition");

        assert!(body.contains("    2"));
    }

    #[tokio::test]
    async fn test_require_path_safety() {
        let (runner, _) = setup_test_runner().await;
        let ctx = RunContext::default();

        // Prohibited sensitive paths
        let res = runner.require_path_safety(&ctx, ".env").await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("SECURITY BLOCKED"));

        let res = runner.require_path_safety(&ctx, "prod.secret").await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("SECURITY BLOCKED"));

        // Valid relative paths
        let res = runner
            .require_path_safety(&ctx, "src/agent/runner/mission_tools.rs")
            .await;
        assert!(res.is_ok());

        // Breadcrumb with absolute external path cannot be used to escape root
        {
            let mut breadcrumbs = ctx.last_accessed_files.lock();
            breadcrumbs.push("C:/Windows/System32/drivers/etc/hosts".to_string());
            breadcrumbs.push("/etc/hosts".to_string());
        }
        let res = runner.require_path_safety(&ctx, "hosts").await;
        if let Ok(safe_p) = res {
            let p_str = safe_p.as_path().to_string_lossy().replace(r"\\?\", "");
            let canon_base = std::fs::canonicalize(&runner.state.base_dir)
                .unwrap_or_else(|_| runner.state.base_dir.clone());
            let base_str = canon_base.to_string_lossy().replace(r"\\?\", "");
            assert!(p_str.starts_with(&base_str));
            assert!(!p_str.contains("System32"));
        }

        // Attempting to record an external absolute path into breadcrumbs must be ignored
        #[cfg(windows)]
        let external_path = std::path::PathBuf::from("C:/Windows/System32/external.txt");
        #[cfg(not(windows))]
        let external_path = std::path::PathBuf::from("/etc/external.txt");

        runner.record_breadcrumb(&ctx, &external_path, "external.txt");
        {
            let breadcrumbs = ctx.last_accessed_files.lock();
            #[cfg(windows)]
            assert!(!breadcrumbs.contains(&"C:/Windows/System32/external.txt".to_string()));
            #[cfg(not(windows))]
            assert!(!breadcrumbs.contains(&"/etc/external.txt".to_string()));
            assert!(!breadcrumbs.contains(&"external.txt".to_string()));
        }
    }

    #[tokio::test]
    async fn test_handle_query_symbol_graph() {
        let (runner, state) = setup_test_runner().await;
        let ctx = RunContext::default();

        let graph_lock = state.resources.get_symbol_graph().await;
        {
            let mut guard = graph_lock.write();
            use crate::intelligence::graph::SymbolNode;

            let node_a = SymbolNode {
                name: "parse_tokens".to_string(),
                path: "src/parser.rs".to_string(),
                kind: "function".to_string(),
                signature: "fn parse_tokens() -> Vec<Token>".to_string(),
                start_line: 10,
                end_line: 25,
                docstring: Some("Parses lexical tokens".to_string()),
                docstring_range: None,
            };
            let node_b = SymbolNode {
                name: "execute_plan".to_string(),
                path: "src/runner.rs".to_string(),
                kind: "function".to_string(),
                signature: "fn execute_plan()".to_string(),
                start_line: 40,
                end_line: 80,
                docstring: None,
                docstring_range: None,
            };
            let idx_a = guard.graph.add_node(node_a);
            let idx_b = guard.graph.add_node(node_b);
            // node_b calls node_a (outgoing from b to a; incoming to a from b)
            guard.graph.add_edge(
                idx_b,
                idx_a,
                crate::intelligence::graph::SymbolEdge {
                    kind: "call".to_string(),
                },
            );
            guard
                .index
                .insert("src/parser.rs\x01parse_tokens".to_string(), idx_a);
            guard
                .index
                .insert("src/runner.rs\x01execute_plan".to_string(), idx_b);
        }

        // 1. Test lookup
        let lookup_call = crate::agent::types::ToolCall {
            name: "query_symbol_graph".to_string(),
            args: serde_json::json!({
                "action": "lookup",
                "symbol": "parse_tokens"
            }),
        };
        let lookup_res = runner
            .handle_query_symbol_graph(&ctx, &lookup_call)
            .await
            .unwrap();
        assert!(lookup_res.contains("SYMBOL GRAPH LOOKUP RESULTS"));
        assert!(lookup_res.contains("parse_tokens"));
        assert!(lookup_res.contains("src/parser.rs:10-25"));
        assert!(lookup_res.contains("execute_plan (src/runner.rs)"));

        // 2. Test blast radius
        let blast_call = crate::agent::types::ToolCall {
            name: "query_symbol_graph".to_string(),
            args: serde_json::json!({
                "action": "blast_radius",
                "symbol": "parse_tokens",
                "path": "src/parser.rs"
            }),
        };
        let blast_res = runner
            .handle_query_symbol_graph(&ctx, &blast_call)
            .await
            .unwrap();
        assert!(blast_res.contains("BLAST RADIUS FOR 'parse_tokens' in src/parser.rs"));
        assert!(blast_res.contains("execute_plan"));
    }

    #[tokio::test]
    async fn test_require_path_polymorphic_aliases() {
        let ctx = RunContext::default();

        let args_path = serde_json::json!({ "path": "src/main.rs" });
        assert_eq!(
            super::require_path(&ctx, &args_path, "test").unwrap(),
            "src/main.rs"
        );

        let args_filename = serde_json::json!({ "filename": "src/lib.rs" });
        assert_eq!(
            super::require_path(&ctx, &args_filename, "test").unwrap(),
            "src/lib.rs"
        );

        let args_file_name = serde_json::json!({ "file_name": "src/App.tsx" });
        assert_eq!(
            super::require_path(&ctx, &args_file_name, "test").unwrap(),
            "src/App.tsx"
        );

        let args_file = serde_json::json!({ "file": "Cargo.toml" });
        assert_eq!(
            super::require_path(&ctx, &args_file, "test").unwrap(),
            "Cargo.toml"
        );

        // Priority check: path beats filename beats file_name beats file
        let args_multi = serde_json::json!({ "path": "p.rs", "filename": "fn.rs", "file": "f.rs" });
        assert_eq!(
            super::require_path(&ctx, &args_multi, "test").unwrap(),
            "p.rs"
        );

        // Missing check
        let args_empty = serde_json::json!({ "unrelated": 123 });
        assert!(super::require_path(&ctx, &args_empty, "test").is_err());
    }

    #[tokio::test]
    async fn test_require_path_safety_workspace_prefix_sanitization() {
        let (runner, state) = setup_test_runner().await;
        let ctx = RunContext::default();

        let test_file = state.base_dir.join("test_module.rs");
        tokio::fs::write(&test_file, "// unit test").await.unwrap();

        let simulated_sandbox_path = "data/workspaces/cl-chain-a/test_module.rs";
        let res = runner
            .require_path_safety(&ctx, simulated_sandbox_path)
            .await;
        assert!(
            res.is_ok(),
            "Expected workspace-prefixed codebase path to be auto-sanitized: {:?}",
            res
        );
        let safe_path = res.unwrap();
        assert_eq!(safe_path.as_path().file_name().unwrap(), "test_module.rs");

        let _ = tokio::fs::remove_file(test_file).await;
    }
}
