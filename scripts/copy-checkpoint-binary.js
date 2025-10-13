#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

console.log('📦 Copying checkpoint binary to VSCode extension...');

// Paths
const sourceFile = path.join(__dirname, '..', '..', '..', 'core', 'checkpoints', 'index.node');
const targetDir = path.join(__dirname, '..', 'out');
const targetFile = path.join(targetDir, 'checkpoints.node');

try {
    // Check if source file exists
    if (!fs.existsSync(sourceFile)) {
        console.error('❌ Checkpoint binary not found at:', sourceFile);
        console.log('💡 Make sure to build the checkpoint system first: cd core/checkpoints && npm run build');
        process.exit(1);
    }

    // Create target directory if it doesn't exist
    if (!fs.existsSync(targetDir)) {
        fs.mkdirSync(targetDir, { recursive: true });
        console.log('📁 Created directory:', targetDir);
    }

    // Copy the binary
    fs.copyFileSync(sourceFile, targetFile);
    
    // Get file size for verification
    const stats = fs.statSync(targetFile);
    const fileSizeInBytes = stats.size;
    const fileSizeInKB = Math.round(fileSizeInBytes / 1024);
    
    console.log('✅ Checkpoint binary copied successfully');
    console.log('   📄 Source:', sourceFile);
    console.log('   📄 Target:', targetFile);
    console.log('   📊 Size:', `${fileSizeInKB} KB`);
    
    // Verify the binary is loadable
    try {
        const checkpointModule = require(targetFile);
        if (checkpointModule && typeof checkpointModule.getConfig === 'function') {
            console.log('✅ Binary verification successful');
        } else {
            console.warn('⚠️  Binary copied but may not be properly structured');
        }
    } catch (loadError) {
        console.warn('⚠️  Binary copied but failed to load:', loadError.message);
        console.log('💡 This may be normal if the binary is for a different platform');
    }

} catch (error) {
    console.error('❌ Failed to copy checkpoint binary:', error.message);
    process.exit(1);
}
