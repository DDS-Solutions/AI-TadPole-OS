//! @docs ARCHITECTURE:Macros
//!
//! ### AI Assist Note
//! **Capability Registry Macros**: Reduces boilerplate for tool registration
//! and dispatch. Provides a declarative way to bridge the `Zero-Trust Tool`
//! trait with categorical handlers.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Macro expansion error or trait implementation mismatch.
//! - **Telemetry Link**: Search `[macros]` in tracing logs.
//! - **Trace Scope**: `server-rs::agent::runner::tools::macros`

/// Generates a categorical handler that dispatches to AgentRunner methods.
#[macro_export]
macro_rules! define_categorical_handler {
    ($name:ident, { $($tool_name:expr => $handler_method:ident $signature:tt ),* $(,)? }) => {
        pub struct $name;

        #[async_trait::async_trait]
        impl $crate::agent::runner::tools::dispatcher::CategoricalHandler for $name {
            async fn handle(
                &self,
                name: &str,
                ctx: &$crate::agent::runner::tools::trait_tool::ToolContext,
                args: serde_json::Value,
                usage: &mut Option<$crate::agent::types::TokenUsage>,
            ) -> Result<String, $crate::agent::runner::tools::error::ToolExecutionError> {
                let runner = $crate::agent::runner::AgentRunner::new(ctx.state.clone());
                let run_ctx = $crate::agent::runner::RunContext::from_tool_ctx(ctx);
                let fc = $crate::agent::types::ToolCall {
                    name: name.to_string(),
                    args,
                };
                tracing::debug!("🧪 [macros] Handling tool: {}", name);

                match name {
                    $(
                        $tool_name => {
                            $crate::define_categorical_handler!(@call runner run_ctx fc usage $handler_method $signature)
                        }
                    )*
                    _ => Err($crate::agent::runner::tools::error::ToolExecutionError::ExecutionFailed(format!(
                        "{} cannot handle '{}'",
                        stringify!($name),
                        name
                    ))),
                }
            }
        }
    };

    // Standard Result<String>
    (@call $runner:ident $run_ctx:ident $fc:ident $usage:ident $method:ident ()) => {
        {
            let _ = $usage;
            $runner.$method(&$run_ctx, &$fc).await.map_err($crate::agent::runner::tools::error::ToolExecutionError::from)
        }
    };

    // Result<String> with Usage
    (@call $runner:ident $run_ctx:ident $fc:ident $usage:ident $method:ident (usage)) => {
        $runner.$method(&$run_ctx, &$fc, $usage).await.map_err($crate::agent::runner::tools::error::ToolExecutionError::from)
    };

    // Mutable Output
    (@call $runner:ident $run_ctx:ident $fc:ident $usage:ident $method:ident (output)) => {
        {
            let _ = $usage;
            let mut output = String::new();
            $runner.$method(&$run_ctx, &$fc, &mut output).await.map_err($crate::agent::runner::tools::error::ToolExecutionError::from)?;
            Ok(output)
        }
    };

    // Mutable Output + Usage + Extra
    (@call $runner:ident $run_ctx:ident $fc:ident $usage:ident $method:ident (output, usage, $extra:expr)) => {
        {
            let mut output = String::new();
            $runner.$method(&$run_ctx, &$fc, &mut output, $usage, $extra).await.map_err($crate::agent::runner::tools::error::ToolExecutionError::from)?;
            Ok(output)
        }
    };
}

// Metadata: [macros]
