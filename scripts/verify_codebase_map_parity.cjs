const fs = require('fs');
const path = require('path');

const REPO_ROOT = path.resolve(__dirname, '..');
const CODEBASE_MAP_PATH = path.join(REPO_ROOT, 'docs', 'CODEBASE_MAP.md');
const TARGET_DIRS = ['server-rs', 'src', 'src-tauri', 'wasm-codec'];
const EXTENSIONS = ['.rs', '.ts', '.tsx']; // Focus on core logic
const IGNORE_DIRS = ['node_modules', 'target', '.git', 'dist', '.tmp', 'public', 'scripts', 'tests'];

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

function parseMap(content) {
    const lines = content.split('\n');
    const paths = [];
    // Matches | `path` | OR | path | at the start of a table row
    const tableRegex = /^\|\s*`?([^`|\s]+)`?\s*\|/; 

    lines.forEach(line => {
        const match = line.match(tableRegex);
        if (match) {
            let p = match[1].trim();
            if (p.endsWith('/')) p = p.slice(0, -1);
            if (p !== 'Directory' && p !== ':---' && p !== '---') {
                paths.push(p);
            }
        }
    });
    return paths;
}

if (!fs.existsSync(CODEBASE_MAP_PATH)) {
    console.error('Error: CODEBASE_MAP.md not found.');
    process.exit(1);
}

const mapContent = fs.readFileSync(CODEBASE_MAP_PATH, 'utf8');
const mapPaths = parseMap(mapContent);

let allActualFiles = [];
TARGET_DIRS.forEach(dir => {
    const fullPath = path.join(REPO_ROOT, dir);
    if (fs.existsSync(fullPath)) {
        allActualFiles = getAllFiles(fullPath, allActualFiles);
    }
});

const results = {
    ghosts: [], // In map, but not in filesystem
    orphans: [], // In filesystem, but not in map (or in a mapped directory)
    directoriesMapped: mapPaths.filter(p => !p.includes('.'))
};

mapPaths.forEach(mp => {
    const fullPath = path.join(REPO_ROOT, mp);
    if (!fs.existsSync(fullPath)) {
        results.ghosts.push(mp);
    }
});

allActualFiles.forEach(file => {
    const relPath = path.relative(REPO_ROOT, file).replace(/\\/g, '/');
    
    // Check if path is explicitly in map or if its parent directory is mapped
    const isExplicitlyMapped = mapPaths.includes(relPath);
    const isParentMapped = results.directoriesMapped.some(dir => relPath.startsWith(dir + '/'));

    if (!isExplicitlyMapped && !isParentMapped) {
        results.orphans.push(relPath);
    }
});

console.log('--- Codebase Map Parity Report ---');
console.log(`Explicit Map Entries: ${mapPaths.length}`);
console.log(`Actual Source Files: ${allActualFiles.length}`);
console.log('----------------------------------');

if (results.ghosts.length > 0) {
    console.log('\n[!] GHOST ENTRIES (In map, but missing from disk):');
    results.ghosts.forEach(g => console.log(`- ${g}`));
} else {
    console.log('\n[OK] No ghost entries found.');
}

if (results.orphans.length > 0) {
    console.log('\n[!] ORPHANED FILES (On disk, but not in map):');
    results.orphans.slice(0, 20).forEach(o => console.log(`- ${o}`));
    if (results.orphans.length > 20) {
        console.log(`... and ${results.orphans.length - 20} more files.`);
    }
} else {
    console.log('\n[OK] No orphaned files found.');
}

if (results.ghosts.length === 0 && results.orphans.length === 0) {
    console.log('\n100% Map Parity Achieved! AI Navigation is accurate.');
}
