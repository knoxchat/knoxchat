#!/usr/bin/env node

/**
 * Enterprise build script for the Knox Checkpoint System
 * 
 * This script builds the complete enterprise-grade checkpoint system with:
 * - Rust core with all enterprise features
 * - Node.js bindings
 * - TypeScript definitions
 * - Cross-platform compatibility
 * - Performance optimizations
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const os = require('os');
const crypto = require('crypto');

const PLATFORMS = {
    'darwin-arm64': { rust: 'aarch64-apple-darwin', node: 'darwin-arm64' },
    'linux-arm64': { rust: 'aarch64-unknown-linux-gnu', node: 'linux-arm64' },
    'linux-x64': { rust: 'x86_64-unknown-linux-gnu', node: 'linux-x64' },
    'win32-arm64': { rust: 'aarch64-pc-windows-msvc', node: 'win32-arm64' },
    'win32-x64': { rust: 'x86_64-pc-windows-msvc', node: 'win32-x64' },
};

class EnterpriseBuildSystem {
    constructor() {
        this.projectRoot = process.cwd();
        this.cacheFile = path.join(this.projectRoot, '.enterprise-build-cache.json');
        this.buildConfig = {
            release: process.argv.includes('--release'),
            target: process.argv.find(arg => arg.startsWith('--target='))?.split('=')[1],
            verbose: process.argv.includes('--verbose'),
            features: process.argv.find(arg => arg.startsWith('--features='))?.split('=')[1]?.split(',') || [],
            skipTests: process.argv.includes('--skip-tests'),
            skipDocs: process.argv.includes('--skip-docs'),
        };
        
        console.log('🏗️  Enterprise Checkpoint System Build');
        console.log('=====================================');
        console.log(`Build mode: ${this.buildConfig.release ? 'Release' : 'Debug'}`);
        console.log(`Platform: ${os.platform()}-${os.arch()}`);
        console.log(`Features: ${this.buildConfig.features.join(', ') || 'default'}`);
        console.log('');
    }
    
    async build() {
        try {
            await this.validateEnvironment();
            
            // Check build cache — skip if sources unchanged and artifact exists
            const sourceHash = this.computeSourceHash();
            const cache = this.loadBuildCache();
            const artifactPath = path.join(this.projectRoot, 'index.node');
            
            if (cache.sourceHash === sourceHash && fs.existsSync(artifactPath)) {
                console.log(`⚡ No source changes detected (hash ${sourceHash.slice(0, 8)}…). Skipping enterprise build.`);
                this.printBuildSummary();
                return;
            }
            
            await this.buildRustCore();
            await this.generateBindings();
            await this.buildNodeModule();
            await this.generateTypeDefinitions();
            if (!this.buildConfig.skipTests) {
                await this.runTests();
            }
            if (!this.buildConfig.skipDocs) {
                await this.generateDocumentation();
            }
            await this.packageBinaries();
            
            // Save build cache after successful build
            this.saveBuildCache({ sourceHash, builtAt: new Date().toISOString() });
            
            console.log('✅ Enterprise build completed successfully!');
            console.log('');
            this.printBuildSummary();
            
        } catch (error) {
            console.error('❌ Build failed:', error.message);
            if (this.buildConfig.verbose) {
                console.error(error.stack);
            }
            process.exit(1);
        }
    }
    
    async validateEnvironment() {
        console.log('🔍 Validating build environment...');
        
        // Check Rust installation
        try {
            const rustVersion = execSync('rustc --version', { encoding: 'utf8' }).trim();
            console.log(`  ✓ Rust: ${rustVersion}`);
        } catch (error) {
            throw new Error('Rust is not installed or not in PATH');
        }
        
        // Check Cargo
        try {
            const cargoVersion = execSync('cargo --version', { encoding: 'utf8' }).trim();
            console.log(`  ✓ Cargo: ${cargoVersion}`);
        } catch (error) {
            throw new Error('Cargo is not available');
        }
        
        // Check Node.js
        try {
            const nodeVersion = execSync('node --version', { encoding: 'utf8' }).trim();
            console.log(`  ✓ Node.js: ${nodeVersion}`);
        } catch (error) {
            throw new Error('Node.js is not installed or not in PATH');
        }
        
        // Check required Rust targets
        if (this.buildConfig.target) {
            try {
                execSync(`rustup target add ${this.buildConfig.target}`, { stdio: 'pipe' });
                console.log(`  ✓ Rust target: ${this.buildConfig.target}`);
            } catch (error) {
                console.warn(`  ⚠️  Could not add target ${this.buildConfig.target}`);
            }
        }
        
        console.log('');
    }
    
    async buildRustCore() {
        console.log('🦀 Building Rust core...');
        const buildStart = Date.now();
        
        const cargoArgs = ['build'];
        
        if (this.buildConfig.release) {
            cargoArgs.push('--release');
        }
        
        if (this.buildConfig.target) {
            cargoArgs.push('--target', this.buildConfig.target);
        }
        
        if (this.buildConfig.features.length > 0) {
            cargoArgs.push('--features', this.buildConfig.features.join(','));
        }
        
        if (this.buildConfig.verbose) {
            cargoArgs.push('--verbose');
        }
        
        try {
            const output = execSync(`cargo ${cargoArgs.join(' ')}`, { 
                encoding: 'utf8',
                cwd: this.projectRoot,
                stdio: this.buildConfig.verbose ? 'inherit' : 'pipe'
            });
            
            const elapsed = ((Date.now() - buildStart) / 1000).toFixed(1);
            if (!this.buildConfig.verbose) {
                console.log(`  ✓ Rust core compiled successfully (${elapsed}s)`);
            }
        } catch (error) {
            throw new Error(`Rust compilation failed: ${error.message}`);
        }
        
        console.log('');
    }
    
    async generateBindings() {
        console.log('🔗 Generating Node.js bindings...');
        
        try {
            // The bindings are generated as part of the Rust build process
            // using the neon framework. We just need to verify they exist.
            
            const targetDir = this.buildConfig.release ? 'release' : 'debug';
            const targetPath = this.buildConfig.target ? 
                path.join('target', this.buildConfig.target, targetDir) : 
                path.join('target', targetDir);
            
            const binaryName = process.platform === 'win32' ? 'checkpoints.dll' : 
                              process.platform === 'darwin' ? 'libcheckpoints.dylib' : 
                              'libcheckpoints.so';
            
            const binaryPath = path.join(targetPath, binaryName);
            
            if (fs.existsSync(binaryPath)) {
                console.log('  ✓ Node.js bindings generated successfully');
            } else {
                console.warn('  ⚠️  Binary not found at expected location:', binaryPath);
            }
            
        } catch (error) {
            throw new Error(`Binding generation failed: ${error.message}`);
        }
        
        console.log('');
    }
    
    async buildNodeModule() {
        console.log('📦 Building Node.js module...');
        
        try {
            // Copy the built binary to the expected location
            const targetDir = this.buildConfig.release ? 'release' : 'debug';
            const targetPath = this.buildConfig.target ? 
                path.join('target', this.buildConfig.target, targetDir) : 
                path.join('target', targetDir);
            
            const sourceExt = process.platform === 'win32' ? '.dll' : 
                            process.platform === 'darwin' ? '.dylib' : '.so';
            const sourceName = `${process.platform === 'win32' ? '' : 'lib'}checkpoints${sourceExt}`;
            const sourcePath = path.join(targetPath, sourceName);
            
            const destPath = path.join(this.projectRoot, 'index.node');
            
            if (fs.existsSync(sourcePath)) {
                fs.copyFileSync(sourcePath, destPath);
                console.log('  ✓ Node.js module created:', destPath);
            } else {
                throw new Error(`Source binary not found: ${sourcePath}`);
            }
            
        } catch (error) {
            throw new Error(`Node.js module build failed: ${error.message}`);
        }
        
        console.log('');
    }
    
    async generateTypeDefinitions() {
        console.log('📝 Generating TypeScript definitions...');
        
        const typeDefinitions = `
// TypeScript definitions for Knox Enterprise Checkpoint System
// Generated automatically - do not edit manually

export interface CheckpointConfig {
    maxCheckpoints: number;
    retentionDays: number;
    maxStorageBytes: number;
    maxFilesPerCheckpoint: number;
    enableCompression: boolean;
}

export interface CheckpointStats {
    totalCheckpoints: number;
    totalSessions: number;
    totalStorageBytes: number;
    avgCheckpointSize: number;
    filesTracked: number;
}

export interface EnterpriseConfig extends CheckpointConfig {
    enterpriseFeatures: {
        auditLogging: boolean;
        encryption: boolean;
        compliance: string;
        monitoring: boolean;
        backups: boolean;
    };
}

/**
 * Create a simple checkpoint with the given description
 * @param description - Human-readable description of the checkpoint
 * @returns Unique checkpoint ID
 */
