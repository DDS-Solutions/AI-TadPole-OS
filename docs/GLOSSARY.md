# 📖 Tadpole OS: Technical Specifications & Glossary
**Intelligence Level**: High (ECC Optimized)
**Source of Truth**: Rust Source Code (`server-rs/src/`)
**Standard Compliance**: ECC-SPEC-01 (Strict Reference Protocol)

---

## 🏗️ Constants & Statics

--- TELEMETRY_TX ---
Type: static
Purpose: Global broadcast channel for high-throughput JSON telemetry emissions from system spans and agent lifecycle events.
Parameters:
- N/A
Return Value: Lazy<broadcast::Sender<serde_json::Value>>. The sender side of a tokio broadcast channel with a capacity of 2000 events.
Side Effects: N/A
Failure Conditions: N/A

---

## 🏗️ Structs

--- AppState ---
Type: struct
Purpose: Primary global application state container and coordination hub for all sovereign subsystems.
Parameters:
- comms (Arc<CommunicationHub>): Definition: Manages real-time communication channels and telemetry.
- governance (Arc<GovernanceHub>): Definition: Manages operational limits and global policy settings.
- registry (Arc<RegistryHub>): Definition: Maintains thread-safe registries for agents, providers, and skills.
- security (Arc<SecurityHub>): Definition: Handles audit trails, budget enforcement, and shell safety scanning.
- resources (Arc<ResourceHub>): Definition: Manages shared system resources including database pools and HTTP clients.
- base_dir (PathBuf): Definition: The root directory for persistent data and workspace sandboxes.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- CommunicationHub ---
Type: struct
Purpose: Orchestrates real-time event broadcasting and human-in-the-loop oversight resolution.
Parameters:
- tx (broadcast::Sender<LogEntry>): Definition: Publishes system logs to all connected UI consumers.
- event_tx (broadcast::Sender<Value>): Definition: Dedicated channel for high-level engine lifecycle events.
- telemetry_tx (broadcast::Sender<Value>): Definition: Dedicated channel for real-time agent status and reasoning updates.
- audio_stream_tx (broadcast::Sender<Vec<u8>>): Definition: Channel for PCM audio chunk streaming.
- pulse_tx (broadcast::Sender<Arc<SwarmPulse>>): Definition: High-speed binary pulse channel for swarm visualization.
- oversight_queue (DashMap<String, OversightEntry>): Definition: Storage for pending tool calls awaiting human decision.
- active_runners (DashMap<String, AbortHandle>): Definition: Registry of active agent tasks for definitive cancellation.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- GovernanceHub ---
Type: struct
Purpose: Centralizes system limits and automated policy enforcement for the agent swarm.
Parameters:
- auto_approve_safe_skills (AtomicBool): Definition: Toggles bypass of HITL for low-risk capabilities.
- max_agents (AtomicU32): Definition: Total agent instances permitted in memory (Default: 50).
- max_clusters (AtomicU32): Definition: Total concurrent mission clusters permitted (Default: 10).
- max_swarm_depth (AtomicU32): Definition: Maximum recursion level for agent recruitment (Default: 5).
- max_task_length (AtomicUsize): Definition: Token threshold for single task payloads (Default: 32768).
- default_budget_usd (RwLock<f64>): Definition: Initial monetary allocation for new agents (Default: 1.0 USD).
- privacy_mode (AtomicBool): Definition: When enabled, all external cloud provider traffic is blocked.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- RegistryHub ---
Type: struct
Purpose: Centralized directory for swarm identities, model providers, and skill discovery.
Parameters:
- agents (DashMap<String, EngineAgent>): Definition: Live registry of all provisioned agents.
- providers (DashMap<String, ProviderConfig>): Definition: Configuration registry for LLM protocols (OpenAI, Anthropic, etc.).
- provider_health (DashMap<String, ProviderStatus>): Definition: Real-time health state for external providers.
- models (DashMap<String, ModelEntry>): Definition: Global catalog of available LLM models.
- skill_registry (Arc<SkillRegistry>): Definition: Manager for dynamic skill manifests and tool definitions.
- mcp_host (Arc<McpHost>): Definition: Host for Model Context Protocol (MCP) tool aggregation.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- ResourceHub ---
Type: struct
Purpose: Manages thread-safe access to heavy system resources and shared mission context.
Parameters:
- pool (SqlitePool): Definition: Primary database connection pool for persistence.
- http_client (Arc<Client>): Definition: Shared client for external API transmissions.
- audio_engine (OnceCell<Arc<NeuralAudioEngine>>): Definition: Lazy-loaded ONNX engine for voice synthesis.
- code_graph (OnceCell<Arc<RwLock<CodeGraph>>>): Definition: Lazy-loaded graph for codebase intelligence.
- initialization_registry (DashMap<String, SubsystemStatus>): Definition: Tracks warmup progress of core subsystems.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- SecurityHub ---
Type: struct
Purpose: Manages tamper-evident auditing and preventative security protocols.
Parameters:
- audit_trail (Arc<MerkleAuditTrail>): Definition: Merkle Hash-Chain engine for non-repudiable logging.
- budget_guard (Arc<BudgetGuard>): Definition: Fiscal enforcement and metering engine.
- shell_scanner (Arc<ShellScanner>): Definition: Whitelist-based validator for shell tool calls.
- secret_redactor (Arc<SecretRedactor>): Definition: Runtime processor for stripping sensitive tokens from logs.
- permission_policy (Arc<PermissionPolicy>): Definition: Governance engine for tool access control.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- LogEntry ---
Type: struct
Purpose: Atomic unit of telemetry mirroring the frontend event interface.
Parameters:
- event_type (String): Definition: Category of event (e.g., "log", "telemetry").
- id (String): Definition: Unique UUID v4 identifier.
- timestamp (DateTime<Utc>): Definition: Event creation time.
- source (String): Definition: Originating subsystem (e.g., "System", "Agent").
- severity (String): Definition: Log level (e.g., "INFO", "CRITICAL").
- text (String): Definition: The redacted event message.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- TelemetryLayer ---
Type: struct
Purpose: Tracing subscriber layer bridging internal spans to OTel-compatible JSON events.
Parameters:
- redactor (SecretRedactor): Definition: Redacts sensitive patterns within log messages.
Return Value: N/A
Side Effects: Broadcasts JSON events to TELEMETRY_TX on span creation and closure.
Failure Conditions: N/A

