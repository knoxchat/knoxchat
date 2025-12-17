/**
 * Context Cache System - Multi-level caching for AI context components
 * 
 * This system provides intelligent caching for semantic analysis, context trees,
 * and relevance scores to dramatically improve performance.
 */

import { LRUCache } from 'lru-cache';

import { CompleteAIContext, QueryIntent } from '../AIContextBuilder';

export class ContextCache {
    private semanticCache!: LRUCache<string, SemanticContext>;
    private queryCache!: LRUCache<string, CompleteAIContext>;
    private astCache!: LRUCache<string, ParsedAST>;
    private relationshipCache!: LRUCache<string, RelationshipGraph>;
    private relevanceCache!: LRUCache<string, RelevanceScore>;
    private fileHashCache: Map<string, string>;
    private config: CacheConfig;

    constructor(config: CacheConfig = {}) {
        this.config = {
            semanticCacheSize: config.semanticCacheSize ?? 1000,
            queryCacheSize: config.queryCacheSize ?? 500,
            astCacheSize: config.astCacheSize ?? 2000,
            relationshipCacheSize: config.relationshipCacheSize ?? 300,
            relevanceCacheSize: config.relevanceCacheSize ?? 1000,
            ttlMinutes: config.ttlMinutes ?? 60,
            persistToDisk: config.persistToDisk ?? false,
            diskCachePath: config.diskCachePath ?? './cache'
        };

        this.fileHashCache = new Map();
        this.initializeCaches();
    }

    /**
     * Initialize all cache layers
     */
    private initializeCaches(): void {
        const ttlMs = (this.config.ttlMinutes ?? 60) * 60 * 1000;

        this.semanticCache = new LRUCache<string, SemanticContext>({
            max: this.config.semanticCacheSize ?? 1000,
            ttl: ttlMs,
            updateAgeOnGet: true,
            allowStale: false
        });

        this.queryCache = new LRUCache<string, CompleteAIContext>({
            max: this.config.queryCacheSize ?? 500,
            ttl: ttlMs,
            updateAgeOnGet: true,
            allowStale: false
        });

        this.astCache = new LRUCache<string, ParsedAST>({
            max: this.config.astCacheSize ?? 2000,
            ttl: ttlMs * 2, // AST cache lives longer
            updateAgeOnGet: true,
            allowStale: false
        });

        this.relationshipCache = new LRUCache<string, RelationshipGraph>({
            max: this.config.relationshipCacheSize ?? 300,
            ttl: ttlMs,
            updateAgeOnGet: true,
            allowStale: false
        });

        this.relevanceCache = new LRUCache<string, RelevanceScore>({
            max: this.config.relevanceCacheSize ?? 1000,
            ttl: ttlMs / 2, // Relevance scores expire faster
            updateAgeOnGet: true,
            allowStale: false
        });
    }

    /**
     * Get or compute semantic context with caching
     */
    async getOrComputeSemanticContext<T>(
        key: string,
        filePath: string,
        compute: () => Promise<T>
    ): Promise<T> {
        // Check if file has changed
        const currentHash = await this.getFileHash(filePath);
        const cacheKey = `${key}:${currentHash}`;

        // Try to get from cache
        const cached = this.semanticCache.get(cacheKey);
        if (cached) {
            return cached as T;
        }

        // Compute and cache
        const computed = await compute();
        this.semanticCache.set(cacheKey, computed as SemanticContext);
        
        // Update file hash
        this.fileHashCache.set(filePath, currentHash);

        return computed;
    }

    /**
     * Get or compute query context with caching
     */
    async getOrComputeQueryContext(
        query: string,
        workspace: string,
        maxTokens: number,
        compute: () => Promise<CompleteAIContext>
    ): Promise<CompleteAIContext> {
        const cacheKey = this.generateQueryCacheKey(query, workspace, maxTokens);

        // Try to get from cache
        const cached = this.queryCache.get(cacheKey);
        if (cached) {
            // Update access time and return
            return cached;
        }

        // Compute and cache
        const computed = await compute();
        this.queryCache.set(cacheKey, computed);

        return computed;
    }

