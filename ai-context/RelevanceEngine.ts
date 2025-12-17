/**
 * Multi-Dimensional Relevance Engine
 * 
 * This engine scores checkpoint relevance across multiple dimensions to select
 * the most appropriate context for AI interactions.
 */

import * as Types from './types';

// Local interface for dimension scores
interface DimensionScores {
    semanticScore: number;
    temporalScore: number;
    architecturalScore: number;
    dependencyScore: number;
    usageScore: number;
}

export class RelevanceEngine {
    private semanticScorer: SemanticScorer;
    private temporalScorer: TemporalScorer;
    private architecturalScorer: ArchitecturalScorer;
    private dependencyScorer: DependencyScorer;
    private usageScorer: UsageScorer;

    constructor() {
        this.semanticScorer = new SemanticScorer();
        this.temporalScorer = new TemporalScorer();
        this.architecturalScorer = new ArchitecturalScorer();
        this.dependencyScorer = new DependencyScorer();
        this.usageScorer = new UsageScorer();
    }

    /**
     * Score checkpoint relevance across all dimensions
     */
    async scoreCheckpointRelevance(
        queryIntent: Types.QueryIntent,
        checkpoint: Types.AIContextCheckpoint,
        workspaceContext: Types.WorkspaceContext
    ): Promise<Types.RelevanceScore> {
        // Get dynamic weights based on query intent
        const weights = this.getDynamicWeights(queryIntent);

        // Calculate individual dimension scores
        const semanticScore = await this.semanticScorer.score(queryIntent, checkpoint, workspaceContext);
        const temporalScore = await this.temporalScorer.score(checkpoint, workspaceContext);
        const architecturalScore = await this.architecturalScorer.score(queryIntent, checkpoint, workspaceContext);
        const dependencyScore = await this.dependencyScorer.score(queryIntent, checkpoint, workspaceContext);
        const usageScore = await this.usageScorer.score(queryIntent, checkpoint, workspaceContext);

        // Calculate weighted composite score
        const compositeScore = this.calculateCompositeScore(
            { semanticScore, temporalScore, architecturalScore, dependencyScore, usageScore },
            weights
        );

        return {
            semantic: semanticScore,
            temporal: temporalScore,
            architectural: architecturalScore,
            dependency: dependencyScore,
            usage: usageScore,
            composite: compositeScore,
            confidence: this.calculateConfidence(semanticScore, temporalScore, architecturalScore, dependencyScore, usageScore),
            reasoning: this.generateReasoning(queryIntent, checkpoint, { semanticScore, temporalScore, architecturalScore, dependencyScore, usageScore })
        };
    }

    /**
     * Get dynamic weights based on query intent
     */
    private getDynamicWeights(intent: Types.QueryIntent): Types.RelevanceWeights {
        switch (intent.query_type) {
            case Types.QueryType.Implementation:
                return {
                    semantic: 0.4,
                    architectural: 0.3,
                    dependency: 0.2,
                    temporal: 0.05,
                    usage: 0.05,
                };

            case Types.QueryType.Debugging:
                return {
                    dependency: 0.35,
                    semantic: 0.25,
                    usage: 0.2,
                    temporal: 0.15,
                    architectural: 0.05,
                };

            case Types.QueryType.Refactoring:
                return {
                    architectural: 0.4,
                    dependency: 0.25,
                    semantic: 0.2,
                    usage: 0.1,
                    temporal: 0.05,
                };

            case Types.QueryType.Architecture:
                return {
                    architectural: 0.5,
                    dependency: 0.2,
                    semantic: 0.15,
                    temporal: 0.1,
                    usage: 0.05,
                };

            case Types.QueryType.Explanation:
                return {
                    semantic: 0.35,
                    architectural: 0.25,
                    dependency: 0.2,
                    usage: 0.15,
                    temporal: 0.05,
                };

            case Types.QueryType.Testing:
                return {
                    semantic: 0.3,
                    usage: 0.3,
                    dependency: 0.2,
                    architectural: 0.15,
                    temporal: 0.05,
                };

            default:
                return {
                    semantic: 0.3,
                    architectural: 0.2,
                    dependency: 0.2,
                    usage: 0.15,
                    temporal: 0.15,
                };
        }
    }

