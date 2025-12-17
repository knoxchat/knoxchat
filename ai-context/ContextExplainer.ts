/**
 * Context Explainer - Provides transparency into why context was included
 * 
 * This service explains the reasoning behind context selection, building trust
 * and enabling users to understand what the AI sees.
 */

import * as Types from './types';

export interface ExplainedContext extends Types.CompleteAIContext {
    explanation: ContextExplanation;
    explained_files: ExplainedFile[];
}

export interface ContextExplanation {
    // Overall summary
    summary: string;
    
    // Why these specific files were chosen
    selection_reasoning: string;
    
    // Confidence breakdown by dimension
    confidence_breakdown: ConfidenceBreakdown;
    
    // What was excluded and why
    excluded_elements: ExcludedElement[];
    
    // Query analysis explanation
    query_analysis: QueryAnalysisExplanation;
    
    // Token budget usage
    token_usage: TokenUsageExplanation;
}

export interface ExplainedFile extends Types.ContextFile {
    // Why this file was included
    inclusion_reason: string;
    
    // Detailed relevance scoring
    relevance_breakdown: RelevanceBreakdown;
    
    // Key elements that made this file relevant
    key_elements: KeyElement[];
    
    // Contribution to overall context
    contribution: FileContribution;
}

export interface ConfidenceBreakdown {
    semantic_match: number;
    temporal_relevance: number;
    architectural_fit: number;
    dependency_coverage: number;
    usage_pattern_match: number;
    overall_confidence: number;
    explanation: string;
}

export interface ExcludedElement {
    type: 'file' | 'function' | 'class' | 'module';
    name: string;
    path?: string;
    exclusion_reason: string;
    relevance_score: number;
    could_be_included_if: string;
}

export interface QueryAnalysisExplanation {
    detected_intent: string;
    extracted_entities: string[];
    inferred_scope: string;
    context_requirements: string[];
    query_complexity: 'simple' | 'moderate' | 'complex';
}

export interface TokenUsageExplanation {
    total_available: number;
    total_used: number;
    usage_percentage: number;
    breakdown_by_category: {
        core_files: number;
        architecture: number;
        relationships: number;
        history: number;
        examples: number;
    };
    optimization_applied: string;
}

export interface RelevanceBreakdown {
    // Individual scoring factors
    entity_matches: EntityMatch[];
    keyword_relevance: number;
    structural_importance: number;
    temporal_score: number;
    dependency_centrality: number;
    
    // Combined score
    overall_score: number;
    
    // Human-readable explanation
    explanation: string;
}

export interface EntityMatch {
    entity_name: string;
    entity_type: string;
    match_type: 'exact' | 'partial' | 'related';
    confidence: number;
    location_in_file: string;
}

export interface KeyElement {
    type: 'function' | 'class' | 'interface' | 'type' | 'import';
    name: string;
    relevance: number;
    reason: string;
    line_numbers: { start: number; end: number };
}

export interface FileContribution {
    provides_core_definitions: boolean;
    provides_dependencies: boolean;
    provides_usage_examples: boolean;
    provides_architectural_context: boolean;
    criticality: 'essential' | 'important' | 'supplementary' | 'optional';
    explanation: string;
}

/**
 * Main Context Explainer Service
 */
export class ContextExplainer {
    /**
     * Add comprehensive explanations to a context
     */
    async explainContext(
        context: Types.CompleteAIContext,
        queryIntent: Types.QueryIntent,
        scoringData: ScoringData
    ): Promise<ExplainedContext> {
        // Build overall explanation
        const explanation = this.buildOverallExplanation(
            context,
            queryIntent,
            scoringData
        );

        // Explain each file
        const explainedFiles = await Promise.all(
            context.core_files.map(file =>
                this.explainFile(file, queryIntent, scoringData)
            )
        );

        return {
            ...context,
            explanation,
            explained_files: explainedFiles,
        };
    }

