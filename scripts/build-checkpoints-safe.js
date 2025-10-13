#!/usr/bin/env node

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('🔧 Building checkpoint system...');

// Paths
const checkpointDir = path.join(__dirname, '..', '..', '..', 'core', 'checkpoints');
const extensionDir = path.join(__dirname, '..');

async function buildCheckpoints() {
    try {
        // Check if checkpoint directory exists
        if (!fs.existsSync(checkpointDir)) {
            console.warn('⚠️  Checkpoint source directory not found:', checkpointDir);
            console.log('💡 Checkpoint functionality will be disabled');
            return false;
        }

        // Check if Rust is available
        try {
            execSync('cargo --version', { stdio: 'pipe' });
        } catch (error) {
            console.warn('⚠️  Rust/Cargo not found. Checkpoint functionality will be disabled.');
            console.log('💡 To enable checkpoints, install Rust from https://rustup.rs/');
            return false;
        }

        // Check if package.json exists in checkpoint directory
        const checkpointPackageJson = path.join(checkpointDir, 'package.json');
        if (!fs.existsSync(checkpointPackageJson)) {
            console.warn('⚠️  Checkpoint package.json not found');
            return false;
        }

        console.log('📦 Building checkpoint native module...');
        
        // Change to checkpoint directory and build
        process.chdir(checkpointDir);
        
        // Install dependencies if needed
        if (!fs.existsSync(path.join(checkpointDir, 'node_modules'))) {
            console.log('📥 Installing checkpoint dependencies...');
            execSync('npm install', { stdio: 'inherit' });
        }
        
        // Build the checkpoint system
        execSync('npm run build', { stdio: 'inherit' });
        
        // Check if the binary was created
        const binaryPath = path.join(checkpointDir, 'index.node');
        if (!fs.existsSync(binaryPath)) {
            console.error('❌ Checkpoint binary not created after build');
            return false;
        }
        
        console.log('✅ Checkpoint system built successfully');
        
        // Copy the binary to the extension directory
        console.log('📋 Copying binary to extension...');
        
        // Change back to extension directory
        process.chdir(extensionDir);
        
        // Run the copy script
        execSync('node scripts/copy-checkpoint-binary.js', { stdio: 'inherit' });
        
        return true;
        
    } catch (error) {
        console.warn('⚠️  Failed to build checkpoint system:', error.message);
        console.log('💡 Extension will continue without checkpoint functionality');
        return false;
    }
}

async function main() {
    const success = await buildCheckpoints();
    
    if (success) {
        console.log('🎉 Checkpoint system integration complete!');
    } else {
        console.log('⚠️  Extension will run without checkpoint functionality');
        console.log('💡 To enable checkpoints later, run: npm run build:checkpoints');
    }
    
    // Always exit successfully to not break the extension build
    process.exit(0);
}

main().catch(error => {
    console.error('❌ Unexpected error in checkpoint build:', error);
    console.log('⚠️  Extension will continue without checkpoint functionality');
    process.exit(0); // Don't fail the main build
});
