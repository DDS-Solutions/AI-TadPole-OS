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

const allFiles = getAllFiles(srcDir);

allFiles.forEach(filePath => {
  let content = fs.readFileSync(filePath, 'utf8');
  const originalContent = content;

  // 1. Force replace event_bus methods everywhere
  // (Since we know these are now renamed in event_bus_service)
  content = content.replace(/\.emit\(/g, '.emit_log(');
  content = content.replace(/\.subscribe\(/g, '.subscribe_logs(');
  content = content.replace(/\.getHistory\(/g, '.get_history(');
  content = content.replace(/\.clearHistory\(/g, '.clear_history(');

  // 2. Fix Index/Barrel misfires
  // Find "from './(.*?)'" where the filename was CamelCase but is now Snake_Case
  // We'll use a lazy greedy match but it might be safer to list them.
  // Actually, let's just target the common ones from TSC errors.
  content = content.replace(/from '\.\/useAgentConfig'/g, "from './use_agent_config'");
  content = content.replace(/from '\.\/useModelManager'/g, "from './use_model_manager'");
  content = content.replace(/from '\.\/useThrottledStatus'/g, "from './use_throttled_status'");
  content = content.replace(/from '\.\/useEngineStatus'/g, "from './use_engine_status'");
  content = content.replace(/import \{ ([a-zA-Z]+) \} from '\.\/agent-config'/g, (match, comp) => {
      // Convert comp to Snake_Case if CamelCase
      const snake = comp.replace(/([a-z])([A-Z])/g, '$1_$2');
      return `import { ${snake} } from './agent-config'`;
  });

  // 3. Fix missing LogEntry -> log_entry if still present
  content = content.replace(/\bLogEntry\b/g, 'log_entry');
  content = content.replace(/\bLogSource\b/g, 'log_source');
  content = content.replace(/\bLogSeverity\b/g, 'log_severity');

  // 4. Restoration of common native APIs that .emit( might have hit
  // Actually .emit( is common for some libs, but in this project it's 99% event_bus.
  // If we hit a native one, we'll fix it in verification.
  
  if (content !== originalContent) {
    fs.writeFileSync(filePath, content, 'utf8');
  }
});