    /**
     * Build overall context explanation
     */
    private buildOverallExplanation(
        context: Types.CompleteAIContext,
        queryIntent: Types.QueryIntent,
        scoringData: ScoringData
    ): ContextExplanation {
        const summary = this.generateSummary(context, queryIntent);
        const selectionReasoning = this.explainSelection(context, queryIntent, scoringData);
        const confidenceBreakdown = this.buildConfidenceBreakdown(context, scoringData);
        const excludedElements = this.identifyExcludedElements(scoringData);
        const queryAnalysis = this.explainQueryAnalysis(queryIntent);
        const tokenUsage = this.explainTokenUsage(context, scoringData);

        return {
            summary,
            selection_reasoning: selectionReasoning,
            confidence_breakdown: confidenceBreakdown,
            excluded_elements: excludedElements,
            query_analysis: queryAnalysis,
            token_usage: tokenUsage,
        };
    }

    /**
     * Generate human-readable summary
     */
    private generateSummary(
        context: Types.CompleteAIContext,
        queryIntent: Types.QueryIntent
    ): string {
        const fileCount = context.core_files.length;
        const queryType = queryIntent.query_type;
        const entityCount = queryIntent.entities.length;

        let summary = `I found ${fileCount} highly relevant file${fileCount !== 1 ? 's' : ''} for your ${queryType.toLowerCase()} query. `;

        if (entityCount > 0) {
            const entityNames = queryIntent.entities.slice(0, 3).map(e => `"${e.name}"`).join(', ');
            summary += `These files contain or relate to ${entityNames}`;
            if (entityCount > 3) {
                summary += ` and ${entityCount - 3} other entities`;
            }
            summary += '. ';
        }

        summary += `The context includes complete semantic understanding, architectural patterns, and relationship graphs to provide comprehensive insight.`;

        return summary;
    }

    /**
     * Explain why these specific files were selected
     */
    private explainSelection(
        context: Types.CompleteAIContext,
        queryIntent: Types.QueryIntent,
        scoringData: ScoringData
    ): string {
        const reasons: string[] = [];

        // Analyze query type influence
        switch (queryIntent.query_type) {
            case Types.QueryType.Architecture:
                reasons.push('Selected files that demonstrate architectural patterns and system design');
                break;
            case Types.QueryType.Debugging:
                reasons.push('Prioritized files in the dependency chain and call graph');
                break;
            case Types.QueryType.Implementation:
                reasons.push('Included similar implementations and usage examples');
                break;
            case Types.QueryType.Refactoring:
                reasons.push('Selected files that would be impacted by refactoring');
                break;
            default:
                reasons.push('Selected files based on semantic relevance and structural importance');
        }

        // Add scope information
        if (queryIntent.scope === Types.QueryScope.System) {
            reasons.push('Expanded scope to include system-level architecture');
        } else if (queryIntent.scope === Types.QueryScope.Module) {
            reasons.push('Focused on module-level context and dependencies');
        }

        // Add entity-specific reasoning
        if (queryIntent.entities.length > 0) {
            reasons.push(`Included files containing direct definitions or usages of mentioned entities`);
        }

        return reasons.join('. ') + '.';
    }

