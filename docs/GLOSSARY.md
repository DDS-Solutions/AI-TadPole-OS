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

--- TELEMETRY_TX ---
Type: static
Purpose: Global broadcast channel for high-throughput JSON telemetry emissions from system spans and agent lifecycle events.
Parameters:
- N/A
Return Value: `Lazy&lt;broadcast::Sender&lt;serde_json::Value&gt;&gt;`. The sender side of a tokio broadcast channel with a capacity of 2000 events.
Side Effects: N/A
Failure Conditions: N/A

---

## 🏗️ Structs

--- AppState ---
Type: struct
Purpose: Primary global application state container and coordination hub for all sovereign subsystems.
Parameters:
- comms (`Arc&lt;CommunicationHub&gt;`): Definition: Manages real-time communication channels and telemetry.
- governance (`Arc&lt;GovernanceHub&gt;`): Definition: Manages operational limits and global policy settings.
- registry (`Arc&lt;RegistryHub&gt;`): Definition: Maintains thread-safe registries for agents, providers, and skills.
- security (`Arc&lt;SecurityHub&gt;`): Definition: Handles audit trails, budget enforcement, and shell safety scanning.
- resources (`Arc&lt;ResourceHub&gt;`): Definition: Manages shared system resources including database pools and HTTP clients.
- base_dir (PathBuf): Definition: The root directory for persistent data and workspace sandboxes.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- CommunicationHub ---
Type: struct
Purpose: Orchestrates real-time event broadcasting and human-in-the-loop oversight resolution.
Parameters:
- tx (`broadcast::Sender&lt;LogEntry&gt;`): Definition: Broadcast system logs to all connected UI WebSockets.
- event_tx (`broadcast::Sender&lt;serde_json::Value&gt;`): Definition: Dedicated broadcast for Engine events (decisions, lifecycle changes).
- telemetry_tx (`broadcast::Sender&lt;serde_json::Value&gt;`): Definition: Dedicated high-speed broadcast for agent telemetry (thinking, status).
- audio_stream_tx (`broadcast::Sender&lt;Vec&lt;u8&gt;&gt;`): Definition: Dedicated high-speed broadcast for neural audio streams (PCM chunks).
- pulse_tx (`broadcast::Sender&lt;Arc&lt;crate::telemetry::pulse_types::SwarmPulse&gt;&gt;`): Definition: High-speed binary pulse broadcasting for swarm visualization.
- oversight_queue (`DashMap&lt;String, OversightEntry&gt;`): Definition: Pending Oversight entries awaiting human decision.
- oversight_resolvers (`DashMap&lt;String, oneshot::Sender&lt;crate::agent::types::OversightResolution&gt;&gt;`): Definition: Resolvers for pending oversight promises.
- active_runners (`DashMap&lt;String, tokio::task::AbortHandle&gt;`): Definition: Active AbortHandles for running agents, allowing for definitive task cancellation.
- runner_semaphore (`tokio::sync::Semaphore`): Definition: Semaphore to limit concurrent executing agents (runner pool throttle).
- event_sequence (`std::sync::atomic::AtomicU64`): Definition: Monotonic sequence counter for outbound engine events.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- GovernanceHub ---
Type: struct
Purpose: Centralizes system limits and automated policy enforcement for the agent swarm.
Parameters:
- auto_approve_safe_skills (AtomicBool): Definition: Global setting: whether to auto-approve low-risk skills.
- max_agents (AtomicU32): Definition: Maximum allowed agents in the swarm (Default: 50).
- max_clusters (AtomicU32): Definition: Maximum allowed concurrent mission clusters permitted (Default: 10).
- max_swarm_depth (AtomicU32): Definition: Maximum depth for agent recursion/spawning (Default: 5).
- max_task_length (AtomicUsize): Definition: Maximum token length for a single task (Default: 32768).
- default_budget_usd (`RwLock&lt;f64&gt;`): Definition: Default budget allocated to new agents in USD (Default: 1.0 USD).
- active_agents (AtomicU32): Definition: Number of agents currently executing tasks.
- recruit_count (AtomicU32): Definition: Total number of recruitment operations performed.
- tpm_accumulator (AtomicUsize): Definition: Global TPM (Tokens Per Minute) accumulator for telemetry.
- privacy_mode (AtomicBool): Definition: Privacy Shield: When true, all external cloud provider traffic is blocked.
- failover_amber_threshold (AtomicU32): Definition: Failover Amber threshold (failures before status becomes Amber, Default: 3).
- failover_red_threshold (AtomicU32): Definition: Failover Red threshold (failures before status becomes Red, Default: 5).
- failover_max_attempts (AtomicU32): Definition: Failover Max attempts (max retries to alternate models, Default: 3).
- provider_timeout_secs (AtomicU32): Definition: Default timeout for LLM provider generation calls in seconds (Default: 60).
- null_providers_test_mode (AtomicBool): Definition: Test mode flag to route all LLMs through NullProvider (TADPOLE_NULL_PROVIDERS=true).
- deprecated_routes (`RwLock&lt;std::collections::HashMap&lt;String, (String, String)&gt;&gt;`): Definition: Deprecated endpoints mapped to (Sunset Date, Alternate Link).
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- RegistryHub ---
Type: struct
Purpose: Centralized directory for swarm identities, model providers, and skill discovery.
Parameters:
- agents (`DashMap&lt;String, EngineAgent&gt;`): Definition: The live agent registry, synced with persistence.
- providers (`DashMap&lt;String, ProviderConfig&gt;`): Definition: Configured LLM providers (e.g., OpenAI, Ollama).
- provider_health (`DashMap&lt;String, ProviderStatus&gt;`): Definition: Real-time health status of providers (Amber/Red state machine).
- provider_failures (`DashMap&lt;String, std::sync::atomic::AtomicU32&gt;`): Definition: Recent failure counts for providers to trigger health transitions.
- models (`DashMap&lt;String, ModelEntry&gt;`): Definition: Available LLM models catalog.
- nodes (`DashMap&lt;String, SwarmNode&gt;`): Definition: Discovery registry for infrastructure nodes in the swarm.
- skills (`Arc&lt;ScriptSkillsRegistry&gt;`): Definition: Registry for dynamic file-based Skills and Workflows.
- skill_registry (`Arc&lt;SkillRegistry&gt;`): Definition: Manager for dynamic Skill Manifests (skill.json).
- mcp_host (`Arc&lt;McpHost&gt;`): Definition: Host for Model Context Protocol (MCP) tool aggregation.
- hooks (`Arc&lt;HooksManager&gt;`): Definition: Manager for Lifecycle Hooks (Pre/Post tool execution).
- tool_registry (`Arc&lt;crate::agent::runner::tools::registry::ToolRegistry&gt;`): Definition: Unified registry for all builtin and categorical tools.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- ResourceHub ---
Type: struct
Purpose: Manages thread-safe access to heavy system resources and shared mission context.
Parameters:
- pool (SqlitePool): Definition: SQLite connection pool for persistent storage.
- http_client (`Arc&lt;Client&gt;`): Definition: Shared HTTP client with optimized connection pooling.
- audio_engine (`OnceCell&lt;Arc&lt;NeuralAudioEngine&gt;&gt;`): Definition: Native engine for local audio synthesis (PCM) and transcription (loaded lazily).
- audio_cache (`Arc&lt;BunkerCache&gt;`): Definition: Zero-latency semantic audio replicate cache for frequent phrases.
- code_graph (`OnceCell&lt;Arc&lt;RwLock&lt;CodeGraph&gt;&gt;&gt;`): Definition: Graph of code relationships for RAG-enhanced tool search.
- symbol_graph (`OnceCell&lt;Arc&lt;RwLock&lt;CodeSymbolGraph&gt;&gt;&gt;`): Definition: Symbol-level Knowledge Graph.
- obfuscation_salt (String): Definition: Dynamic boot-time cryptographically secure salt. Used for path obfuscation.
- identity_context (`OnceCell&lt;String&gt;`): Definition: Global system identity context loaded from `directives/IDENTITY.md`.
- memory_context (`OnceCell&lt;String&gt;`): Definition: Global long-term memory context loaded from `directives/LONG_TERM_MEMORY.md`.
- swarm_vault (`OnceCell&lt;Arc&lt;VectorMemory&gt;&gt;`): Definition: Global swarm-wide knowledge vault for cross-mission intelligence (compiled when vector-memory feature is active).
- knowledge_store (`OnceCell&lt;Arc&lt;crate::agent::knowledge_store::KnowledgeStore&gt;&gt;`): Definition: Persistent cross-cluster Institutional Knowledge Store (compiled when vector-memory feature is active).
- rate_limiters (`DashMap&lt;String, Arc&lt;RateLimiter&gt;&gt;`): Definition: Cached rate limiters partitioned by model and provider.
- initialization_registry (`DashMap&lt;String, SubsystemStatus&gt;`): Definition: Tracks the initialization status of all subsystems.
- hardware_profiler (`Arc&lt;crate::system::profiler::HardwareProfiler&gt;`): Definition: System hardware profiler for sovereign compute telemetry.
- acl (`Arc&lt;dyn crate::agent::runner::service_traits::AclServiceTrait&gt;`): Definition: Global Access Control List service for tool governance.
- renderer (`Arc&lt;dyn crate::agent::runner::service_traits::PromptRendererTrait&gt;`): Definition: System prompt template renderer.
- base_dir (std::path::PathBuf): Definition: Base directory for relative path resolution.
- tool_cache (`Arc&lt;parking_lot::Mutex&lt;crate::agent::runner::tools::cache::SharedToolCache&gt;&gt;`): Definition: Shared cache for read-only tool results.
- conflict_manager (`Arc&lt;crate::security::conflict::ConflictManager&gt;`): Definition: Registry for conflict locks during concurrent file operations.
- blueprint_cache (`tokio::sync::OnceCell&lt;Arc&lt;crate::services::blueprint_service::Blueprint&gt;&gt;`): Definition: Cached codebase blueprint for System 2 indexing.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- SecurityHub ---
Type: struct
Purpose: Manages tamper-evident auditing and preventative security protocols.
Parameters:
- audit_trail (`Arc&lt;MerkleAuditTrail&gt;`): Definition: Tamper-evident audit trail engine (Merkle Hash Chain).
- budget_guard (`Arc&lt;BudgetGuard&gt;`): Definition: Persistent budget governance and metering engine.
- shell_scanner (`Arc&lt;ShellScanner&gt;`): Definition: Proactive shell safety scanner (API key leak protection).
- secret_redactor (`Arc&lt;SecretRedactor&gt;`): Definition: Runtime secret redactor for logs and telemetry.
- system_monitor (`Arc&lt;SecurityMonitor&gt;`): Definition: System resource and environment monitor.
- permission_policy (`Arc&lt;PermissionPolicy&gt;`): Definition: Dynamic tool permission and governance policy engine.
- deploy_token (String): Definition: Authentication token for administrative/deploy requests.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- LogEntry ---
Type: struct
Purpose: Atomic unit of telemetry mirroring the frontend event interface.
Parameters:
- event_type (String): Definition: Category of event (renamed to `"type"` in JSON, e.g. `"log"`).
- id (String): Definition: Unique UUID v4 identifier.
- timestamp (`DateTime&lt;Utc&gt;`): Definition: Event creation time.
- source (String): Definition: Originating subsystem (e.g., "System", "Agent").
- severity (String): Definition: Log level (e.g., "INFO", "CRITICAL").
- agent_id (`Option&lt;String&gt;`): Definition: Optional ID of the agent associated with this log.
- agent_name (`Option&lt;String&gt;`): Definition: Optional name of the agent associated with this log.
- mission_id (`Option&lt;String&gt;`): Definition: Optional correlation ID for the swarm mission.
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
- state (`Arc&lt;AppState&gt;`): Definition: Reference to global application state.
- model_router (`Arc&lt;dyn ModelRouter&gt;`): Definition: Interface for routing LLM inference calls to appropriate providers.
- prompt_service (`Arc&lt;dyn PromptService&gt;`): Definition: Interface for loading and building prompt templates.
- tool_orchestrator (`Arc&lt;dyn ToolOrchestrator&gt;`): Definition: Interface for executing external tools with budget checks.
- mission_state_manager (`Arc&lt;dyn MissionStateManager&gt;`): Definition: Interface for saving/loading mission context.
- workflow_coordinator (`Arc&lt;dyn WorkflowCoordinator&gt;`): Definition: Coordinator for executing deterministic SOP sequences.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- RunContext ---
Type: struct
Purpose: Data container carrying mission state and environment throughout an execution loop.
Parameters:
- agent_id (String): Definition: Unique identifier for the agent.
- name (String): Definition: Human-readable name of the agent.
- role (String): Definition: Dynamic role assigned to the agent.
- department (String): Definition: Functional department of the agent.
- description (String): Definition: Summary of the agent's responsibilities.
- model_config (ModelConfig): Definition: Primary LLM configuration.
- skills (`Vec&lt;String&gt;`): Definition: List of enabled custom files/skills.
- workflows (`Vec&lt;String&gt;`): Definition: List of SOP workflows registered to the agent.
- agent_models (AgentModels): Definition: Alternate slots for planning and execution LLMs.
- mcp_tools (`Vec&lt;String&gt;`): Definition: List of tool names exposed by active MCP servers.
- mission_id (String): Definition: Unique identifier for the current cluster mission.
- user_id (`Option&lt;String&gt;`): Definition: Optional identifier of the initiating user.
- depth (u32): Definition: Swarm recursion depth level (limit-controlled).
- lineage (`Vec&lt;String&gt;`): Definition: Parent agent IDs leading to this instance.
- provider_name (String): Definition: Name of the active LLM provider.
- workspace_root (PathBuf): Definition: Sandbox directory for file operations.
- fs_adapter (FilesystemAdapter): Definition: Zero-trust adapter restricting path traversals.
- safe_mode (bool): Definition: When enabled, hazardous commands/actions require user confirmation.
- analysis (bool): Definition: Toggle flag for deep tracing metrics.
- traceparent (`Option&lt;String&gt;`): Definition: OTel trace context propagation header.
- last_accessed_files (`Arc&lt;Mutex&lt;Vec&lt;String&gt;&gt;&gt;`): Definition: Dynamic tracking of files read during execution.
- modified_files (`Arc&lt;Mutex&lt;Vec&lt;String&gt;&gt;&gt;`): Definition: Dynamic tracking of files modified during execution.
- commands_run (`Arc&lt;Mutex&lt;HashSet&lt;String&gt;&gt;&gt;`): Definition: Set of external shell commands run by the agent.
- allowed_files (`Option&lt;Vec&lt;String&gt;&gt;`): Definition: Specific file whitelist constraints (if sandbox-configured).
- recent_findings (`Option&lt;String&gt;`): Definition: Cached discoveries/findings for cross-turn retention.
- working_memory (Value): Definition: Ephemeral JSON scratchpad for the agent's internal state.
- base_dir (PathBuf): Definition: Root persistence directory of the engine server.
- summarized_history (`Option&lt;String&gt;`): Definition: Compacted representation of past turns to reduce token window pressure.
- structured_output (bool): Definition: Toggle flag requesting structured JSON format.
- backlog (`Option&lt;Arc&lt;Mutex&lt;MissionBacklog&gt;&gt;&gt;`): Definition: Queue of pending subtasks generated during execution.
- primary_goal (`Option&lt;String&gt;`): Definition: The top-level mission goal string.
- budget_usd (f64): Definition: Monetary cap allocated for the run.
- current_cost_usd (f64): Definition: Total cost incurred during the active run (cumulative).
- reasoning_depth (u32): Definition: Number of steps used for reasoning/CoT.
- act_threshold (f32): Definition: Score threshold for taking actions.
- max_turns (u32): Definition: Maximum iterations allowed for the run loop.
- authority_level (RoleAuthorityLevel): Definition: Dynamic permission tier mapped to the agent's role.
- resource_weights (`HashMap&lt;String, f32&gt;`): Definition: Resource priority distribution parameters.
- graph_context (`Option&lt;String&gt;`): Definition: Codebase symbol relationships injected for target files.
- verification_passed (bool): Definition: Flag tracking testing verification results.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- ModelConfig ---
Type: struct
Purpose: Configuration for an LLM instance including system prompts and performance thresholds.
Parameters:
- provider (ModelProvider): Definition: Target backend provider protocol.
- model_id (String): Definition: Specific model identifier.
- api_key (`Option&lt;String&gt;`): Definition: Optional access key override.
- base_url (`Option&lt;String&gt;`): Definition: Optional backend endpoint URL override.
- system_prompt (`Option&lt;String&gt;`): Definition: Custom system prompt instruction override.
- temperature (`Option&lt;f32&gt;`): Definition: LLM sampling temperature.
- max_tokens (`Option&lt;u32&gt;`): Definition: Maximum response tokens.
- external_id (`Option&lt;String&gt;`): Definition: Telemetry / tracking identifier.
- rpm (`Option&lt;u32&gt;`): Definition: Rate limit threshold (Requests Per Minute).
- rpd (`Option&lt;u32&gt;`): Definition: Rate limit threshold (Requests Per Day).
- tpm (`Option&lt;u32&gt;`): Definition: Rate limit threshold (Tokens Per Minute).
- tpd (`Option&lt;u32&gt;`): Definition: Rate limit threshold (Tokens Per Day).
- skills (`Option&lt;Vec&lt;String&gt;&gt;`): Definition: Enabled skill whitelists.
- workflows (`Option&lt;Vec&lt;String&gt;&gt;`): Definition: Enabled SOP workflow mappings.
- mcp_tools (`Option&lt;Vec&lt;String&gt;&gt;`): Definition: Enabled MCP tool selections.
- steering_vectors (`Option&lt;Vec&lt;String&gt;&gt;`): Definition: Model alignment steering configurations.
- reasoning_depth (`Option&lt;u32&gt;`): Definition: Maximum internal reasoning turns (e.g. for o1/r1 models).
- act_threshold (`Option&lt;f32&gt;`): Definition: Confidence threshold for autonomous actions.
- max_turns (`Option&lt;u32&gt;`): Definition: Maximum turn limit override.
- connector_configs (`Option&lt;Vec&lt;ConnectorConfig&gt;&gt;`): Definition: Database or knowledge connector details.
- extra_parameters (`Option&lt;HashMap&lt;String, Value&gt;&gt;`): Definition: Additional provider-specific parameters.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- ModelCapabilities ---
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