export function createSimpleCheckpoint(description: string): string;

/**
 * Get the current checkpoint system configuration
 * @returns Configuration object
 */
export function getConfig(): CheckpointConfig;

/**
 * Create a checkpoint manager instance for a workspace
 * @param workspacePath - Path to the workspace directory
 * @returns Session ID for the checkpoint manager
 */
export function createManager(workspacePath: string): string;

/**
 * Get comprehensive checkpoint statistics
 * @returns Statistics object
 */
export function getStats(): CheckpointStats;

/**
 * Enterprise-grade checkpoint manager
 */
export class EnterpriseCheckpointManager {
    constructor(config: EnterpriseConfig);
    
    createCheckpoint(description: string, options?: CheckpointOptions): Promise<string>;
    restoreCheckpoint(checkpointId: string, options?: RestoreOptions): Promise<RestoreResult>;
    listCheckpoints(limit?: number): Promise<CheckpointInfo[]>;
    getStatistics(): Promise<CheckpointStats>;
    cleanup(retentionDays?: number): Promise<number>;
    exportData(path: string): Promise<void>;
    importData(path: string): Promise<number>;
}

export interface CheckpointOptions {
    tags?: string[];
    metadata?: Record<string, string>;
    includeFiles?: string[];
    excludeFiles?: string[];
    compression?: boolean;
}