    /**
     * Build detailed confidence breakdown
     */
    private buildConfidenceBreakdown(
        context: Types.CompleteAIContext,
        scoringData: ScoringData
    ): ConfidenceBreakdown {
        // Calculate individual dimension scores
        const semanticMatch = this.calculateAverageScore(scoringData.semanticScores);
        const temporalRelevance = this.calculateAverageScore(scoringData.temporalScores);
        const architecturalFit = this.calculateAverageScore(scoringData.architecturalScores);
        const dependencyCoverage = this.calculateAverageScore(scoringData.dependencyScores);
        const usagePatternMatch = this.calculateAverageScore(scoringData.usageScores);

        const overallConfidence = context.confidence_score;

        // Generate explanation
        let explanation = 'Confidence assessment: ';
        const highScores: string[] = [];
        const lowScores: string[] = [];

        if (semanticMatch > 0.8) highScores.push('strong semantic match');
        else if (semanticMatch < 0.5) lowScores.push('weak semantic match');

        if (temporalRelevance > 0.8) highScores.push('very recent code');
        else if (temporalRelevance < 0.5) lowScores.push('older code');

        if (architecturalFit > 0.8) highScores.push('excellent architectural alignment');
        else if (architecturalFit < 0.5) lowScores.push('limited architectural context');

        if (dependencyCoverage > 0.8) highScores.push('complete dependency coverage');
        else if (dependencyCoverage < 0.5) lowScores.push('partial dependency coverage');

        if (highScores.length > 0) {
            explanation += highScores.join(', ');
        }
        if (lowScores.length > 0) {
            if (highScores.length > 0) explanation += '; however, ';
            explanation += lowScores.join(', ');
        }
        explanation += '.';

        return {
            semantic_match: semanticMatch,
            temporal_relevance: temporalRelevance,
            architectural_fit: architecturalFit,
            dependency_coverage: dependencyCoverage,
            usage_pattern_match: usagePatternMatch,
            overall_confidence: overallConfidence,
            explanation,
        };
    }

    /**
     * Identify and explain excluded elements
     */
    private identifyExcludedElements(scoringData: ScoringData): ExcludedElement[] {
        const excluded: ExcludedElement[] = [];

        // Add excluded files from scoring data
        for (const [fileName, score] of Object.entries(scoringData.allFileScores || {})) {
            if (score < 0.5) {
                excluded.push({
                    type: 'file',
                    name: fileName,
                    path: fileName,
                    exclusion_reason: this.generateExclusionReason(score),
                    relevance_score: score,
                    could_be_included_if: this.generateInclusionCondition(score),
                });
            }
        }

        // Sort by relevance (most relevant excluded items first)
        excluded.sort((a, b) => b.relevance_score - a.relevance_score);

        // Return top 10 most relevant exclusions
        return excluded.slice(0, 10);
    }

    /**
     * Generate exclusion reason
     */
    private generateExclusionReason(score: number): string {
        if (score < 0.2) {
            return 'Very low relevance to query - no matching entities or keywords';
        } else if (score < 0.35) {
            return 'Low relevance - only tangentially related to query';
        } else if (score < 0.5) {
            return 'Below relevance threshold - excluded to preserve token budget for more critical context';
        }
        return 'Excluded to fit within token limits';
    }

    /**
     * Generate inclusion condition
     */
    private generateInclusionCondition(score: number): string {
        if (score < 0.2) {
            return 'Query explicitly mentions entities in this file';
        } else if (score < 0.35) {
            return 'Query scope expanded to include related modules';
        } else if (score < 0.5) {
            return 'Token budget increased or query refined to focus on this area';
        }
        return 'Higher token limit available';
    }

    /**
     * Explain query analysis
     */
    private explainQueryAnalysis(queryIntent: Types.QueryIntent): QueryAnalysisExplanation {
        const detectedIntent = this.describeQueryType(queryIntent.query_type);
        const extractedEntities = queryIntent.entities.map(e => `${e.name} (${e.type})`);
        const inferredScope = this.describeScope(queryIntent.scope);
        const contextRequirements = queryIntent.context_requirements
            .sort((a, b) => b.priority - a.priority)
            .map(req => `${req.requirement_type} (priority: ${req.priority.toFixed(2)})`);

        const queryComplexity = this.determineQueryComplexity(queryIntent);

        return {
            detected_intent: detectedIntent,
            extracted_entities: extractedEntities,
            inferred_scope: inferredScope,
            context_requirements: contextRequirements,
            query_complexity: queryComplexity,
        };
    }

