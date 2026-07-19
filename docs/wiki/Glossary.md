---
title: "Glossary"
tier: "reference"
status: "verified"
version: "1.3.0"
last-verified: "2026-07-15"
commit: "b1c347b1"
network-badge: "none"
risk-tags: []
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[GLOSSARY]` in audit logs.
>
> ### AI Assist Note
> 📖 Tadpole OS: Technical Specifications & Glossary
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py`.

# 📖 Tadpole OS: Technical Specifications & Glossary
**Intelligence Level**: High (ECC Optimized)
**Source of Truth**: Rust Source Code (`server-rs/src/`)
**Standard Compliance**: ECC-SPEC-01 (Strict Reference Protocol)

---

## 🏗️ Constants & Statics

### TELEMETRY_TX
[Tier 3: Security]
Type: static
Purpose: Global broadcast channel for high-throughput JSON telemetry emissions from system spans and agent lifecycle events.
Parameters: N/A
Return Value: `Lazy<broadcast::Sender<serde_json::Value>>`. The sender side of a tokio broadcast channel with a capacity of 2000 events.
Side Effects: N/A
Failure Conditions: N/A

---

## 🏗️ Structs

### AppState
[Tier 2: Admin]
Type: struct
Purpose: Primary global application state container and coordination hub for all sovereign subsystems.
Parameters:
- comms (`Arc<CommunicationHub>`): Definition: Manages real-time communication channels and telemetry.
- governance (`Arc<GovernanceHub>`): Definition: Manages operational limits and global policy settings.
- registry (`Arc<RegistryHub>`): Definition: Maintains thread-safe registries for agents, providers, and skills.
- security (`Arc<SecurityHub>`): Definition: Handles audit trails, budget enforcement, and shell safety scanning.
- resources (`Arc<ResourceHub>`): Definition: Manages shared system resources including database pools and HTTP clients.
- base_dir (PathBuf): Definition: The root directory for persistent data and workspace sandboxes.
- mirror_mode (`bool`): Definition: When true, the engine operates in mirror/read-only mode, preventing state mutations.
- drift_alerts (`Arc<DashMap<String, String>>`): Definition: Live registry of active architectural drift alerts keyed by subsystem name.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### CommunicationHub
[Tier 2: Admin]
Type: struct
Purpose: Orchestrates real-time event broadcasting and human-in-the-loop oversight resolution.
Parameters:
- tx (`broadcast::Sender<LogEntry>`): Definition: Broadcast system logs to all connected UI WebSockets.
- event_tx (`broadcast::Sender<serde_json::Value>`): Definition: Dedicated broadcast for Engine events (decisions, lifecycle changes).
- telemetry_tx (`broadcast::Sender<serde_json::Value>`): Definition: Dedicated high-speed broadcast for agent telemetry (thinking, status).
- audio_stream_tx (`broadcast::Sender<Vec<u8>>`): Definition: Dedicated high-speed broadcast for neural audio streams (PCM chunks).
- pulse_tx (`broadcast::Sender<Arc<crate::telemetry::pulse_types::SwarmPulse>>`): Definition: High-speed binary pulse broadcasting for swarm visualization.
- oversight_queue (`DashMap<String, OversightEntry>`): Definition: Pending Oversight entries awaiting human decision.
- oversight_resolvers (`DashMap<String, oneshot::Sender<crate::agent::types::OversightResolution>>`): Definition: Resolvers for pending oversight promises.
- active_runners (`DashMap<String, RunnerHandle>`): Definition: Active `RunnerHandle` structs for running agents, providing abort control and real-time status inspection.
- runner_semaphore (`tokio::sync::Semaphore`): Definition: Semaphore to limit concurrent executing agents (runner pool throttle).
- event_sequence (`std::sync::atomic::AtomicU64`): Definition: Monotonic sequence counter for outbound engine events.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### GovernanceHub
[Tier 2: Admin]
Type: struct
Purpose: Centralizes system limits and automated policy enforcement for the agent swarm.
Parameters:
- auto_approve_safe_skills (AtomicBool): Definition: Global setting: whether to auto-approve low-risk skills.
- max_agents (AtomicU32): Definition: Maximum allowed agents in the swarm (Default: 50).
- max_clusters (AtomicU32): Definition: Maximum allowed concurrent mission clusters permitted (Default: 10).
- max_swarm_depth (AtomicU32): Definition: Maximum depth for agent recursion/spawning (Default: 5).
- max_task_length (AtomicUsize): Definition: Maximum token length for a single task (Default: 32768).
- default_budget_usd (`RwLock<f64>`): Definition: Default budget allocated to new agents in USD (Default: 1.0 USD).
- active_agents (AtomicU32): Definition: Number of agents currently executing tasks.
- recruit_count (AtomicU32): Definition: Total number of recruitment operations performed.
- tpm_accumulator (AtomicUsize): Definition: Global TPM (Tokens Per Minute) accumulator for telemetry.
- privacy_mode (AtomicBool): Definition: Privacy Shield: When true, all external cloud provider traffic is blocked.
- failover_amber_threshold (AtomicU32): Definition: Failover Amber threshold (failures before status becomes Amber, Default: 3).
- failover_red_threshold (AtomicU32): Definition: Failover Red threshold (failures before status becomes Red, Default: 5).
- failover_max_attempts (AtomicU32): Definition: Failover Max attempts (max retries to alternate models, Default: 3).
- provider_timeout_secs (AtomicU32): Definition: Default timeout for LLM provider generation calls in seconds (Default: 60).
- null_providers_test_mode (AtomicBool): Definition: Test mode flag to route all LLMs through NullProvider (TADPOLE_NULL_PROVIDERS=true).
- deprecated_routes (`RwLock<std::collections::HashMap<String, (String, String)>>`): Definition: Deprecated endpoints mapped to (Sunset Date, Alternate Link).
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### RegistryHub
[Tier 2: Admin]
Type: struct
Purpose: Centralized directory for swarm identities, model providers, and skill discovery.
Parameters:
- agents (`DashMap<String, EngineAgent>`): Definition: The live agent registry, synced with persistence.
- providers (`DashMap<String, ProviderConfig>`): Definition: Configured LLM providers (e.g., OpenAI, Ollama).
- provider_health (`DashMap<String, ProviderStatus>`): Definition: Real-time health status of providers (Amber/Red state machine).
- provider_failures (`DashMap<String, std::sync::atomic::AtomicU32>`): Definition: Recent failure counts for providers to trigger health transitions.
- models (`DashMap<String, ModelEntry>`): Definition: Available LLM models catalog.
- nodes (`DashMap<String, SwarmNode>`): Definition: Discovery registry for infrastructure nodes in the swarm.
- skills (`Arc<ScriptSkillsRegistry>`): Definition: Registry for dynamic file-based Skills and Workflows.
- skill_registry (`Arc<SkillRegistry>`): Definition: Manager for dynamic Skill Manifests (skill.json).
- mcp_host (`Arc<McpHost>`): Definition: Host for Model Context Protocol (MCP) tool aggregation.
- hooks (`Arc<HooksManager>`): Definition: Manager for Lifecycle Hooks (Pre/Post tool execution).
- tool_registry (`Arc<crate::agent::runner::tools::registry::ToolRegistry>`): Definition: Unified registry for all builtin and categorical tools.
- mission_backlogs (`DashMap<String, Arc<Mutex<MissionBacklog>>>`): Definition: Per-mission backlog queues for pending subtasks generated during conductor execution.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### ResourceHub
[Tier 2: Admin]
Type: struct
Purpose: Manages thread-safe access to heavy system resources and shared mission context.
Parameters:
- pool (SqlitePool): Definition: SQLite connection pool for persistent storage.
- http_client (`Arc<Client>`): Definition: Shared HTTP client with optimized connection pooling.
- audio_engine (`OnceCell<Arc<NeuralAudioEngine>>`): Definition: Native engine for local audio synthesis (PCM) and transcription (loaded lazily).
- audio_cache (`Arc<BunkerCache>`): Definition: Zero-latency semantic audio replicate cache for frequent phrases.
- code_graph (`OnceCell<Arc<RwLock<CodeGraph>>>`): Definition: Graph of code relationships for RAG-enhanced tool search.
- symbol_graph (`OnceCell<Arc<RwLock<CodeSymbolGraph>>>`): Definition: Symbol-level Knowledge Graph.
- obfuscation_salt (String): Definition: Dynamic boot-time cryptographically secure salt. Used for path obfuscation.
- identity_context (`OnceCell<String>`): Definition: Global system identity context loaded from `directives/IDENTITY.md`.
- memory_context (`OnceCell<String>`): Definition: Global long-term memory context loaded from `directives/LONG_TERM_MEMORY.md`.
- swarm_vault (`OnceCell<Arc<VectorMemory>>`): Definition: Global swarm-wide knowledge vault for cross-mission intelligence (compiled when vector-memory feature is active).
- knowledge_store (`OnceCell<Arc<crate::agent::knowledge_store::KnowledgeStore>>`): Definition: Persistent cross-cluster Institutional Knowledge Store (compiled when vector-memory feature is active).
- rate_limiters (`DashMap<String, Arc<RateLimiter>>`): Definition: Cached rate limiters partitioned by model and provider.
- initialization_registry (`DashMap<String, SubsystemStatus>`): Definition: Tracks the initialization status of all subsystems.
- hardware_profiler (`Arc<crate::system::profiler::HardwareProfiler>`): Definition: System hardware profiler for sovereign compute telemetry.
- acl (`Arc<dyn crate::agent::runner::service_traits::AclServiceTrait>`): Definition: Global Access Control List service for tool governance.
- renderer (`Arc<dyn crate::agent::runner::service_traits::PromptRendererTrait>`): Definition: System prompt template renderer.
- base_dir (std::path::PathBuf): Definition: Base directory for relative path resolution.
- tool_cache (`Arc<parking_lot::Mutex<crate::agent::runner::tools::cache::SharedToolCache>>`): Definition: Shared cache for read-only tool results.
- conflict_manager (`Arc<crate::security::conflict::ConflictManager>`): Definition: Registry for conflict locks during concurrent file operations.
- blueprint_cache (`tokio::sync::OnceCell<Arc<crate::services::blueprint_service::Blueprint>>`): Definition: Cached codebase blueprint for System 2 indexing.
- payment_router (`Arc<crate::agent::runner::a2a_router::PaymentRouter>`): Definition: PaymentRouter managing local mock transactions, L3 hybrid blockchain exits, economic zones, and daily spend limit checks.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### SecurityHub
[Tier 3: Security]
Type: struct
Purpose: Manages tamper-evident auditing and preventative security protocols.
Parameters:
- audit_trail (`Arc<MerkleAuditTrail>`): Definition: Tamper-evident audit trail engine (Merkle Hash Chain).
- budget_guard (`Arc<BudgetGuard>`): Definition: Persistent budget governance and metering engine.
- shell_scanner (`Arc<ShellScanner>`): Definition: Proactive shell safety scanner (API key leak protection).
- secret_redactor (`Arc<SecretRedactor>`): Definition: Runtime secret redactor for logs and telemetry.
- system_monitor (`Arc<SecurityMonitor>`): Definition: System resource and environment monitor.
- permission_policy (`Arc<PermissionPolicy>`): Definition: Dynamic tool permission and governance policy engine.
- deploy_token (`String`): Definition: Authentication token for administrative/deploy requests.
- admin_token (`String`): Definition: Secondary administrative access token for privileged system operations.
- oversight_public_key (`Option<String>`): Definition: Optional RSA/EC public key used to verify cryptographically signed oversight decisions.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### LogEntry
[Tier 3: Security]
Type: struct
Purpose: Atomic unit of telemetry mirroring the frontend event interface.
Parameters:
- event_type (String): Definition: Category of event (renamed to `"type"` in JSON, e.g. `"log"`).
- id (String): Definition: Unique UUID v4 identifier.
- timestamp (`DateTime<Utc>`): Definition: Event creation time.
- source (String): Definition: Originating subsystem (e.g., "System", "Agent").
- severity (String): Definition: Log level (e.g., "INFO", "CRITICAL").
- agent_id (`Option<String>`): Definition: Optional ID of the agent associated with this log.
- agent_name (`Option<String>`): Definition: Optional name of the agent associated with this log.
- mission_id (`Option<String>`): Definition: Optional correlation ID for the swarm mission.
- text (String): Definition: The redacted event message.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### TelemetryLayer
[Tier 3: Security]
Type: struct
Purpose: Tracing subscriber layer bridging internal spans to OTel-compatible JSON events.
Parameters:
- redactor (SecretRedactor): Definition: Redacts sensitive patterns within log messages.
Return Value: N/A
Side Effects: Broadcasts JSON events to TELEMETRY_TX on span creation and closure.
Failure Conditions: N/A

