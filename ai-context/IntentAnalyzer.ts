/**
 * Intent Analyzer - Analyzes query intent for context building
 */

export interface QueryIntent {
    original_query: string;
    query_type: QueryType;
    scope: QueryScope;
    entities: CodeEntity[];
    context_requirements: ContextRequirement[];
    priority_indicators: PriorityIndicator[];
    expected_response_type: ResponseType;
    confidence: number;
}

export enum QueryType {
    Implementation = "implementation",
    Debugging = "debugging",
    Refactoring = "refactoring", 
    Explanation = "explanation",
    Architecture = "architecture",
    Testing = "testing",
    Documentation = "documentation",
    Performance = "performance",
    Security = "security"
}

export enum QueryScope {
    Function = "function",
    Class = "class",
    Module = "module", 
    Package = "package",
    System = "system",
    Component = "component"
}

export interface CodeEntity {
    name: string;
    type: EntityType;
    confidence: number;
    location?: string;
    context_hint?: string;
}

export enum EntityType {
    Function = "function",
    Class = "class",
    Interface = "interface",
    Type = "type",
    Variable = "variable",
    Constant = "constant",
    Module = "module",
    Package = "package",
    Component = "component",
    Service = "service"
}

export interface ContextRequirement {
    requirement_type: ContextType;
    priority: number;
    reasoning: string;
    scope: string[];
}

export enum ContextType {
    SemanticContext = "semantic",
    ArchitecturalContext = "architectural",
    EvolutionContext = "evolution", 
    DependencyContext = "dependency",
    UsageContext = "usage",
    ExampleContext = "example",
    TestingContext = "testing",
    DocumentationContext = "documentation"
}

export interface PriorityIndicator {
    indicator: string;
    weight: number;
    reasoning: string;
    category: PriorityCategory;
}

export enum PriorityCategory {
    Urgency = "urgency",
    Complexity = "complexity",
    Impact = "impact",
    Specificity = "specificity"
}

export enum ResponseType {
    CodeImplementation = "code_implementation",
    Explanation = "explanation", 
    Debugging = "debugging",
    Refactoring = "refactoring",
    Architecture = "architecture",
    Testing = "testing",
    Documentation = "documentation"
}

export class IntentAnalyzer {
    private entityExtractor: CodeEntityExtractor;
    private patternMatcher: PatternMatcher;
    private contextPredictor: ContextPredictor;

    constructor() {
        this.entityExtractor = new CodeEntityExtractor();
        this.patternMatcher = new PatternMatcher();
        this.contextPredictor = new ContextPredictor();
    }

    /**
     * Analyze query to understand intent and requirements
     */
    async analyzeQuery(query: string): Promise<QueryIntent> {
        // Extract code entities mentioned in the query
        const entities = await this.entityExtractor.extract(query);
        
        // Classify the query type
        const queryType = await this.classifyQueryType(query);
        
        // Determine the scope of the query
        const scope = await this.determineScopeRequirements(query, entities);
        
        // Identify required context types
        const contextRequirements = await this.identifyContextRequirements(
            query,
            queryType,
            entities
        );
        
        // Extract priority indicators
        const priorityIndicators = await this.extractPriorityIndicators(query);
        
        // Predict expected response type
        const expectedResponseType = await this.predictResponseType(query, queryType);
        
        // Calculate overall confidence
        const confidence = this.calculateAnalysisConfidence(
            query,
            entities,
            queryType,
            contextRequirements
        );

        return {
            original_query: query,
            query_type: queryType,
            scope,
            entities,
            context_requirements: contextRequirements,
            priority_indicators: priorityIndicators,
            expected_response_type: expectedResponseType,
            confidence
        };
    }