    /**
     * Calculate weighted composite score
     */
    private calculateCompositeScore(
        scores: DimensionScores,
        weights: Types.RelevanceWeights
    ): number {
        return (
            scores.semanticScore * weights.semantic +
            scores.temporalScore * weights.temporal +
            scores.architecturalScore * weights.architectural +
            scores.dependencyScore * weights.dependency +
            scores.usageScore * weights.usage
        );
    }

    /**
     * Calculate confidence in the relevance score
     */
    private calculateConfidence(
        semantic: number,
        temporal: number,
        architectural: number,
        dependency: number,
        usage: number
    ): number {
        // Confidence is higher when scores are consistent across dimensions
        const scores = [semantic, temporal, architectural, dependency, usage];
        const mean = scores.reduce((sum, score) => sum + score, 0) / scores.length;
        const variance = scores.reduce((sum, score) => sum + Math.pow(score - mean, 2), 0) / scores.length;
        const stdDev = Math.sqrt(variance);

        // Lower standard deviation means higher confidence
        return Math.max(0, 1 - (stdDev / mean));
    }

    /**
     * Generate human-readable reasoning for the score
     */
    private generateReasoning(
        intent: Types.QueryIntent,
        checkpoint: Types.AIContextCheckpoint,
        scores: DimensionScores
    ): string {
        const reasons: string[] = [];

        // Semantic reasoning
        if (scores.semanticScore > 0.7) {
            reasons.push(`High semantic relevance: checkpoint contains entities mentioned in query`);
        } else if (scores.semanticScore > 0.4) {
            reasons.push(`Moderate semantic relevance: some related entities found`);
        }

        // Temporal reasoning
        if (scores.temporalScore > 0.8) {
            reasons.push(`Very recent checkpoint: likely contains current state`);
        } else if (scores.temporalScore > 0.5) {
            reasons.push(`Recent checkpoint: reasonably current`);
        }

        // Architectural reasoning
        if (scores.architecturalScore > 0.7) {
            reasons.push(`Strong architectural relevance: matches system design patterns`);
        }

        // Dependency reasoning
        if (scores.dependencyScore > 0.7) {
            reasons.push(`High dependency relevance: contains required dependencies`);
        }

        // Usage reasoning
        if (scores.usageScore > 0.6) {
            reasons.push(`Good usage patterns: shows how code is typically used`);
        }

        return reasons.length > 0 ? reasons.join('; ') : 'General relevance based on composite scoring';
    }

    /**
     * Select optimal combination of checkpoints for a query
     */
    async selectOptimalCheckpointCombination(
        scoredCheckpoints: Types.ScoredCheckpoint[],
        intent: Types.QueryIntent,
        maxCheckpoints: number = 5
    ): Promise<Types.AIContextCheckpoint[]> {
        // Sort by composite score
        const sortedCheckpoints = scoredCheckpoints.sort((a, b) => b.score.composite - a.score.composite);

        // Apply diversity selection to avoid redundant checkpoints
        const selected = await this.applyDiversitySelection(sortedCheckpoints, intent, maxCheckpoints);

        return selected.map(scored => scored.checkpoint);
    }

    /**
     * Apply diversity selection to avoid redundant checkpoints
     */
    private async applyDiversitySelection(
        sortedCheckpoints: Types.ScoredCheckpoint[],
        intent: Types.QueryIntent,
        maxCheckpoints: number
    ): Promise<Types.ScoredCheckpoint[]> {
        const selected: Types.ScoredCheckpoint[] = [];
        
        for (const candidate of sortedCheckpoints) {
            if (selected.length >= maxCheckpoints) {
                break;
            }

            // Check if this checkpoint adds new value
            const addsDiversity = await this.checkDiversity(candidate, selected, intent);
            if (addsDiversity || selected.length === 0) {
                selected.push(candidate);
            }
        }

        return selected;
    }