--- ProblemDetails ---
Type: struct
Purpose: RFC 9457 compliant structure for machine-readable API error responses.
Parameters:
- type_uri (String): Definition: URI identifying the specific error type (formatted as `https://tadpole.os/errors/{slug}`).
- title (String): Definition: Human-readable summary of the error.
- status (u16): Definition: HTTP status code.
- detail (String): Definition: Sanitized and redacted explanation of the failure.
- instance (`Option&lt;String&gt;`): Definition: Unique request/transaction URI path context.
- error_code (`Option&lt;String&gt;`): Definition: Uppercase alphanumeric error code representation.
- help_link (`Option&lt;String&gt;`): Definition: Publicly accessible documentation link for troubleshooting.
- severity (String): Definition: Logging severity tier (e.g., `"CRITICAL"`, `"ERROR"`, `"WARNING"`).
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
- `MultiError(Vec&lt;AppError&gt;)`: Aggregated collection of multiple concurrent errors.
- `Anyhow(anyhow::Error)`: Wildcard wrapper for custom contextual errors.
- `Sqlx(sqlx::Error)`: Database persistence failure.
- `Io(std::io::Error)`: Standard system input/output failure.
- `Reqwest(reqwest::Error)`: Downstream HTTP client failure.
- `Serde(serde_json::Error)`: Serialization/deserialization failure.
- `WalkDir(walkdir::Error)`: Filesystem directory traversal failure.
- `Graph(GraphError)`: Error occurring within the code structure or symbol knowledge graphs.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- RunnerError ---
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