    /**
     * Classify the type of query based on content and patterns
     */
    private async classifyQueryType(query: string): Promise<QueryType> {
        const queryLower = query.toLowerCase();
        
        // Define pattern weights for different query types
        const patterns: Record<QueryType, { patterns: RegExp[]; weight: number }> = {
            [QueryType.Implementation]: {
                patterns: [
                    /how to|implement|create|build|add|write|make|develop/i,
                    /function|method|class|component/i,
                    /code|solution|example/i
                ],
                weight: 1.0
            },
            [QueryType.Debugging]: {
                patterns: [
                    /error|bug|fix|issue|problem|not working|debug|broken/i,
                    /why.*not|what.*wrong|doesn't work/i,
                    /exception|crash|fail/i
                ],
                weight: 1.0
            },
            [QueryType.Refactoring]: {
                patterns: [
                    /refactor|improve|optimize|clean up|restructure|reorganize/i,
                    /better way|best practice|code smell/i,
                    /performance|efficiency/i
                ],
                weight: 1.0
            },
            [QueryType.Explanation]: {
                patterns: [
                    /what is|explain|how does|understand|clarify|describe/i,
                    /what.*do|how.*work|why.*use/i,
                    /meaning|purpose|concept/i
                ],
                weight: 1.0
            },
            [QueryType.Architecture]: {
                patterns: [
                    /architecture|design|pattern|structure|organize|system/i,
                    /module|component|service|layer/i,
                    /dependency|relationship|interaction/i
                ],
                weight: 1.0
            },
            [QueryType.Testing]: {
                patterns: [
                    /test|testing|unit test|integration test|e2e/i,
                    /mock|stub|assertion|coverage/i,
                    /spec|specification/i
                ],
                weight: 1.0
            },
            [QueryType.Documentation]: {
                patterns: [
                    /document|documentation|comment|jsdoc|readme/i,
                    /api.*doc|guide|tutorial/i,
                    /example.*usage/i
                ],
                weight: 1.0
            },
            [QueryType.Performance]: {
                patterns: [
                    /performance|speed|slow|fast|optimize|efficiency/i,
                    /memory|cpu|benchmark|profile/i,
                    /bottleneck|latency/i
                ],
                weight: 1.0
            },
            [QueryType.Security]: {
                patterns: [
                    /security|secure|vulnerability|auth|permission/i,
                    /encrypt|hash|sanitize|validate/i,
                    /xss|csrf|injection/i
                ],
                weight: 1.0
            }
        };

        // Score each query type
        const scores: Record<QueryType, number> = {} as any;
        
        for (const [type, config] of Object.entries(patterns)) {
            scores[type as QueryType] = 0;
            
            for (const pattern of config.patterns) {
                const matches = queryLower.match(pattern);
                if (matches) {
                    scores[type as QueryType] += config.weight * matches.length;
                }
            }
        }

        // Find the highest scoring type
        const sortedTypes = Object.entries(scores)
            .sort(([, a], [, b]) => b - a)
            .map(([type]) => type as QueryType);

        return sortedTypes[0] || QueryType.Explanation;
    }

    /**
     * Determine the scope of the query
     */
    private async determineScopeRequirements(
        query: string,
        entities: CodeEntity[]
    ): Promise<QueryScope> {
        const queryLower = query.toLowerCase();
        
        // Check for explicit scope indicators
        if (queryLower.includes('system') || queryLower.includes('application') || queryLower.includes('entire')) {
            return QueryScope.System;
        }
        
        if (queryLower.includes('package') || queryLower.includes('library')) {
            return QueryScope.Package;
        }
        
        if (queryLower.includes('module') || queryLower.includes('file')) {
            return QueryScope.Module;
        }
        
        if (queryLower.includes('component') || queryLower.includes('service')) {
            return QueryScope.Component;
        }
        
        if (queryLower.includes('class') || queryLower.includes('interface')) {
            return QueryScope.Class;
        }
        
        if (queryLower.includes('function') || queryLower.includes('method')) {
            return QueryScope.Function;
        }

        // Infer scope from entities
        const entityTypes = entities.map(e => e.type);
        if (entityTypes.includes(EntityType.Class)) {
            return QueryScope.Class;
        }
        if (entityTypes.includes(EntityType.Function)) {
            return QueryScope.Function;
        }
        if (entityTypes.includes(EntityType.Module)) {
            return QueryScope.Module;
        }
        if (entityTypes.includes(EntityType.Component)) {
            return QueryScope.Component;
        }

        // Default to module scope
        return QueryScope.Module;
    }