    /**
     * Check if a checkpoint adds diversity to the selected set
     */
    private async checkDiversity(
        candidate: Types.ScoredCheckpoint,
        selected: Types.ScoredCheckpoint[],
        intent: Types.QueryIntent
    ): Promise<boolean> {
        // Check for file overlap
        const candidateFileChanges = candidate.checkpoint.base_checkpoint?.file_changes || candidate.checkpoint.file_changes || [];
        const candidateFiles = new Set(candidateFileChanges.map(fc => fc.path));
        
        for (const selectedCheckpoint of selected) {
            const selectedFileChanges = selectedCheckpoint.checkpoint.base_checkpoint?.file_changes || selectedCheckpoint.checkpoint.file_changes || [];
            const selectedFiles = new Set(selectedFileChanges.map(fc => fc.path));
            const overlap = this.calculateSetOverlap(candidateFiles, selectedFiles);
            
            // If there's significant file overlap, check for semantic diversity
            if (overlap > 0.7) {
                const semanticDiversity = await this.calculateSemanticDiversity(
                    candidate.checkpoint,
                    selectedCheckpoint.checkpoint,
                    intent
                );
                
                // Only include if there's significant semantic diversity
                if (semanticDiversity < 0.3) {
                    return false;
                }
            }
        }

        return true;
    }

    /**
     * Calculate overlap between two sets
     */
    private calculateSetOverlap<T>(set1: Set<T>, set2: Set<T>): number {
        const intersection = new Set(Array.from(set1).filter(x => set2.has(x)));
        const union = new Set([...Array.from(set1), ...Array.from(set2)]);
        return intersection.size / union.size;
    }

    /**
     * Calculate semantic diversity between two checkpoints
     */
    private async calculateSemanticDiversity(
        checkpoint1: Types.AIContextCheckpoint,
        checkpoint2: Types.AIContextCheckpoint,
        intent: Types.QueryIntent
    ): Promise<number> {
        // Compare semantic contexts
        const context1 = checkpoint1.semantic_context;
        const context2 = checkpoint2.semantic_context;

        // Calculate diversity in functions, classes, etc.
        const functionDiversity = this.calculateEntityDiversity(
            Object.keys(context1.functions || {}),
            Object.keys(context2.functions || {})
        );

        const classDiversity = this.calculateEntityDiversity(
            Object.keys(context1.classes || {}),
            Object.keys(context2.classes || {})
        );

        const interfaceDiversity = this.calculateEntityDiversity(
            Object.keys(context1.interfaces || {}),
            Object.keys(context2.interfaces || {})
        );

        // Average diversity across entity types
        return (functionDiversity + classDiversity + interfaceDiversity) / 3;
    }

    /**
     * Calculate diversity between two entity lists
     */
    private calculateEntityDiversity(entities1: string[], entities2: string[]): number {
        const set1 = new Set(entities1);
        const set2 = new Set(entities2);
        const intersection = new Set(Array.from(set1).filter(x => set2.has(x)));
        const union = new Set([...Array.from(set1), ...Array.from(set2)]);
        
        // Diversity is the complement of overlap
        return union.size === 0 ? 0 : 1 - (intersection.size / union.size);
    }
}

/**
 * Semantic relevance scorer
 */
class SemanticScorer {
    async score(
        intent: Types.QueryIntent,
        checkpoint: Types.AIContextCheckpoint,
        context: Types.WorkspaceContext
    ): Promise<number> {
        let score = 0;

        // Score based on entity matches
        const entityMatchScore = this.scoreEntityMatches(intent, checkpoint);
        score += entityMatchScore * 0.4;

        // Score based on keyword relevance
        const keywordScore = this.scoreKeywordRelevance(intent, checkpoint);
        score += keywordScore * 0.3;

        // Score based on semantic similarity
        const semanticSimilarity = await this.calculateSemanticSimilarity(intent, checkpoint);
        score += semanticSimilarity * 0.3;

        return Math.min(score, 1.0);
    }