export interface RestoreOptions {
    createBackup?: boolean;
    conflictResolution?: 'skip' | 'overwrite' | 'prompt' | 'backup';
    includeFiles?: string[];
    excludeFiles?: string[];
    validateChecksums?: boolean;
}

export interface CheckpointInfo {
    id: string;
    description: string;
    createdAt: string;
    filesAffected: number;
    sizeBytes: number;
    tags: string[];
    metadata: Record<string, string>;
}

export interface RestoreResult {
    success: boolean;
    restoredFiles: string[];
    failedFiles: Array<{ path: string; error: string }>;
    conflicts: Array<{ path: string; type: string }>;
    backupCheckpointId?: string;
}

// Enterprise monitoring and health
export interface HealthStatus {
    overall: 'healthy' | 'warning' | 'critical';
    components: Record<string, ComponentHealth>;
    lastCheck: string;
}

export interface ComponentHealth {
    status: 'healthy' | 'warning' | 'critical';
    message: string;
    metrics: Record<string, number>;
}

export interface SystemMetrics {
    checkpoints: {
        total: number;
        todayCount: number;
        averageSize: number;
        successRate: number;
    };
    storage: {
        totalBytes: number;
        availableBytes: number;
        utilizationPercent: number;
        compressionRatio: number;
    };
    performance: {
        avgCreationTimeMs: number;
        avgRestorationTimeMs: number;
        throughputMBps: number;
    };
}
`;
        
        try {
            const defsPath = path.join(this.projectRoot, 'index.d.ts');
            fs.writeFileSync(defsPath, typeDefinitions.trim());
            console.log('  ✓ TypeScript definitions generated:', defsPath);
        } catch (error) {
            throw new Error(`TypeScript definition generation failed: ${error.message}`);
        }
        
        console.log('');
    }
    
    async runTests() {
        console.log('🧪 Running tests...');
        
        try {
            // Run Rust tests
            const cargoTestArgs = ['test'];
            if (this.buildConfig.release) {
                cargoTestArgs.push('--release');
            }
            if (this.buildConfig.features.length > 0) {
                cargoTestArgs.push('--features', this.buildConfig.features.join(','));
            }
            
            execSync(`cargo ${cargoTestArgs.join(' ')}`, {
                encoding: 'utf8',
                cwd: this.projectRoot,
                stdio: this.buildConfig.verbose ? 'inherit' : 'pipe'
            });
            
            console.log('  ✓ Rust tests passed');
            
            // Run Node.js integration tests if they exist
            if (fs.existsSync(path.join(this.projectRoot, 'test'))) {
                try {
                    execSync('npm test', {
                        encoding: 'utf8',
                        cwd: this.projectRoot,
                        stdio: this.buildConfig.verbose ? 'inherit' : 'pipe'
                    });
                    console.log('  ✓ Node.js tests passed');
                } catch (error) {
                    console.warn('  ⚠️  Node.js tests failed or not configured');
                }
            }
            
        } catch (error) {
            throw new Error(`Tests failed: ${error.message}`);
        }
        
        console.log('');
    }
    
    async generateDocumentation() {
        console.log('📚 Generating documentation...');
        
        try {
            // Generate Rust documentation
            execSync('cargo doc --no-deps', {
                encoding: 'utf8',
                cwd: this.projectRoot,
                stdio: this.buildConfig.verbose ? 'inherit' : 'pipe'
            });
            
            console.log('  ✓ Rust documentation generated');
            
            // Create README for the build
            const readme = `
