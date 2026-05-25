const fs = require('fs');
const path = require('path');

const srcDir = path.join(__dirname, 'src');
const testsDir = path.join(__dirname, 'tests');

const replacements = [
    { from: /\bopenTab\b/g, to: 'open_tab' },
    { from: /\bactiveTabId\b/g, to: 'active_tab_id' },
    { from: /\bsetActiveTab\b/g, to: 'set_active_tab' },
    { from: /\bcloseTab\b/g, to: 'close_tab' },
    { from: /\btoggleTabDetachment\b/g, to: 'toggle_tab_detachment' },
    { from: /\bisActive\b/g, to: 'is_active' },
    { from: /\balphaId\b/g, to: 'alpha_id' },
    { from: /\bonClose\b/g, to: 'on_close' },
    { from: /\bisDetached\b/g, to: 'is_detached' },
    { from: /\bisChecked\b/g, to: 'is_checked' },
    { from: /\bpendingTasks\b/g, to: 'pending_tasks' },
    { from: /\btokensUsed\b/g, to: 'tokens_used' },
    { from: /\bcurrentTask\b/g, to: 'current_task' },
    { from: /\bmodelConfig\b/g, to: 'model_config' },
    { from: /\bmodelConfig2\b/g, to: 'model_config2' },
    { from: /\bmodelConfig3\b/g, to: 'model_config3' },
    { from: /\bactiveModelSlot\b/g, to: 'active_model_slot' },
    { from: /\bcreatedAt\b/g, to: 'created_at' },
    { from: /\blastPulse\b/g, to: 'last_pulse' },
    { from: /\binputTokens\b/g, to: 'input_tokens' },
    { from: /\boutputTokens\b/g, to: 'output_tokens' },
    { from: /\bfailureCount\b/g, to: 'failure_count' },
    { from: /\bmaxClusters\b/g, to: 'max_clusters' },
    { from: /\bprivacyMode\b/g, to: 'privacy_mode' },
    { from: /\bdefaultModel\b/g, to: 'default_model' },
    { from: /\bdefaultTemperature\b/g, to: 'default_temperature' },
    { from: /\bvoiceEngine\b/g, to: 'voice_engine' },
    { from: /\bmcpTools\b/g, to: 'mcp_tools' },
    { from: /\baddProvider\b/g, to: 'add_provider' },
    { from: /\baddModel\b/g, to: 'add_model' },
    { from: /\beditModel\b/g, to: 'edit_model' },
    { from: /\beditProvider\b/g, to: 'edit_provider' },
    { from: /\bsetProviderConfig\b/g, to: 'set_provider_config' },
    { from: /\bfetchAgents\b/g, to: 'fetch_agents' },
    { from: /\bprocessCommand\b/g, to: 'process_command' },
    { from: /\bmissionObjective\b/g, to: 'mission_objective' },
    { from: /\bonModelChange\b/g, to: 'on_model_change' },
    { from: /\bupdateSetting\b/g, to: 'update_setting' },
    { from: /\bgetModelColor\b/g, to: 'get_model_color' },
    { from: /\bresolveProvider\b/g, to: 'resolve_provider' },
    { from: /\buseSwarmMetrics\b/g, to: 'use_swarm_metrics' },
    { from: /\bTraceSpan\b/g, to: 'Trace_Span' },
    { from: /\bclearAll\b/g, to: 'clear_all' },
    { from: /\bparentId\b/g, to: 'parent_id' },
    { from: /\bhandlePulse\b/g, to: 'handle_pulse' },
    { from: /\bnavItemClass\b/g, to: 'nav_item_class' },
    { from: /\bsetActiveTrace\b/g, to: 'set_active_trace' },
    { from: /\bsetActions\b/g, to: 'set_actions' },
    { from: /\bclearActions\b/g, to: 'clear_actions' },
    { from: /\bimportCapability\b/g, to: 'import_capability' },
    { from: /\bregisterCapability\b/g, to: 'register_capability' },
    { from: /\bvoiceClient\b/g, to: 'voice_client' },
    { from: /\bresolveAgentModelConfig\b/g, to: 'resolve_agent_model_config' },
    { from: /\baddSpan\b/g, to: 'add_span' },
    { from: /\bupdateSpan\b/g, to: 'update_span' },
    { from: /\bfetch_agents\b/g, to: 'fetch_agents' }, // Ensure consistency
    { from: /\bisOpen\b/g, to: 'is_open' }
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

  replacements.forEach(rep => {
    content = content.replace(rep.from, rep.to);
  });

  if (content !== originalContent) {
    fs.writeFileSync(filePath, content, 'utf8');
    console.log(`Updated: ${filePath}`);
  }
});

console.log('Casing fix complete.');
