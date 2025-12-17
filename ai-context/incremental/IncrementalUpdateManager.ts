/**
 * Incremental Update Manager - Efficient incremental updates for semantic context
 * 
 * This system tracks file changes and updates only the affected parts of the
 * semantic context, dramatically improving performance for large codebases.
 */

import { ContextCache } from '../cache/ContextCache';
import { Symbol } from '../parsers/LanguageParser';
import { parserRegistry } from '../parsers/ParserRegistry';

export class IncrementalUpdateManager {
    private cache: ContextCache;
    private fileWatchers: Map<string, FileWatcher> = new Map();
    private dependencyGraph: DependencyGraph = new DependencyGraph();
    private updateQueue: UpdateTask[] = [];
    private isProcessing = false;
    private config: IncrementalConfig;

    constructor(cache: ContextCache, config: IncrementalConfig = {}) {
        this.cache = cache;
        this.config = {
            batchSize: config.batchSize || 10,
            debounceMs: config.debounceMs || 100,
            maxRetries: config.maxRetries || 3,
            enableDependencyTracking: config.enableDependencyTracking !== false
        };
    }

    /**
     * Start watching files for changes
     */
    async startWatching(workspacePath: string, filePatterns: string[] = ['**/*.ts', '**/*.js', '**/*.py', '**/*.rs']): Promise<void> {
        // In a real implementation, this would use fs.watch or chokidar
        console.log(`Starting file watching for ${workspacePath} with patterns:`, filePatterns);
        
        // Simulate file watching setup
        for (const pattern of filePatterns) {
            const watcher = new FileWatcher(pattern, this.handleFileChange.bind(this));
            this.fileWatchers.set(pattern, watcher);
        }
    }

    /**
     * Stop watching files
     */
    async stopWatching(): Promise<void> {
        for (const watcher of this.fileWatchers.values()) {
            await watcher.stop();
        }
        this.fileWatchers.clear();
    }

    /**
     * Handle file change events
     */
    private async handleFileChange(event: FileChangeEvent): Promise<void> {
        // Add to update queue
        const task: UpdateTask = {
            id: `${event.filePath}-${Date.now()}`,
            filePath: event.filePath,
            changeType: event.changeType,
            timestamp: Date.now(),
            retries: 0,
            dependencies: []
        };

        this.updateQueue.push(task);

        // Debounce processing
        setTimeout(() => {
            if (!this.isProcessing) {
                this.processUpdateQueue();
            }
        }, this.config.debounceMs);
    }

    /**
     * Process queued updates
     */
    private async processUpdateQueue(): Promise<void> {
        if (this.isProcessing || this.updateQueue.length === 0) {return;}

        this.isProcessing = true;

        try {
            // Group updates by file and take the most recent
            const latestUpdates = this.consolidateUpdates();
            
            // Process in batches
            const batches = this.createBatches(latestUpdates, this.config.batchSize ?? 10);
            
            for (const batch of batches) {
                await this.processBatch(batch);
            }
        } finally {
            this.isProcessing = false;
            
            // Process any new updates that arrived while processing
            if (this.updateQueue.length > 0) {
                setTimeout(() => this.processUpdateQueue(), this.config.debounceMs);
            }
        }
    }

    /**
     * Consolidate updates to keep only the latest for each file
     */
    private consolidateUpdates(): UpdateTask[] {
        const fileMap = new Map<string, UpdateTask>();
        
        for (const task of this.updateQueue) {
            const existing = fileMap.get(task.filePath);
            if (!existing || task.timestamp > existing.timestamp) {
                fileMap.set(task.filePath, task);
            }
        }
        
        // Clear processed updates
        this.updateQueue = [];
        
        return Array.from(fileMap.values());
    }

    /**
     * Create batches for parallel processing
     */
    private createBatches<T>(items: T[], batchSize: number): T[][] {
        const batches: T[][] = [];
        for (let i = 0; i < items.length; i += batchSize) {
            batches.push(items.slice(i, i + batchSize));
        }
        return batches;
    }

