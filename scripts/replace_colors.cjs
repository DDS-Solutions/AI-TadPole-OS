const fs = require('fs');
const path = require('path');

const directoryPath = path.join(__dirname);

function walk(dir, callback) {
    fs.readdir(dir, (err, files) => {
        if (err) throw err;
        files.forEach(file => {
            const filepath = path.join(dir, file);
            fs.stat(filepath, (err, stats) => {
                if (stats.isDirectory()) {
                    walk(filepath, callback);
                } else if (stats.isFile() && (filepath.endsWith('.tsx') || filepath.endsWith('.ts'))) {
                    callback(filepath);
                }
            });
        });
    });
}

walk(directoryPath, (filepath) => {
    fs.readFile(filepath, 'utf8', (err, data) => {
        if (err) throw err;
        let result = data;

        result = result.replace(/bg-\[\#09090b\]/g, 'bg-zinc-950');
        result = result.replace(/bg-\[\#18181b\]/g, 'bg-zinc-900');
        result = result.replace(/bg-\[\#0f0f11\]/g, 'bg-zinc-950');
        result = result.replace(/bg-\[\#111113\]/g, 'bg-zinc-950');
        result = result.replace(/bg-\[\#27272a\]/g, 'bg-zinc-800');
        result = result.replace(/border-\[\#09090b\]/g, 'border-zinc-950');
        result = result.replace(/border-\[\#18181b\]/g, 'border-zinc-900');

        result = result.replace(/\[\#09090b\]/g, 'zinc-950');
        result = result.replace(/\[\#18181b\]/g, 'zinc-900');
        result = result.replace(/\[\#0f0f11\]/g, 'zinc-950');
        result = result.replace(/\[\#111113\]/g, 'zinc-950');
        result = result.replace(/\[\#27272a\]/g, 'zinc-800');

        if (result !== data) {
            fs.writeFile(filepath, result, 'utf8', (err) => {
                if (err) throw err;
                console.log(`Updated: ${filepath}`);
            });
        }
    });
});