### AgentRunner
[Tier 2: Admin]
Type: struct
Purpose: Execution engine orchestrating the autonomous mission lifecycle.
Parameters:
- state (`Arc<AppState>`): Definition: Reference to global application state.
- model_router (`Arc<dyn ModelRouter>`): Definition: Interface for routing LLM inference calls to appropriate providers.
- prompt_service (`Arc<dyn PromptService>`): Definition: Interface for loading and building prompt templates.
- tool_orchestrator (`Arc<dyn ToolOrchestrator>`): Definition: Interface for executing external tools with budget checks.
- mission_state_manager (`Arc<dyn MissionStateManager>`): Definition: Interface for saving/loading mission context.
- workflow_coordinator (`Arc<dyn WorkflowCoordinator>`): Definition: Coordinator for executing deterministic SOP sequences.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### RunContext
[Tier 2: Admin]
Type: struct
Purpose: Data container carrying mission state and environment throughout an execution loop.
Parameters:
- agent_id (String): Definition: Unique identifier for the agent.
- name (String): Definition: Human-readable name of the agent.
- role (String): Definition: Dynamic role assigned to the agent.
- department (String): Definition: Functional department of the agent.
- description (String): Definition: Summary of the agent's responsibilities.
- model_config (ModelConfig): Definition: Primary LLM configuration.
- skills (`Vec<String>`): Definition: List of enabled custom files/skills.
- workflows (`Vec<String>`): Definition: List of SOP workflows registered to the agent.
- agent_models (AgentModels): Definition: Alternate slots for planning and execution LLMs.
- mcp_tools (`Vec<String>`): Definition: List of tool names exposed by active MCP servers.
- mission_id (String): Definition: Unique identifier for the current cluster mission.
- user_id (`Option<String>`): Definition: Optional identifier of the initiating user.
- depth (u32): Definition: Swarm recursion depth level (limit-controlled).
- lineage (`Vec<String>`): Definition: Parent agent IDs leading to this instance.
- provider_name (String): Definition: Name of the active LLM provider.
- workspace_root (PathBuf): Definition: Sandbox directory for file operations.
- fs_adapter (FilesystemAdapter): Definition: Zero-trust adapter restricting path traversals.
- safe_mode (bool): Definition: When enabled, hazardous commands/actions require user confirmation.
- analysis (bool): Definition: Toggle flag for deep tracing metrics.
- traceparent (`Option<String>`): Definition: OTel trace context propagation header.
- last_accessed_files (`Arc<Mutex<Vec<String>>>`): Definition: Dynamic tracking of files read during execution.
- modified_files (`Arc<Mutex<Vec<String>>>`): Definition: Dynamic tracking of files modified during execution.
- commands_run (`Arc<Mutex<HashSet<String>>>`): Definition: Set of external shell commands run by the agent.
- allowed_files (`Option<Vec<String>>`): Definition: Specific file whitelist constraints (if sandbox-configured).
- recent_findings (`Option<String>`): Definition: Cached discoveries/findings for cross-turn retention.
- working_memory (Value): Definition: Ephemeral JSON scratchpad for the agent's internal state.
- base_dir (PathBuf): Definition: Root persistence directory of the engine server.
- summarized_history (`Option<String>`): Definition: Compacted representation of past turns to reduce token window pressure.
- structured_output (bool): Definition: Toggle flag requesting structured JSON format.
- backlog (`Option<Arc<Mutex<MissionBacklog>>>`): Definition: Queue of pending subtasks generated during execution.
- primary_goal (`Option<String>`): Definition: The top-level mission goal string.
- budget_usd (f64): Definition: Monetary cap allocated for the run.
- current_cost_usd (f64): Definition: Total cost incurred during the active run (cumulative).
- reasoning_depth (u32): Definition: Number of steps used for reasoning/CoT.
- act_threshold (f32): Definition: Score threshold for taking actions.
- max_turns (u32): Definition: Maximum iterations allowed for the run loop.
- authority_level (RoleAuthorityLevel): Definition: Dynamic permission tier mapped to the agent's role.
- resource_weights (`HashMap<String, f32>`): Definition: Resource priority distribution parameters.
- graph_context (`Option<String>`): Definition: Codebase symbol relationships injected for target files.
- verification_passed (bool): Definition: Flag tracking testing verification results.
- visible_transcript (`Vec<serde_json::Value>`): Definition: The agent's current turn history as raw JSON, visible for context-compression decisions.
- conductor_plan (`Option<Arc<ConductorPlan>>`): Definition: The active Conductor DAG plan for the current mission, if one has been generated.
- sub_budget_usd (`Option<f64>`): Definition: Optional per-sub-agent budget cap, overriding the global `budget_usd` for spawned child agents.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### ModelConfig
[Tier 2: Admin]
Type: struct
Purpose: Configuration for an LLM instance including system prompts and performance thresholds.
Parameters:
- provider (ModelProvider): Definition: Target backend provider protocol.
- model_id (String): Definition: Specific model identifier.
- api_key (`Option<String>`): Definition: Optional access key override.
- base_url (`Option<String>`): Definition: Optional backend endpoint URL override.
- system_prompt (`Option<String>`): Definition: Custom system prompt instruction override.
- temperature (`Option<f32>`): Definition: LLM sampling temperature.
- max_tokens (`Option<u32>`): Definition: Maximum response tokens.
- external_id (`Option<String>`): Definition: Telemetry / tracking identifier.
- rpm (`Option<u32>`): Definition: Rate limit threshold (Requests Per Minute).
- rpd (`Option<u32>`): Definition: Rate limit threshold (Requests Per Day).
- tpm (`Option<u32>`): Definition: Rate limit threshold (Tokens Per Minute).
- tpd (`Option<u32>`): Definition: Rate limit threshold (Tokens Per Day).
- skills (`Option<Vec<String>>`): Definition: Enabled skill whitelists.
- workflows (`Option<Vec<String>>`): Definition: Enabled SOP workflow mappings.
- mcp_tools (`Option<Vec<String>>`): Definition: Enabled MCP tool selections.
- steering_vectors (`Option<Vec<String>>`): Definition: Model alignment steering configurations.
- reasoning_depth (`Option<u32>`): Definition: Maximum internal reasoning turns (e.g. for o1/r1 models).
- act_threshold (`Option<f32>`): Definition: Confidence threshold for autonomous actions.
- max_turns (`Option<u32>`): Definition: Maximum turn limit override.
- connector_configs (`Option<Vec<ConnectorConfig>>`): Definition: Database or knowledge connector details.
- extra_parameters (`Option<HashMap<String, Value>>`): Definition: Additional provider-specific parameters.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### ModelCapabilities
[Tier 2: Admin]
Type: struct
Purpose: Feature matrix defining the functional limits and supported modalities of an LLM.
Parameters:
- supports_tools (bool): Definition: Support for external function calling.
- supports_vision (bool): Definition: Support for image processing.
- supports_structured_output (bool): Definition: Support for structured JSON schemas in responses.
- supports_reasoning (bool): Definition: Support for internal reasoning loops (CoT).
- supports_halting_tool (bool): Definition: Support for explicitly yielding/stopping execution turns.
- context_window (u32): Definition: Total token capacity.
- max_output_tokens (u32): Definition: Maximum generation limit per API call.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### ProblemDetails
[Tier 3: Security]
Type: struct
Purpose: RFC 9457 compliant structure for machine-readable API error responses.
Parameters:
- type_uri (String): Definition: URI identifying the specific error type (formatted as `https://tadpole.os/errors/{slug}`).
- title (String): Definition: Human-readable summary of the error.
- status (u16): Definition: HTTP status code.
- detail (String): Definition: Sanitized and redacted explanation of the failure.
- instance (`Option<String>`): Definition: Unique request/transaction URI path context.
- error_code (`Option<String>`): Definition: Uppercase alphanumeric error code representation.
- help_link (`Option<String>`): Definition: Publicly accessible documentation link for troubleshooting.
- severity (String): Definition: Logging severity tier (e.g., `"CRITICAL"`, `"ERROR"`, `"WARNING"`).
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### InstallTemplateRequest
[Tier 2: Admin]
Type: struct
Purpose: Request payload for installing a swarm template from a remote repository.
Parameters:
- repository_url (String): Definition: Git URL of the template source.
- path (String): Definition: Path within the repository for the template configuration.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### KnowledgeEntry
[Tier 1: Operator]
Type: struct
Purpose: Represents a single record of institutional memory stored in the Institutional Knowledge Store.
Parameters:
- id (String): Definition: Unique UUID v4 identifying the entry.
- text (String): Definition: The raw content snippet.
- topic (String): Definition: Categorical classification or tag.
- cluster_id (Option<String>): Definition: Optional cluster/project workspace context.
- source_node_id (Option<String>): Definition: The Bunker node that authored this entry.
- source_agent_id (Option<String>): Definition: The agent that authored this entry.
- content_hash (String): Definition: SHA-256 fingerprint for deduplication.
- confidence (f32): Definition: Accuracy score (0.0 to 1.0) decays over time unless human-confirmed.
- human_confirmed (bool): Definition: Flag indicating manual validation by an operator.
- ttl (Option<i64>): Definition: Expiration timestamp.
- created_at (i64): Definition: Creation timestamp.
- access_count (i64): Definition: Number of times this node has been read.
- concept_type (String): Definition: OKF concept classification.
- title (Option<String>): Definition: Descriptive header.
- description (Option<String>): Definition: Extended documentation.
- resource_uri (Option<String>): Definition: Canonical target link.
- tags (Option<String>): Definition: Comma-separated descriptors.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### AddKnowledgeRequest
[Tier 1: Operator]
Type: struct
Purpose: Payload parameters for submitting a new knowledge asset to the IKS.
Parameters:
- text (String): Definition: Raw textual content of the knowledge node.
- topic (String): Definition: General category.
- cluster_id (Option<String>): Definition: Optional cluster scope.
- source_node_id (Option<String>): Definition: Local/remote node ID.
- source_agent_id (Option<String>): Definition: Authoring agent ID.
- confidence (Option<f32>): Definition: Initial confidence score.
- ttl_days (Option<i64>): Definition: Number of days until expiry.
- human_confirmed (Option<bool>): Definition: Pre-confirm flag.
- concept_type (Option<String>): Definition: OKF concept type.
- title (Option<String>): Definition: Descriptive title.
- description (Option<String>): Definition: Extended description.
- resource_uri (Option<String>): Definition: Canonical reference link.
- tags (Option<String>): Definition: Tags.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