--- SkillError ---
Type: enum
Purpose: Specific custom script skill and recruitment errors wrapped by `AppError`.
Variants:
- `ValidationError(String)`: Input or configuration data failed validation checks.
- `RecruitmentFailure { recipe_id, role, detail }`: Spawning of a swarm sub-agent failed.
- `SanitizationViolation(String)`: Whitelist security scanner blocked unsafe content or command execution.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- RoleAuthorityLevel ---
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

--- ModelProvider ---
Type: enum
Purpose: Supported backend protocols and hosting platforms for LLM interaction.
Variants:
- `Openai`, `Anthropic`, `Google`, `Gemini`, `Ollama`, `Groq`, `Mistral`, `Perplexity`, `Fireworks`, `Together`, `Deepseek`, `Xai`, `Inception`, `Openrouter`, `Cerebras`, `Sambanova`, `OllamaCloud`.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- SubsystemStatus ---
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

--- SystemHealthState ---
Type: enum
Purpose: Overall status reporting for the swarm engine.
Variants:
- `Warming`: Critical subsystems are starting up.
- `Ready`: All critical subsystems operational.
- `Degraded`: Subsystems failed or are in a degraded state.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- Modality ---
Type: enum
Purpose: Feature modality categories.
Variants:
- `Llm`, `Vision`, `Voice`, `Audio`, `Reasoning`.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