    /**
     * Explain token usage
     */
    private explainTokenUsage(
        context: Types.CompleteAIContext,
        scoringData: ScoringData
    ): TokenUsageExplanation {
        const totalAvailable = scoringData.maxTokens || 8000;
        const totalUsed = context.metadata?.token_count || 0;
        const usagePercentage = (totalUsed / totalAvailable) * 100;

        // Estimate breakdown (rough approximation)
        const coreFilesTokens = Math.floor(totalUsed * 0.7);
        const architectureTokens = Math.floor(totalUsed * 0.1);
        const relationshipsTokens = Math.floor(totalUsed * 0.1);
        const historyTokens = Math.floor(totalUsed * 0.05);
        const examplesTokens = totalUsed - coreFilesTokens - architectureTokens - relationshipsTokens - historyTokens;

        let optimization = 'No optimization needed';
        if (usagePercentage > 95) {
            optimization = 'Aggressive compression applied - removed comments and redundant code';
        } else if (usagePercentage > 85) {
            optimization = 'Moderate compression applied - prioritized essential elements';
        } else if (usagePercentage > 70) {
            optimization = 'Light optimization applied - excluded low-priority supplementary context';
        }

        return {
            total_available: totalAvailable,
            total_used: totalUsed,
            usage_percentage: usagePercentage,
            breakdown_by_category: {
                core_files: coreFilesTokens,
                architecture: architectureTokens,
                relationships: relationshipsTokens,
                history: historyTokens,
                examples: examplesTokens,
            },
            optimization_applied: optimization,
        };
    }

    /**
     * Explain individual file inclusion
     */
    private async explainFile(
        file: Types.ContextFile,
        queryIntent: Types.QueryIntent,
        scoringData: ScoringData
    ): Promise<ExplainedFile> {
        const inclusionReason = this.generateFileInclusionReason(file, queryIntent);
        const relevanceBreakdown = this.buildFileRelevanceBreakdown(file, queryIntent, scoringData);
        const keyElements = this.identifyKeyElements(file, queryIntent);
        const contribution = this.assessFileContribution(file, queryIntent);

        return {
            ...file,
            inclusion_reason: inclusionReason,
            relevance_breakdown: relevanceBreakdown,
            key_elements: keyElements,
            contribution: contribution,
        };
    }

    /**
     * Generate file inclusion reason
     */
    private generateFileInclusionReason(
        file: Types.ContextFile,
        queryIntent: Types.QueryIntent
    ): string {
        const reasons: string[] = [];

        // Check for entity matches
        const matchingEntities = queryIntent.entities.filter(entity =>
            this.fileContainsEntity(file, entity)
        );

        if (matchingEntities.length > 0) {
            const entityNames = matchingEntities.map(e => `"${e.name}"`).join(', ');
            reasons.push(`Directly contains ${entityNames} mentioned in your query`);
        }

        // Check for architectural relevance
        if (file.semantic_info.classes.length > 0) {
            reasons.push(`Defines ${file.semantic_info.classes.length} class${file.semantic_info.classes.length > 1 ? 'es' : ''} central to system architecture`);
        }

        // Check for recently modified
        if (file.last_modified) {
            const daysSince = (Date.now() - file.last_modified.getTime()) / (1000 * 60 * 60 * 24);
            if (daysSince < 7) {
                reasons.push(`Recently modified (${Math.floor(daysSince)} days ago)`);
            }
        }

        // Check for import/export significance
        if (file.semantic_info.exports.length > 0) {
            reasons.push(`Exports ${file.semantic_info.exports.length} reusable component${file.semantic_info.exports.length > 1 ? 's' : ''}`);
        }

        if (reasons.length === 0) {
            reasons.push('Part of relevant execution path or dependency chain');
        }

        return reasons.join('; ');
    }

