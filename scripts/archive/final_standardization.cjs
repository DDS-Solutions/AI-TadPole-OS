const fs = require('fs');
const path = require('path');

const srcDir = path.join(__dirname, 'src');
const testsDir = path.join(__dirname, 'tests');

// 1. Definition of all project-defined symbols to refactor
const serviceMap = {
    'Agent_Api_Service': 'agent_api_service',
    'Mission_Api_Service': 'mission_api_service',
    'System_Api_Service': 'system_api_service',
    'TadpoleOSService': 'tadpole_os_service',
    'Base_Api_Service': 'base_api_service',
    'Proposal_Service': 'proposal_service',
    'Local_Swarm': 'local_swarm',
    'TadpoleOSSocket': 'tadpole_os_socket',
    'Voice_Client': 'voice_client'
};

const methodMap = {
    'getAgents': 'get_agents',
    'updateAgent': 'update_agent',
    'createAgent': 'create_agent',
    'pauseAgent': 'pause_agent',
    'resumeAgent': 'resume_agent',
    'sendCommand': 'send_command',
    'getAgentMemory': 'get_agent_memory',
    'searchMemory': 'search_memory',
    'deleteAgentMemory': 'delete_agent_memory',
    'saveAgentMemory': 'save_agent_memory',
    'syncMission': 'sync_mission',
    'getSkillManifests': 'get_skill_manifests',
    'getUnifiedSkills': 'get_unified_skills',
    'saveSkillScript': 'save_skill_script',
    'deleteSkillScript': 'delete_skill_script',
    'saveWorkflow': 'save_workflow',
    'deleteWorkflow': 'delete_workflow',
    'saveHook': 'save_hook',
    'deleteHook': 'delete_hook',
    'getMcpTools': 'get_mcp_tools',
    'executeMcpTool': 'execute_mcp_tool',
    'checkHealth': 'check_health',
    'deployEngine': 'deploy_engine',
    'killAgents': 'kill_agents',
    'shutdownEngine': 'shutdown_engine',
    'testProvider': 'test_provider',
    'getNodes': 'get_nodes',
    'discoverNodes': 'discover_nodes',
    'getBenchmarks': 'get_benchmarks',
    'runBenchmark': 'run_benchmark',
    'getScheduledJobs': 'get_scheduled_jobs',
    'createScheduledJob': 'create_scheduled_job',
    'updateScheduledJob': 'update_scheduled_job',
    'deleteScheduledJob': 'delete_scheduled_job',
    'getScheduledJobRuns': 'get_scheduled_job_runs',
    'getPendingOversight': 'get_pending_oversight',
    'getOversightLedger': 'get_oversight_ledger',
    'decideOversight': 'decide_oversight',
    'getKnowledgeDocs': 'get_knowledge_docs',
    'getKnowledgeDoc': 'get_knowledge_doc',
    'getOperationsManual': 'get_operations_manual',
    'getProviders': 'get_providers',
    'updateProvider': 'update_provider',
    'deleteProvider': 'delete_provider',
    'updateModel': 'update_model',
    'deleteModel': 'delete_model',
    'getModels': 'get_models',
    'getSecurityQuotas': 'get_security_quotas',
    'updateSecurityQuota': 'update_security_quota',
    'getMissionQuotas': 'get_mission_quotas',
    'updateMissionQuota': 'update_mission_quota',
    'getAuditTrail': 'get_audit_trail',
    'getAgentHealth': 'get_agent_health',
    'listContinuityWorkflows': 'list_continuity_workflows',
    'createContinuityWorkflow': 'create_continuity_workflows',
    'addContinuityWorkflowStep': 'add_continuity_workflows_step',
    'deleteContinuityWorkflow': 'delete_continuity_workflows',
    'getIntegrityStatus': 'get_integrity_status',
    'updateGovernanceSettings': 'update_governance_settings',
    'getModelCatalog': 'get_model_catalog',
    'pullModel': 'pull_model',
    'installTemplate': 'install_template'
};

function getAllFiles(dir, fileList = []) {
  if (!fs.existsSync(dir)) return fileList;
  const files = fs.readdirSync(dir);
  files.forEach(file => {
    const filePath = path.join(dir, file);
    if (fs.statSync(filePath).isDirectory()) {
      getAllFiles(filePath, fileList);
    } else {
      fileList.push(filePath);
    }
  });
  return fileList;
}

const allFiles = [...getAllFiles(srcDir), ...getAllFiles(testsDir)];

allFiles.forEach(filePath => {
    const ext = path.extname(filePath);
    if (ext !== '.ts' && ext !== '.tsx') return;

    let content = fs.readFileSync(filePath, 'utf8');
    const originalContent = content;

    // 1. Refactor Service singletons
    Object.keys(serviceMap).forEach(key => {
        const regex = new RegExp(`\\b${key}\\b`, 'g');
        content = content.replace(regex, serviceMap[key]);
    });

    // 2. Refactor method names
    Object.keys(methodMap).forEach(key => {
        const regex = new RegExp(`\\b${key}\\b`, 'g');
        content = content.replace(regex, methodMap[key]);
    });

    // 3. Fix standard casing issues specifically for paths
    content = content.replace(/from '\.\/Use_Agent_Config'/g, "from './use_agent_config'");
    content = content.replace(/from '\.\.?\/(P|p)ages\/model_store'/g, "from '../pages/Model_Store'");
    content = content.replace(/import \{ Model_Store \} from '\.\.\/pages\/Model_Store'/g, "import Model_Store from '../pages/Model_Store'");

    // 4. Final casing restoration for critical React/Web/Standard symbols incorrectly refactored in previous runs
    content = content.replace(/\bArray\.is_array\b/g, 'Array.isArray');
    content = content.replace(/\.to_iso_string\(\)/g, '.toISOString()');
    content = content.replace(/\.to_locale_string\(\)/g, '.toLocaleString()');
    content = content.replace(/\.to_fixed\(/g, '.toFixed(');
    content = content.replace(/\bStop_Propagation\(\)/g, 'stopPropagation()');
    content = content.replace(/\bRandom_UUID\(\)/g, 'randomUUID()');

    if (content !== originalContent) {
        fs.writeFileSync(filePath, content, 'utf8');
    }
});

console.log('Final standardization polish complete.');
