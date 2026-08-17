/**
 * 🏷️ Tadpole OS Version Synchronization Utility
 * Usage: node scripts/bump_version.cjs <new_version>
 * Example: node scripts/bump_version.cjs 1.1.412
 */

const fs = require('fs');
const path = require('path');

const newVersion = process.argv[2];

if (!newVersion) {
    console.error('❌ Error: Please specify target version. Example: node scripts/bump_version.cjs 1.1.412');
    process.exit(1);
}

if (!/^\d+\.\d+\.\d+$/.test(newVersion)) {
    console.error(`❌ Error: Invalid semver string '${newVersion}'. Format must be X.Y.Z (e.g. 1.1.412)`);
    process.exit(1);
}

const rootDir = path.resolve(__dirname, '..');
const today = new Date().toISOString().split('T')[0];

let updatedCount = 0;

// 1. Update version.json
const versionJsonPath = path.join(rootDir, 'version.json');
if (fs.existsSync(versionJsonPath)) {
    const data = JSON.parse(fs.readFileSync(versionJsonPath, 'utf8'));
    data.version = newVersion;
    data.last_updated = today;
    fs.writeFileSync(versionJsonPath, JSON.stringify(data, null, 2) + '\n');
    console.log(`✅ [version.json] -> ${newVersion} (last_updated: ${today})`);
    updatedCount++;
}

// 2. Update package.json
const packageJsonPath = path.join(rootDir, 'package.json');
if (fs.existsSync(packageJsonPath)) {
    const data = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
    data.version = newVersion;
    fs.writeFileSync(packageJsonPath, JSON.stringify(data, null, 2) + '\n');
    console.log(`✅ [package.json] -> ${newVersion}`);
    updatedCount++;
}

// 3. Update server-rs/Cargo.toml
const cargoTomlPath = path.join(rootDir, 'server-rs', 'Cargo.toml');
if (fs.existsSync(cargoTomlPath)) {
    let content = fs.readFileSync(cargoTomlPath, 'utf8');
    content = content.replace(/^version\s*=\s*"[^"]+"/m, `version = "${newVersion}"`);
    fs.writeFileSync(cargoTomlPath, content);
    console.log(`✅ [server-rs/Cargo.toml] -> ${newVersion}`);
    updatedCount++;
}

// 4. Update apps/mobile-android/app/build.gradle.kts
const androidGradlePath = path.join(rootDir, 'apps', 'mobile-android', 'app', 'build.gradle.kts');
if (fs.existsSync(androidGradlePath)) {
    let content = fs.readFileSync(androidGradlePath, 'utf8');
    content = content.replace(/versionName\s*=\s*"[^"]+"/g, `versionName = "${newVersion}"`);
    fs.writeFileSync(androidGradlePath, content);
    console.log(`✅ [apps/mobile-android/app/build.gradle.kts] -> ${newVersion}`);
    updatedCount++;
}

console.log(`\n🎉 Successfully synchronized ${updatedCount} files to version ${newVersion}\n`);