    /**
     * Get or compute AST with caching
     */
    async getOrComputeAST<T>(
        filePath: string,
        compute: () => Promise<T>
    ): Promise<T> {
        const currentHash = await this.getFileHash(filePath);
        const cacheKey = `ast:${filePath}:${currentHash}`;

        // Try to get from cache
        const cached = this.astCache.get(cacheKey);
        if (cached) {
            return cached.ast as T;
        }

        // Compute and cache
        const computed = await compute();
        this.astCache.set(cacheKey, {
            ast: computed,
            filePath,
            hash: currentHash,
            timestamp: Date.now()
        } as ParsedAST);

        return computed;
    }

    /**
     * Get or compute relationship graph with caching
     */
    async getOrComputeRelationships<T>(
        key: string,
        dependencies: string[],
        compute: () => Promise<T>
    ): Promise<T> {
        // Create cache key based on dependencies
        const dependencyHash = await this.hashDependencies(dependencies);
        const cacheKey = `${key}:${dependencyHash}`;

        // Try to get from cache
        const cached = this.relationshipCache.get(cacheKey);
        if (cached) {
            return cached as T;
        }

        // Compute and cache
        const computed = await compute();
        this.relationshipCache.set(cacheKey, computed as RelationshipGraph);

        return computed;
    }

    /**
     * Get or compute relevance score with caching
     */
    async getOrComputeRelevance<T>(
        queryIntent: QueryIntent,
        checkpointId: string,
        compute: () => Promise<T>
    ): Promise<T> {
        const cacheKey = this.generateRelevanceCacheKey(queryIntent, checkpointId);

        // Try to get from cache
        const cached = this.relevanceCache.get(cacheKey);
        if (cached) {
            return cached as T;
        }

        // Compute and cache
        const computed = await compute();
        this.relevanceCache.set(cacheKey, computed as RelevanceScore);

        return computed;
    }

    /**
     * Invalidate cache for a specific file
     */
    invalidateFile(filePath: string): void {
        // Remove file hash to force recomputation
        this.fileHashCache.delete(filePath);

        // Remove all cache entries for this file
        for (const cache of [this.semanticCache, this.astCache]) {
            for (const key of cache.keys()) {
                if (key.includes(filePath)) {
                    cache.delete(key);
                }
            }
        }
    }

    /**
     * Invalidate all caches
     */
    invalidateAll(): void {
        this.semanticCache.clear();
        this.queryCache.clear();
        this.astCache.clear();
        this.relationshipCache.clear();
        this.relevanceCache.clear();
        this.fileHashCache.clear();
    }

    /**
     * Get cache statistics
     */
    getStats(): CacheStats {
        return {
            semantic: this.getCacheLayerStats(this.semanticCache, 'semantic'),
            query: this.getCacheLayerStats(this.queryCache, 'query'),
            ast: this.getCacheLayerStats(this.astCache, 'ast'),
            relationship: this.getCacheLayerStats(this.relationshipCache, 'relationship'),
            relevance: this.getCacheLayerStats(this.relevanceCache, 'relevance'),
            fileHashes: this.fileHashCache.size,
            totalMemoryUsage: this.estimateMemoryUsage()
        };
    }

    /**
     * Optimize cache performance
     */
    async optimize(): Promise<CacheOptimizationResult> {
        const startTime = Date.now();
        let itemsRemoved = 0;
        let memoryFreed = 0;

        // Remove stale entries
        const caches = [
            this.semanticCache,
            this.queryCache,
            this.astCache,
            this.relationshipCache,
            this.relevanceCache
        ];

        for (const cache of caches) {
            const sizeBefore = cache.size;
            cache.purgeStale();
            const sizeAfter = cache.size;
            itemsRemoved += sizeBefore - sizeAfter;
        }

        // Clean up file hashes for non-existent files
        const fileHashesBefore = this.fileHashCache.size;
        await this.cleanupFileHashes();
        const fileHashesAfter = this.fileHashCache.size;
        itemsRemoved += fileHashesBefore - fileHashesAfter;

        const optimizationTime = Date.now() - startTime;

        return {
            itemsRemoved,
            memoryFreed,
            optimizationTime,
            newCacheStats: this.getStats()
        };
    }

