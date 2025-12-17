/**
 * AI Context Builder - Core service for building complete AI context from checkpoints
 * 
 * This service transforms the checkpoint-based approach into a complete context
 * system that provides semantic understanding of code for AI interactions.
 */

// Import actual implementations
import { ContextOptimizer } from './ContextOptimizer';
import { ContextTreeBuilder } from './ContextTreeBuilder';
import { QueryIntentAnalyzer } from './QueryIntentAnalyzer';
import { RelevanceEngine } from './RelevanceEngine';
import { ContextExplainer, ExplainedContext, ScoringData } from './ContextExplainer';
import { MultiModalIntegrator, EnrichedAIContext } from './MultiModalIntegrator';
import { IntelligentQueryExpander, ExpandedQuery } from './IntelligentQueryExpander';
import { IncrementalContextUpdater, IncrementalUpdate } from './IncrementalContextUpdater';
// Import unified types
import * as Types from './types';

// Mock CheckpointManager for now - will be replaced with actual import
interface CheckpointManager {
    getCheckpointsForWorkspace(workspace: string): Promise<Types.AIContextCheckpoint[]>;
}

// Re-export types for convenience
export * from './types';
export * from './ContextExplainer';
export * from './MultiModalIntegrator';
export * from './IntelligentQueryExpander';
export * from './IncrementalContextUpdater';
export * from './PredictiveContextLoader';

/**
 * Main AI Context Builder Service
 */
export class AIContextBuilder {
    private checkpointManager: CheckpointManager;
    private intentAnalyzer: QueryIntentAnalyzer;
    private contextOptimizer: ContextOptimizer;
    private contextTreeBuilder: ContextTreeBuilder;
    private relevanceEngine: RelevanceEngine;
    private contextExplainer: ContextExplainer;
    private multiModalIntegrator: MultiModalIntegrator;
    private queryExpander: IntelligentQueryExpander;
    private incrementalUpdater: IncrementalContextUpdater;
    private contextCache: Map<string, Types.CompleteAIContext>;
    private explainedContextCache: Map<string, ExplainedContext>;
    private enrichedContextCache: Map<string, EnrichedAIContext>;

    constructor(
        checkpointManager: CheckpointManager,
        intentAnalyzer?: QueryIntentAnalyzer,
        contextOptimizer?: ContextOptimizer
    ) {
        this.checkpointManager = checkpointManager;
        this.intentAnalyzer = intentAnalyzer || new QueryIntentAnalyzer();
        this.contextOptimizer = contextOptimizer || new ContextOptimizer();
        this.contextTreeBuilder = new ContextTreeBuilder();
        this.relevanceEngine = new RelevanceEngine();
        this.contextExplainer = new ContextExplainer();
        this.multiModalIntegrator = new MultiModalIntegrator();
        this.queryExpander = new IntelligentQueryExpander();
        this.incrementalUpdater = new IncrementalContextUpdater();
        this.contextCache = new Map();
        this.explainedContextCache = new Map();
        this.enrichedContextCache = new Map();
    }

    /**
     * Handle incremental file changes (for real-time updates)
     */
    async handleFileChange(
        fileChange: IncrementalUpdate,
        workspace: string
    ): Promise<void> {
        try {
            const result = await this.incrementalUpdater.updateOnFileChange(fileChange, workspace);

            // Invalidate affected caches
            for (const checkpointId of result.affected_checkpoints) {
                this.invalidateCachesForCheckpoint(checkpointId);
            }

            console.log(`✅ File change processed incrementally`);
            console.log(`   - File: ${fileChange.file_path}`);
            console.log(`   - Update time: ${result.update_time_ms}ms`);
            console.log(`   - Nodes updated: ${result.updated_nodes.length}`);
            console.log(`   - Caches invalidated: ${result.cache_invalidations}`);

        } catch (error) {
            console.error('Failed to handle incremental file change:', error);
            // Clear all caches for this file to force full rebuild
            this.invalidateCachesForFile(fileChange.file_path);
        }
    }