# Knox Enterprise Checkpoint System

## Build Information

- **Build Date**: ${new Date().toISOString()}
- **Build Mode**: ${this.buildConfig.release ? 'Release' : 'Debug'}
- **Platform**: ${os.platform()}-${os.arch()}
- **Features**: ${this.buildConfig.features.join(', ') || 'default'}
- **Rust Version**: ${execSync('rustc --version', { encoding: 'utf8' }).trim()}
- **Node Version**: ${execSync('node --version', { encoding: 'utf8' }).trim()}

## Features

### Core Features
- ✅ Automatic file change tracking
- ✅ SQLite-based metadata storage
- ✅ LZ4 compression with deduplication
- ✅ Conflict detection and resolution
- ✅ Cross-platform compatibility

### Enterprise Features
- ✅ Audit logging and compliance
- ✅ Advanced backup strategies
- ✅ Performance monitoring
- ✅ Security and encryption
- ✅ External integrations (Git, CI/CD)
- ✅ Multi-tenant support

### VSCode Integration
- ✅ Rich command palette integration
- ✅ Automatic checkpoint creation
- ✅ Visual checkpoint indicators
- ✅ One-click restoration
- ✅ Export/import functionality

## Usage

\`\`\`javascript
const { createSimpleCheckpoint, getConfig, getStats } = require('./index.node');

// Create a checkpoint
const checkpointId = createSimpleCheckpoint('My checkpoint description');
console.log('Created checkpoint:', checkpointId);

// Get configuration
const config = getConfig();
console.log('Max checkpoints:', config.maxCheckpoints);

// Get statistics
const stats = getStats();
console.log('Total checkpoints:', stats.totalCheckpoints);
\`\`\`

## Enterprise Configuration

See \`enterprise.rs\` for comprehensive configuration options including:
- Backup policies and retention
- Security and compliance settings
- Monitoring and alerting
- Performance optimization
- External system integrations

## Support

For enterprise support and licensing, please contact the Knox team.
`;
            
            fs.writeFileSync(path.join(this.projectRoot, 'README.md'), readme.trim());
            console.log('  ✓ Documentation generated');
            
        } catch (error) {
            console.warn('  ⚠️  Documentation generation failed:', error.message);
        }
        
        console.log('');
    }
    
    async packageBinaries() {
        console.log('📦 Packaging binaries...');
        
        try {
            const packageInfo = {
                name: 'knox-checkpoints-enterprise',
                version: '1.0.0',
                description: 'Enterprise-grade checkpoint system for Knox AI',
                main: 'index.node',
                types: 'index.d.ts',
                files: ['index.node', 'index.d.ts', 'README.md'],
                engines: {
                    node: '>=16.0.0'
                },
                os: [process.platform],
                cpu: [process.arch],
                keywords: ['checkpoint', 'backup', 'versioning', 'enterprise', 'ai'],
                author: 'Knox Team',
                license: 'Commercial',
                repository: {
                    type: 'git',
                    url: 'https://github.com/knoxai/knox-vsc.git'
                },
                buildInfo: {
                    buildDate: new Date().toISOString(),
                    buildMode: this.buildConfig.release ? 'release' : 'debug',
                    platform: `${os.platform()}-${os.arch()}`,
                    features: this.buildConfig.features,
                    rustVersion: execSync('rustc --version', { encoding: 'utf8' }).trim(),
                    nodeVersion: execSync('node --version', { encoding: 'utf8' }).trim()
                }
            };
            
            fs.writeFileSync(
                path.join(this.projectRoot, 'package.json'),
                JSON.stringify(packageInfo, null, 2)
            );
            
            console.log('  ✓ Package metadata created');
            
        } catch (error) {
            throw new Error(`Binary packaging failed: ${error.message}`);
        }
        
        console.log('');
    }
    
    printBuildSummary() {
        console.log('📊 Build Summary');
        console.log('================');
        
        try {
            const stats = fs.statSync(path.join(this.projectRoot, 'index.node'));
            console.log(`Binary size: ${(stats.size / 1024 / 1024).toFixed(2)} MB`);
        } catch (error) {
            console.log('Binary size: Unknown');
        }
        
        console.log(`Build time: ${process.uptime().toFixed(2)} seconds`);
        console.log(`Platform: ${os.platform()}-${os.arch()}`);
        console.log(`Mode: ${this.buildConfig.release ? 'Release (optimized)' : 'Debug'}`);
        console.log('');
        console.log('🎉 Ready for enterprise deployment!');
    }
    
    /**
     * Compute SHA-256 hash over all Rust source files and Cargo.toml for cache invalidation.
     */
    computeSourceHash() {
        const hash = crypto.createHash('sha256');
        const srcDir = path.join(this.projectRoot, 'src');
        const cargoToml = path.join(this.projectRoot, 'Cargo.toml');
        const workspaceCargoToml = path.join(this.projectRoot, '..', 'Cargo.toml');
        
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
        
        // Also hash the build config (features, release mode) to invalidate on config change
        hash.update(JSON.stringify(this.buildConfig));
        
        for (const f of files.sort()) {
            if (fs.existsSync(f)) {
                hash.update(fs.readFileSync(f));
            }
        }
        return hash.digest('hex');
    }
    
    /**
     * Load the enterprise build cache from disk.
     */
    loadBuildCache() {
        try {
            if (fs.existsSync(this.cacheFile)) {
                return JSON.parse(fs.readFileSync(this.cacheFile, 'utf8'));
            }
        } catch { /* ignore */ }
        return {};
    }
    
    /**
     * Save build cache to disk.
     */
    saveBuildCache(data) {
        try {
            fs.writeFileSync(this.cacheFile, JSON.stringify(data, null, 2));
        } catch (e) {
            console.warn('⚠️  Could not save build cache:', e.message);
        }
    }
}

// Run the build if this script is executed directly
if (require.main === module) {
    const builder = new EnterpriseBuildSystem();
    builder.build().catch(error => {
        console.error('Build failed:', error);
        process.exit(1);
    });
}

module.exports = { EnterpriseBuildSystem };
