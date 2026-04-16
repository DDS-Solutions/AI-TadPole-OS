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
    missing: [],
    byDir: {}
};

allFiles.forEach(file => {
    const content = fs.readFileSync(file, 'utf8');
    const relPath = path.relative(__dirname, file);
    const dir = relPath.split(path.sep)[0];
    
    if (!results.byDir[dir]) {
        results.byDir[dir] = { total: 0, covered: 0 };
    }
    results.byDir[dir].total++;

    if (content.includes('@docs ARCHITECTURE:')) {
        results.covered++;
        results.byDir[dir].covered++;
    } else {
        results.missing.push(relPath);
    }
});

console.log('--- AI Awakening Coverage Report ---');
console.log(`Total Files: ${results.total}`);
console.log(`Covered: ${results.covered} (${((results.covered / results.total) * 100).toFixed(2)}%)`);
console.log('------------------------------------');
console.log('Breakdown by Directory:');
for (const [dir, stats] of Object.entries(results.byDir)) {
    console.log(`${dir}: ${stats.covered}/${stats.total} (${((stats.covered / stats.total) * 100).toFixed(2)}%)`);
}

if (results.missing.length > 0) {
    console.log('\nMissing Header:');
    results.missing.forEach(file => console.log(`[ ] ${file}`));
} else {
    console.log('\nAll files covered! 100% Parity achieved.');
}