    /**
     * Process a batch of updates
     */
    private async processBatch(batch: UpdateTask[]): Promise<void> {
        const batchPromises = batch.map(task => this.processUpdate(task));
        const results = await Promise.allSettled(batchPromises);
        
        // Handle failed updates
        for (let i = 0; i < results.length; i++) {
            const result = results[i];
            if (result.status === 'rejected') {
                const task = batch[i];
                await this.handleUpdateFailure(task, result.reason);
            }
        }
    }

    /**
     * Process a single update
     */
    private async processUpdate(task: UpdateTask): Promise<void> {
        try {
            switch (task.changeType) {
                case 'created':
                case 'modified':
                    await this.handleFileModified(task);
                    break;
                case 'deleted':
                    await this.handleFileDeleted(task);
                    break;
                case 'renamed':
                    await this.handleFileRenamed(task);
                    break;
            }
        } catch (error) {
            throw new Error(`Failed to process update for ${task.filePath}: ${error}`);
        }
    }

    /**
     * Handle file modification
     */
    private async handleFileModified(task: UpdateTask): Promise<void> {
        const filePath = task.filePath;
        
        // Parse the file to get new semantic information
        const fileContent = await this.readFile(filePath);
        const parseResult = await parserRegistry.parseFile(fileContent, filePath);
        
        if (!parseResult.success) {
            throw new Error(`Failed to parse ${filePath}: ${parseResult.diagnostics?.[0]?.message}`);
        }

        // Get previous semantic context
        const previousContext = await this.getPreviousSemanticContext(filePath);
        
        // Calculate changes
        const changes = this.calculateSemanticChanges(previousContext, parseResult.symbols);
        
        // Update cache
        await this.cache.getOrComputeSemanticContext(
            `semantic:${filePath}`,
            filePath,
            async () => ({
                symbols: parseResult.symbols,
                callGraph: parseResult.callGraph,
                dependencies: parseResult.dependencies,
                lastUpdated: Date.now()
            })
        );

        // Update dependency graph
        if (this.config.enableDependencyTracking) {
            await this.updateDependencyGraph(filePath, parseResult.dependencies);
        }

        // Propagate changes to dependent files
        if (changes.hasSignificantChanges) {
            await this.propagateChanges(filePath, changes);
        }

        console.log(`Updated semantic context for ${filePath}`, {
            symbolsAdded: changes.added.length,
            symbolsModified: changes.modified.length,
            symbolsRemoved: changes.removed.length
        });
    }

    /**
     * Handle file deletion
     */
    private async handleFileDeleted(task: UpdateTask): Promise<void> {
        const filePath = task.filePath;
        
        // Remove from cache
        this.cache.invalidateFile(filePath);
        
        // Remove from dependency graph
        this.dependencyGraph.removeFile(filePath);
        
        // Find files that depend on this deleted file
        const dependentFiles = this.dependencyGraph.getDependentFiles(filePath);
        
        // Queue updates for dependent files
        for (const dependentFile of dependentFiles) {
            this.updateQueue.push({
                id: `dependent-${dependentFile}-${Date.now()}`,
                filePath: dependentFile,
                changeType: 'modified', // Trigger reanalysis
                timestamp: Date.now(),
                retries: 0,
                dependencies: []
            });
        }

        console.log(`Removed ${filePath} and queued ${dependentFiles.length} dependent files for update`);
    }

    /**
     * Handle file rename
     */
    private async handleFileRenamed(task: UpdateTask): Promise<void> {
        if (!task.oldFilePath) {
            throw new Error('Old file path required for rename operation');
        }

        // Remove old file
        await this.handleFileDeleted({ ...task, filePath: task.oldFilePath });
        
        // Add new file
        await this.handleFileModified(task);

        console.log(`Renamed ${task.oldFilePath} to ${task.filePath}`);
    }