--- AgentRunner ---
Type: struct
Purpose: Execution engine orchestrating the autonomous mission lifecycle.
Parameters:
- state (Arc<AppState>): Definition: Reference to global application state.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- RunContext ---
Type: struct
Purpose: Data container carrying mission state and environment throughout an execution loop.
Parameters:
- agent_id (String): Definition: Unique identifier for the agent.
- mission_id (String): Definition: Unique identifier for the mission.
- workspace_root (PathBuf): Definition: Path to the sandboxed file environment.
- budget_usd (f64): Definition: Fiscal limit for the current mission.
- current_cost_usd (f64): Definition: Cumulative monetary cost incurred.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- ModelConfig ---
Type: struct
Purpose: Configuration for an LLM instance including system prompts and performance thresholds.
Parameters:
- provider (ModelProvider): Definition: Backend protocol.
- model_id (String): Definition: Specific model identifier.
- reasoning_depth (Option<u32>): Definition: Maximum internal reasoning turns.
- act_threshold (Option<f32>): Definition: Confidence threshold for ACT.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- ModelCapabilities ---
Type: struct
Purpose: Feature matrix defining the functional limits and supported modalities of an LLM.
Parameters:
- supports_tools (bool): Definition: Support for external function calling.
- supports_vision (bool): Definition: Support for image processing.
- supports_reasoning (bool): Definition: Support for chain-of-thought protocols.
- context_window (u32): Definition: Total token capacity.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- ProblemDetails ---
Type: struct
Purpose: RFC 9457 compliant structure for machine-readable API error responses.
Parameters:
- type_uri (String): Definition: URI identifying the specific error type.
- title (String): Definition: Human-readable summary of the error.
- status (u16): Definition: HTTP status code.
- detail (String): Definition: Detailed explanation of the failure.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- InstallTemplateRequest ---
Type: struct
Purpose: Request payload for installing a swarm template from a remote repository.
Parameters:
- repository_url (String): Definition: Git URL of the template source.
- path (String): Definition: Path within the repository for the template configuration.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

---

## 🏗️ Enums