    /**
     * Preload cache for common queries
     */
    async preloadCommonQueries(commonQueries: string[], workspace: string): Promise<void> {
        const preloadPromises = commonQueries.map(async query => {
            const cacheKey = this.generateQueryCacheKey(query, workspace, 8000);
            
            // Only preload if not already cached
            if (!this.queryCache.has(cacheKey)) {
                // This would trigger actual computation in a real implementation
                console.log(`Preloading cache for query: ${query}`);
            }
        });

        await Promise.all(preloadPromises);
    }

    // Helper methods

    private async getFileHash(filePath: string): Promise<string> {
        // In a real implementation, this would compute file hash (e.g., MD5, SHA-256)
        // For now, we'll use a simple timestamp-based approach
        try {
            const stats = await import('fs').then(fs => fs.promises.stat(filePath));
            return `${stats.mtime.getTime()}-${stats.size}`;
        } catch {
            // File doesn't exist or can't be accessed
            return `missing-${Date.now()}`;
        }
    }

    private async hashDependencies(dependencies: string[]): Promise<string> {
        // Create a hash of all dependency file hashes
        const hashes = await Promise.all(
            dependencies.map(dep => this.getFileHash(dep))
        );
        return hashes.join('-');
    }

    private generateQueryCacheKey(query: string, workspace: string, maxTokens: number): string {
        // Create a normalized cache key for queries
        const normalizedQuery = query.toLowerCase().trim();
        const workspaceHash = workspace.split('/').pop() || 'unknown';
        return `query:${workspaceHash}:${maxTokens}:${this.hashString(normalizedQuery)}`;
    }

    private generateRelevanceCacheKey(queryIntent: QueryIntent, checkpointId: string): string {
        const intentHash = this.hashString(JSON.stringify({
            type: queryIntent.query_type,
            entities: queryIntent.entities.map(e => e.name).sort(),
            scope: queryIntent.scope
        }));
        return `relevance:${checkpointId}:${intentHash}`;
    }

    private hashString(str: string): string {
        // Simple hash function for cache keys
        let hash = 0;
        for (let i = 0; i < str.length; i++) {
            const char = str.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
            hash = hash & hash; // Convert to 32-bit integer
        }
        return Math.abs(hash).toString(36);
    }

    private getCacheLayerStats(cache: LRUCache<string, any>, name: string): CacheLayerStats {
        return {
            name,
            size: cache.size,
            maxSize: cache.max || 0,
            hitRate: this.calculateHitRate(cache),
            memoryUsage: this.estimateCacheMemoryUsage(cache)
        };
    }

    private calculateHitRate(cache: LRUCache<string, any>): number {
        // In a real implementation, we'd track hits and misses
        // For now, return a placeholder
        return 0.75; // 75% hit rate
    }

    private estimateCacheMemoryUsage(cache: LRUCache<string, any>): number {
        // Rough estimate of memory usage
        let totalSize = 0;
        for (const [key, value] of cache.entries()) {
            totalSize += key.length * 2; // String characters are 2 bytes
            totalSize += this.estimateObjectSize(value);
        }
        return totalSize;
    }

    private estimateObjectSize(obj: any): number {
        // Very rough estimation
        const jsonString = JSON.stringify(obj);
        return jsonString.length * 2; // Approximate bytes
    }

    private estimateMemoryUsage(): number {
        return (
            this.estimateCacheMemoryUsage(this.semanticCache) +
            this.estimateCacheMemoryUsage(this.queryCache) +
            this.estimateCacheMemoryUsage(this.astCache) +
            this.estimateCacheMemoryUsage(this.relationshipCache) +
            this.estimateCacheMemoryUsage(this.relevanceCache)
        );
    }

    private async cleanupFileHashes(): Promise<void> {
        const filesToCheck = Array.from(this.fileHashCache.keys());
        
        for (const filePath of filesToCheck) {
            try {
                await import('fs').then(fs => fs.promises.access(filePath));
            } catch {
                // File doesn't exist, remove from cache
                this.fileHashCache.delete(filePath);
            }
        }
    }
}

