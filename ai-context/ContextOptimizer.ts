/**
 * Context Optimizer - Optimizes AI context for token efficiency and relevance
 * 
 * This service takes complete context and optimizes it for AI consumption by:
 * - Prioritizing context elements by relevance
 * - Compressing less critical information
 * - Ensuring token limits are respected
 * - Maintaining semantic completeness
 */

import { CompleteAIContext, QueryIntent, QueryType, ContextFile } from './AIContextBuilder';

export interface OptimizedAIContext extends CompleteAIContext {
    optimization_metadata: OptimizationMetadata;
}

export interface OptimizationMetadata {
    original_token_count: number;
    optimized_token_count: number;
    compression_ratio: number;
    elements_included: number;
    elements_excluded: number;
    optimization_strategy: string;
    confidence_impact: number;
}

export interface ContextElement {
    id: string;
    type: ElementType;
    content: any;
    priority: number;
    token_count: number;
    essential: boolean;
    compressible: boolean;
}

export enum ElementType {
    CoreFile = "core_file",
    Function = "function",
    Class = "class",
    Interface = "interface",
    Type = "type",
    Import = "import",
    Export = "export",
    CallGraph = "call_graph",
    Dependency = "dependency",
    Pattern = "pattern",
    Example = "example",
    Documentation = "documentation"
}

export interface CompressionResult {
    original_content: string;
    compressed_content: string;
    compression_ratio: number;
    information_loss: number;
}

export class ContextOptimizer {
    private tokenEstimator: TokenEstimator;
    private priorityCalculator: PriorityCalculator;
    private compressionEngine: ContextCompressionEngine;

    constructor() {
        this.tokenEstimator = new TokenEstimator();
        this.priorityCalculator = new PriorityCalculator();
        this.compressionEngine = new ContextCompressionEngine();
    }

    /**
     * Optimize context for AI consumption within token limits
     */
    async optimizeForAI(
        context: CompleteAIContext,
        maxTokens: number,
        queryIntent: QueryIntent
    ): Promise<OptimizedAIContext> {
        const startTime = Date.now();

        // 1. Break down context into prioritizable elements
        const contextElements = await this.decomposeContext(context, queryIntent);

        // 2. Calculate priority scores for all elements
        const prioritizedElements = await this.prioritizeElements(contextElements, queryIntent);

        // 3. Estimate initial token count
        const originalTokenCount = await this.estimateTotalTokens(prioritizedElements);

        // 4. Build optimized context within token limits
        const optimizedContext = await this.buildOptimizedContext(
            prioritizedElements,
            maxTokens,
            queryIntent
        );

        // 5. Calculate optimization metadata
        const optimizedTokenCount = await this.tokenEstimator.estimate(optimizedContext);
        const optimizationMetadata: OptimizationMetadata = {
            original_token_count: originalTokenCount,
            optimized_token_count: optimizedTokenCount,
            compression_ratio: originalTokenCount > 0 ? optimizedTokenCount / originalTokenCount : 1,
            elements_included: prioritizedElements.filter(e => this.isElementIncluded(e, optimizedContext)).length,
            elements_excluded: prioritizedElements.filter(e => !this.isElementIncluded(e, optimizedContext)).length,
            optimization_strategy: this.determineOptimizationStrategy(queryIntent),
            confidence_impact: this.calculateConfidenceImpact(context, optimizedContext)
        };

        return {
            ...optimizedContext,
            optimization_metadata: optimizationMetadata
        };
    }

    /**
     * Decompose context into prioritizable elements
     */
    private async decomposeContext(
        context: CompleteAIContext,
        queryIntent: QueryIntent
    ): Promise<ContextElement[]> {
        const elements: ContextElement[] = [];

        // Decompose core files
        for (const file of context.core_files) {
            elements.push(await this.createFileElement(file, queryIntent));

            // Decompose file contents into smaller elements
            const fileElements = await this.decomposeFile(file, queryIntent);
            elements.push(...fileElements);
        }

        // Decompose architectural context
        elements.push(...await this.decomposeArchitecturalContext(context.architecture, queryIntent));

        // Decompose relationships
        elements.push(...await this.decomposeRelationships(context.relationships, queryIntent));

        // Decompose history
        elements.push(...await this.decomposeHistory(context.history, queryIntent));

        // Decompose examples
        elements.push(...await this.decomposeExamples(context.examples, queryIntent));

        return elements;
    }