    private scoreEntityMatches(intent: Types.QueryIntent, checkpoint: Types.AIContextCheckpoint): number {
        const semanticContext = checkpoint.semantic_context;
        let matches = 0;
        let total = intent.entities.length;

        for (const entity of intent.entities) {
            if (this.entityExistsInContext(entity, semanticContext)) {
                matches++;
            }
        }

        return total === 0 ? 0 : matches / total;
    }

    private scoreKeywordRelevance(intent: Types.QueryIntent, checkpoint: Types.AIContextCheckpoint): number {
        // Extract keywords from the query and match against checkpoint content
        const queryKeywords = intent.original_query.toLowerCase().split(/\s+/);
        const checkpointText = this.extractTextFromCheckpoint(checkpoint).toLowerCase();
        
        let matches = 0;
        for (const keyword of queryKeywords) {
            if (keyword.length > 2 && checkpointText.includes(keyword)) {
                matches++;
            }
        }

        return queryKeywords.length === 0 ? 0 : matches / queryKeywords.length;
    }

    private async calculateSemanticSimilarity(intent: Types.QueryIntent, checkpoint: Types.AIContextCheckpoint): Promise<number> {
        // Placeholder for semantic similarity calculation
        return 0.5;
    }

    private entityExistsInContext(entity: any, semanticContext: any): boolean {
        const entityName = entity.name;
        
        return (
            (semanticContext.functions && semanticContext.functions[entityName]) ||
            (semanticContext.classes && semanticContext.classes[entityName]) ||
            (semanticContext.interfaces && semanticContext.interfaces[entityName]) ||
            (semanticContext.types && semanticContext.types[entityName])
        );
    }

    private extractTextFromCheckpoint(checkpoint: Types.AIContextCheckpoint): string {
        // Extract all text content from the checkpoint for keyword matching
        let text = '';
        
        const fileChanges = checkpoint.base_checkpoint?.file_changes || checkpoint.file_changes || [];
        for (const fileChange of fileChanges) {
            if (fileChange.new_content) {
                text += fileChange.new_content + ' ';
            }
        }

        return text;
    }
}

/**
 * Temporal relevance scorer
 */
class TemporalScorer {
    async score(checkpoint: Types.AIContextCheckpoint, context: Types.WorkspaceContext): Promise<number> {
        const now = new Date();
        const checkpointTime = new Date(checkpoint.created_at);
        const ageInHours = (now.getTime() - checkpointTime.getTime()) / (1000 * 60 * 60);

        // Exponential decay with half-life of 24 hours
        const halfLife = 24;
        const decayFactor = Math.pow(0.5, ageInHours / halfLife);

        // Boost score for stability (checkpoints that haven't been superseded)
        const stabilityBoost = await this.calculateStabilityBoost(checkpoint, context);

        return Math.min(decayFactor * stabilityBoost, 1.0);
    }

    private async calculateStabilityBoost(checkpoint: Types.AIContextCheckpoint, context: Types.WorkspaceContext): Promise<number> {
        // Boost checkpoints that represent stable states
        // (e.g., haven't been modified by subsequent checkpoints)
        return 1.0; // Placeholder
    }
}

/**
 * Architectural relevance scorer
 */
class ArchitecturalScorer {
    async score(
        intent: Types.QueryIntent,
        checkpoint: Types.AIContextCheckpoint,
        context: Types.WorkspaceContext
    ): Promise<number> {
        let score = 0;

        // Score based on architectural patterns
        const patternScore = this.scoreArchitecturalPatterns(intent, checkpoint);
        score += patternScore * 0.4;

        // Score based on system layer relevance
        const layerScore = this.scoreSystemLayerRelevance(intent, checkpoint);
        score += layerScore * 0.3;

        // Score based on component relationships
        const relationshipScore = this.scoreComponentRelationships(intent, checkpoint);
        score += relationshipScore * 0.3;

        return Math.min(score, 1.0);
    }