    /**
     * Invalidate caches for a specific checkpoint
     */
    private invalidateCachesForCheckpoint(checkpointId: string): void {
        // Clear context caches that used this checkpoint
        for (const [key, context] of Array.from(this.contextCache.entries())) {
            if (context.source_checkpoints?.includes(checkpointId)) {
                this.contextCache.delete(key);
            }
        }

        for (const [key, context] of Array.from(this.explainedContextCache.entries())) {
            if (context.source_checkpoints?.includes(checkpointId)) {
                this.explainedContextCache.delete(key);
            }
        }

        for (const [key, context] of Array.from(this.enrichedContextCache.entries())) {
            if (context.source_checkpoints?.includes(checkpointId)) {
                this.enrichedContextCache.delete(key);
            }
        }
    }

    /**
     * Invalidate all caches for a file
     */
    private invalidateCachesForFile(filePath: string): void {
        // For now, clear all caches (could be smarter)
        this.contextCache.clear();
        this.explainedContextCache.clear();
        this.enrichedContextCache.clear();
        console.log(`🗑️ Cleared all caches due to file change: ${filePath}`);
    }

    /**
     * Build enriched AI context with multi-modal insights
     */
    async buildEnrichedContextForQuery(
        query: string,
        currentWorkspace: string,
        maxTokens: number = 8000
    ): Promise<EnrichedAIContext> {
        const startTime = Date.now();

        // Check cache first
        const cacheKey = this.generateCacheKey(query, currentWorkspace, maxTokens);
        const cachedEnriched = this.enrichedContextCache.get(cacheKey);
        if (cachedEnriched) {
            return cachedEnriched;
        }

        try {
            // Build base context first
            const baseContext = await this.buildContextForQuery(query, currentWorkspace, maxTokens);
            
            // Analyze query intent for relevance scoring
            const queryIntent = await this.intentAnalyzer.analyzeQuery(query);

            // Enrich with multi-modal insights
            const enrichedContext = await this.multiModalIntegrator.enrichContext(baseContext, queryIntent);

            // Cache the result
            this.enrichedContextCache.set(cacheKey, enrichedContext);

            console.log(`✨ Enriched context with multi-modal insights in ${Date.now() - startTime}ms`);
            console.log(`   - Comments analyzed: ${enrichedContext.multi_modal_insights.enrichment_metadata.comments_analyzed}`);
            console.log(`   - Tests analyzed: ${enrichedContext.multi_modal_insights.enrichment_metadata.tests_analyzed}`);
            console.log(`   - Confidence boost: +${(enrichedContext.multi_modal_insights.enrichment_metadata.confidence_boost * 100).toFixed(1)}%`);

            return enrichedContext;

        } catch (error) {
            console.error('Failed to build enriched AI context:', error);
            throw new Error(`Context enrichment failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Build complete AI context for a given query with full explanations
     */
    async buildExplainedContextForQuery(
        query: string,
        currentWorkspace: string,
        maxTokens: number = 8000
    ): Promise<ExplainedContext> {
        const startTime = Date.now();

        // Check cache first
        const cacheKey = this.generateCacheKey(query, currentWorkspace, maxTokens);
        const cachedExplained = this.explainedContextCache.get(cacheKey);
        if (cachedExplained) {
            return cachedExplained;
        }

        try {
            // 1. Analyze query intent
            const queryIntent = await this.intentAnalyzer.analyzeQuery(query);

            // 2. Select relevant checkpoints and collect scoring data
            const { checkpoints, scoringData } = await this.selectSemanticallyRelevantCheckpointsWithScores(
                queryIntent,
                currentWorkspace
            );

            // 3. Build complete context tree
            const contextTree = await this.buildCompleteContextTree(
                checkpoints,
                queryIntent
            );

            // 4. Optimize for AI consumption
            const optimizedContext = await this.contextOptimizer.optimizeForAI(
                contextTree,
                maxTokens,
                queryIntent
            );

            // Add metadata
            optimizedContext.metadata = {
                checkpoints_used: checkpoints.map(cp => cp.base_checkpoint.id.toString()),
                context_type: this.determineContextType(queryIntent),
                confidence_score: this.calculateConfidenceScore(optimizedContext),
                token_count: await this.estimateTokenCount(optimizedContext),
                build_time_ms: Date.now() - startTime,
                cache_hit_rate: 0,
                generated_at: new Date()
            };

            // 5. Add comprehensive explanations
            const explainedContext = await this.contextExplainer.explainContext(
                optimizedContext,
                queryIntent,
                scoringData
            );

            // Cache the result
            this.explainedContextCache.set(cacheKey, explainedContext);

            return explainedContext;

        } catch (error) {
            console.error('Failed to build explained AI context:', error);
            throw new Error(`Context building failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Build complete AI context for a given query
     */
    async buildContextForQuery(
        query: string,
        currentWorkspace: string,
        maxTokens: number = 8000
    ): Promise<Types.CompleteAIContext> {
        const startTime = Date.now();

        // Check cache first
        const cacheKey = this.generateCacheKey(query, currentWorkspace, maxTokens);
        const cached = this.contextCache.get(cacheKey);
        if (cached) {
            return cached;
        }

        try {
            // 1. Analyze query intent
            const queryIntent = await this.intentAnalyzer.analyzeQuery(query);

            // 2. Expand query with related entities and concepts
            const workspaceContext: Types.WorkspaceContext = {
                workspace_path: currentWorkspace,
                current_files: [],
                recent_changes: [],
                project_metadata: {}
            };
            const expandedQuery = await this.queryExpander.expandQuery(query, queryIntent, workspaceContext);

            // Merge expanded entities into query intent
            const enhancedIntent: Types.QueryIntent = {
                ...queryIntent,
                entities: [...queryIntent.entities, ...expandedQuery.expanded_entities],
                context_requirements: [
                    ...queryIntent.context_requirements,
                    ...expandedQuery.implicit_requirements.map(req => ({
                        requirement_type: req.context_type,
                        priority: req.priority,
                        reasoning: req.reason
                    }))
                ]
            };

            console.log(`🔍 Query expanded: +${expandedQuery.expansion_metadata.entities_added} entities, +${expandedQuery.expansion_metadata.concepts_identified} concepts`);

            // 3. Select relevant checkpoints based on enhanced intent
            const relevantCheckpoints = await this.selectSemanticallyRelevantCheckpoints(
                enhancedIntent,
                currentWorkspace
            );

            // 3. Build complete context tree
            const contextTree = await this.buildCompleteContextTree(
                relevantCheckpoints,
                queryIntent
            );

            // 4. Optimize for AI consumption
            const optimizedContext = await this.contextOptimizer.optimizeForAI(
                contextTree,
                maxTokens,
                queryIntent
            );

            // Add metadata
            optimizedContext.metadata = {
                checkpoints_used: relevantCheckpoints.map(cp => cp.base_checkpoint.id.toString()),
                context_type: this.determineContextType(queryIntent),
                confidence_score: this.calculateConfidenceScore(optimizedContext),
                token_count: await this.estimateTokenCount(optimizedContext),
                build_time_ms: Date.now() - startTime,
                cache_hit_rate: 0, // First time building
                generated_at: new Date()
            };

            // Cache the result
            this.contextCache.set(cacheKey, optimizedContext);

            return optimizedContext;

        } catch (error) {
            console.error('Failed to build AI context:', error);
            throw new Error(`Context building failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Select checkpoints that are semantically relevant to the query (with scoring data)
     */
    private async selectSemanticallyRelevantCheckpointsWithScores(
        intent: Types.QueryIntent,
        workspace: string
    ): Promise<{ checkpoints: Types.AIContextCheckpoint[]; scoringData: ScoringData }> {
        // Get all checkpoints for the workspace
        const allCheckpoints: Types.AIContextCheckpoint[] = await this.checkpointManager.getCheckpointsForWorkspace(workspace);

        // Create workspace context
        const workspaceContext: Types.WorkspaceContext = {
            workspace_path: workspace,
            current_files: [], // Would be populated with current file list
            recent_changes: [], // Would be populated with recent changes
            project_metadata: {} // Would be populated with project metadata
        };

        // Score each checkpoint using the enhanced relevance engine
        const scoredCheckpoints: Types.ScoredCheckpoint[] = await Promise.all(
            allCheckpoints.map(async (checkpoint): Promise<Types.ScoredCheckpoint> => ({
                checkpoint,
                score: await this.relevanceEngine.scoreCheckpointRelevance(intent, checkpoint, workspaceContext)
            }))
        );

        // Collect scoring data
        const scoringData: ScoringData = {
            semanticScores: scoredCheckpoints.map(sc => sc.score.semantic),
            temporalScores: scoredCheckpoints.map(sc => sc.score.temporal),
            architecturalScores: scoredCheckpoints.map(sc => sc.score.architectural),
            dependencyScores: scoredCheckpoints.map(sc => sc.score.dependency),
            usageScores: scoredCheckpoints.map(sc => sc.score.usage),
            allFileScores: {},
            maxTokens: 8000,
        };

        // Build file scores map
        for (const scored of scoredCheckpoints) {
            for (const fileChange of scored.checkpoint.base_checkpoint.file_changes) {
                scoringData.allFileScores![fileChange.path] = scored.score.composite;
            }
        }

        // Select optimal combination using diversity and relevance
        const checkpoints = await this.relevanceEngine.selectOptimalCheckpointCombination(scoredCheckpoints, intent, 5);

        return { checkpoints, scoringData };
    }

    /**
     * Select checkpoints that are semantically relevant to the query
     */
    private async selectSemanticallyRelevantCheckpoints(
        intent: Types.QueryIntent,
        workspace: string
    ): Promise<Types.AIContextCheckpoint[]> {
        const { checkpoints } = await this.selectSemanticallyRelevantCheckpointsWithScores(intent, workspace);
        return checkpoints;
    }

    /**
     * Build complete context tree from selected checkpoints
     */
    private async buildCompleteContextTree(
        checkpoints: Types.AIContextCheckpoint[],
        intent: Types.QueryIntent
    ): Promise<Types.CompleteAIContext> {
        // Use the enhanced context tree builder
        const contextTree: Types.ContextTree = await this.contextTreeBuilder.buildCompleteContextTree(checkpoints, intent);

        const context: Types.CompleteAIContext = {
            core_files: await this.extractCoreFiles(checkpoints, intent),
            architecture: this.buildArchitecturalContextFromTree(contextTree),
            relationships: this.buildRelationshipContextFromTree(contextTree),
            history: this.buildEvolutionContextFromTree(contextTree),
            examples: this.buildExampleContextFromTree(contextTree, intent),
            metadata: {} as Types.ContextMetadata,
            source_checkpoints: checkpoints.map(cp => cp.base_checkpoint.id.toString()),
            context_type: this.determineContextType(intent),
            confidence_score: this.calculateContextConfidenceFromTree(contextTree, intent)
        };

        return context;
    }

    /**
     * Extract core files with complete content (not fragments)
     */
    private async extractCoreFiles(
        checkpoints: Types.AIContextCheckpoint[],
        intent: Types.QueryIntent
    ): Promise<Types.ContextFile[]> {
        const coreFiles: Types.ContextFile[] = [];
        const processedFiles = new Set<string>();

        for (const checkpoint of checkpoints) {
            for (const fileChange of checkpoint.base_checkpoint.file_changes) {
                const filePath = fileChange.path.toString();
                
                if (!processedFiles.has(filePath) && this.isRelevantFile(fileChange, intent)) {
                    const contextFile: Types.ContextFile = {
                        path: filePath,
                        complete_content: fileChange.new_content || fileChange.original_content || "",
                        language: this.detectLanguage(filePath),
                        encoding: fileChange.encoding.toString(),
                        size: fileChange.size_bytes,
                        last_modified: fileChange.modified_at,
                        semantic_info: this.extractSemanticInfo(checkpoint, filePath)
                    };

                    coreFiles.push(contextFile);
                    processedFiles.add(filePath);
                }
            }
        }

        return coreFiles;
    }

    // Helper methods to extract information from context tree
    private buildArchitecturalContextFromTree(contextTree: Types.ContextTree): Types.ArchitecturalContext {
        const archContext = contextTree.getArchitecturalContext();
        return {
            project_structure: { root_directories: [], module_structure: [], package_dependencies: [] },
            patterns_used: [],
            dependency_graph: { nodes: [], edges: [], cycles: [] },
            data_flow_diagram: { entry_points: [], data_transformations: [], storage_interactions: [] },
            layers: []
        };
    }

    private buildRelationshipContextFromTree(contextTree: Types.ContextTree): Types.RelationshipContext {
        const relContext = contextTree.getRelationshipContext();
        return {
            complete_call_graph: { functions: [], relationships: [] },
            type_hierarchy: { root_types: [], inheritance_chains: [], interface_implementations: [] },
            import_graph: { modules: [], dependencies: [] },
            usage_patterns: []
        };
    }

    private buildEvolutionContextFromTree(contextTree: Types.ContextTree): Types.HistoryContext {
        const evolContext = contextTree.getEvolutionContext();
        return {
            change_timeline: [],
            architectural_decisions: [],
            refactoring_history: []
        };
    }

    private buildExampleContextFromTree(contextTree: Types.ContextTree, intent: Types.QueryIntent): Types.ExampleContext[] {
        return [];
    }

    private calculateContextConfidenceFromTree(contextTree: Types.ContextTree, intent: Types.QueryIntent): number {
        // For testing purposes, return a static confidence score
        return 0.85;
    }

    // Helper methods
    private extractSemanticInfo(checkpoint: Types.AIContextCheckpoint, filePath: string): Types.SemanticInfo {
        return {
            functions: [],
            classes: [],
            interfaces: [],
            types: [],
            imports: [],
            exports: []
        };
    }

    private determineContextType(intent: Types.QueryIntent): Types.ContextType {
        switch (intent.query_type) {
            case Types.QueryType.Architecture:
                return Types.ContextType.ArchitecturalContext;
            case Types.QueryType.Debugging:
                return Types.ContextType.DependencyContext;
            default:
                return Types.ContextType.SemanticContext;
        }
    }

    private calculateConfidenceScore(context: Types.CompleteAIContext): number {
        return 0.8; // Placeholder
    }

    private async estimateTokenCount(context: Types.CompleteAIContext): Promise<number> {
        // Simple estimation - in practice, would use proper tokenization
        const totalContent = context.core_files.reduce((acc, file) => acc + file.complete_content.length, 0);
        return Math.floor(totalContent / 4); // Rough estimate: 4 chars per token
    }

    private generateCacheKey(query: string, workspace: string, maxTokens: number): string {
        return `${workspace}:${query.substring(0, 50)}:${maxTokens}`;
    }

    private checkpointContainsEntity(checkpoint: Types.AIContextCheckpoint, entity: Types.CodeEntity): boolean {
        // Check if checkpoint contains the specified entity
        switch (entity.type) {
            case Types.EntityType.Function:
                return checkpoint.semantic_context.functions.has(entity.name);
            case Types.EntityType.Class:
                return checkpoint.semantic_context.classes.has(entity.name);
            default:
                return false;
        }
    }

    private isRelevantFile(fileChange: any, intent: Types.QueryIntent): boolean {
        // Determine if file is relevant to the query intent
        return true; // Placeholder - would implement actual relevance logic
    }

    private detectLanguage(filePath: string): string {
        const extension = filePath.split('.').pop()?.toLowerCase();
        switch (extension) {
            case 'ts':
            case 'tsx':
                return 'typescript';
            case 'js':
            case 'jsx':
                return 'javascript';
            case 'py':
                return 'python';
            case 'rs':
                return 'rust';
            case 'go':
                return 'go';
            case 'java':
                return 'java';
            default:
                return 'text';
        }
    }
}

// All types are now imported from types.ts