--- MissionStatus ---
Type: enum
Purpose: Swarm mission lifecycle status.
Variants:
- `Pending`, `SpecReview`, `Active`, `Completed`, `Failed`, `Paused`.
Return Value: N/A
Side Effects: N/A
Failure Conditions: N/A

---

## 📖 Terminology & Concepts

--- Alpha Node / Department Lead ---
Type: Terminology
Purpose: The primary coordinator agent in a workspace responsible for breaking down goals, recruiting specialist sub-agents, and compiling reports. Represents the management tier in the command chain.
Implementation Hook: `RoleAuthorityLevel::Management` / `state.registry.list_active_specialists()`

--- Mission Cluster / Project Workspace ---
Type: Terminology
Purpose: An isolated local project directory containing state files, agent logs, configurations, and workspaces grouped toward a single business goal.
Implementation Hook: `GovernanceHub::max_clusters` / `RunContext::mission_id`

--- Swarm Depth ---
Type: Terminology
Purpose: The recursion level of sub-agent recruitment within a mission. Enforced to prevent infinite delegation loops.
Implementation Hook: `MAX_SWARM_DEPTH` / `GovernanceHub::max_swarm_depth`

--- Neural Vault / Secure Credentials Vault ---
Type: Terminology
Purpose: Client-side credentials storage encrypted using `PBKDF2 + AES-256-GCM` via standard `SubtleCrypto` APIs. Stores LLM provider credentials securely in the browser.
Implementation Hook: `use_vault_store` / `crypto.worker.ts`

