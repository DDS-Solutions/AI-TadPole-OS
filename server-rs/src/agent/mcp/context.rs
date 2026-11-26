//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Context & Scope
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Non-optional agent identity and capability declarations in AgentScope.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none

/// Explicit execution context for an agent turn.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub agent_id: String,
    pub role: Option<String>,
    pub request_id: Option<String>,
}

impl ExecutionContext {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            role: None,
            request_id: None,
        }
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn as_scope<'a>(&'a self, mcp_declarations: &'a [String]) -> AgentScope<'a> {
        AgentScope {
            agent_id: &self.agent_id,
            role: self.role.as_deref(),
            mcp_declarations,
        }
    }
}

/// Bounded borrow scope for an agent during tool execution.
/// Invariant: An agent identity is mandatory; capability declarations cannot be absent.
#[derive(Debug, Clone, Copy)]
pub struct AgentScope<'a> {
    pub agent_id: &'a str,
    pub role: Option<&'a str>,
    pub mcp_declarations: &'a [String],
}

impl<'a> AgentScope<'a> {
    pub fn new(agent_id: &'a str, mcp_declarations: &'a [String]) -> Self {
        Self {
            agent_id,
            role: None,
            mcp_declarations,
        }
    }

    pub fn with_role(mut self, role: &'a str) -> Self {
        self.role = Some(role);
        self
    }
}
