const fs = require('fs');
const path = require('path');

const SRC_DIR = path.join(__dirname, '../src');
const OUTPUT_FILE = path.join(__dirname, '../docs/TELEMETRY_MAP.json');

function scanDirectory(dir, results = []) {
    const files = fs.readdirSync(dir);
    for (const file of files) {
        const fullPath = path.join(dir, file);
        if (fs.statSync(fullPath).isDirectory()) {
            scanDirectory(fullPath, results);
        } else if (file.endsWith('.ts') || file.endsWith('.tsx')) {
            results.push(fullPath);
        }
    }
    return results;
}

const TELEMETRY_REGEX = /Telemetry Link\*\*(?:: Search (for|`))?\s*`([^`]+)`/g;

function generateMap() {
    const files = scanDirectory(SRC_DIR);
    const telemetryMap = {};

    files.forEach(filePath => {
        const content = fs.readFileSync(filePath, 'utf8');
        let match;
        
        while ((match = TELEMETRY_REGEX.exec(content)) !== null) {
            const link = match[2];
            const relativePath = path.relative(path.join(__dirname, '..'), filePath).replace(/\\/g, '/');
            
            // Generate a clean key from filename
            const baseName = path.basename(filePath, path.extname(filePath));
            const key = baseName.replace(/[^a-zA-Z0-9]/g, '_');
            
            // If key exists, append suffix
            let finalKey = key;
            let counter = 1;
            while (telemetryMap[finalKey]) {
                finalKey = `${key}_${counter++}`;
            }

            telemetryMap[finalKey] = {
                module: relativePath,
                emitter: baseName,
                desc: `Auto-registered telemetry link for ${baseName}.`,
                telemetry_link: link
            };
        }
    });

    fs.writeFileSync(OUTPUT_FILE, JSON.stringify(telemetryMap, null, 2));
    console.log(`✅ Generated ${Object.keys(telemetryMap).length} entries in ${OUTPUT_FILE}`);
}

generateMap();
