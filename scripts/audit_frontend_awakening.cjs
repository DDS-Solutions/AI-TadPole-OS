const fs = require('fs');
const path = require('path');

const TARGET_DIRS = [
    'src/components',
    'src/hooks',
    'src/layouts',
    'src/pages',
    'src/services',
    'src/stores'
];

const REQUIRED_MARKERS = [
    '* @docs ARCHITECTURE:',
    '### AI Assist Note',
    '### 🔍 Debugging & Observability'
];

function auditFile(filePath) {
    const content = fs.readFileSync(filePath, 'utf8');
    const missing = REQUIRED_MARKERS.filter(marker => !content.includes(marker));
    return {
        path: filePath,
        isCompliant: missing.length === 0,
        missing
    };
}

function walkDir(dir, fileList = []) {
    if (!fs.existsSync(dir)) return fileList;
    const files = fs.readdirSync(dir);
    files.forEach(file => {
        const filePath = path.join(dir, file);
        if (fs.statSync(filePath).isDirectory()) {
            walkDir(filePath, fileList);
        } else if (filePath.endsWith('.ts') || filePath.endsWith('.tsx')) {
            fileList.push(filePath);
        }
    });
    return fileList;
}

const allFiles = TARGET_DIRS.flatMap(dir => walkDir(path.join(process.cwd(), dir)));
const results = allFiles.map(auditFile);

const nonCompliant = results.filter(r => !r.isCompliant);

console.log(`\n--- Frontend AI Awakening Audit Report ---`);
console.log(`Total Files Checked: ${allFiles.length}`);
console.log(`Compliant: ${allFiles.length - nonCompliant.length}`);
console.log(`Non-Compliant: ${nonCompliant.length}`);

if (nonCompliant.length > 0) {
    console.log(`\nNon-Compliant Files (First 20):`);
    nonCompliant.slice(0, 20).forEach(r => {
        console.log(`- ${r.path}`);
        console.log(`  Missing: ${r.missing.join(', ')}`);
    });
}