    /**
     * Prioritize context elements based on query intent and relevance
     */
    private async prioritizeElements(
        elements: ContextElement[],
        queryIntent: QueryIntent
    ): Promise<ContextElement[]> {
        for (const element of elements) {
            element.priority = await this.priorityCalculator.calculatePriority(element, queryIntent);
            element.essential = await this.isEssentialElement(element, queryIntent);
        }

        // Sort by priority (highest first)
        return elements.sort((a, b) => b.priority - a.priority);
    }

    /**
     * Build optimized context within token limits
     */
    private async buildOptimizedContext(
        prioritizedElements: ContextElement[],
        maxTokens: number,
        queryIntent: QueryIntent
    ): Promise<CompleteAIContext> {
        // Start with essential elements
        const essentialElements = prioritizedElements.filter(e => e.essential);
        let currentTokens = await this.estimateTokensForElements(essentialElements);

        // Initialize optimized context with essential elements
        let optimizedContext = await this.buildContextFromElements(essentialElements);

        // Add additional elements by priority until token limit
        const remainingElements = prioritizedElements.filter(e => !e.essential);
        
        for (const element of remainingElements) {
            const elementTokens = element.token_count;
            
            if (currentTokens + elementTokens <= maxTokens) {
                // Add element as-is
                optimizedContext = await this.addElementToContext(optimizedContext, element);
                currentTokens += elementTokens;
            } else if (element.priority > 0.7 && element.compressible) {
                // Try compression for high-priority elements
                const compressed = await this.compressionEngine.compress(element);
                const compressedTokens = await this.tokenEstimator.estimateElement(compressed);
                
                if (currentTokens + compressedTokens <= maxTokens) {
                    optimizedContext = await this.addElementToContext(optimizedContext, compressed);
                    currentTokens += compressedTokens;
                }
            } else if (element.priority > 0.5) {
                // Add as reference for moderately important elements
                optimizedContext = await this.addElementReference(optimizedContext, element);
            }
        }

        return optimizedContext;
    }

    /**
     * Create a context element for a file
     */
    private async createFileElement(file: ContextFile, queryIntent: QueryIntent): Promise<ContextElement> {
        const tokenCount = await this.tokenEstimator.estimateText(file.complete_content);
        const priority = await this.calculateFilePriority(file, queryIntent);

        return {
            id: `file:${file.path}`,
            type: ElementType.CoreFile,
            content: file,
            priority: priority,
            token_count: tokenCount,
            essential: this.isEssentialFile(file, queryIntent),
            compressible: true
        };
    }

    /**
     * Decompose a file into smaller semantic elements
     */
    private async decomposeFile(file: ContextFile, queryIntent: QueryIntent): Promise<ContextElement[]> {
        const elements: ContextElement[] = [];

        // Create elements for functions
        for (const func of file.semantic_info.functions) {
            elements.push({
                id: `function:${file.path}:${func.name}`,
                type: ElementType.Function,
                content: func,
                priority: await this.calculateFunctionPriority(func, queryIntent),
                token_count: await this.estimateFunctionTokens(func),
                essential: this.isEssentialFunction(func, queryIntent),
                compressible: true
            });
        }

        // Create elements for classes
        for (const cls of file.semantic_info.classes) {
            elements.push({
                id: `class:${file.path}:${cls.name}`,
                type: ElementType.Class,
                content: cls,
                priority: await this.calculateClassPriority(cls, queryIntent),
                token_count: await this.estimateClassTokens(cls),
                essential: this.isEssentialClass(cls, queryIntent),
                compressible: true
            });
        }

        // Create elements for imports
        for (const imp of file.semantic_info.imports) {
            elements.push({
                id: `import:${file.path}:${imp.module}`,
                type: ElementType.Import,
                content: imp,
                priority: 0.3, // Generally low priority unless specifically relevant
                token_count: await this.estimateImportTokens(imp),
                essential: false,
                compressible: false // Imports are already minimal
            });
        }

        return elements;
    }