    /**
     * Identify what types of context are needed for this query
     */
    private async identifyContextRequirements(
        query: string,
        queryType: QueryType,
        entities: CodeEntity[]
    ): Promise<ContextRequirement[]> {
        const requirements: ContextRequirement[] = [];
        const queryLower = query.toLowerCase();

        // Base requirements by query type
        switch (queryType) {
            case QueryType.Implementation:
                requirements.push({
                    requirement_type: ContextType.SemanticContext,
                    priority: 0.9,
                    reasoning: "Need semantic understanding for implementation",
                    scope: entities.map(e => e.name)
                });
                requirements.push({
                    requirement_type: ContextType.ExampleContext,
                    priority: 0.7,
                    reasoning: "Examples help with implementation patterns",
                    scope: ["similar_implementations"]
                });
                break;

            case QueryType.Debugging:
                requirements.push({
                    requirement_type: ContextType.DependencyContext,
                    priority: 0.9,
                    reasoning: "Dependencies often cause bugs",
                    scope: ["direct_dependencies", "transitive_dependencies"]
                });
                requirements.push({
                    requirement_type: ContextType.UsageContext,
                    priority: 0.8,
                    reasoning: "Usage patterns reveal common issues",
                    scope: ["call_patterns", "error_patterns"]
                });
                break;

            case QueryType.Refactoring:
                requirements.push({
                    requirement_type: ContextType.ArchitecturalContext,
                    priority: 0.9,
                    reasoning: "Architecture understanding needed for refactoring",
                    scope: ["design_patterns", "dependencies"]
                });
                requirements.push({
                    requirement_type: ContextType.TestingContext,
                    priority: 0.7,
                    reasoning: "Tests ensure refactoring safety",
                    scope: ["test_coverage", "test_cases"]
                });
                break;

            case QueryType.Architecture:
                requirements.push({
                    requirement_type: ContextType.ArchitecturalContext,
                    priority: 1.0,
                    reasoning: "Primary requirement for architectural queries",
                    scope: ["system_design", "patterns", "boundaries"]
                });
                requirements.push({
                    requirement_type: ContextType.EvolutionContext,
                    priority: 0.6,
                    reasoning: "Evolution shows architectural decisions",
                    scope: ["design_decisions", "refactoring_history"]
                });
                break;

            case QueryType.Explanation:
                requirements.push({
                    requirement_type: ContextType.SemanticContext,
                    priority: 0.8,
                    reasoning: "Semantic understanding for explanations",
                    scope: entities.map(e => e.name)
                });
                requirements.push({
                    requirement_type: ContextType.DocumentationContext,
                    priority: 0.7,
                    reasoning: "Documentation provides context for explanations",
                    scope: ["comments", "docs", "examples"]
                });
                break;

            case QueryType.Testing:
                requirements.push({
                    requirement_type: ContextType.TestingContext,
                    priority: 1.0,
                    reasoning: "Primary requirement for testing queries",
                    scope: ["test_patterns", "mocking", "assertions"]
                });
                requirements.push({
                    requirement_type: ContextType.SemanticContext,
                    priority: 0.8,
                    reasoning: "Need to understand code being tested",
                    scope: entities.map(e => e.name)
                });
                break;
        }

        // Additional requirements based on query content
        if (queryLower.includes('history') || queryLower.includes('change') || queryLower.includes('evolution')) {
            requirements.push({
                requirement_type: ContextType.EvolutionContext,
                priority: 0.8,
                reasoning: "Query explicitly mentions historical context",
                scope: ["change_timeline", "evolution_patterns"]
            });
        }

        if (queryLower.includes('dependency') || queryLower.includes('import') || queryLower.includes('require')) {
            requirements.push({
                requirement_type: ContextType.DependencyContext,
                priority: 0.9,
                reasoning: "Query explicitly mentions dependencies",
                scope: ["dependency_graph", "import_analysis"]
            });
        }

        if (queryLower.includes('example') || queryLower.includes('sample') || queryLower.includes('demo')) {
            requirements.push({
                requirement_type: ContextType.ExampleContext,
                priority: 0.8,
                reasoning: "Query explicitly requests examples",
                scope: ["code_examples", "usage_examples"]
            });
        }

        return requirements;
    }