---

## 🏗️ Enums

### AppError
[Tier 3: Security]
Type: enum
Purpose: Unified application error enumeration for the engine, mapping failures to RFC 9457 / HTTP protocols.
Variants:
- `Runner(RunnerError)`: Wrapper for execution errors occurring in the agent runner.
- `Skill(SkillError)`: Wrapper for validation, safety, or recruitment errors inside custom script skills.
- `BadRequest(String)`: Malformed HTTP request or validation failure.
- `Unauthorized(String)`: Missing or invalid authentication token.
- `Forbidden(String)`: Action blocked by access control policies.
- `NotFound(String)`: Requested resource or entity does not exist.
- `DomainError { code, detail, help_link }`: Business logic failure, utilizing `DomainCode`.
- `InfrastructureError { provider_id, kind, detail, help_link }`: Downstream provider service failure, utilizing `ProviderId` and `InfrastructureErrorKind`.
- `QuantizationFallback { model_id, suggested_quant, detail }`: Mismatch in local model resource allocation.
- `NotImplemented(String)`: Action or protocol is not implemented.
- `RateLimit(String)`: API or provider rate limits exceeded.
- `InternalServerError(String)`: Unhandled internal system error.
- `Conflict(String)`: Conflicting concurrent operations (e.g. file lock).
- `DegradedState(String)`: Subsystem is in a degraded or unavailable state.
- `MultiError(Vec<AppError>)`: Aggregated collection of multiple concurrent errors.
- `Anyhow(anyhow::Error)`: Wildcard wrapper for custom contextual errors.
- `Sqlx(sqlx::Error)`: Database persistence failure.
- `Io(std::io::Error)`: Standard system input/output failure.
- `Reqwest(reqwest::Error)`: Downstream HTTP client failure.
- `Serde(serde_json::Error)`: Serialization/deserialization failure.
- `WalkDir(walkdir::Error)`: Filesystem directory traversal failure.
- `Graph(GraphError)`: Error occurring within the code structure or symbol knowledge graphs.
- `SecurityBoundaryViolation(String)`: A Zero-Trust boundary was crossed; the request was denied by the security policy engine.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### RunnerError
[Tier 3: Security]
Type: enum
Purpose: Specific execution and reasoning loop errors wrapped by `AppError`.
Variants:
- `BudgetExhausted(String)`: Mission cost exceeds the allocated budget limit.
- `RecursionBlocked(String)`: Swarm recursion depth limit has been reached.
- `SentinelGate(String)`: Guardrails or sentinel checks blocked execution.
- `Compression(String)`: Failure during agent turn history compression.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### SkillError
[Tier 3: Security]
Type: enum
Purpose: Specific custom script skill and recruitment errors wrapped by `AppError`.
Variants:
- `ValidationError(String)`: Input or configuration data failed validation checks.
- `RecruitmentFailure { recipe_id, role, detail }`: Spawning of a swarm sub-agent failed.
- `SanitizationViolation(String)`: Whitelist security scanner blocked unsafe content or command execution.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### RoleAuthorityLevel
[Tier 2: Admin]
Type: enum
Purpose: Hierarchical authority of an agent within the swarm.
Variants:
- `Executive`: Strategic oversight and delegation (CEO, Overlord).
- `Management`: Tactical coordination (Alpha Node).
- `Specialist`: Standard task execution.
- `Observer`: Read-only auditing.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### ModelProvider
[Tier 2: Admin]
Type: enum
Purpose: Supported backend protocols and hosting platforms for LLM interaction.
Variants:
- `Openai`, `Anthropic`, `Google`, `Gemini`, `Ollama`, `Groq`, `Mistral`, `Perplexity`, `Fireworks`, `Together`, `Deepseek`, `Xai`, `Inception`, `Openrouter`, `Cerebras`, `Sambanova`, `OllamaCloud`.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### SubsystemStatus
[Tier 3: Security]
Type: enum
Purpose: Lifecycle state machine for core engine components.
Variants:
- `NotStarted`: Component is uninitialized.
- `Warming(f32)`: Component is warming up, carrying the progress float (0.0 to 1.0).
- `Ready`: Component is fully initialized and operational.
- `Failed(String)`: Terminal initialization failure state.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### SystemHealthState
[Tier 3: Security]
Type: enum
Purpose: Overall status reporting for the swarm engine.
Variants:
- `Warming`: Critical subsystems are starting up.
- `Ready`: All critical subsystems operational.
- `Degraded`: Subsystems failed or are in a degraded state.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### Modality
[Tier 2: Admin]
Type: enum
Purpose: Feature modality categories.
Variants:
- `Llm`, `Vision`, `Voice`, `Audio`, `Reasoning`.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