    /**
     * Decompose architectural context into elements
     */
    private async decomposeArchitecturalContext(
        architecture: any,
        queryIntent: QueryIntent
    ): Promise<ContextElement[]> {
        const elements: ContextElement[] = [];

        // Add project structure
        elements.push({
            id: "architecture:project_structure",
            type: ElementType.Pattern,
            content: architecture.project_structure,
            priority: queryIntent.query_type === QueryType.Architecture ? 0.9 : 0.4,
            token_count: await this.tokenEstimator.estimateObject(architecture.project_structure),
            essential: queryIntent.query_type === QueryType.Architecture,
            compressible: true
        });

        // Add design patterns
        for (let i = 0; i < architecture.patterns_used.length; i++) {
            const pattern = architecture.patterns_used[i];
            elements.push({
                id: `pattern:${i}`,
                type: ElementType.Pattern,
                content: pattern,
                priority: 0.6,
                token_count: await this.tokenEstimator.estimateObject(pattern),
                essential: false,
                compressible: true
            });
        }

        return elements;
    }

    /**
     * Decompose relationship context into elements
     */
    private async decomposeRelationships(
        relationships: any,
        queryIntent: QueryIntent
    ): Promise<ContextElement[]> {
        const elements: ContextElement[] = [];

        // Add call graph
        elements.push({
            id: "relationships:call_graph",
            type: ElementType.CallGraph,
            content: relationships.complete_call_graph,
            priority: queryIntent.query_type === QueryType.Debugging ? 0.8 : 0.5,
            token_count: await this.tokenEstimator.estimateObject(relationships.complete_call_graph),
            essential: queryIntent.query_type === QueryType.Debugging,
            compressible: true
        });

        return elements;
    }

    /**
     * Decompose history context into elements
     */
    private async decomposeHistory(
        history: any,
        queryIntent: QueryIntent
    ): Promise<ContextElement[]> {
        const elements: ContextElement[] = [];

        // Add recent changes
        elements.push({
            id: "history:recent_changes",
            type: ElementType.Documentation,
            content: history.change_timeline.slice(-5), // Last 5 changes
            priority: 0.4,
            token_count: await this.tokenEstimator.estimateObject(history.change_timeline.slice(-5)),
            essential: false,
            compressible: true
        });

        return elements;
    }

    /**
     * Decompose examples into elements
     */
    private async decomposeExamples(
        examples: any[],
        queryIntent: QueryIntent
    ): Promise<ContextElement[]> {
        const elements: ContextElement[] = [];

        for (let i = 0; i < examples.length; i++) {
            const example = examples[i];
            elements.push({
                id: `example:${i}`,
                type: ElementType.Example,
                content: example,
                priority: queryIntent.query_type === QueryType.Implementation ? 0.7 : 0.3,
                token_count: await this.tokenEstimator.estimateObject(example),
                essential: false,
                compressible: true
            });
        }

        return elements;
    }

    // Helper methods for priority calculation
    private async calculateFilePriority(file: ContextFile, queryIntent: QueryIntent): Promise<number> {
        let priority = 0.5; // Base priority

        // Boost priority if file contains entities mentioned in query
        for (const entity of queryIntent.entities) {
            if (this.fileContainsEntity(file, entity)) {
                priority += 0.3;
            }
        }

        // Boost priority for recently modified files
        const age = Date.now() - (file.last_modified?.getTime() || 0);
        const daysSinceModified = age / (1000 * 60 * 60 * 24);
        if (daysSinceModified < 7) {
            priority += 0.2;
        }

        return Math.min(priority, 1.0);
    }

    private async calculateFunctionPriority(func: any, queryIntent: QueryIntent): Promise<number> {
        let priority = 0.4; // Base priority

        // Boost if function is mentioned in query
        if (queryIntent.entities.some(e => e.name === func.name)) {
            priority += 0.5;
        }

        // Boost for complex functions (might be more important)
        if (func.complexity > 10) {
            priority += 0.1;
        }

        return Math.min(priority, 1.0);
    }

    private async calculateClassPriority(cls: any, queryIntent: QueryIntent): Promise<number> {
        let priority = 0.4; // Base priority

        // Boost if class is mentioned in query
        if (queryIntent.entities.some(e => e.name === cls.name)) {
            priority += 0.5;
        }

        // Boost for classes with many methods (likely important)
        if (cls.methods.length > 5) {
            priority += 0.1;
        }

        return Math.min(priority, 1.0);
    }

    // Helper methods for determining essential elements
    private isEssentialFile(file: ContextFile, queryIntent: QueryIntent): boolean {
        // File is essential if it contains entities mentioned in the query
        return queryIntent.entities.some(entity => this.fileContainsEntity(file, entity));
    }