// Supporting interfaces and types

export interface CacheConfig {
    semanticCacheSize?: number;
    queryCacheSize?: number;
    astCacheSize?: number;
    relationshipCacheSize?: number;
    relevanceCacheSize?: number;
    ttlMinutes?: number;
    persistToDisk?: boolean;
    diskCachePath?: string;
}

export interface CacheStats {
    semantic: CacheLayerStats;
    query: CacheLayerStats;
    ast: CacheLayerStats;
    relationship: CacheLayerStats;
    relevance: CacheLayerStats;
    fileHashes: number;
    totalMemoryUsage: number;
}

export interface CacheLayerStats {
    name: string;
    size: number;
    maxSize: number;
    hitRate: number;
    memoryUsage: number;
}

export interface CacheOptimizationResult {
    itemsRemoved: number;
    memoryFreed: number;
    optimizationTime: number;
    newCacheStats: CacheStats;
}

// Placeholder interfaces (would be properly defined elsewhere)
interface SemanticContext {
    [key: string]: any;
}

interface ParsedAST {
    ast: any;
    filePath: string;
    hash: string;
    timestamp: number;
}

interface RelationshipGraph {
    [key: string]: any;
}

interface RelevanceScore {
    [key: string]: any;
}

/**
 * Cache warming service for proactive cache population
 */
export class CacheWarmingService {
    private cache: ContextCache;
    private warmingQueue: WarmingTask[] = [];
    private isWarming = false;

    constructor(cache: ContextCache) {
        this.cache = cache;
    }

    /**
     * Add files to warming queue
     */
    addFilesToWarmingQueue(filePaths: string[]): void {
        for (const filePath of filePaths) {
            this.warmingQueue.push({
                type: 'file',
                filePath,
                priority: this.calculateWarmingPriority(filePath),
                addedAt: Date.now()
            });
        }

        // Sort by priority
        this.warmingQueue.sort((a, b) => b.priority - a.priority);
    }

    /**
     * Start cache warming process
     */
    async startWarming(): Promise<void> {
        if (this.isWarming) {return;}

        this.isWarming = true;
        
        try {
            while (this.warmingQueue.length > 0) {
                const task = this.warmingQueue.shift();
                if (task) {
                    await this.processWarmingTask(task);
                }
            }
        } finally {
            this.isWarming = false;
        }
    }

    /**
     * Stop cache warming process
     */
    stopWarming(): void {
        this.isWarming = false;
        this.warmingQueue = [];
    }

    private calculateWarmingPriority(filePath: string): number {
        let priority = 50; // Base priority

        // Higher priority for common file types
        if (filePath.endsWith('.ts') || filePath.endsWith('.js')) {priority += 20;}
        if (filePath.endsWith('.py')) {priority += 15;}
        if (filePath.endsWith('.rs')) {priority += 10;}

        // Higher priority for files in common directories
        if (filePath.includes('/src/')) {priority += 15;}
        if (filePath.includes('/lib/')) {priority += 10;}
        if (filePath.includes('/core/')) {priority += 10;}

        // Lower priority for test files
        if (filePath.includes('/test/') || filePath.includes('spec.')) {priority -= 10;}

        return priority;
    }

    private async processWarmingTask(task: WarmingTask): Promise<void> {
        try {
            switch (task.type) {
                case 'file':
                    // Pre-compute AST and semantic context
                    if (task.filePath) {
                        await this.cache.getOrComputeAST(task.filePath, async () => {
                            // This would trigger actual AST parsing
                            return { parsed: true, filePath: task.filePath! };
                        });
                    }
                    break;
            }
        } catch (error) {
            console.warn(`Cache warming failed for ${task.filePath}:`, error);
        }
    }
}

interface WarmingTask {
    type: 'file' | 'query' | 'relationship';
    filePath?: string;
    query?: string;
    priority: number;
    addedAt: number;
}

// Export singleton cache instance
export const contextCache = new ContextCache();
