#!/usr/bin/env node
/**
 * Build the checkpoints module (uses workspace build)
 * Supports incremental builds with hash-based invalidation.
 */

const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const crypto = require('crypto');

console.log('Building Knox Checkpoint System...');

const workspaceRoot = path.join(__dirname, '..');
const targetFile = path.join(__dirname, 'index.node');
const cacheFile = path.join(__dirname, '.build-cache.json');

/**
 * Compute a SHA-256 hash over all Rust source files and Cargo.toml to detect changes.
 */
function computeSourceHash() {
    const hash = crypto.createHash('sha256');
    const srcDir = path.join(__dirname, 'src');
    const cargoToml = path.join(__dirname, 'Cargo.toml');
    const workspaceCargoToml = path.join(workspaceRoot, 'Cargo.toml');

    const files = [cargoToml, workspaceCargoToml];
    if (fs.existsSync(srcDir)) {
        (function walk(dir) {
            for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
                if (entry.isDirectory()) {
                    walk(path.join(dir, entry.name));
                } else if (entry.name.endsWith('.rs') || entry.name === 'Cargo.toml') {
                    files.push(path.join(dir, entry.name));
                }
            }
        })(srcDir);
    }

    for (const f of files.sort()) {
        if (fs.existsSync(f)) {
            hash.update(fs.readFileSync(f));
        }
    }
    return hash.digest('hex');
}

function loadCache() {
    try {
        if (fs.existsSync(cacheFile)) return JSON.parse(fs.readFileSync(cacheFile, 'utf8'));
    } catch { /* ignore */ }
    return {};
}

function saveCache(data) {
    fs.writeFileSync(cacheFile, JSON.stringify(data, null, 2));
}

const startTime = Date.now();
const sourceHash = computeSourceHash();
const cache = loadCache();

// Skip build if nothing changed and artifact exists
if (cache.sourceHash === sourceHash && fs.existsSync(targetFile)) {
    console.log(`⚡ No source changes detected (hash ${sourceHash.slice(0, 8)}…). Skipping build.`);
    process.exit(0);
}

// Clean previous build
if (fs.existsSync(targetFile)) {
    fs.unlinkSync(targetFile);
}

try {
    // Build using workspace (this is faster if other modules are already built)
    console.log('Compiling Rust library (workspace build)...');
    execSync('cargo build --release -p checkpoints', {
        cwd: workspaceRoot,
        stdio: 'inherit'
    });
    
    // Copy the built library to the expected location
    const sourceFile = path.join(workspaceRoot, 'target', 'release', getLibraryName());
    
    if (fs.existsSync(sourceFile)) {
        fs.copyFileSync(sourceFile, targetFile);
        const elapsed = ((Date.now() - startTime) / 1000).toFixed(1);
        saveCache({ sourceHash, builtAt: new Date().toISOString() });
        console.log(`✅ Built checkpoint system: ${targetFile} (${elapsed}s)`);
    } else {
        throw new Error(`Built library not found at ${sourceFile}`);
    }
    
} catch (error) {
    console.error('❌ Build failed:', error.message);
    process.exit(1);
}

function getLibraryName() {
    const platform = process.platform;
    
    switch (platform) {
        case 'win32':
            return 'checkpoints.dll';
        case 'darwin':
            return 'libcheckpoints.dylib';
        case 'linux':
            return 'libcheckpoints.so';
        default:
            throw new Error(`Unsupported platform: ${platform}`);
    }
}