    /**
     * Build file relevance breakdown
     */
    private buildFileRelevanceBreakdown(
        file: Types.ContextFile,
        queryIntent: Types.QueryIntent,
        scoringData: ScoringData
    ): RelevanceBreakdown {
        // Find entity matches
        const entityMatches: EntityMatch[] = [];
        for (const entity of queryIntent.entities) {
            const matches = this.findEntityMatchesInFile(file, entity);
            entityMatches.push(...matches);
        }

        // Calculate scores
        const keywordRelevance = this.calculateKeywordRelevance(file, queryIntent.original_query);
        const structuralImportance = this.calculateStructuralImportance(file);
        const temporalScore = this.calculateTemporalScore(file);
        const dependencyCentrality = this.calculateDependencyCentrality(file);

        const overallScore = (
            (entityMatches.length > 0 ? 0.4 : 0) +
            keywordRelevance * 0.25 +
            structuralImportance * 0.15 +
            temporalScore * 0.1 +
            dependencyCentrality * 0.1
        );

        const explanation = this.generateRelevanceExplanation(
            entityMatches,
            keywordRelevance,
            structuralImportance,
            temporalScore,
            dependencyCentrality
        );

        return {
            entity_matches: entityMatches,
            keyword_relevance: keywordRelevance,
            structural_importance: structuralImportance,
            temporal_score: temporalScore,
            dependency_centrality: dependencyCentrality,
            overall_score: overallScore,
            explanation: explanation,
        };
    }

    /**
     * Identify key elements in file
     */
    private identifyKeyElements(
        file: Types.ContextFile,
        queryIntent: Types.QueryIntent
    ): KeyElement[] {
        const keyElements: KeyElement[] = [];

        // Add functions that match query entities
        for (const func of file.semantic_info.functions) {
            if (queryIntent.entities.some(e => e.name.toLowerCase() === func.name.toLowerCase())) {
                keyElements.push({
                    type: 'function',
                    name: func.name,
                    relevance: 0.95,
                    reason: 'Directly mentioned in query',
                    line_numbers: { start: 0, end: 0 }, // Would need actual line numbers
                });
            } else if (func.complexity > 10) {
                keyElements.push({
                    type: 'function',
                    name: func.name,
                    relevance: 0.7,
                    reason: 'High complexity function central to logic',
                    line_numbers: { start: 0, end: 0 },
                });
            }
        }

        // Add classes
        for (const cls of file.semantic_info.classes) {
            if (queryIntent.entities.some(e => e.name.toLowerCase() === cls.name.toLowerCase())) {
                keyElements.push({
                    type: 'class',
                    name: cls.name,
                    relevance: 0.95,
                    reason: 'Directly mentioned in query',
                    line_numbers: { start: 0, end: 0 },
                });
            } else if (cls.methods.length > 5) {
                keyElements.push({
                    type: 'class',
                    name: cls.name,
                    relevance: 0.75,
                    reason: 'Core class with many methods',
                    line_numbers: { start: 0, end: 0 },
                });
            }
        }

        // Sort by relevance
        keyElements.sort((a, b) => b.relevance - a.relevance);

        return keyElements.slice(0, 10); // Top 10
    }

    /**
     * Assess file contribution
     */
    private assessFileContribution(
        file: Types.ContextFile,
        queryIntent: Types.QueryIntent
    ): FileContribution {
        const providesCoreDefinitions = queryIntent.entities.some(entity =>
            this.fileContainsEntity(file, entity)
        );

        const providesDependencies = file.semantic_info.imports.length > 0;
        const providesUsageExamples = file.semantic_info.functions.length > 0;
        const providesArchitecturalContext = file.semantic_info.classes.length > 0 ||
            file.semantic_info.interfaces.length > 0;

        let criticality: 'essential' | 'important' | 'supplementary' | 'optional' = 'optional';
        if (providesCoreDefinitions) {
            criticality = 'essential';
        } else if (providesArchitecturalContext) {
            criticality = 'important';
        } else if (providesDependencies || providesUsageExamples) {
            criticality = 'supplementary';
        }

        const explanation = this.generateContributionExplanation(
            providesCoreDefinitions,
            providesDependencies,
            providesUsageExamples,
            providesArchitecturalContext,
            criticality
        );

        return {
            provides_core_definitions: providesCoreDefinitions,
            provides_dependencies: providesDependencies,
            provides_usage_examples: providesUsageExamples,
            provides_architectural_context: providesArchitecturalContext,
            criticality: criticality,
            explanation: explanation,
        };
    }