    private isEssentialFunction(func: any, queryIntent: QueryIntent): boolean {
        return queryIntent.entities.some(e => e.name === func.name && e.type === "function");
    }

    private isEssentialClass(cls: any, queryIntent: QueryIntent): boolean {
        return queryIntent.entities.some(e => e.name === cls.name && e.type === "class");
    }

    private async isEssentialElement(element: ContextElement, queryIntent: QueryIntent): Promise<boolean> {
        // Elements are essential if they directly relate to query entities
        return element.priority > 0.8 || element.essential;
    }

    // Token estimation helpers
    private async estimateTotalTokens(elements: ContextElement[]): Promise<number> {
        return elements.reduce((total, element) => total + element.token_count, 0);
    }

    private async estimateTokensForElements(elements: ContextElement[]): Promise<number> {
        return elements.reduce((total, element) => total + element.token_count, 0);
    }

    private async estimateFunctionTokens(func: any): Promise<number> {
        // Rough estimation based on function complexity and documentation
        return 50 + (func.complexity * 10) + (func.documentation?.length || 0) / 4;
    }

    private async estimateClassTokens(cls: any): Promise<number> {
        // Rough estimation based on methods and properties
        return 100 + (cls.methods.length * 20) + (cls.properties.length * 10);
    }

    private async estimateImportTokens(imp: any): Promise<number> {
        return 10 + imp.imported_items.length * 2;
    }

    // Context building helpers
    private async buildContextFromElements(elements: ContextElement[]): Promise<CompleteAIContext> {
        // Build a complete context from selected elements
        // This is a simplified implementation
        return {
            core_files: elements.filter(e => e.type === ElementType.CoreFile).map(e => e.content),
            architecture: { 
                project_structure: { 
                    root_directories: [], 
                    module_structure: [], 
                    package_dependencies: [] 
                }, 
                patterns_used: [], 
                dependency_graph: { nodes: [], edges: [], cycles: [] }, 
                data_flow_diagram: { entry_points: [], data_transformations: [], storage_interactions: [] }, 
                layers: [] 
            },
            relationships: { complete_call_graph: { functions: [], relationships: [] }, type_hierarchy: { root_types: [], inheritance_chains: [], interface_implementations: [] }, import_graph: { modules: [], dependencies: [] }, usage_patterns: [] },
            history: { change_timeline: [], architectural_decisions: [], refactoring_history: [] },
            examples: elements.filter(e => e.type === ElementType.Example).map(e => e.content),
            metadata: {} as any,
            source_checkpoints: [],
            context_type: "semantic" as any,
            confidence_score: 0.8
        };
    }

    private async addElementToContext(context: CompleteAIContext, element: ContextElement): Promise<CompleteAIContext> {
        // Add element to appropriate section of context
        // This is a simplified implementation
        return context;
    }

    private async addElementReference(context: CompleteAIContext, element: ContextElement): Promise<CompleteAIContext> {
        // Add reference to element without full content
        return context;
    }

    // Utility methods
    private fileContainsEntity(file: ContextFile, entity: any): boolean {
        // Check if file contains the specified entity
        switch (entity.type) {
            case "function":
                return file.semantic_info.functions.some(f => f.name === entity.name);
            case "class":
                return file.semantic_info.classes.some(c => c.name === entity.name);
            default:
                return false;
        }
    }

    private isElementIncluded(element: ContextElement, context: CompleteAIContext): boolean {
        // Check if element was included in the optimized context
        return true; // Simplified implementation
    }

    private determineOptimizationStrategy(queryIntent: QueryIntent): string {
        switch (queryIntent.query_type) {
            case QueryType.Implementation:
                return "implementation-focused";
            case QueryType.Debugging:
                return "dependency-focused";
            case QueryType.Architecture:
                return "architecture-focused";
            default:
                return "balanced";
        }
    }

    private calculateConfidenceImpact(original: CompleteAIContext, optimized: CompleteAIContext): number {
        // Calculate how optimization affected confidence
        // Simplified: assume 5% confidence loss for every 20% compression
        const compressionRatio = optimized.core_files.length / original.core_files.length;
        return Math.max(0, 1 - ((1 - compressionRatio) * 0.25));
    }
}