    /**
     * Extract priority indicators from the query
     */
    private async extractPriorityIndicators(query: string): Promise<PriorityIndicator[]> {
        const indicators: PriorityIndicator[] = [];
        const queryLower = query.toLowerCase();

        // Urgency indicators
        const urgencyPatterns = [
            { pattern: /urgent|asap|immediately|quickly|fast/i, weight: 1.0 },
            { pattern: /need.*now|right now/i, weight: 0.9 },
            { pattern: /deadline|due/i, weight: 0.8 }
        ];

        for (const { pattern, weight } of urgencyPatterns) {
            if (pattern.test(query)) {
                indicators.push({
                    indicator: pattern.source,
                    weight,
                    reasoning: "Query indicates urgency",
                    category: PriorityCategory.Urgency
                });
            }
        }

        // Complexity indicators
        const complexityPatterns = [
            { pattern: /complex|complicated|difficult|advanced/i, weight: 0.9 },
            { pattern: /simple|easy|basic|straightforward/i, weight: 0.3 },
            { pattern: /enterprise|production|scalable/i, weight: 0.8 }
        ];

        for (const { pattern, weight } of complexityPatterns) {
            if (pattern.test(query)) {
                indicators.push({
                    indicator: pattern.source,
                    weight,
                    reasoning: "Query indicates complexity level",
                    category: PriorityCategory.Complexity
                });
            }
        }

        // Impact indicators
        const impactPatterns = [
            { pattern: /critical|important|essential|vital/i, weight: 1.0 },
            { pattern: /breaking|major|significant/i, weight: 0.9 },
            { pattern: /minor|small|trivial/i, weight: 0.2 }
        ];

        for (const { pattern, weight } of impactPatterns) {
            if (pattern.test(query)) {
                indicators.push({
                    indicator: pattern.source,
                    weight,
                    reasoning: "Query indicates impact level",
                    category: PriorityCategory.Impact
                });
            }
        }

        // Specificity indicators
        const specificityPatterns = [
            { pattern: /specific|exactly|precisely|particular/i, weight: 0.9 },
            { pattern: /general|overview|broadly|in general/i, weight: 0.4 },
            { pattern: /detailed|comprehensive|thorough/i, weight: 0.8 }
        ];

        for (const { pattern, weight } of specificityPatterns) {
            if (pattern.test(query)) {
                indicators.push({
                    indicator: pattern.source,
                    weight,
                    reasoning: "Query indicates specificity level",
                    category: PriorityCategory.Specificity
                });
            }
        }

        return indicators;
    }