--- AppError ---
Type: enum
Purpose: Unified error enumeration for the engine, mapping failures to RFC 9457 / HTTP protocols.
Parameters:
- BadRequest(String): Definition: Malformed request or validation failure.
- Unauthorized(String): Definition: Missing or invalid authentication.
- Forbidden(String): Definition: Insufficient permissions.
- NotFound(String): Definition: Resource does not exist.
- ValidationError(String): Definition: Input data failed validation checks.
- DomainError { code, detail, help_link }: Definition: Specific business logic failure.
- BudgetExhausted(String): Definition: Mission cost exceeds allocated budget.
- InfrastructureError { provider_id, detail, help_link }: Definition: Downstream service failure.
- QuantizationFallback { model_id, suggested_quant, detail }: Definition: Model resource mismatch.
- RecruitmentFailure { recipe_id, role, detail }: Definition: Failed to spawn sub-agent.
- SanitizationViolation(String): Definition: Security scanner blocked unsafe content.
- RecursionBlocked(String): Definition: Swarm depth limit reached.
- RateLimit(String): Definition: Provider rate limits exceeded.
- InternalServerError(String): Definition: Unhandled system failure.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- RoleAuthorityLevel ---
Type: enum
Purpose: Hierarchical authority of an agent within the swarm.
Parameters:
- Executive: Definition: Strategic oversight (CEO, Overlord).
- Management: Definition: Tactical coordination (Alpha Node).
- Specialist: Definition: Task execution.
- Observer: Definition: Read-only auditing.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- ModelProvider ---
Type: enum
Purpose: Supported backend protocols for LLM interaction.
Parameters:
- Openai, Anthropic, Google, Ollama, Groq, Mistral, Deepseek, Xai, etc.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- SubsystemStatus ---
Type: enum
Purpose: Lifecycle state machine for core engine components.
Parameters:
- NotStarted: Definition: Component is uninitialized.
- Warming(f32): Definition: Component is in transition (0.0-1.0 progress).
- Ready: Definition: Component is fully functional.
- Failed(String): Definition: Terminal error state.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

---

## 🏗️ Functions

--- install_template ---
Type: fn
Purpose: Clones, validates, and installs a swarm template into the local engine environment.
Parameters:
- state (State<Arc<AppState>>): Definition: Reference to global state.
- payload (Json<InstallTemplateRequest>): Definition: Repository URL and internal path.
Return Value: Result<Response, AppError>. Returns 200 OK or failure.
Side Effects:
- Clones repository to `data/.bunker_cache`.
- Copies configs to `data/swarm_config/agents`, `directives`, and `execution`.
- Merges MCP configuration into `.agent/mcp_config.json`.
- Persists agent records to SQLite.
Failure Conditions:
- AppError::InternalServerError: Git failure.
- AppError::BadRequest: Invalid path.
- AppError::NotFound: Path missing in repository.

--- yield_phase_transition ---
Type: fn
Purpose: Forces the current task to yield to the Tokio scheduler and emits a phase transition event.
Parameters:
- agent_id (&str): Definition: ID of the yielding agent.
- phase (&str): Definition: Name of the target phase.
Return Value: N/A (async void)
Side Effects:
- Emits `agent:phase_transition` event to `event_tx`.
- Suspends the current task via `tokio::task::yield_now()`.
Failure Conditions: N/A

--- broadcast_sys ---
Type: fn
Purpose: Redacts and publishes a high-priority system log to telemetry consumers.
Parameters:
- text (&str): Definition: Raw log message.
- severity (&str): Definition: Log level.
- mission_id (Option<String>): Definition: Correlation ID.
Return Value: N/A
Side Effects: Redacts secrets via `SecretRedactor` and sends `LogEntry` to `comms.tx`.
Failure Conditions: N/A

--- list_active_specialists ---
Type: fn
Purpose: Returns a formatted directory of specialized agents, excluding administrative nodes.
Parameters:
- N/A (RegistryHub method)
Return Value: Vec<String>. Formatted markdown list of specialists.
Side Effects: N/A
Failure Conditions: N/A

--- set_subsystem_status ---
Type: fn
Purpose: Updates the initialization registry with the state of a specific subsystem.
Parameters:
- name (&str): Definition: Subsystem identifier.
- status (SubsystemStatus): Definition: Target state.
Return Value: N/A
Side Effects: Inserts status into `initialization_registry` DashMap.
Failure Conditions: N/A

--- get_audio_engine ---
Type: fn
Purpose: Lazily initializes the ONNX audio engine and returns an Arc pointer.
Parameters:
- N/A (Async ResourceHub method)
Return Value: Result<Arc<NeuralAudioEngine>, AppError>.
Side Effects: Transitions "Audio" status and allocates memory for model weights.
Failure Conditions: AppError::Io or AppError::InternalServerError on initialization failure.

--- run ---
Type: fn
Purpose: Executes the autonomous mission lifecycle for a specific agent.
Parameters:
- agent_id (String): Definition: Assigned agent ID.
- payload (TaskPayload): Definition: Mission details and budget.
Return Value: Result<String, AppError>. Returns the mission response text.
Side Effects: Transitions agent status, consumes budget, and writes to memory/history databases.
Failure Conditions: AppError::BudgetExhausted, AppError::RecursionBlocked, or AppError::NotFound.

---

[//]: # (Metadata: [GLOSSARY])