    /**
     * Calculate semantic changes between old and new symbols
     */
    private calculateSemanticChanges(
        previousSymbols: Symbol[] = [],
        newSymbols: Symbol[] = []
    ): SemanticChanges {
        const oldSymbolMap = new Map(previousSymbols.map(s => [s.name, s]));
        const newSymbolMap = new Map(newSymbols.map(s => [s.name, s]));

        const added: Symbol[] = [];
        const modified: Symbol[] = [];
        const removed: Symbol[] = [];

        // Find added and modified symbols
        for (const [name, newSymbol] of newSymbolMap) {
            const oldSymbol = oldSymbolMap.get(name);
            if (!oldSymbol) {
                added.push(newSymbol);
            } else if (this.hasSymbolChanged(oldSymbol, newSymbol)) {
                modified.push(newSymbol);
            }
        }

        // Find removed symbols
        for (const [name, oldSymbol] of oldSymbolMap) {
            if (!newSymbolMap.has(name)) {
                removed.push(oldSymbol);
            }
        }

        const hasSignificantChanges = added.length > 0 || modified.length > 0 || removed.length > 0;

        return {
            added,
            modified,
            removed,
            hasSignificantChanges,
            changeScore: this.calculateChangeScore(added, modified, removed)
        };
    }

    /**
     * Check if a symbol has changed significantly
     */
    private hasSymbolChanged(oldSymbol: Symbol, newSymbol: Symbol): boolean {
        // Compare key properties
        if (oldSymbol.type !== newSymbol.type) {return true;}
        
        // Type-specific comparisons
        switch (oldSymbol.type) {
            case 'function':
                const oldFunc = oldSymbol as any;
                const newFunc = newSymbol as any;
                return (
                    oldFunc.returnType !== newFunc.returnType ||
                    JSON.stringify(oldFunc.parameters) !== JSON.stringify(newFunc.parameters) ||
                    oldFunc.visibility !== newFunc.visibility
                );
                
            case 'class':
                const oldClass = oldSymbol as any;
                const newClass = newSymbol as any;
                return (
                    oldClass.superClass !== newClass.superClass ||
                    JSON.stringify(oldClass.interfaces) !== JSON.stringify(newClass.interfaces) ||
                    JSON.stringify(oldClass.methods?.map((m: any) => m.name)) !== 
                    JSON.stringify(newClass.methods?.map((m: any) => m.name))
                );
                
            default:
                return false;
        }
    }

    /**
     * Calculate a score representing the magnitude of changes
     */
    private calculateChangeScore(added: Symbol[], modified: Symbol[], removed: Symbol[]): number {
        // Weight different types of changes
        const addedScore = added.length * 1.0;
        const modifiedScore = modified.length * 0.7;
        const removedScore = removed.length * 1.2; // Removals are more impactful
        
        return addedScore + modifiedScore + removedScore;
    }

    /**
     * Propagate changes to dependent files
     */
    private async propagateChanges(filePath: string, changes: SemanticChanges): Promise<void> {
        if (!this.config.enableDependencyTracking) {return;}

        const dependentFiles = this.dependencyGraph.getDependentFiles(filePath);
        
        // Only propagate if changes are significant enough
        if (changes.changeScore > 2.0) {
            for (const dependentFile of dependentFiles) {
                // Invalidate cache for dependent file
                this.cache.invalidateFile(dependentFile);
                
                // Queue for re-analysis
                this.updateQueue.push({
                    id: `propagated-${dependentFile}-${Date.now()}`,
                    filePath: dependentFile,
                    changeType: 'modified',
                    timestamp: Date.now(),
                    retries: 0,
                    dependencies: [filePath]
                });
            }
        }
    }

    /**
     * Update dependency graph with new dependencies
     */
    private async updateDependencyGraph(filePath: string, dependencies: any[]): Promise<void> {
        this.dependencyGraph.updateDependencies(filePath, dependencies);
    }

    /**
     * Handle update failure with retry logic
     */
    private async handleUpdateFailure(task: UpdateTask, error: any): Promise<void> {
        task.retries++;
        
        if (task.retries < (this.config.maxRetries ?? 3)) {
            // Add back to queue for retry
            setTimeout(() => {
                this.updateQueue.push(task);
            }, Math.pow(2, task.retries) * 1000); // Exponential backoff
            
            console.warn(`Update failed for ${task.filePath}, retrying (${task.retries}/${this.config.maxRetries ?? 3}):`, error);
        } else {
            console.error(`Update permanently failed for ${task.filePath} after ${task.retries} retries:`, error);
        }
    }