### MissionStatus
[Tier 1: Operator]
Type: enum
Purpose: Swarm mission lifecycle status.
Variants:
- `Pending`, `SpecReview`, `Active`, `Completed`, `Failed`, `Paused`.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

---

## 📖 Terminology & Concepts

### Agent Scope
[Tier 2: Admin]
> [!NOTE]
> **Redirect**: See [RAG Scope](#rag-scope) for the canonical definition of Agent Scope / RAG Scope.

### Alpha Node
[Tier 1: Operator]
> [!NOTE]
> **Renamed Term**: This term has been renamed to **Department Lead**. See [Department Lead](#department-lead) for the canonical definition.

### A2A Ledger
[Tier 2: Admin]
Type: Terminology
Purpose: The persistent, SQLite-backed ledger storing cryptographically audited financial transaction logs between swarm agents executing paid services.
Implementation Hook: `a2a_ledger.rs` / `server-rs/src/agent/runner/a2a_ledger.rs`

### AXUM
[Tier 2: Admin]
Type: Technical Framework
Purpose: The high-performance, asynchronous Rust web framework used to run the Tadpole OS backend engine and expose HTTP/WebSocket service APIs.
Implementation Hook: `server-rs/src/router.rs`

### Circuit Breaker
[Tier 3: Security]
Type: Terminology
Purpose: Client-side resilience mechanism that monitors communication failures inside service namespaces (e.g. `infra`, `engine`) and short-circuits further requests to prevent dashboard locks.
Implementation Hook: `VITE_SYSTEM_BREAKER_FAILURE_THRESHOLD` / `VITE_SYSTEM_BREAKER_COOLDOWN_MS`

### Conductor DAG
[Tier 2: Admin]
Type: Terminology
Purpose: The Directed Acyclic Graph plan generated by the Conductor engine representing decomposed subtask dependencies for a complex swarm objective.
Implementation Hook: `conductor.rs` / `server-rs/src/agent/runner/conductor.rs`

### Context Compaction
[Tier 2: Admin]
Type: Terminology
Purpose: The automated, $O(N)$ linear dialogue cleanup procedure that preserves system prompt limits by shrinking or omitting older turns' logs while leaving the last 4 turns raw.
Implementation Hook: `ContextManager::compact` / `server-rs/src/agent/context_manager.rs`

### Department Lead
[Tier 1: Operator]
Type: Terminology
Purpose: The primary coordinator agent in a workspace responsible for breaking down goals, recruiting specialist sub-agents, and compiling reports. Represents the management tier in the command chain.
Implementation Hook: `RoleAuthorityLevel::Management` / `state.registry.list_active_specialists()`

### Department Scope
[Tier 1: Operator]
Type: Terminology
Purpose: A communications constraint in the SovereignChat that restricts broadcasted instructions to all agents and sub-agents assigned to a specific workspace folder or cluster.
Implementation Hook: `SovereignChat` / `RunContext::department`

### Engine Shutdown
[Tier 3: Security]
Type: Terminology
Purpose: The graceful server termination protocol that persists all live agent states to the database and cleanly exits the backend process.
Implementation Hook: `POST /v1/engine/shutdown` / `routes::engine_control::shutdown_engine`

### Institutional Knowledge Store (IKS)
[Tier 1: Operator]
Type: Terminology
Purpose: The local, SQLite-backed repository managing swarm-wide insights, memory vectors, and validated operational procedures.
Implementation Hook: `ResourceHub::knowledge_store` / `routes::knowledge`

### Kill Switch
[Tier 3: Security]
Type: Terminology
Purpose: An emergency intervention mechanism that immediately halts all active agent thinking and execution loops and rejects pending oversight requests.
Implementation Hook: `POST /v1/engine/kill` / `routes::engine_control::kill_agents`

### Master Passphrase
[Tier 3: Security]
Type: Terminology
Purpose: The cryptographic password entered by the operator to derive key material via PBKDF2 for decrypting stored provider API keys in memory.
Implementation Hook: `crypto.worker.ts` / `use_vault_store`

### Mission Cluster
[Tier 1: Operator]
> [!NOTE]
> **Redirect**: This term has been renamed to **Project Workspace**. See [Project Workspace](#project-workspace) for the canonical definition.

### Neural Vault
[Tier 3: Security]
> [!NOTE]
> **Redirect**: This term has been renamed to **Secure Credentials Vault**. See [Secure Credentials Vault](#secure-credentials-vault) for the canonical definition.

### Open Knowledge Format (OKF)
[Tier 1: Operator]
Type: Terminology
Purpose: A standardized, self-describing draft specification (v0.1) for structuring institutional knowledge nodes with structured frontmatter metadata fields.
Implementation Hook: `KnowledgeEntry` / `AddKnowledgeRequest`

### Parity Guard
[Tier 2: Admin]
Type: Tooling
Purpose: A Python-driven validation command-line script that scans code files, routes, manifests, and documentation trees to flag drift or omissions.
Implementation Hook: `execution/parity_guard.py`

### Private Swarm Environment
[Tier 1: Operator]
Type: Terminology
Purpose: The secure, local-first execution environment where virtual agents run tools, process documents, and interact with LLMs without external data leaks.
Implementation Hook: `AppState` / `AgentRunner`

### Project Workspace
[Tier 1: Operator]
Type: Terminology
Purpose: An isolated local project directory containing state files, agent logs, configurations, and workspaces grouped toward a single business goal.
Implementation Hook: `GovernanceHub::max_clusters` / `RunContext::mission_id`

### RAG Scope
[Tier 2: Admin]
Type: Terminology
Purpose: The data ingestion context constraint limiting an agent's retrieval to specific files, workspace directories, or long-term memory.
Implementation Hook: `ResourceHub::symbol_graph` / `ResourceHub::code_graph`

### Secure Credentials Vault
[Tier 3: Security]
Type: Terminology
Purpose: Client-side credentials storage encrypted using `PBKDF2 + AES-256-GCM` via standard `SubtleCrypto` APIs. Stores LLM provider credentials securely in the browser.
Implementation Hook: `use_vault_store` / `crypto.worker.ts`

### Sovereign Reality
[Tier 1: Operator]
> [!NOTE]
> **Redirect**: This term has been renamed to **Private Swarm Environment**. See [Private Swarm Environment](#private-swarm-environment) for the canonical definition.

### STATIC_DIR
[Tier 2: Admin]
Type: Environment Variable
Purpose: The environment configuration specifying the directory path from which static frontend assets are served by the AXUM web server.
Implementation Hook: `std::env::var("STATIC_DIR")` / `server-rs/src/router.rs`

### Swarm Depth
[Tier 2: Admin]
Type: Terminology
Purpose: The recursion level of sub-agent recruitment within a mission. Enforced to prevent infinite delegation loops.
Implementation Hook: `MAX_SWARM_DEPTH` / `GovernanceHub::max_swarm_depth`

### Swarm Pulse
[Tier 3: Security]
Type: Terminology
Purpose: A real-time binary telemetry event stream showing system communications, agent thinking state, and tool calls.
Implementation Hook: `CommunicationHub::pulse_tx` / `SwarmPulse`

### Topological Swarm Scheduler
[Tier 2: Admin]
Type: Terminology
Purpose: The scheduling mechanism that executes Conductor steps according to topological dependency ordering, resolving cycles and deadlocks.
Implementation Hook: `topological_sort_conductor_steps` / `server-rs/src/agent/runner/conductor.rs`

### Two-Phase Commit (2PC) Swarm Transaction
[Tier 3: Security]
Type: Terminology
Purpose: The transaction protocol ensuring atomic commitment of financial balances across sub-missions, featuring a prepare phase and a rollback guarantee upon failure.
Implementation Hook: `prepare_transaction` / `commit_transaction` / `rollback_transaction`

---

## 🏗️ Compliance & Quality Standards

### ALCOA+
[Tier 3: Security]
Type: Standard
Purpose: International data integrity guidelines (Attributable, Legible, Contemporaneous, Original, Accurate, Complete, Consistent, Enduring, Available) enforced across the engine telemetry streams.
Implementation Hook: `server-rs/src/security/audit.rs`

### GxP
[Tier 3: Security]
Type: Standard
Purpose: Good Practice quality guidelines and regulations (e.g., GCP, GMP, GLP) to ensure the system is safe for regulated domains.
Implementation Hook: `execution/verify_all.py`

### Software Validation Package (SVP)
[Tier 3: Security]
Type: Compliance Artifact
Purpose: Formally compiled validation documentation package (including FMEA risk assessment, OQ/PQ test scripts) for regulatory sign-off.
Implementation Hook: `docs/TEST_MISSIONS.md`

---

## 🏗️ Developer Tooling

### Active Documentation Guard (ADG)
[Tier 1: Developer Tooling]
Type: System / CLI Feature
Purpose: Enforces symbol-to-docstring parity between live code and Rustdoc/JSDoc comments. Extracts backtick-quoted symbol references from doc comments and verifies they resolve against the `CodeSymbolGraph`. Mismatches are surfaced with Jaro-Winkler fuzzy suggestions and can be auto-corrected in-place.
Implementation Hook: `server-rs/src/bin/graph_query/doc_guard.rs`
CLI: `graph_query validate [--strict] [--diff] [--fix]`

### docstring_range
[Tier 1: Developer Tooling]
Type: Field (`Option<SymbolRange>`)
Purpose: Optional field on the `Symbol` struct capturing the exact byte start/end and line start/end coordinates of associated JSDoc or Rustdoc comments. Propagated to `SymbolNode` in the graph. Enables back-to-front in-place byte-coordinate auto-fixes without invalidating prior offsets.
Implementation Hook: `server-rs/src/utils/parser.rs`

### SymbolNode
[Tier 1: Developer Tooling]
Type: struct
Purpose: Graph vertex in the `CodeSymbolGraph` representing a single symbol (function, struct, trait, interface, etc.) with its name, kind, file path, line range, documentation range, and incoming/outgoing reference edges.
Implementation Hook: `server-rs/src/intelligence/graph.rs`

### tauri_ipc: (Symbol Prefix)
[Tier 1: Developer Tooling]
Type: Synthetic symbol prefix
Purpose: Symbols emitted by the parser for `#[tauri::command]` annotated Rust functions and matching `invoke()` call-sites in TSX files. Allows the `CodeSymbolGraph` to detect frontend/backend boundary drift by comparing paired `tauri_ipc:<command_name>` nodes across both language ASTs.
Implementation Hook: `server-rs/src/utils/parser.rs`

### Jaro-Winkler Similarity
[Tier 1: Developer Tooling]
Type: Algorithm
Purpose: String similarity metric used by the Active Documentation Guard to suggest the closest live symbol name when a docstring reference cannot be resolved. Implemented via the `strsim` crate (`strsim::jaro_winkler`). Suggestions are shown at similarity ≥ 0.85 and applied automatically when `--fix` is passed.
Implementation Hook: `server-rs/src/bin/graph_query/doc_guard.rs`

---

[//]: # (Metadata: [GLOSSARY])