/**
 * Token estimator for calculating token counts
 */
class TokenEstimator {
    async estimate(context: CompleteAIContext): Promise<number> {
        let totalTokens = 0;

        // Estimate tokens for core files
        for (const file of context.core_files) {
            totalTokens += await this.estimateText(file.complete_content);
        }

        // Add estimates for other context sections
        totalTokens += await this.estimateObject(context.architecture);
        totalTokens += await this.estimateObject(context.relationships);
        totalTokens += await this.estimateObject(context.history);
        totalTokens += await this.estimateObject(context.examples);

        return totalTokens;
    }

    async estimateText(text: string): Promise<number> {
        // Simple estimation: roughly 4 characters per token
        return Math.ceil(text.length / 4);
    }

    async estimateObject(obj: any): Promise<number> {
        const jsonString = JSON.stringify(obj);
        return this.estimateText(jsonString);
    }

    async estimateElement(element: ContextElement): Promise<number> {
        return element.token_count;
    }
}

/**
 * Priority calculator for context elements
 */
class PriorityCalculator {
    async calculatePriority(element: ContextElement, queryIntent: QueryIntent): Promise<number> {
        let priority = 0.5; // Base priority

        // Adjust based on element type and query type
        priority += this.getTypeRelevanceBoost(element.type, queryIntent.query_type);

        // Adjust based on entity relevance
        priority += this.getEntityRelevanceBoost(element, queryIntent);

        return Math.min(priority, 1.0);
    }

    private getTypeRelevanceBoost(elementType: ElementType, queryType: QueryType): number {
        const relevanceMatrix: Partial<Record<QueryType, Record<ElementType, number>>> = {
            [QueryType.Implementation]: {
                [ElementType.Function]: 0.4,
                [ElementType.Class]: 0.3,
                [ElementType.Interface]: 0.3,
                [ElementType.Example]: 0.3,
                [ElementType.CoreFile]: 0.2,
                [ElementType.Type]: 0.2,
                [ElementType.Import]: 0.1,
                [ElementType.Export]: 0.1,
                [ElementType.CallGraph]: 0.1,
                [ElementType.Dependency]: 0.1,
                [ElementType.Pattern]: 0.1,
                [ElementType.Documentation]: 0.0
            },
            [QueryType.Debugging]: {
                [ElementType.CallGraph]: 0.4,
                [ElementType.Dependency]: 0.3,
                [ElementType.Function]: 0.3,
                [ElementType.CoreFile]: 0.2,
                [ElementType.Class]: 0.2,
                [ElementType.Import]: 0.2,
                [ElementType.Interface]: 0.1,
                [ElementType.Type]: 0.1,
                [ElementType.Export]: 0.1,
                [ElementType.Pattern]: 0.1,
                [ElementType.Example]: 0.1,
                [ElementType.Documentation]: 0.0
            },
            [QueryType.Architecture]: {
                [ElementType.Pattern]: 0.4,
                [ElementType.Dependency]: 0.3,
                [ElementType.CoreFile]: 0.2,
                [ElementType.Class]: 0.2,
                [ElementType.Interface]: 0.2,
                [ElementType.CallGraph]: 0.2,
                [ElementType.Function]: 0.1,
                [ElementType.Type]: 0.1,
                [ElementType.Import]: 0.1,
                [ElementType.Export]: 0.1,
                [ElementType.Example]: 0.1,
                [ElementType.Documentation]: 0.1
            },
            [QueryType.Refactoring]: {
                [ElementType.Pattern]: 0.3,
                [ElementType.Class]: 0.3,
                [ElementType.Function]: 0.3,
                [ElementType.Dependency]: 0.2,
                [ElementType.CoreFile]: 0.2,
                [ElementType.CallGraph]: 0.2,
                [ElementType.Interface]: 0.1,
                [ElementType.Type]: 0.1,
                [ElementType.Import]: 0.1,
                [ElementType.Export]: 0.1,
                [ElementType.Example]: 0.1,
                [ElementType.Documentation]: 0.0
            },
            [QueryType.Explanation]: {
                [ElementType.Documentation]: 0.4,
                [ElementType.Function]: 0.3,
                [ElementType.Class]: 0.3,
                [ElementType.CoreFile]: 0.2,
                [ElementType.Interface]: 0.2,
                [ElementType.Type]: 0.2,
                [ElementType.Example]: 0.2,
                [ElementType.Pattern]: 0.1,
                [ElementType.CallGraph]: 0.1,
                [ElementType.Dependency]: 0.1,
                [ElementType.Import]: 0.0,
                [ElementType.Export]: 0.0
            },
            [QueryType.Testing]: {
                [ElementType.Function]: 0.4,
                [ElementType.Class]: 0.3,
                [ElementType.Example]: 0.3,
                [ElementType.CoreFile]: 0.2,
                [ElementType.Interface]: 0.2,
                [ElementType.CallGraph]: 0.1,
                [ElementType.Dependency]: 0.1,
                [ElementType.Type]: 0.1,
                [ElementType.Pattern]: 0.1,
                [ElementType.Import]: 0.0,
                [ElementType.Export]: 0.0,
                [ElementType.Documentation]: 0.0
            },
            [QueryType.Documentation]: {
                [ElementType.Documentation]: 0.5,
                [ElementType.Function]: 0.2,
                [ElementType.Class]: 0.2,
                [ElementType.Interface]: 0.2,
                [ElementType.Type]: 0.2,
                [ElementType.Example]: 0.2,
                [ElementType.CoreFile]: 0.1,
                [ElementType.Pattern]: 0.1,
                [ElementType.CallGraph]: 0.0,
                [ElementType.Dependency]: 0.0,
                [ElementType.Import]: 0.0,
                [ElementType.Export]: 0.0
            }
        };

        return relevanceMatrix[queryType]?.[elementType] || 0;
    }