    // Helper methods

    private calculateAverageScore(scores: number[]): number {
        if (scores.length === 0) return 0;
        return scores.reduce((sum, score) => sum + score, 0) / scores.length;
    }

    private describeQueryType(queryType: Types.QueryType): string {
        const descriptions: Record<Types.QueryType, string> = {
            [Types.QueryType.Implementation]: 'Implementation request - needs code examples and patterns',
            [Types.QueryType.Debugging]: 'Debugging query - requires dependency chain and execution flow',
            [Types.QueryType.Refactoring]: 'Refactoring task - needs impact analysis and architectural context',
            [Types.QueryType.Explanation]: 'Explanation request - requires semantic understanding and documentation',
            [Types.QueryType.Architecture]: 'Architecture query - needs system design and pattern analysis',
            [Types.QueryType.Testing]: 'Testing query - requires test coverage and usage examples',
            [Types.QueryType.Documentation]: 'Documentation request - needs clear explanations and examples',
            [Types.QueryType.Performance]: 'Performance query - requires complexity analysis and optimization opportunities',
            [Types.QueryType.Security]: 'Security query - needs vulnerability analysis and security patterns',
        };
        return descriptions[queryType] || 'General query';
    }

    private describeScope(scope: Types.QueryScope): string {
        const descriptions: Record<Types.QueryScope, string> = {
            [Types.QueryScope.Function]: 'Function-level - focused on specific function implementation',
            [Types.QueryScope.Class]: 'Class-level - encompasses class methods and properties',
            [Types.QueryScope.Module]: 'Module-level - includes entire module/file scope',
            [Types.QueryScope.Package]: 'Package-level - spans multiple related modules',
            [Types.QueryScope.System]: 'System-level - covers entire application architecture',
            [Types.QueryScope.Component]: 'Component-level - focuses on specific component boundary',
        };
        return descriptions[scope] || 'General scope';
    }

    private determineQueryComplexity(queryIntent: Types.QueryIntent): 'simple' | 'moderate' | 'complex' {
        const entityCount = queryIntent.entities.length;
        const requirementCount = queryIntent.context_requirements.length;
        const scopeBreadth = queryIntent.scope === Types.QueryScope.System ? 2 : 
                            queryIntent.scope === Types.QueryScope.Package ? 1 : 0;

        const complexityScore = entityCount + requirementCount + scopeBreadth;

        if (complexityScore <= 3) return 'simple';
        if (complexityScore <= 7) return 'moderate';
        return 'complex';
    }

    private fileContainsEntity(file: Types.ContextFile, entity: Types.CodeEntity): boolean {
        const lowerName = entity.name.toLowerCase();
        
        return (
            file.semantic_info.functions.some(f => f.name.toLowerCase() === lowerName) ||
            file.semantic_info.classes.some(c => c.name.toLowerCase() === lowerName) ||
            file.semantic_info.interfaces.some(i => i.name.toLowerCase() === lowerName) ||
            file.semantic_info.types.some(t => t.name.toLowerCase() === lowerName)
        );
    }

    private findEntityMatchesInFile(
        file: Types.ContextFile,
        entity: Types.CodeEntity
    ): EntityMatch[] {
        const matches: EntityMatch[] = [];
        const lowerName = entity.name.toLowerCase();

        // Check functions
        for (const func of file.semantic_info.functions) {
            if (func.name.toLowerCase() === lowerName) {
                matches.push({
                    entity_name: func.name,
                    entity_type: 'function',
                    match_type: 'exact',
                    confidence: 1.0,
                    location_in_file: `function ${func.name}`,
                });
            } else if (func.name.toLowerCase().includes(lowerName)) {
                matches.push({
                    entity_name: func.name,
                    entity_type: 'function',
                    match_type: 'partial',
                    confidence: 0.7,
                    location_in_file: `function ${func.name}`,
                });
            }
        }

        // Check classes
        for (const cls of file.semantic_info.classes) {
            if (cls.name.toLowerCase() === lowerName) {
                matches.push({
                    entity_name: cls.name,
                    entity_type: 'class',
                    match_type: 'exact',
                    confidence: 1.0,
                    location_in_file: `class ${cls.name}`,
                });
            }
        }

        return matches;
    }

