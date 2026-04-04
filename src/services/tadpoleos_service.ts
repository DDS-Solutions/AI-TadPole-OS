/**
 * Tadpole_OS_Service
 * Unified HTTP client for the TadpoleOS Rust backend.
 * This is a proxy service that delegates to domain-specific services
 * to maintain backward compatibility while improving maintainability.
 * Refactored for strict snake_case compliance for backend parity.
 */
import { agent_api_service } from './agent_api_service';
import { mission_api_service } from './mission_api_service';
import { system_api_service } from './system_api_service';
import { api_request, get_headers } from './base_api_service';

import { resolve_provider } from '../utils/model_utils';

// Removed duplicate definitions - now correctly re-exported from system_api_service

export const tadpole_os_service = {
    // Shared / Base
    get_headers,
    request: api_request,

    // Agent Domain
    get get_agents() { return agent_api_service.get_agents; },
    get update_agent() { return agent_api_service.update_agent; },
    get create_agent() { return agent_api_service.create_agent; },
    get pause_agent() { return agent_api_service.pause_agent; },
    get resume_agent() { return agent_api_service.resume_agent; },
    resolve_provider: resolve_provider,
    get send_command() { return agent_api_service.send_command; },
    get get_agent_memory() { return agent_api_service.get_agent_memory; },
    get search_memory() { return agent_api_service.search_memory; },
    get delete_agent_memory() { return agent_api_service.delete_agent_memory; },
    get save_agent_memory() { return agent_api_service.save_agent_memory; },

    // Mission / Skill Domain
    get sync_mission() { return mission_api_service.sync_mission; },
    get get_skill_manifests() { return mission_api_service.get_skill_manifests; },
    get get_unified_skills() { return mission_api_service.get_unified_skills; },
    get save_skill_script() { return mission_api_service.save_skill_script; },
    get delete_skill_script() { return mission_api_service.delete_skill_script; },
    get save_workflow() { return mission_api_service.save_workflow; },
    get delete_workflow() { return mission_api_service.delete_workflow; },
    get save_hook() { return mission_api_service.save_hook; },
    get delete_hook() { return mission_api_service.delete_hook; },
    get get_mcp_tools() { return mission_api_service.get_mcp_tools; },
    get execute_mcp_tool() { return mission_api_service.execute_mcp_tool; },

    // System / Engine Domain
    get check_health() { return system_api_service.check_health; },
    get deploy_engine() { return system_api_service.deploy_engine; },
    get speak() { return system_api_service.speak; },
    get kill_agents() { return system_api_service.kill_agents; },
    get shutdown_engine() { return system_api_service.shutdown_engine; },
    get transcribe() { return system_api_service.transcribe; },
    get test_provider() { return system_api_service.test_provider; },
    get get_nodes() { return system_api_service.get_nodes; },
    get discover_nodes() { return system_api_service.discover_nodes; },
    get get_benchmarks() { return system_api_service.get_benchmarks; },
    get run_benchmark() { return system_api_service.run_benchmark; },
    get get_scheduled_jobs() { return system_api_service.get_scheduled_jobs; },
    get create_scheduled_job() { return system_api_service.create_scheduled_job; },
    get update_scheduled_job() { return system_api_service.update_scheduled_job; },
    get delete_scheduled_job() { return system_api_service.delete_scheduled_job; },
    get get_scheduled_job_runs() { return system_api_service.get_scheduled_job_runs; },
    get get_pending_oversight() { return system_api_service.get_pending_oversight; },
    get get_oversight_ledger() { return system_api_service.get_oversight_ledger; },
    get decide_oversight() { return system_api_service.decide_oversight; },
    get get_knowledge_docs() { return system_api_service.get_knowledge_docs; },
    get get_knowledge_doc() { return system_api_service.get_knowledge_doc; },
    get get_operations_manual() { return system_api_service.get_operations_manual; },
    get get_providers() { return system_api_service.get_providers; },
    get update_provider() { return system_api_service.update_provider; },
    get delete_provider() { return system_api_service.delete_provider; },
    get get_models() { return system_api_service.get_models; },
    get update_model() { return system_api_service.update_model; },
    get delete_model() { return system_api_service.delete_model; },
    get get_security_quotas() { return system_api_service.get_security_quotas; },
    get update_security_quota() { return system_api_service.update_security_quota; },
    get get_mission_quotas() { return system_api_service.get_mission_quotas; },
    get update_mission_quota() { return system_api_service.update_mission_quota; },
    get get_audit_trail() { return system_api_service.get_audit_trail; },
    get get_agent_health() { return system_api_service.get_agent_health; },
    get get_integrity_status() { return system_api_service.get_integrity_status; },
    get list_continuity_workflows() { return system_api_service.list_continuity_workflows; },
    get create_continuity_workflows() { return system_api_service.create_continuity_workflows; },
    get add_continuity_workflows_step() { return system_api_service.add_continuity_workflows_step; },
    get delete_continuity_workflows() { return system_api_service.delete_continuity_workflows; },
    get get_model_catalog() { return system_api_service.get_model_catalog; },
    get pull_model() { return system_api_service.pull_model; }
};

// Re-export types for consumers
export type { 
    Provider_Test_Config, 
    Infra_Node, 
    Benchmark_Record, 
    Scheduled_Job, 
    Scheduled_Job_Run, 
    Workflow_Entry, 
    Audit_Entry, 
    Agent_Health, 
    Quotas, 
    Swarm_Node, 
    Store_Model 
} from './system_api_service';
export type Connection_State = 'connecting' | 'connected' | 'disconnected' | 'reconnecting' | 'error';
export type { Skill_Manifest } from './mission_api_service';