    private scoreArchitecturalPatterns(intent: Types.QueryIntent, checkpoint: Types.AIContextCheckpoint): number {
        // Score based on architectural patterns present in the checkpoint
        const intentAnalysis = checkpoint.intent_analysis;
        if (!intentAnalysis || !intentAnalysis.design_patterns_used) {
            return 0.3; // Default score
        }

        // Higher score for checkpoints that use relevant design patterns
        return intentAnalysis.design_patterns_used.length > 0 ? 0.8 : 0.3;
    }

    private scoreSystemLayerRelevance(intent: Types.QueryIntent, checkpoint: Types.AIContextCheckpoint): number {
        // Score based on system layer (UI, business logic, data, etc.)
        return 0.5; // Placeholder
    }

    private scoreComponentRelationships(intent: Types.QueryIntent, checkpoint: Types.AIContextCheckpoint): number {
        // Score based on component interactions and relationships
        return 0.5; // Placeholder
    }
}

/**
 * Dependency relevance scorer
 */
class DependencyScorer {
    async score(
        intent: Types.QueryIntent,
        checkpoint: Types.AIContextCheckpoint,
        context: Types.WorkspaceContext
    ): Promise<number> {
        let score = 0;

        // Score based on dependency chain relevance
        const dependencyChainScore = this.scoreDependencyChain(intent, checkpoint);
        score += dependencyChainScore * 0.5;

        // Score based on import relevance
        const importScore = this.scoreImportRelevance(intent, checkpoint);
        score += importScore * 0.3;

        // Score based on call graph connectivity
        const callGraphScore = this.scoreCallGraphConnectivity(intent, checkpoint);
        score += callGraphScore * 0.2;

        return Math.min(score, 1.0);
    }

    private scoreDependencyChain(intent: Types.QueryIntent, checkpoint: Types.AIContextCheckpoint): number {
        // Score based on dependency chain relevance to the query
        return 0.5; // Placeholder
    }

    private scoreImportRelevance(intent: Types.QueryIntent, checkpoint: Types.AIContextCheckpoint): number {
        const semanticContext = checkpoint.semantic_context;
        if (!semanticContext.imports) {
            return 0;
        }

        // Score based on relevant imports
        let relevantImports = 0;
        for (const importStmt of semanticContext.imports) {
            if (this.isImportRelevant(importStmt, intent)) {
                relevantImports++;
            }
        }

        return semanticContext.imports.length === 0 ? 0 : relevantImports / semanticContext.imports.length;
    }

    private scoreCallGraphConnectivity(intent: Types.QueryIntent, checkpoint: Types.AIContextCheckpoint): number {
        // Score based on call graph connectivity
        return 0.5; // Placeholder
    }

    private isImportRelevant(importStmt: any, intent: Types.QueryIntent): boolean {
        // Check if import is relevant to the query intent
        for (const entity of intent.entities) {
            if (importStmt.imported_items && importStmt.imported_items.includes(entity.name)) {
                return true;
            }
        }
        return false;
    }
}

/**
 * Usage pattern relevance scorer
 */
class UsageScorer {
    async score(
        intent: Types.QueryIntent,
        checkpoint: Types.AIContextCheckpoint,
        context: Types.WorkspaceContext
    ): Promise<number> {
        let score = 0;

        // Score based on usage frequency
        const frequencyScore = this.scoreUsageFrequency(checkpoint);
        score += frequencyScore * 0.4;

        // Score based on usage patterns
        const patternScore = this.scoreUsagePatterns(intent, checkpoint);
        score += patternScore * 0.3;

        // Score based on example quality
        const exampleScore = this.scoreExampleQuality(checkpoint);
        score += exampleScore * 0.3;

        return Math.min(score, 1.0);
    }

    private scoreUsageFrequency(checkpoint: Types.AIContextCheckpoint): number {
        // Score based on how frequently the code in this checkpoint is used
        return 0.5; // Placeholder
    }

    private scoreUsagePatterns(intent: Types.QueryIntent, checkpoint: Types.AIContextCheckpoint): number {
        // Score based on relevant usage patterns
        return 0.5; // Placeholder
    }

    private scoreExampleQuality(checkpoint: Types.AIContextCheckpoint): number {
        // Score based on quality of usage examples
        return 0.5; // Placeholder
    }
}