--- Sovereign Reality / Private Swarm Environment ---
Type: Terminology
Purpose: The secure, local-first execution environment where virtual agents run tools, process documents, and interact with LLMs without external data leaks.
Implementation Hook: `AppState` / `AgentRunner`

--- RAG Scope / Agent Scope ---
Type: Terminology
Purpose: The data ingestion context constraint limiting an agent's retrieval to specific files, workspace directories, or long-term memory.
Implementation Hook: `ResourceHub::symbol_graph` / `ResourceHub::code_graph`

--- Swarm Pulse ---
Type: Terminology
Purpose: A real-time binary telemetry event stream showing system communications, agent thinking state, and tool calls.
Implementation Hook: `CommunicationHub::pulse_tx` / `SwarmPulse`

---

## 🏗️ Compliance & Quality Standards

--- ALCOA+ ---
Type: Standard
Purpose: International data integrity guidelines (Attributable, Legible, Contemporaneous, Original, Accurate, Complete, Consistent, Enduring, Available) enforced across the engine telemetry streams.
Implementation Hook: `server-rs/src/security/audit.rs`

--- GxP ---
Type: Standard
Purpose: Good Practice quality guidelines and regulations (e.g., GCP, GMP, GLP) to ensure the system is safe for regulated domains.
Implementation Hook: `execution/verify_all.py`

--- Software Validation Package (SVP) ---
Type: Compliance Artifact
Purpose: Formally compiled validation documentation package (including FMEA risk assessment, OQ/PQ test scripts) for regulatory sign-off.
Implementation Hook: `docs/TEST_MISSIONS.md`

---

[//]: # (Metadata: [GLOSSARY])
