const fs = require('fs');
const path = require('path');

const srcDir = path.join(__dirname, 'src');

function getAllFiles(dir, fileList = []) {
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

function camelToSnakeCase(str) {
  return str.replace(/[a-z0-9][A-Z]/g, letter => letter[0] + '_' + letter[1].toLowerCase());
}

function pascalToUpperSnakeCase(str) {
  return str.replace(/[a-z0-9][A-Z]/g, letter => letter[0] + '_' + letter[1]);
}

// 1. Build the symbol map BEFORE renaming files
const allFiles = getAllFiles(srcDir);
const symbolMap = new Map();

allFiles.forEach(oldPath => {
  const ext = path.extname(oldPath);
  if (ext !== '.ts' && ext !== '.tsx') return;

  const baseName = path.basename(oldPath, ext);
  let newName = baseName;
  
  if (/^[A-Z]/.test(baseName)) {
    if (baseName !== 'App') {
       newName = pascalToUpperSnakeCase(baseName);
    }
  } else {
    newName = camelToSnakeCase(baseName).toLowerCase();
  }

  if (newName !== baseName) {
    symbolMap.set(baseName, newName);
  }
});

// Add extra required project symbols
const extraSymbols = {
    'missionId': 'mission_id',
    'agentId': 'agent_id',
    'agentName': 'agent_name',
    'themeColor': 'theme_color',
    'isNew': 'is_new',
    'costUsd': 'cost_usd',
    'budgetUsd': 'budget_usd',
    'requiresOversight': 'requires_oversight',
    'useAgentStore': 'use_agent_store',
    'useDropdownStore': 'use_dropdown_store',
    'useHeaderStore': 'use_header_store',
    'useMemoryStore': 'use_memory_store',
    'useModelStore': 'use_model_store',
    'useNodeStore': 'use_node_store',
    'useNotificationStore': 'use_notification_store',
    'useProviderStore': 'use_provider_store',
    'useRoleStore': 'use_role_store',
    'useSettingsStore': 'use_settings_store',
    'useSkillStore': 'use_skill_store',
    'useSovereignStore': 'use_sovereign_store',
    'useTabStore': 'use_tab_store',
    'useTraceStore': 'use_trace_store',
    'useVaultStore': 'use_vault_store',
    'useWorkspaceStore': 'use_workspace_store',
    'LogEntry': 'log_entry',
    'LogSource': 'log_source',
    'LogSeverity': 'log_severity',
    'EventBus': 'event_bus',
    'EventBusService': 'event_bus_service',
    'subscribe': 'subscribe_logs',
    'emit': 'emit_log',
    'getHistory': 'get_history',
    'clearHistory': 'clear_history'
};

Object.keys(extraSymbols).forEach(key => {
    symbolMap.set(key, extraSymbols[key]);
});

console.log(`Mapped ${symbolMap.size} symbols. Starting content refactor...`);

// 2. Update all file contents using the map
allFiles.forEach(filePath => {
  let content = fs.readFileSync(filePath, 'utf8');
  const originalContent = content;

  symbolMap.forEach((snake, camel) => {
    // a) Update Imports/Exports paths surgically
    // Handles from './MyComponent' -> from './My_Component'
    const pathRegex = new RegExp(`(['"])\\.\\.?/[^'"]*?\\/${camel}(['"])`, 'g');
    content = content.replace(pathRegex, (match) => match.replace(camel, snake));

    // b) Update Identifiers surgically (whole word only)
    // Avoid replacing standard library names if possible (we only use safe mapping here)
    const idRegex = new RegExp(`\\b${camel}\\b`, 'g');
    
    // For general symbols from files, we only replace them if they are PascalCase (likely components)
    // or if they are in our explicitly safe list.
    if (/^[A-Z]/.test(camel) || extraSymbols[camel]) {
        // Special case: don't replace 'subscribe' or 'emit' if they are NOT following 'event_bus.' 
        // to avoid breaking standard library or other object methods.
        if (camel === 'subscribe' || camel === 'emit' || camel === 'getHistory' || camel === 'clearHistory') {
            // Only replace if preceded by event_bus or EventBus (pre-transform)
            const busIdRegex = new RegExp(`(event_bus|EventBus)\\.${camel}\\b`, 'g');
            content = content.replace(busIdRegex, (match, bus) => {
                const newBus = bus === 'EventBus' ? 'event_bus' : bus;
                return `${newBus}.${snake}`;
            });
        } else {
            content = content.replace(idRegex, snake);
        }
    }
  });

  if (content !== originalContent) {
    fs.writeFileSync(filePath, content, 'utf8');
  }
});

console.log('Content refactor complete. Starting file renaming...');

// 3. Rename files on disk
// We must do this LAST so as not to break the path logic of the content refactor
allFiles.forEach(oldPath => {
  const ext = path.extname(oldPath);
  if (ext !== '.ts' && ext !== '.tsx') return;

  const dir = path.dirname(oldPath);
  const baseName = path.basename(oldPath, ext);
  
  const newName = symbolMap.get(baseName);
  if (newName && newName !== baseName) {
    const newPath = path.join(dir, newName + ext);
    if (!fs.existsSync(newPath)) {
      fs.renameSync(oldPath, newPath);
    }
  }
});

console.log('Refactor complete.');