    /**
     * Predict the expected response type
     */
    private async predictResponseType(query: string, queryType: QueryType): Promise<ResponseType> {
        const queryLower = query.toLowerCase();

        // Direct response type indicators
        if (queryLower.includes('show me code') || queryLower.includes('write') || queryLower.includes('implement')) {
            return ResponseType.CodeImplementation;
        }

        if (queryLower.includes('explain') || queryLower.includes('what is') || queryLower.includes('how does')) {
            return ResponseType.Explanation;
        }

        if (queryLower.includes('fix') || queryLower.includes('debug') || queryLower.includes('error')) {
            return ResponseType.Debugging;
        }

        if (queryLower.includes('refactor') || queryLower.includes('improve') || queryLower.includes('optimize')) {
            return ResponseType.Refactoring;
        }

        if (queryLower.includes('test') || queryLower.includes('testing')) {
            return ResponseType.Testing;
        }

        if (queryLower.includes('document') || queryLower.includes('comment')) {
            return ResponseType.Documentation;
        }

        // Map query type to response type
        switch (queryType) {
            case QueryType.Implementation:
                return ResponseType.CodeImplementation;
            case QueryType.Debugging:
                return ResponseType.Debugging;
            case QueryType.Refactoring:
                return ResponseType.Refactoring;
            case QueryType.Architecture:
                return ResponseType.Architecture;
            case QueryType.Testing:
                return ResponseType.Testing;
            case QueryType.Documentation:
                return ResponseType.Documentation;
            default:
                return ResponseType.Explanation;
        }
    }

    /**
     * Calculate confidence in the analysis
     */
    private calculateAnalysisConfidence(
        query: string,
        entities: CodeEntity[],
        queryType: QueryType,
        contextRequirements: ContextRequirement[]
    ): number {
        let confidence = 0.5; // Base confidence

        // Boost confidence based on entity extraction
        if (entities.length > 0) {
            const avgEntityConfidence = entities.reduce((sum, e) => sum + e.confidence, 0) / entities.length;
            confidence += avgEntityConfidence * 0.3;
        }

        // Boost confidence based on query clarity
        const queryWords = query.split(/\s+/).length;
        if (queryWords >= 5 && queryWords <= 20) {
            confidence += 0.2; // Good length queries are clearer
        }

        // Boost confidence based on specific indicators
        const specificIndicators = [
            'function', 'class', 'method', 'variable', 'component',
            'implement', 'create', 'fix', 'debug', 'refactor'
        ];
        
        const foundIndicators = specificIndicators.filter(indicator => 
            query.toLowerCase().includes(indicator)
        );
        
        confidence += foundIndicators.length * 0.05;

        // Boost confidence based on context requirements specificity
        const specificRequirements = contextRequirements.filter(req => req.priority > 0.7);
        confidence += specificRequirements.length * 0.05;

        return Math.min(confidence, 1.0);
    }
}

// Supporting classes (simplified implementations)

class CodeEntityExtractor {
    async extract(query: string): Promise<CodeEntity[]> {
        const entities: CodeEntity[] = [];
        const queryLower = query.toLowerCase();

        // Simple pattern-based entity extraction
        const patterns = [
            { type: EntityType.Function, pattern: /(\w+)\s*\(.*\)/g },
            { type: EntityType.Class, pattern: /class\s+(\w+)/gi },
            { type: EntityType.Component, pattern: /(\w+Component|\w+Service|\w+Controller)/gi },
            { type: EntityType.Variable, pattern: /variable\s+(\w+)/gi },
            { type: EntityType.Module, pattern: /module\s+(\w+)/gi }
        ];

        for (const { type, pattern } of patterns) {
            let match;
            while ((match = pattern.exec(query)) !== null) {
                entities.push({
                    name: match[1],
                    type,
                    confidence: 0.8,
                    context_hint: match[0]
                });
            }
        }

        // Look for camelCase/PascalCase identifiers
        const identifierPattern = /\b[A-Z][a-zA-Z0-9]*|[a-z][a-zA-Z0-9]*[A-Z][a-zA-Z0-9]*/g;
        let match;
        while ((match = identifierPattern.exec(query)) !== null) {
            const name = match[0];
            if (name.length > 2 && !entities.some(e => e.name === name)) {
                entities.push({
                    name,
                    type: name[0] === name[0].toUpperCase() ? EntityType.Class : EntityType.Function,
                    confidence: 0.6
                });
            }
        }

        return entities;
    }
}

class PatternMatcher {
    // Pattern matching utilities
}

class ContextPredictor {
    // Context prediction utilities
}