    private calculateKeywordRelevance(file: Types.ContextFile, query: string): number {
        const keywords = query.toLowerCase().split(/\s+/).filter(w => w.length > 2);
        const fileContent = file.complete_content.toLowerCase();
        
        let matches = 0;
        for (const keyword of keywords) {
            if (fileContent.includes(keyword)) {
                matches++;
            }
        }
        
        return keywords.length > 0 ? matches / keywords.length : 0;
    }

    private calculateStructuralImportance(file: Types.ContextFile): number {
        // Files with more exports are generally more important
        const exportCount = file.semantic_info.exports.length;
        const classCount = file.semantic_info.classes.length;
        const interfaceCount = file.semantic_info.interfaces.length;
        
        return Math.min((exportCount * 0.3 + classCount * 0.5 + interfaceCount * 0.2) / 5, 1);
    }

    private calculateTemporalScore(file: Types.ContextFile): number {
        if (!file.last_modified) return 0.5;
        
        const daysSince = (Date.now() - file.last_modified.getTime()) / (1000 * 60 * 60 * 24);
        const decayFactor = Math.pow(0.5, daysSince / 30); // Half-life of 30 days
        
        return decayFactor;
    }

    private calculateDependencyCentrality(file: Types.ContextFile): number {
        // Files with many imports/exports are more central
        const importCount = file.semantic_info.imports.length;
        const exportCount = file.semantic_info.exports.length;
        
        return Math.min((importCount + exportCount * 2) / 20, 1);
    }

    private generateRelevanceExplanation(
        entityMatches: EntityMatch[],
        keywordRelevance: number,
        structuralImportance: number,
        temporalScore: number,
        dependencyCentrality: number
    ): string {
        const parts: string[] = [];

        if (entityMatches.length > 0) {
            parts.push(`Contains ${entityMatches.length} entity match${entityMatches.length > 1 ? 'es' : ''}`);
        }

        if (keywordRelevance > 0.7) {
            parts.push('high keyword relevance');
        }

        if (structuralImportance > 0.7) {
            parts.push('structurally important');
        }

        if (temporalScore > 0.8) {
            parts.push('very recent');
        }

        if (dependencyCentrality > 0.7) {
            parts.push('central to dependency graph');
        }

        return parts.length > 0 ? parts.join(', ') : 'General relevance to query';
    }

    private generateContributionExplanation(
        providesCoreDefinitions: boolean,
        providesDependencies: boolean,
        providesUsageExamples: boolean,
        providesArchitecturalContext: boolean,
        criticality: string
    ): string {
        const contributions: string[] = [];

        if (providesCoreDefinitions) {
            contributions.push('defines core entities mentioned in query');
        }
        if (providesArchitecturalContext) {
            contributions.push('provides architectural context through classes/interfaces');
        }
        if (providesUsageExamples) {
            contributions.push('shows usage patterns through functions');
        }
        if (providesDependencies) {
            contributions.push('includes dependency information');
        }

        const contributionText = contributions.join(', ');
        return `${criticality.charAt(0).toUpperCase() + criticality.slice(1)} file that ${contributionText}`;
    }
}

/**
 * Supporting type for scoring data
 */
export interface ScoringData {
    semanticScores: number[];
    temporalScores: number[];
    architecturalScores: number[];
    dependencyScores: number[];
    usageScores: number[];
    allFileScores?: Record<string, number>;
    maxTokens?: number;
}

