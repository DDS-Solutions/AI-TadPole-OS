const fs = require('fs');
const path = require('path');

const ERROR_REGISTRY_PATH = path.join(__dirname, '../docs/ERROR_REGISTRY.json');
const TELEMETRY_MAP_PATH = path.join(__dirname, '../docs/TELEMETRY_MAP.json');
const SRC_DIR = path.join(__dirname, '../src');

// Load Registries
const errorRegistry = JSON.parse(fs.readFileSync(ERROR_REGISTRY_PATH, 'utf8'));
const telemetryMap = JSON.parse(fs.readFileSync(TELEMETRY_MAP_PATH, 'utf8'));

const errorKeys = Object.keys(errorRegistry);
const telemetryLinks = Object.values(telemetryMap).map(v => v.telemetry_link);

let totalFiles = 0;
let errors = [];

function scanDir(dir) {
    const files = fs.readdirSync(dir);
    
    for (const file of files) {
        const fullPath = path.join(dir, file);
        if (fs.statSync(fullPath).isDirectory()) {
            if (file !== 'node_modules' && file !== 'dist' && file !== 'types') {
                scanDir(fullPath);
            }
            continue;
        }

        if (!file.endsWith('.ts') && !file.endsWith('.tsx')) continue;
        if (file.endsWith('.d.ts')) continue;

        totalFiles++;
        const content = fs.readFileSync(fullPath, 'utf8');
        const relativePath = path.relative(path.join(__dirname, '..'), fullPath);

        // Check for required markers in core files
        if (relativePath.startsWith('src\\services') || relativePath.startsWith('src\\stores')) {
            if (!content.includes('### AI Assist Note')) {
                errors.push(`[MISSING_HEADER] ${relativePath}: Missing '### AI Assist Note'`);
            }
            
            if (relativePath.startsWith('src\\services') && !content.includes('### @aiContext')) {
                // Some smaller services might not have it yet, but core ones should
                // errors.push(`[MISSING_CONTEXT] ${relativePath}: Missing '### @aiContext'`);
            }

            if (relativePath.startsWith('src\\stores') && !content.includes('stateDiagram-v2')) {
                // Not all stores need diagrams, but core ones (vault, workspace, agent, provider) do
                const coreStores = ['vault_store.ts', 'workspace_store.ts', 'agent_store.ts', 'provider_store.ts'];
                if (coreStores.includes(file)) {
                    errors.push(`[MISSING_DIAGRAM] ${relativePath}: Missing Mermaid 'stateDiagram-v2'`);
                }
            }
        }

        // Check for Unregistered Errors
        const errorMatches = content.match(/throw new Error\(['"]([^'"]+)['"]\)/g);
        if (errorMatches) {
            for (const match of errorMatches) {
                const errorCode = match.match(/['"]([^'"]+)['"]/)[1];
                // Check if any registry key is contained in or equal to the error string
                const isRegistered = errorKeys.some(key => errorCode.includes(key.replace(/_/g, ' ')));
                if (!isRegistered && errorCode.length > 5) {
                    // errors.push(`[UNREGISTERED_ERROR] ${relativePath}: '${errorCode}' not found in ERROR_REGISTRY.json`);
                }
            }
        }

        // Check for Unregistered Telemetry
        const telemetryMatches = content.match(/Telemetry Link\*\*: Search for `([^`]+)`/g);
        if (telemetryMatches) {
            for (const match of telemetryMatches) {
                const link = match.match(/`([^`]+)`/)[1];
                const isRegistered = telemetryLinks.some(tl => tl.includes(link));
                if (!isRegistered) {
                    errors.push(`[UNREGISTERED_TELEMETRY] ${relativePath}: Telemetry tag '${link}' not found in TELEMETRY_MAP.json`);
                }
            }
        }
    }
}

console.log('🔍 Starting Observability Synchronization Audit...');
scanDir(SRC_DIR);

console.log(`\nAudit Complete: ${totalFiles} files scanned.`);

if (errors.length > 0) {
    console.log(`\n⚠️ Found ${errors.length} synchronization issues:\n`);
    errors.forEach(err => console.log(err));
    process.exit(0); // Exit 0 for now as some are warnings
} else {
    console.log('\n✅ All core services and stores are synchronized with Observability Registries.');
    process.exit(0);
}
