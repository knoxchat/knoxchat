/**
 * Git Checkpoint Bridge - Integration between Git commits and AI Context System
 * 
 * This component bridges Git version control with the AI Context System,
 * converting Git commits to semantic checkpoints as specified in checkpoint-system-design.md.
 */

import { execSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

import { CodeUnderstandingEngine } from '../CodeUnderstandingEngine';
import * as Types from '../types';

// Define missing types for Git integration
type SemanticMap = Record<string, any>;
type SemanticContext = Record<string, any>;
type IntentAnalysis = Record<string, any>;
type ArchitecturalImpact = Record<string, any>;
type CodeRelationships = Record<string, any>;

// Interface for CheckpointManager that this class will implement
interface CheckpointManager {
    getCheckpointsForWorkspace(workspace: string): Promise<Types.AIContextCheckpoint[]>;
}

export class GitCheckpointBridge implements CheckpointManager {
    private codeUnderstanding: CodeUnderstandingEngine;
    private gitRepo: GitRepository;
    private checkpointCache: Map<string, Types.AIContextCheckpoint[]> = new Map();
    private repoPath: string;

    constructor(repoPath: string) {
        this.repoPath = repoPath;
        this.codeUnderstanding = new CodeUnderstandingEngine();
        this.gitRepo = new GitRepository(repoPath);
    }

    /**
     * Implementation of CheckpointManager interface
     */
    async getCheckpointsForWorkspace(workspace: string): Promise<Types.AIContextCheckpoint[]> {
        // Check cache first
        if (this.checkpointCache.has(workspace)) {
            return this.checkpointCache.get(workspace)!;
        }

        try {
            // Get recent commits and convert to checkpoints
            const recentCommits = await this.gitRepo.getRecentCommits(10);
            const checkpoints = await Promise.all(
                recentCommits.map(commit => this.syncWithGit(commit))
            );

            // Cache the results
            this.checkpointCache.set(workspace, checkpoints);
            return checkpoints;
        } catch (error) {
            console.error('Failed to get checkpoints for workspace:', error);
            return [];
        }
    }

    /**
     * Convert Git commit to semantic checkpoint
     */
    async syncWithGit(gitCommit: GitCommit): Promise<Types.AIContextCheckpoint> {
        try {
            // Analyze commit semantics
            const semanticChanges = await this.analyzeCommitSemantics(gitCommit);
            
            // Create checkpoint from commit
            const checkpoint = await this.createCheckpointFromCommit(gitCommit, semanticChanges);
            
            return checkpoint;
        } catch (error) {
            console.error('Failed to sync with Git:', error);
            throw new Error(`Git sync failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Sync multiple commits to create checkpoint timeline
     */
    async syncCommitRange(fromCommit: string, toCommit: string): Promise<Types.AIContextCheckpoint[]> {
        const commits = await this.gitRepo.getCommitRange(fromCommit, toCommit);
        const checkpoints: Types.AIContextCheckpoint[] = [];

        for (const commit of commits) {
            try {
                const checkpoint = await this.syncWithGit(commit);
                checkpoints.push(checkpoint);
            } catch (error) {
                console.warn(`Failed to process commit ${commit.hash}:`, error);
            }
        }

        return checkpoints;
    }

    /**
     * Get latest changes and create checkpoint
     */
    async syncLatestChanges(): Promise<Types.AIContextCheckpoint | null> {
        const latestCommit = await this.gitRepo.getLatestCommit();
        
        if (!latestCommit) {
            return null;
        }

        return await this.syncWithGit(latestCommit);
    }

    /**
     * Create checkpoint from Git branch
     */
    async syncBranch(branchName: string): Promise<Types.AIContextCheckpoint[]> {
        const commits = await this.gitRepo.getBranchCommits(branchName);
        const checkpoints: Types.AIContextCheckpoint[] = [];

        for (const commit of commits) {
            const checkpoint = await this.syncWithGit(commit);
            checkpoints.push(checkpoint);
        }

        return checkpoints;
    }

    /**
     * Analyze semantic meaning of Git commit
     */
    private async analyzeCommitSemantics(gitCommit: GitCommit): Promise<CommitSemanticAnalysis> {
        // Analyze commit message
        const messageAnalysis = await this.analyzeCommitMessage(gitCommit.message);
        
        // Analyze changed files
        const fileAnalysis = await this.analyzeChangedFiles(gitCommit);
        
        // Analyze code changes
        const codeAnalysis = await this.analyzeCodeChanges(gitCommit);
        
        // Determine change impact
        const impactAnalysis = await this.analyzeChangeImpact(gitCommit);

        return {
            messageAnalysis,
            fileAnalysis,
            codeAnalysis,
            impactAnalysis,
            changeType: this.determineChangeType(messageAnalysis, fileAnalysis, codeAnalysis),
            semanticScope: this.determineSemanticScope(fileAnalysis, codeAnalysis),
            architecturalChanges: await this.identifyArchitecturalChanges(gitCommit)
        };
    }

    /**
     * Create AI Context checkpoint from Git commit
     */
    private async createCheckpointFromCommit(
        gitCommit: GitCommit,
        semanticChanges: CommitSemanticAnalysis
    ): Promise<Types.AIContextCheckpoint> {
        // Get current codebase state
        const currentFiles = await this.gitRepo.getFilesAtCommit(gitCommit.hash);
        const semanticMap = await this.codeUnderstanding.analyzeCodebase(currentFiles);

        // Build semantic context
        const semanticContext = await this.buildSemanticContext(semanticMap, currentFiles);
        
        // Build intent analysis
        const intentAnalysis = await this.buildIntentAnalysis(gitCommit, semanticChanges);
        
        // Build architectural impact
        const architecturalImpact = await this.buildArchitecturalImpact(semanticChanges);
        
        // Build code relationships
        const codeRelationships = await this.buildCodeRelationships(semanticMap);

        // Create base checkpoint
        const baseCheckpoint: Types.BaseCheckpoint = {
            id: this.generateCheckpointId(gitCommit),
            session_id: `git-${gitCommit.hash}`,
            description: gitCommit.message,
            created_at: new Date(gitCommit.timestamp * 1000),
            file_changes: await this.buildFileChanges(gitCommit),
            files_affected: semanticChanges.fileAnalysis.added.length + semanticChanges.fileAnalysis.modified.length,
            size_bytes: 0, // Would be calculated from file changes
            tags: [semanticChanges.changeType],
            metadata: {
                git_hash: gitCommit.hash,
                git_author: gitCommit.author,
                git_branch: gitCommit.branch || 'main'
            }
        };

        return {
            id: this.generateCheckpointId(gitCommit),
            base_checkpoint: baseCheckpoint,
            semantic_context: semanticContext,
            intent_analysis: intentAnalysis,
            architectural_impact: architecturalImpact,
            code_relationships: codeRelationships,
            confidence_score: 0.8, // Default confidence score
            created_at: new Date(gitCommit.timestamp * 1000),
            file_changes: await this.buildFileChanges(gitCommit),
            git_metadata: {
                commit_hash: gitCommit.hash,
                author: gitCommit.author,
                message: gitCommit.message,
                branch: gitCommit.branch,
                parent_commits: gitCommit.parents,
                merge_commit: gitCommit.parents.length > 1
            }
        };
    }

    /**
     * Analyze commit message for intent and type
     */
    private async analyzeCommitMessage(message: string): Promise<CommitMessageAnalysis> {
        const analysis: CommitMessageAnalysis = {
            intent: this.extractIntentFromMessage(message),
            type: this.classifyCommitType(message),
            scope: this.extractScopeFromMessage(message),
            breakingChange: this.isBreakingChange(message),
            issueReferences: this.extractIssueReferences(message),
            keywords: this.extractKeywords(message)
        };

        return analysis;
    }

    /**
     * Analyze changed files in commit
     */
    private async analyzeChangedFiles(gitCommit: GitCommit): Promise<FileAnalysis> {
        const changedFiles = await this.gitRepo.getChangedFiles(gitCommit.hash);
        
        const analysis: FileAnalysis = {
            added: changedFiles.filter(f => f.status === 'A'),
            modified: changedFiles.filter(f => f.status === 'M'),
            deleted: changedFiles.filter(f => f.status === 'D'),
            renamed: changedFiles.filter(f => f.status === 'R'),
            fileTypes: this.categorizeFileTypes(changedFiles),
            impactedModules: await this.identifyImpactedModules(changedFiles),
            testFiles: changedFiles.filter(f => this.isTestFile(f.path)),
            configFiles: changedFiles.filter(f => this.isConfigFile(f.path))
        };

        return analysis;
    }

    /**
     * Analyze code changes semantically
     */
    private async analyzeCodeChanges(gitCommit: GitCommit): Promise<CodeChangeAnalysis> {
        const diff = await this.gitRepo.getCommitDiff(gitCommit.hash);
        
        return {
            functionsAdded: await this.extractAddedFunctions(diff),
            functionsModified: await this.extractModifiedFunctions(diff),
            functionsDeleted: await this.extractDeletedFunctions(diff),
            classesAdded: await this.extractAddedClasses(diff),
            classesModified: await this.extractModifiedClasses(diff),
            classesDeleted: await this.extractDeletedClasses(diff),
            interfacesChanged: await this.extractInterfaceChanges(diff),
            importsChanged: await this.extractImportChanges(diff),
            complexityChange: await this.calculateComplexityChange(diff),
            linesAdded: diff.additions,
            linesDeleted: diff.deletions
        };
    }

    /**
     * Analyze change impact across the system
     */
    private async analyzeChangeImpact(gitCommit: GitCommit): Promise<ChangeImpactAnalysis> {
        const changedFiles = await this.gitRepo.getChangedFiles(gitCommit.hash);
        
        return {
            directImpact: changedFiles.map(f => f.path),
            indirectImpact: await this.findIndirectlyImpactedFiles(changedFiles),
            testImpact: await this.findImpactedTests(changedFiles),
            apiChanges: await this.identifyAPIChanges(gitCommit),
            databaseChanges: await this.identifyDatabaseChanges(changedFiles),
            configurationChanges: await this.identifyConfigurationChanges(changedFiles),
            riskLevel: this.assessChangeRisk(changedFiles)
        };
    }

    /**
     * Build semantic context from analyzed codebase
     */
    private async buildSemanticContext(
        semanticMap: SemanticMap,
        files: Map<string, any>
    ): Promise<Types.SemanticContext> {
        return {
            functions: new Map(),
            classes: new Map(),
            interfaces: new Map(),
            types: new Map(),
            constants: new Map(),
            imports: [],
            exports: [],
            call_chains: [],
            inheritance_tree: {},
            dependency_graph: {},
            usage_patterns: []
        };
    }

    /**
     * Build intent analysis from commit and semantic changes
     */
    private async buildIntentAnalysis(
        gitCommit: GitCommit,
        semanticChanges: CommitSemanticAnalysis
    ): Promise<Types.IntentAnalysis> {
        return {
            change_intent: {},
            affected_features: [],
            design_patterns_used: [],
            architectural_decisions: [],
            refactoring_type: 'none',
            confidence: 0.8
        };
    }

    /**
     * Build architectural impact analysis
     */
    private async buildArchitecturalImpact(
        semanticChanges: CommitSemanticAnalysis
    ): Promise<Types.ArchitecturalImpact> {
        return {
            layers_affected: [],
            patterns_introduced: [],
            patterns_modified: [],
            dependency_changes: [],
            boundary_changes: [],
            significance: 'Medium' as const
        };
    }

    /**
     * Build code relationships from semantic map
     */
    private async buildCodeRelationships(semanticMap: SemanticMap): Promise<Types.CodeRelationships> {
        return {
            direct_dependencies: [],
            transitive_dependencies: [],
            dependents: [],
            coupling_strength: {},
            cohesion_metrics: {
                functional_cohesion: 0,
                sequential_cohesion: 0,
                communicational_cohesion: 0,
                procedural_cohesion: 0,
                temporal_cohesion: 0,
                logical_cohesion: 0,
                coincidental_cohesion: 0
            }
        };
    }

    // Helper methods for commit analysis
    private extractIntentFromMessage(message: string): string {
        // Extract intent from conventional commit messages or natural language
        const conventionalMatch = message.match(/^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: (.+)/);
        
        if (conventionalMatch) {
            return conventionalMatch[3];
        }
        
        // Extract first line as intent
        return message.split('\n')[0];
    }

    private classifyCommitType(message: string): CommitType {
        const messageLower = message.toLowerCase();
        
        if (/^feat|add|implement|create/.test(messageLower)) {return 'feature';}
        if (/^fix|bug|resolve|patch/.test(messageLower)) {return 'bugfix';}
        if (/^refactor|restructure|reorganize/.test(messageLower)) {return 'refactor';}
        if (/^docs|documentation/.test(messageLower)) {return 'documentation';}
        if (/^test|spec/.test(messageLower)) {return 'test';}
        if (/^style|format/.test(messageLower)) {return 'style';}
        if (/^chore|maintenance/.test(messageLower)) {return 'maintenance';}
        if (/^perf|performance|optimize/.test(messageLower)) {return 'performance';}
        if (/^security|secure/.test(messageLower)) {return 'security';}
        
        return 'other';
    }

    private extractScopeFromMessage(message: string): string | null {
        const scopeMatch = message.match(/\(([^)]+)\)/);
        return scopeMatch ? scopeMatch[1] : null;
    }

    private isBreakingChange(message: string): boolean {
        return message.includes('BREAKING CHANGE') || message.includes('!:');
    }

    private extractIssueReferences(message: string): string[] {
        const issueMatches = message.match(/#\d+/g);
        return issueMatches || [];
    }

    private extractKeywords(message: string): string[] {
        // Extract meaningful keywords from commit message
        const words = message.toLowerCase().match(/\b[a-z]{3,}\b/g) || [];
        return [...new Set(words)].filter(word => !this.isStopWord(word));
    }

    private isStopWord(word: string): boolean {
        const stopWords = new Set([
            'the', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for', 'of', 'with', 'by'
        ]);
        return stopWords.has(word);
    }

    private categorizeFileTypes(files: ChangedFile[]): FileTypeCategories {
        return {
            source: files.filter(f => this.isSourceFile(f.path)),
            test: files.filter(f => this.isTestFile(f.path)),
            config: files.filter(f => this.isConfigFile(f.path)),
            documentation: files.filter(f => this.isDocumentationFile(f.path)),
            assets: files.filter(f => this.isAssetFile(f.path))
        };
    }

    private isSourceFile(filePath: string): boolean {
        return /\.(ts|tsx|js|jsx|py|rs|go|java|cpp|c|h)$/.test(filePath);
    }

    private isTestFile(filePath: string): boolean {
        return /\.(test|spec)\.(ts|tsx|js|jsx)$/.test(filePath) || filePath.includes('/test/');
    }

    private isConfigFile(filePath: string): boolean {
        return /\.(json|yaml|yml|toml|ini|conf|config)$/.test(filePath) || 
               ['package.json', 'tsconfig.json', 'webpack.config.js'].includes(path.basename(filePath));
    }

    private isDocumentationFile(filePath: string): boolean {
        return /\.(md|rst|txt)$/.test(filePath) || filePath.includes('/docs/');
    }

    private isAssetFile(filePath: string): boolean {
        return /\.(png|jpg|jpeg|gif|svg|css|scss|sass|less)$/.test(filePath);
    }

    private generateCheckpointId(gitCommit: GitCommit): string {
        return `git-${gitCommit.hash}-${Date.now()}`;
    }

    // Placeholder implementations for complex analysis methods
    private async identifyImpactedModules(files: ChangedFile[]): Promise<string[]> {
        // Implementation would analyze module structure
        return [];
    }

    private async extractAddedFunctions(diff: GitDiff): Promise<string[]> {
        // Implementation would parse diff for added functions
        return [];
    }

    private async extractModifiedFunctions(diff: GitDiff): Promise<string[]> {
        // Implementation would parse diff for modified functions
        return [];
    }

    private async extractDeletedFunctions(diff: GitDiff): Promise<string[]> {
        // Implementation would parse diff for deleted functions
        return [];
    }

    private async extractAddedClasses(diff: GitDiff): Promise<string[]> {
        // Implementation would parse diff for added classes
        return [];
    }

    private async extractModifiedClasses(diff: GitDiff): Promise<string[]> {
        // Implementation would parse diff for modified classes
        return [];
    }

    private async extractDeletedClasses(diff: GitDiff): Promise<string[]> {
        // Implementation would parse diff for deleted classes
        return [];
    }

    private async extractInterfaceChanges(diff: GitDiff): Promise<string[]> {
        // Implementation would parse diff for interface changes
        return [];
    }

    private async extractImportChanges(diff: GitDiff): Promise<string[]> {
        // Implementation would parse diff for import changes
        return [];
    }

    private async calculateComplexityChange(diff: GitDiff): Promise<number> {
        // Implementation would calculate complexity change
        return 0;
    }

    // Additional helper methods would be implemented here...
    private async findIndirectlyImpactedFiles(files: ChangedFile[]): Promise<string[]> { return []; }
    private async findImpactedTests(files: ChangedFile[]): Promise<string[]> { return []; }
    private async identifyAPIChanges(commit: GitCommit): Promise<string[]> { return []; }
    private async identifyDatabaseChanges(files: ChangedFile[]): Promise<string[]> { return []; }
    private async identifyConfigurationChanges(files: ChangedFile[]): Promise<string[]> { return []; }
    private assessChangeRisk(files: ChangedFile[]): RiskLevel { return 'low'; }
    private extractFunctionDefinitions(semanticMap: SemanticMap): Map<string, any> { return new Map(); }
    private extractClassDefinitions(semanticMap: SemanticMap): Map<string, any> { return new Map(); }
    private extractInterfaceDefinitions(semanticMap: SemanticMap): Map<string, any> { return new Map(); }
    private extractTypeDefinitions(semanticMap: SemanticMap): Map<string, any> { return new Map(); }
    private extractConstantDefinitions(semanticMap: SemanticMap): Map<string, any> { return new Map(); }
    private extractImportStatements(semanticMap: SemanticMap): any[] { return []; }
    private extractExportStatements(semanticMap: SemanticMap): any[] { return []; }
    private extractCallChains(semanticMap: SemanticMap): any[] { return []; }
    private buildInheritanceTree(semanticMap: SemanticMap): any { return {}; }
    private buildDependencyGraph(semanticMap: SemanticMap): any { return {}; }
    private identifyUsagePatterns(semanticMap: SemanticMap): any[] { return []; }
    private mapToChangeIntent(changeType: string): any { return {}; }
    private async identifyAffectedFeatures(changes: CommitSemanticAnalysis): Promise<string[]> { return []; }
    private async identifyDesignPatterns(changes: CommitSemanticAnalysis): Promise<any[]> { return []; }
    private async extractArchitecturalDecisions(commit: GitCommit, changes: CommitSemanticAnalysis): Promise<any[]> { return []; }
    private identifyRefactoringType(changes: CommitSemanticAnalysis): string | null { return null; }
    private async identifyPatternChanges(changes: CommitSemanticAnalysis): Promise<any[]> { return []; }
    private async identifyDependencyChanges(changes: CommitSemanticAnalysis): Promise<any[]> { return []; }
    private async identifyInterfaceChanges(changes: CommitSemanticAnalysis): Promise<any[]> { return []; }
    private async assessPerformanceImplications(changes: CommitSemanticAnalysis): Promise<any[]> { return []; }
    private async assessSecurityImplications(changes: CommitSemanticAnalysis): Promise<any[]> { return []; }
    private async assessMaintainabilityImpact(changes: CommitSemanticAnalysis): Promise<number> { return 0; }
    private async assessScalabilityImpact(changes: CommitSemanticAnalysis): Promise<number> { return 0; }
    private extractInheritanceRelationships(semanticMap: SemanticMap): any[] { return []; }
    private extractCompositionRelationships(semanticMap: SemanticMap): any[] { return []; }
    private extractUsageRelationships(semanticMap: SemanticMap): any[] { return []; }
    private async buildTemporalRelationships(semanticMap: SemanticMap): Promise<any[]> { return []; }
    private determineChangeType(messageAnalysis: any, fileAnalysis: any, codeAnalysis: any): string { return 'other'; }
    private determineSemanticScope(fileAnalysis: any, codeAnalysis: any): string { return 'module'; }
    private async identifyArchitecturalChanges(commit: GitCommit): Promise<any[]> { return []; }
    private async buildFileChanges(commit: GitCommit): Promise<any[]> { return []; }
}

// Git Repository wrapper class
class GitRepository {
    constructor(private repoPath: string) {}

    async getCommitRange(fromCommit: string, toCommit: string): Promise<GitCommit[]> {
        try {
            const output = execSync(`git -C "${this.repoPath}" log --oneline --pretty=format:"%H|%an|%ae|%ct|%s|%D" ${fromCommit}..${toCommit}`, 
                { encoding: 'utf8' });
            return this.parseGitLogOutput(output);
        } catch (error) {
            console.warn('Failed to get commit range:', error);
            return [];
        }
    }

    async getLatestCommit(): Promise<GitCommit | null> {
        try {
            const output = execSync(`git -C "${this.repoPath}" log -1 --pretty=format:"%H|%an|%ae|%ct|%s|%D"`, 
                { encoding: 'utf8' });
            const commits = this.parseGitLogOutput(output);
            return commits.length > 0 ? commits[0] : null;
        } catch (error) {
            console.warn('Failed to get latest commit:', error);
            return null;
        }
    }

    async getRecentCommits(count: number = 10): Promise<GitCommit[]> {
        try {
            const output = execSync(`git -C "${this.repoPath}" log -${count} --pretty=format:"%H|%an|%ae|%ct|%s|%D"`, 
                { encoding: 'utf8' });
            return this.parseGitLogOutput(output);
        } catch (error) {
            console.warn('Failed to get recent commits:', error);
            return [];
        }
    }

    async getBranchCommits(branchName: string): Promise<GitCommit[]> {
        try {
            const output = execSync(`git -C "${this.repoPath}" log ${branchName} --pretty=format:"%H|%an|%ae|%ct|%s|%D"`, 
                { encoding: 'utf8' });
            return this.parseGitLogOutput(output);
        } catch (error) {
            console.warn('Failed to get branch commits:', error);
            return [];
        }
    }

    private parseGitLogOutput(output: string): GitCommit[] {
        if (!output.trim()) {return [];}
        
        return output.trim().split('\n').map(line => {
            const [hash, author, email, timestamp, message, refs] = line.split('|');
            const branch = this.extractBranchFromRefs(refs);
            const parents = this.getCommitParents(hash);
            
            return {
                hash,
                author,
                email,
                timestamp: parseInt(timestamp),
                message: message || '',
                branch: branch || 'main',
                parents
            };
        });
    }

    private extractBranchFromRefs(refs: string): string {
        if (!refs) {return 'main';}
        const branchMatch = refs.match(/origin\/(\w+)/);
        return branchMatch ? branchMatch[1] : 'main';
    }

    private getCommitParents(hash: string): string[] {
        try {
            const output = execSync(`git -C "${this.repoPath}" log --pretty=%P -n 1 ${hash}`, 
                { encoding: 'utf8' });
            return output.trim().split(' ').filter(p => p.length > 0);
        } catch (error) {
            return [];
        }
    }

    async getFilesAtCommit(commitHash: string): Promise<Map<string, any>> {
        // Implementation would get file contents at specific commit
        return new Map();
    }

    async getChangedFiles(commitHash: string): Promise<ChangedFile[]> {
        // Implementation would get changed files in commit
        return [];
    }

    async getCommitDiff(commitHash: string): Promise<GitDiff> {
        // Implementation would get commit diff
        return { additions: 0, deletions: 0, files: [] };
    }
}

// Type definitions
export interface GitCommit {
    hash: string;
    author: string;
    email: string;
    timestamp: number;
    message: string;
    branch: string;
    parents: string[];
}

interface CommitSemanticAnalysis {
    messageAnalysis: CommitMessageAnalysis;
    fileAnalysis: FileAnalysis;
    codeAnalysis: CodeChangeAnalysis;
    impactAnalysis: ChangeImpactAnalysis;
    changeType: string;
    semanticScope: string;
    architecturalChanges: any[];
}

interface CommitMessageAnalysis {
    intent: string;
    type: CommitType;
    scope: string | null;
    breakingChange: boolean;
    issueReferences: string[];
    keywords: string[];
}

interface FileAnalysis {
    added: ChangedFile[];
    modified: ChangedFile[];
    deleted: ChangedFile[];
    renamed: ChangedFile[];
    fileTypes: FileTypeCategories;
    impactedModules: string[];
    testFiles: ChangedFile[];
    configFiles: ChangedFile[];
}

interface CodeChangeAnalysis {
    functionsAdded: string[];
    functionsModified: string[];
    functionsDeleted: string[];
    classesAdded: string[];
    classesModified: string[];
    classesDeleted: string[];
    interfacesChanged: string[];
    importsChanged: string[];
    complexityChange: number;
    linesAdded: number;
    linesDeleted: number;
}

interface ChangeImpactAnalysis {
    directImpact: string[];
    indirectImpact: string[];
    testImpact: string[];
    apiChanges: string[];
    databaseChanges: string[];
    configurationChanges: string[];
    riskLevel: RiskLevel;
}

interface ChangedFile {
    path: string;
    status: 'A' | 'M' | 'D' | 'R';
    additions?: number;
    deletions?: number;
}

interface FileTypeCategories {
    source: ChangedFile[];
    test: ChangedFile[];
    config: ChangedFile[];
    documentation: ChangedFile[];
    assets: ChangedFile[];
}

interface GitDiff {
    additions: number;
    deletions: number;
    files: ChangedFile[];
}

type CommitType = 
    | 'feature' 
    | 'bugfix' 
    | 'refactor' 
    | 'documentation' 
    | 'test' 
    | 'style' 
    | 'maintenance' 
    | 'performance' 
    | 'security' 
    | 'other';

type RiskLevel = 'low' | 'medium' | 'high' | 'critical';

export default GitCheckpointBridge;