    /**
     * Get statistics about incremental updates
     */
    getUpdateStats(): UpdateStats {
        return {
            queueSize: this.updateQueue.length,
            isProcessing: this.isProcessing,
            watchedFiles: this.fileWatchers.size,
            dependencyGraphSize: this.dependencyGraph.getSize(),
            totalUpdatesProcessed: 0, // Would track this in real implementation
            averageUpdateTime: 0, // Would track this in real implementation
            failureRate: 0 // Would track this in real implementation
        };
    }

    // Helper methods

    private async readFile(filePath: string): Promise<string> {
        // In a real implementation, this would read the file from disk
        return `// Mock file content for ${filePath}`;
    }

    private async getPreviousSemanticContext(filePath: string): Promise<Symbol[]> {
        // Try to get from cache
        try {
            const cached = await this.cache.getOrComputeSemanticContext(
                `semantic:${filePath}`,
                filePath,
                async () => ({ symbols: [] })
            );
            return (cached as any).symbols || [];
        } catch {
            return [];
        }
    }
}

// Supporting classes and interfaces

class FileWatcher {
    constructor(
        private pattern: string,
        private onChange: (event: FileChangeEvent) => void
    ) {
        // Initialize file watching
    }

    async stop(): Promise<void> {
        // Stop file watching
    }
}

class DependencyGraph {
    private dependencies: Map<string, Set<string>> = new Map();
    private dependents: Map<string, Set<string>> = new Map();

    updateDependencies(filePath: string, dependencies: any[]): void {
        // Clear existing dependencies
        const oldDeps = this.dependencies.get(filePath) || new Set();
        for (const oldDep of oldDeps) {
            const dependents = this.dependents.get(oldDep);
            if (dependents) {
                dependents.delete(filePath);
            }
        }

        // Add new dependencies
        const newDeps = new Set<string>();
        for (const dep of dependencies) {
            const depPath = dep.target; // Assuming target is the file path
            newDeps.add(depPath);
            
            if (!this.dependents.has(depPath)) {
                this.dependents.set(depPath, new Set());
            }
            this.dependents.get(depPath)!.add(filePath);
        }
        
        this.dependencies.set(filePath, newDeps);
    }

    getDependentFiles(filePath: string): string[] {
        const dependents = this.dependents.get(filePath);
        return dependents ? Array.from(dependents) : [];
    }

    removeFile(filePath: string): void {
        // Remove as dependency
        const deps = this.dependencies.get(filePath) || new Set();
        for (const dep of deps) {
            const dependents = this.dependents.get(dep);
            if (dependents) {
                dependents.delete(filePath);
            }
        }
        this.dependencies.delete(filePath);

        // Remove as dependent
        this.dependents.delete(filePath);
    }

    getSize(): number {
        return this.dependencies.size;
    }
}

// Interfaces

export interface IncrementalConfig {
    batchSize?: number;
    debounceMs?: number;
    maxRetries?: number;
    enableDependencyTracking?: boolean;
}

interface UpdateTask {
    id: string;
    filePath: string;
    oldFilePath?: string; // For rename operations
    changeType: 'created' | 'modified' | 'deleted' | 'renamed';
    timestamp: number;
    retries: number;
    dependencies: string[];
}

interface FileChangeEvent {
    filePath: string;
    changeType: 'created' | 'modified' | 'deleted' | 'renamed';
    oldFilePath?: string;
}

interface SemanticChanges {
    added: Symbol[];
    modified: Symbol[];
    removed: Symbol[];
    hasSignificantChanges: boolean;
    changeScore: number;
}

interface UpdateStats {
    queueSize: number;
    isProcessing: boolean;
    watchedFiles: number;
    dependencyGraphSize: number;
    totalUpdatesProcessed: number;
    averageUpdateTime: number;
    failureRate: number;
}
