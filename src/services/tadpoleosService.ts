/**
 * @module TadpoleOSService
 * Unified HTTP client for the TadpoleOS Rust backend.
 * This is a proxy service that delegates to domain-specific services
 * to maintain backward compatibility while improving maintainability.
 */
import { AgentApiService } from './AgentApiService';
import { MissionApiService } from './MissionApiService';
import { SystemApiService } from './SystemApiService';
import { apiRequest, getHeaders } from './BaseApiService';

import { resolveProvider } from '../utils/modelUtils';

export const TadpoleOSService = {
    // Shared / Base
    getHeaders,
    request: apiRequest,

    // Agent Domain
    get getAgents() { return AgentApiService.getAgents; },
    get updateAgent() { return AgentApiService.updateAgent; },
    get createAgent() { return AgentApiService.createAgent; },
    get pauseAgent() { return AgentApiService.pauseAgent; },
    get resumeAgent() { return AgentApiService.resumeAgent; },
    resolveProvider: resolveProvider,
    get sendCommand() { return AgentApiService.sendCommand; },
    get getAgentMemory() { return AgentApiService.getAgentMemory; },
    get deleteAgentMemory() { return AgentApiService.deleteAgentMemory; },
    get saveAgentMemory() { return AgentApiService.saveAgentMemory; },

    // Mission / Skill Domain
    get syncMission() { return MissionApiService.syncMission; },
    get getSkillManifests() { return MissionApiService.getSkillManifests; },
    get getUnifiedSkills() { return MissionApiService.getUnifiedSkills; },
    get saveSkillScript() { return MissionApiService.saveSkillScript; },
    get deleteSkillScript() { return MissionApiService.deleteSkillScript; },
    get saveWorkflow() { return MissionApiService.saveWorkflow; },
    get deleteWorkflow() { return MissionApiService.deleteWorkflow; },
    get saveHook() { return MissionApiService.saveHook; },
    get deleteHook() { return MissionApiService.deleteHook; },
    get getMcpTools() { return MissionApiService.getMcpTools; },
    get executeMcpTool() { return MissionApiService.executeMcpTool; },

    // System / Engine Domain
    get checkHealth() { return SystemApiService.checkHealth; },
    get deployEngine() { return SystemApiService.deployEngine; },
    get speak() { return SystemApiService.speak; },
    get killAgents() { return SystemApiService.killAgents; },
    get shutdownEngine() { return SystemApiService.shutdownEngine; },
    get transcribe() { return SystemApiService.transcribe; },
    get testProvider() { return SystemApiService.testProvider; },
    get getNodes() { return SystemApiService.getNodes; },
    get discoverNodes() { return SystemApiService.discoverNodes; },
    get getBenchmarks() { return SystemApiService.getBenchmarks; },
    get runBenchmark() { return SystemApiService.runBenchmark; },
    get getScheduledJobs() { return SystemApiService.getScheduledJobs; },
    get createScheduledJob() { return SystemApiService.createScheduledJob; },
    get updateScheduledJob() { return SystemApiService.updateScheduledJob; },
    get deleteScheduledJob() { return SystemApiService.deleteScheduledJob; },
    get getScheduledJobRuns() { return SystemApiService.getScheduledJobRuns; },
    get getPendingOversight() { return SystemApiService.getPendingOversight; },
    get getOversightLedger() { return SystemApiService.getOversightLedger; },
    get decideOversight() { return SystemApiService.decideOversight; },
    get getKnowledgeDocs() { return SystemApiService.getKnowledgeDocs; },
    get getKnowledgeDoc() { return SystemApiService.getKnowledgeDoc; },
    get getOperationsManual() { return SystemApiService.getOperationsManual; },
    get getProviders() { return SystemApiService.getProviders; },
    get updateProvider() { return SystemApiService.updateProvider; },
    get deleteProvider() { return SystemApiService.deleteProvider; },
    get getModels() { return SystemApiService.getModels; },
    get updateModel() { return SystemApiService.updateModel; },
    get deleteModel() { return SystemApiService.deleteModel; },
    get getSecurityQuotas() { return SystemApiService.getSecurityQuotas; },
    get updateSecurityQuota() { return SystemApiService.updateSecurityQuota; },
    get getMissionQuotas() { return SystemApiService.getMissionQuotas; },
    get updateMissionQuota() { return SystemApiService.updateMissionQuota; },
    get getAuditTrail() { return SystemApiService.getAuditTrail; },
    get getAgentHealth() { return SystemApiService.getAgentHealth; },
    get getIntegrityStatus() { return SystemApiService.getIntegrityStatus; },
    get listContinuityWorkflows() { return SystemApiService.listContinuityWorkflows; },
    get createContinuityWorkflow() { return SystemApiService.createContinuityWorkflow; },
    get addContinuityWorkflowStep() { return SystemApiService.addContinuityWorkflowStep; },
    get deleteContinuityWorkflow() { return SystemApiService.deleteContinuityWorkflow; },
    get getModelCatalog() { return SystemApiService.getModelCatalog; },
    get pullModel() { return SystemApiService.pullModel; }
};

// Re-export types for consumers
export type { ProviderTestConfig, InfraNode, BenchmarkRecord, ScheduledJob, ScheduledJobRun, WorkflowEntry, AuditEntry, AgentHealth, Quotas, SwarmNode, StoreModel } from './SystemApiService';
export type { SkillManifest } from './MissionApiService';
