const fs = require('fs');
const path = require('path');

const TARGET_DIRS = ['server-rs', 'src', 'src-tauri', 'wasm-codec'];
const EXTENSIONS = ['.rs', '.ts', '.tsx', '.css', '.html'];
const IGNORE_DIRS = ['node_modules', 'target', '.git', 'dist', '.tmp', 'public', 'scripts'];

function getAllFiles(dirPath, arrayOfFiles) {
  let files = fs.readdirSync(dirPath);

  arrayOfFiles = arrayOfFiles || [];

  files.forEach(function(file) {
    const fullPath = path.join(dirPath, file);
    if (fs.statSync(fullPath).isDirectory()) {
      if (!IGNORE_DIRS.includes(file)) {
        arrayOfFiles = getAllFiles(fullPath, arrayOfFiles);
      }
    } else {
      if (EXTENSIONS.includes(path.extname(file))) {
        arrayOfFiles.push(fullPath);
      }
    }
  });

  return arrayOfFiles;
}

let allFiles = [];
TARGET_DIRS.forEach(dir => {
    const fullPath = path.resolve(__dirname, dir);
    if (fs.existsSync(fullPath)) {
        allFiles = getAllFiles(fullPath, allFiles);
    }
});

const results = {
    total: allFiles.length,
    covered: 0,
    hiFi: 0,
    missingHeader: [],
    missingHiFi: [],
    byDir: {}
};

allFiles.forEach(file => {
    const content = fs.readFileSync(file, 'utf8');
    const relPath = path.relative(__dirname, file);
    const dir = relPath.split(path.sep)[0];
    
    if (!results.byDir[dir]) {
        results.byDir[dir] = { total: 0, covered: 0, hiFi: 0 };
    }
    results.byDir[dir].total++;

    const hasHeader = content.includes('@docs ARCHITECTURE:');
    const hasAssist = content.includes('AI Assist') || content.includes('### AI Assist');
    const hasDebug = content.includes('### 🔍 Debugging & Observability') || content.includes('### Debugging & Observability');

    if (hasHeader) {
        results.covered++;
        results.byDir[dir].covered++;
    } else {
        results.missingHeader.push(relPath);
    }

    if (hasHeader && (hasAssist || hasDebug)) {
        // High-Fi means it has at least one extra detail section beyond just the tag
        results.hiFi++;
        results.byDir[dir].hiFi++;
    } else if (hasHeader) {
        results.missingHiFi.push(relPath);
    }
});

console.log('--- Super AI Awakening (v2) Coverage Report ---');
console.log(`Total Files: ${results.total}`);
console.log(`Standard Covered (@docs): ${results.covered} (${((results.covered / results.total) * 100).toFixed(2)}%)`);
console.log(`High-Fidelity Covered (Debug): ${results.hiFi} (${((results.hiFi / results.total) * 100).toFixed(2)}%)`);
console.log('-----------------------------------------------');
console.log('Breakdown by Directory (Standard / High-Fi):');
for (const [dir, stats] of Object.entries(results.byDir)) {
    const standardPct = ((stats.covered / stats.total) * 100).toFixed(2);
    const hiFiPct = ((stats.hiFi / stats.total) * 100).toFixed(2);
    console.log(`${dir.padEnd(12)} | Std: ${stats.covered}/${stats.total} (${standardPct}%) | HiFi: ${stats.hiFi}/${stats.total} (${hiFiPct}%)`);
}

if (results.missingHeader.length > 0) {
    console.log('\n[!] Missing BASE Header (@docs ARCHITECTURE:):');
    results.missingHeader.forEach(file => console.log(`[ ] ${file}`));
}

if (results.missingHiFi.length > 0) {
    console.log('\n[!] Missing HIGH-FIDELITY Sections (Debug/Assist):');
    results.missingHiFi.forEach(file => console.log(`- ${file}`));
}

if (results.missingHeader.length === 0 && results.missingHiFi.length === 0) {
    console.log('\n100% Super-Awakened! Codebase is High-Fidelity Agent Ready.');
}
