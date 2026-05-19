const fs = require('fs');
const path = require('path');

const srcDir = path.join(__dirname, 'src');
const testsDir = path.join(__dirname, 'tests');

const targets = [
    'agent_api_service',
    'base_api_service',
    'mission_api_service',
    'system_api_service',
    'proposal_service',
    'tadpoleos_service',
    'agent_service',
    'crypto_service',
    'event_bus',
    'local_swarm',
    'socket',
    'voice_client',
    'agent_store',
    'dropdown_store',
    'header_store',
    'memory_store',
    'model_store',
    'node_store',
    'notification_store',
    'provider_store',
    'role_store',
    'settings_store',
    'skill_store',
    'sovereign_store',
    'tab_store',
    'trace_store',
    'vault_store',
    'workspace_store'
];

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

  targets.forEach(t => {
      // Create a case-insensitive regex for the target in an import context
      // e.g. import ... from './Agent_Api_Service' or './agentApiService'
      // regex matches the target string between / and ' or "
      const regex = new RegExp(`(/)${t.replace(/_/g, '[_]?')}(['"])`, 'gi');
      
      // We also need to catch the ones that don't have a leading / if they are the first part of the path
      // but usually imports are ./ or ../ or src/
      
      content = content.replace(regex, (match, p1, p2) => {
          return p1 + t + p2;
      });
      
      // Also catch ones that might be Mission_Api_Service without leading slash in the regex above
      // This matches 'Mission_Api_Service' or "Mission_Api_Service" exactly if it's the whole path
      const exactRegex = new RegExp(`(['"])${t.replace(/_/g, '[_]?')}(['"])`, 'gi');
      content = content.replace(exactRegex, (match, p1, p2) => {
          return p1 + t + p2;
      });
  });

  // Specific fix for tests calling CamelCase exports
  content = content.replace(/\bAgentApiService\b/g, 'agent_api_service');
  content = content.replace(/\bMissionApiService\b/g, 'mission_api_service');
  content = content.replace(/\bSystemApiService\b/g, 'system_api_service');
  content = content.replace(/\bBaseApiService\b/g, 'base_api_service');
  content = content.replace(/\bProposalService\b/g, 'proposal_service');

  if (content !== originalContent) {
    fs.writeFileSync(filePath, content, 'utf8');
  }
});

console.log('Nuclear casing fix complete.');