    private getEntityRelevanceBoost(element: ContextElement, queryIntent: QueryIntent): number {
        // Check if element relates to entities mentioned in query
        for (const entity of queryIntent.entities) {
            if (this.elementMatchesEntity(element, entity)) {
                return entity.confidence * 0.3; // Boost based on entity confidence
            }
        }
        return 0;
    }

    private elementMatchesEntity(element: ContextElement, entity: any): boolean {
        // Check if element matches the entity
        if (element.type === ElementType.Function && entity.type === "function") {
            return element.content.name === entity.name;
        }
        if (element.type === ElementType.Class && entity.type === "class") {
            return element.content.name === entity.name;
        }
        return false;
    }
}

/**
 * Context compression engine for reducing token usage while preserving meaning
 */
class ContextCompressionEngine {
    async compress(element: ContextElement): Promise<ContextElement> {
        let compressedContent = element.content;
        let compressionRatio = 1.0;

        switch (element.type) {
            case ElementType.CoreFile:
                compressedContent = await this.compressFile(element.content);
                compressionRatio = 0.7; // Assume 30% compression
                break;

            case ElementType.Function:
                compressedContent = await this.compressFunction(element.content);
                compressionRatio = 0.8; // Assume 20% compression
                break;

            case ElementType.Documentation:
                compressedContent = await this.compressDocumentation(element.content);
                compressionRatio = 0.6; // Assume 40% compression
                break;

            default:
                // No compression for other types
                break;
        }

        return {
            ...element,
            content: compressedContent,
            token_count: Math.floor(element.token_count * compressionRatio)
        };
    }

    private async compressFile(file: any): Promise<any> {
        // Remove comments and unnecessary whitespace
        let compressedContent = file.complete_content
            .replace(/\/\*[\s\S]*?\*\//g, '') // Remove block comments
            .replace(/\/\/.*$/gm, '') // Remove line comments
            .replace(/\s+/g, ' ') // Collapse whitespace
            .trim();

        return {
            ...file,
            complete_content: compressedContent
        };
    }

    private async compressFunction(func: any): Promise<any> {
        // Keep signature and key information, summarize body
        return {
            ...func,
            documentation: func.documentation ? this.summarizeText(func.documentation) : undefined
        };
    }

    private async compressDocumentation(docs: any): Promise<any> {
        // Summarize documentation to key points
        if (typeof docs === 'string') {
            return this.summarizeText(docs);
        }
        return docs;
    }

    private summarizeText(text: string): string {
        // Simple text summarization - keep first and last sentences
        const sentences = text.split(/[.!?]+/).filter(s => s.trim().length > 0);
        if (sentences.length <= 2) return text;
        
        return `${sentences[0].trim()}. ... ${sentences[sentences.length - 1].trim()}.`;
    }
}
