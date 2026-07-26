/**
 * @docs ARCHITECTURE:Services
 * @docs API_REFERENCE:Endpoints
 * 
 * ### AI Assist Note
 * **Root Proxy Service**: Unified entry point for all domain-specific API services. 
 * Orchestrates health checks, deployment, and cross-subsystem orchestration.
 * 
 * ### @aiContext
 * - **Dependencies**: `agent_api_service`, `mission_api_service`, `system_api_service` (Delegates).
 * - **Side Effects**: Aggregated side effects of all domain services (REST/Health/Deployment).
 * - **Mocking**: Mocking this single object allows for complete backend isolation in E2E/UI-only vitest suites.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Service resolution error (incorrectly mapped delegate), backend 404/500 propagation, or base URL mismatch in settings_store.
 * - **Telemetry Link**: Search for `[TadpoleOSService]` in browser logs or look for v1/ routes in backend audit trails.
 */


/**
 * Tadpole_OS_Service
 * Unified HTTP client for the TadpoleOS Rust backend.
 * This is a proxy service that delegates to domain-specific services
 * to maintain backward compatibility while improving maintainability.
 * Refactored for strict snake_case compliance for backend parity.
 */
import { agent_api_service } from './agent';
import { mission_api_service } from './mission_api_service';
import { system_api_service } from './system_api_service';
import { intelligence_api_service } from './intelligence_api_service';
import { api_request, get_headers } from './base_api_service';

import { resolve_provider } from '../utils/model_utils';

// Export domain services directly for consumer modules
export { agent_api_service, mission_api_service, system_api_service, intelligence_api_service };

/**
 * @deprecated Use domain services directly (agent_api_service, system_api_service, etc.)
 */
export const tadpole_os_service = {
    // Shared / Base
    get_headers,
    request: api_request,
    resolve_provider,

    // Domain Delegate References
    agent: agent_api_service,
    mission: mission_api_service,
    system: system_api_service,
    intelligence: intelligence_api_service,

    // Legacy method bindings
    get_agents: agent_api_service.get_agents,
    update_agent: agent_api_service.update_agent,
    create_agent: agent_api_service.create_agent,
    pause_agent: agent_api_service.pause_agent,
    resume_agent: agent_api_service.resume_agent,
    send_command: agent_api_service.send_command,
    get_agent_memory: agent_api_service.get_agent_memory,
    search_memory: agent_api_service.search_memory,
    delete_agent_memory: agent_api_service.delete_agent_memory,
    save_agent_memory: agent_api_service.save_agent_memory,
    save_role_blueprint: agent_api_service.save_role_blueprint,
    delete_role_blueprint: agent_api_service.delete_role_blueprint,

    get_unified_skills: mission_api_service.get_unified_skills,
    save_skill_script: mission_api_service.save_skill_script,
    delete_skill_script: mission_api_service.delete_skill_script,
    save_workflow: mission_api_service.save_workflow,
    delete_workflow: mission_api_service.delete_workflow,
    save_hook: mission_api_service.save_hook,
    delete_hook: mission_api_service.delete_hook,
    get_mcp_tools: mission_api_service.get_mcp_tools,
    execute_mcp_tool: mission_api_service.execute_mcp_tool,

    check_health: system_api_service.engine.check_health,
    deploy_engine: system_api_service.engine.deploy_engine,
    speak: system_api_service.engine.speak,
    kill_agents: system_api_service.engine.kill_agents,
    shutdown_engine: system_api_service.engine.shutdown_engine,
    transcribe: system_api_service.engine.transcribe,
    test_provider: system_api_service.infra.test_provider,
    get_nodes: system_api_service.infra.get_nodes,
    discover_nodes: system_api_service.infra.discover_nodes,
    get_benchmarks: system_api_service.benchmarks.get_benchmarks,
    run_benchmark: system_api_service.benchmarks.run_benchmark,
    get_scheduled_jobs: system_api_service.continuity.get_scheduled_jobs,
    create_scheduled_job: system_api_service.continuity.create_scheduled_job,
    update_scheduled_job: system_api_service.continuity.update_scheduled_job,
    delete_scheduled_job: system_api_service.continuity.delete_scheduled_job,
    get_scheduled_job_runs: system_api_service.continuity.get_scheduled_job_runs,
    get_pending_oversight: system_api_service.oversight.get_pending_oversight,
    get_oversight_ledger: system_api_service.oversight.get_oversight_ledger,
    decide_oversight: system_api_service.oversight.decide_oversight,
    get_knowledge_docs: system_api_service.docs.get_knowledge_docs,
    get_knowledge_doc: system_api_service.docs.get_knowledge_doc,
    get_operations_manual: system_api_service.docs.get_operations_manual,
    get_providers: system_api_service.infra.get_providers,
    update_provider: system_api_service.infra.update_provider,
    delete_provider: system_api_service.infra.delete_provider,
    get_models: system_api_service.infra.get_models,
    update_model: system_api_service.infra.update_model,
    delete_model: system_api_service.infra.delete_model,
    update_security_quota: system_api_service.oversight.update_security_quota,
    get_mission_quotas: system_api_service.oversight.get_mission_quotas,
    update_mission_quota: system_api_service.oversight.update_mission_quota,
    get_security_snapshot: system_api_service.oversight.get_security_snapshot,
    list_continuity_workflows: system_api_service.continuity.list_continuity_workflows,
    trigger_scheduled_job: system_api_service.continuity.trigger_scheduled_job,
    get_workflow_run_steps: system_api_service.continuity.get_workflow_run_steps,
    get_model_catalog: system_api_service.infra.get_model_catalog,
    pull_model: system_api_service.infra.pull_model,
    sync_provider_models: system_api_service.infra.sync_provider_models,

    get_code_graph: intelligence_api_service.get_code_graph,
    get_blast_radius: intelligence_api_service.get_blast_radius
};

// Re-export types for consumers
export type { 
    Provider_Test_Config, 
    Benchmark_Record, 
    Scheduled_Job, 
    Scheduled_Job_Run, 
    Workflow_Entry, 
    Audit_Entry, 
    Agent_Health, 
    Quotas, 
    Quota_Details,
    Swarm_Node, 
    Store_Model 
} from './system_api_service';
export type { Connection_State } from './socket';
export type { Skill_Manifest } from './mission_api_service';


// Metadata: [tadpoleos_service]
