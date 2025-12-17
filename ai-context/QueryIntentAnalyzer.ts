/**
 * Query Intent Analyzer - Analyzes user queries to understand intent and requirements
 * 
 * This service uses NLP and pattern matching to understand what the user is asking for
 * and what context will be needed to provide a complete answer.
 */

import { QueryIntent, QueryType, QueryScope, EntityType, ContextType, ResponseType, CodeEntity, ContextRequirement, PriorityIndicator } from './AIContextBuilder';

export class QueryIntentAnalyzer {
    private nlpProcessor: NLPProcessor;
    private codeEntityExtractor: CodeEntityExtractor;
    private patternMatcher: PatternMatcher;

    constructor() {
        this.nlpProcessor = new NLPProcessor();
        this.codeEntityExtractor = new CodeEntityExtractor();
        this.patternMatcher = new PatternMatcher();
    }

    /**
     * Analyze a user query to understand intent and context requirements
     */
    async analyzeQuery(query: string): Promise<QueryIntent> {
        // Extract code entities (functions, classes, variables, etc.)
        const codeEntities = await this.codeEntityExtractor.extract(query);
        
        // Classify query type
        const queryType = await this.classifyQueryType(query);
        
        // Determine scope and context requirements
        const scope = await this.determineScopeRequirements(query, codeEntities);
        
        // Identify required context types
        const contextRequirements = await this.identifyContextRequirements(
            query, 
            queryType, 
            codeEntities
        );

        // Extract priority indicators
        const priorityIndicators = await this.extractPriorityIndicators(query);

        // Predict expected response type
        const expectedResponseType = await this.predictResponseType(query, queryType);

        // Calculate overall confidence based on individual component confidences
        const confidence = this.calculateOverallConfidence(
            codeEntities,
            contextRequirements,
            priorityIndicators
        );

        return {
            original_query: query,
            query_type: queryType,
            scope: scope,
            entities: codeEntities,
            context_requirements: contextRequirements,
            priority_indicators: priorityIndicators,
            expected_response_type: expectedResponseType,
            confidence: confidence
        };
    }

    /**
     * Calculate overall confidence based on individual component confidences
     */
    private calculateOverallConfidence(
        entities: CodeEntity[],
        requirements: ContextRequirement[],
        priorities: PriorityIndicator[]
    ): number {
        // Average entity confidence
        const entityConfidence = entities.length > 0 
            ? entities.reduce((sum, entity) => sum + entity.confidence, 0) / entities.length 
            : 0.5;
        
        // Requirement confidence (higher if more specific requirements)
        const requirementConfidence = Math.min(requirements.length * 0.2 + 0.3, 1.0);
        
        // Priority confidence (higher if more priority indicators found)
        const priorityConfidence = Math.min(priorities.length * 0.15 + 0.4, 1.0);
        
        // Weighted average
        return (entityConfidence * 0.5 + requirementConfidence * 0.3 + priorityConfidence * 0.2);
    }

    /**
     * Classify the type of query based on patterns and keywords
     */
    private async classifyQueryType(query: string): Promise<QueryType> {
        const patterns = {
            implementation: /(?:how to|implement|create|build|add|write|make|develop|code|generate)/i,
            debugging: /(?:error|bug|fix|issue|problem|not working|debug|broken|fails?|crash)/i,
            refactoring: /(?:refactor|improve|optimize|restructure|clean up|reorganize|simplify)/i,
            explanation: /(?:what is|explain|how does|understand|clarify|describe|tell me about)/i,
            architecture: /(?:architecture|design|pattern|structure|organize|system|framework)/i,
            testing: /(?:test|testing|unit test|integration test|mock|spec|coverage)/i,
            documentation: /(?:document|docs|comment|readme|guide|tutorial)/i,
        };

        // Check patterns in order of specificity
        for (const type in patterns) {
            const pattern = patterns[type as keyof typeof patterns];
            if (pattern.test(query)) {
                return QueryType[type as keyof typeof QueryType];
            }
        }

        // Use ML-based classification for ambiguous cases
        return await this.mlClassifyQuery(query);
    }

    /**
     * Determine the scope of the query (function, class, module, etc.)
     */
    private async determineScopeRequirements(query: string, entities: CodeEntity[]): Promise<QueryScope> {
        // Analyze scope indicators in the query
        const scopePatterns = {
            function: /(?:function|method|procedure|routine)/i,
            class: /(?:class|object|constructor|instance)/i,
            module: /(?:module|file|component|service)/i,
            package: /(?:package|library|framework|dependency)/i,
            system: /(?:system|application|architecture|entire|whole|all)/i,
        };

        for (const scope in scopePatterns) {
            const pattern = scopePatterns[scope as keyof typeof scopePatterns];
            if (pattern.test(query)) {
                return QueryScope[scope as keyof typeof QueryScope];
            }
        }

        // Infer scope from entities
        if (entities.some(e => e.type === EntityType.Function)) {
            return QueryScope.Function;
        } else if (entities.some(e => e.type === EntityType.Class)) {
            return QueryScope.Class;
        } else if (entities.some(e => e.type === EntityType.Module)) {
            return QueryScope.Module;
        }

        return QueryScope.Function; // Default scope
    }

    /**
     * Identify what types of context will be needed
     */
    private async identifyContextRequirements(
        query: string,
        queryType: QueryType,
        entities: CodeEntity[]
    ): Promise<ContextRequirement[]> {
        const requirements: ContextRequirement[] = [];

        // Base requirements by query type
        switch (queryType) {
            case QueryType.Implementation:
                requirements.push(
                    { requirement_type: ContextType.SemanticContext, priority: 0.9, reasoning: "Need function signatures and types" },
                    { requirement_type: ContextType.ArchitecturalContext, priority: 0.7, reasoning: "Need to understand system design" },
                    { requirement_type: ContextType.ExampleContext, priority: 0.6, reasoning: "Examples help with implementation" }
                );
                break;

            case QueryType.Debugging:
                requirements.push(
                    { requirement_type: ContextType.DependencyContext, priority: 0.9, reasoning: "Need to trace dependencies" },
                    { requirement_type: ContextType.SemanticContext, priority: 0.8, reasoning: "Need complete function context" },
                    { requirement_type: ContextType.EvolutionContext, priority: 0.6, reasoning: "Recent changes might be relevant" }
                );
                break;

            case QueryType.Refactoring:
                requirements.push(
                    { requirement_type: ContextType.ArchitecturalContext, priority: 0.9, reasoning: "Need architectural understanding" },
                    { requirement_type: ContextType.DependencyContext, priority: 0.8, reasoning: "Need impact analysis" },
                    { requirement_type: ContextType.UsageContext, priority: 0.7, reasoning: "Need to understand usage patterns" }
                );
                break;

            case QueryType.Architecture:
                requirements.push(
                    { requirement_type: ContextType.ArchitecturalContext, priority: 1.0, reasoning: "Primary focus on architecture" },
                    { requirement_type: ContextType.EvolutionContext, priority: 0.8, reasoning: "Evolution shows architectural decisions" },
                    { requirement_type: ContextType.DependencyContext, priority: 0.7, reasoning: "Dependencies show system structure" }
                );
                break;

            default:
                requirements.push(
                    { requirement_type: ContextType.SemanticContext, priority: 0.8, reasoning: "General semantic understanding needed" }
                );
        }

        // Additional requirements based on entities
        if (entities.some(e => e.type === EntityType.Class)) {
            requirements.push({
                requirement_type: ContextType.SemanticContext,
                priority: 0.9,
                reasoning: "Class context needed for inheritance and methods"
            });
        }

        if (entities.some(e => e.type === EntityType.Function)) {
            requirements.push({
                requirement_type: ContextType.DependencyContext,
                priority: 0.7,
                reasoning: "Function dependencies and call chains needed"
            });
        }

        return requirements;
    }

    /**
     * Extract priority indicators from the query
     */
    private async extractPriorityIndicators(query: string): Promise<PriorityIndicator[]> {
        const indicators: PriorityIndicator[] = [];

        // Urgency indicators
        if (/(?:urgent|asap|quickly|immediately|now)/i.test(query)) {
            indicators.push({
                indicator: "urgency",
                weight: 0.8,
                reasoning: "Query contains urgency keywords"
            });
        }

        // Complexity indicators
        if (/(?:complex|complicated|advanced|sophisticated)/i.test(query)) {
            indicators.push({
                indicator: "complexity",
                weight: 0.7,
                reasoning: "Query indicates complex requirements"
            });
        }

        // Specific entity mentions
        const entityCount = (query.match(/[A-Z][a-zA-Z]*(?:Class|Function|Interface|Service)/g) || []).length;
        if (entityCount > 0) {
            indicators.push({
                indicator: "entity_specificity",
                weight: 0.6 + (entityCount * 0.1),
                reasoning: `Query mentions ${entityCount} specific entities`
            });
        }

        // Error/problem indicators
        if (/(?:error|problem|issue|broken|not working)/i.test(query)) {
            indicators.push({
                indicator: "problem_solving",
                weight: 0.9,
                reasoning: "Query indicates a problem that needs solving"
            });
        }

        return indicators;
    }

    /**
     * Predict what type of response the user expects
     */
    private async predictResponseType(query: string, queryType: QueryType): Promise<ResponseType> {
        // Direct response type mapping
        switch (queryType) {
            case QueryType.Implementation:
                return ResponseType.CodeImplementation;
            case QueryType.Debugging:
                return ResponseType.Debugging;
            case QueryType.Refactoring:
                return ResponseType.Refactoring;
            case QueryType.Architecture:
                return ResponseType.Architecture;
            default:
                return ResponseType.Explanation;
        }
    }

    /**
     * Advanced ML-based query classification with confidence scoring
     */
    private async mlClassifyQuery(query: string): Promise<QueryType> {
        const mlClassifier = new MLQueryClassifier();
        return await mlClassifier.classifyWithConfidence(query);
    }

    /**
     * Enhanced context requirements with predictive analysis
     */
    private async identifyContextRequirementsAdvanced(
        query: string,
        queryType: QueryType,
        entities: CodeEntity[]
    ): Promise<ContextRequirement[]> {
        const baseRequirements = await this.identifyContextRequirements(query, queryType, entities);
        
        // Add predictive context requirements based on query patterns
        const predictiveRequirements = await this.predictAdditionalContextNeeds(query, queryType, entities);
        
        // Merge and prioritize requirements
        return this.mergeAndPrioritizeRequirements(baseRequirements, predictiveRequirements);
    }

    /**
     * Predict additional context needs based on query analysis
     */
    private async predictAdditionalContextNeeds(
        query: string, 
        queryType: QueryType, 
        entities: CodeEntity[]
    ): Promise<ContextRequirement[]> {
        const additionalRequirements: ContextRequirement[] = [];

        // If query mentions testing, add test context
        if (/test|testing|spec|mock/i.test(query)) {
            additionalRequirements.push({
                requirement_type: ContextType.ExampleContext, // Use existing context type
                priority: 0.8,
                reasoning: "Query mentions testing concerns"
            });
        }

        // If query mentions performance, add performance context
        if (/performance|optimize|slow|fast|efficient/i.test(query)) {
            additionalRequirements.push({
                requirement_type: ContextType.ArchitecturalContext, // Use existing context type
                priority: 0.7,
                reasoning: "Query has performance implications"
            });
        }

        // If query mentions security, add security context
        if (/security|auth|permission|secure|vulnerability/i.test(query)) {
            additionalRequirements.push({
                requirement_type: ContextType.ArchitecturalContext, // Use existing context type
                priority: 0.9,
                reasoning: "Query has security implications"
            });
        }

        return additionalRequirements;
    }

    /**
     * Merge and prioritize context requirements
     */
    private mergeAndPrioritizeRequirements(
        base: ContextRequirement[], 
        additional: ContextRequirement[]
    ): ContextRequirement[] {
        const merged = [...base];
        
        for (const req of additional) {
            const existing = merged.find(r => r.requirement_type === req.requirement_type);
            if (existing) {
                // Boost priority if requirement appears multiple times
                existing.priority = Math.max(existing.priority, req.priority * 1.1);
                existing.reasoning += ` + ${req.reasoning}`;
            } else {
                merged.push(req);
            }
        }

        return merged.sort((a, b) => b.priority - a.priority);
    }
}

/**
 * Extracts code entities (functions, classes, etc.) from natural language queries
 */
class CodeEntityExtractor {
    async extract(query: string): Promise<CodeEntity[]> {
        const entities: CodeEntity[] = [];

        // Extract function names (camelCase or snake_case followed by parentheses)
        const functionPattern = /\b([a-z][a-zA-Z0-9_]*)\s*\(/g;
        let match;
        while ((match = functionPattern.exec(query)) !== null) {
            entities.push({
                name: match[1],
                type: EntityType.Function,
                confidence: 0.8,
                location: `position ${match.index}`
            });
        }

        // Extract class names (PascalCase)
        const classPattern = /\b([A-Z][a-zA-Z0-9]*(?:Class|Service|Component|Manager|Handler|Controller)?)\b/g;
        while ((match = classPattern.exec(query)) !== null) {
            entities.push({
                name: match[1],
                type: EntityType.Class,
                confidence: 0.7,
                location: `position ${match.index}`
            });
        }

        // Extract interface names
        const interfacePattern = /\b(I[A-Z][a-zA-Z0-9]*|[A-Z][a-zA-Z0-9]*Interface)\b/g;
        while ((match = interfacePattern.exec(query)) !== null) {
            entities.push({
                name: match[1],
                type: EntityType.Interface,
                confidence: 0.6,
                location: `position ${match.index}`
            });
        }

        // Extract variable names (mentioned with specific patterns)
        const variablePattern = /(?:variable|var|let|const)\s+([a-zA-Z_][a-zA-Z0-9_]*)/g;
        while ((match = variablePattern.exec(query)) !== null) {
            entities.push({
                name: match[1],
                type: EntityType.Variable,
                confidence: 0.9,
                location: `position ${match.index}`
            });
        }

        // Extract module/file names
        const modulePattern = /\b([a-zA-Z][a-zA-Z0-9]*\.[jt]sx?|[a-zA-Z][a-zA-Z0-9]*\.py|[a-zA-Z][a-zA-Z0-9]*\.rs)\b/g;
        while ((match = modulePattern.exec(query)) !== null) {
            entities.push({
                name: match[1],
                type: EntityType.Module,
                confidence: 0.8,
                location: `position ${match.index}`
            });
        }

        return entities;
    }
}

/**
 * Pattern matcher for identifying common query patterns
 */
class PatternMatcher {
    private patterns: QueryPattern[] = [
        {
            name: "how_to_implement",
            pattern: /how (?:do I|can I|to) (?:implement|create|build|make)/i,
            type: QueryType.Implementation,
            confidence: 0.9
        },
        {
            name: "fix_error",
            pattern: /(?:fix|solve|resolve) (?:this|the) (?:error|issue|problem)/i,
            type: QueryType.Debugging,
            confidence: 0.9
        },
        {
            name: "explain_code",
            pattern: /(?:what does|explain|how does) (?:this|the) (?:code|function|class)/i,
            type: QueryType.Explanation,
            confidence: 0.8
        },
        {
            name: "refactor_code",
            pattern: /(?:refactor|improve|optimize|clean up) (?:this|the) (?:code|function|class)/i,
            type: QueryType.Refactoring,
            confidence: 0.8
        }
    ];

    match(query: string): QueryPattern | null {
        for (const pattern of this.patterns) {
            if (pattern.pattern.test(query)) {
                return pattern;
            }
        }
        return null;
    }
}

/**
 * NLP processor for advanced text analysis
 */
class NLPProcessor {
    async processText(text: string): Promise<NLPResult> {
        // Placeholder for NLP processing
        // In a real implementation, this might use libraries like:
        // - natural.js for basic NLP
        // - compromise.js for text analysis
        // - or call external APIs like OpenAI, Google NLP, etc.
        
        return {
            tokens: text.split(/\s+/),
            sentiment: this.analyzeSentiment(text),
            keywords: this.extractKeywords(text),
            entities: [], // Would extract named entities
            intent_confidence: 0.7
        };
    }

    private analyzeSentiment(text: string): 'positive' | 'negative' | 'neutral' {
        // Simple sentiment analysis
        const positiveWords = ['good', 'great', 'excellent', 'perfect', 'awesome'];
        const negativeWords = ['bad', 'terrible', 'awful', 'broken', 'error', 'problem'];
        
        const words = text.toLowerCase().split(/\s+/);
        const positiveScore = words.filter(w => positiveWords.indexOf(w) !== -1).length;
        const negativeScore = words.filter(w => negativeWords.indexOf(w) !== -1).length;
        
        if (positiveScore > negativeScore) {return 'positive';}
        if (negativeScore > positiveScore) {return 'negative';}
        return 'neutral';
    }

    private extractKeywords(text: string): string[] {
        // Simple keyword extraction
        const stopWords = ['the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for', 'of', 'with', 'by'];
        return text.toLowerCase()
            .split(/\s+/)
            .filter(word => word.length > 2 && stopWords.indexOf(word) === -1)
            .slice(0, 10); // Top 10 keywords
    }
}

// Supporting interfaces
interface QueryPattern {
    name: string;
    pattern: RegExp;
    type: QueryType;
    confidence: number;
}

interface NLPResult {
    tokens: string[];
    sentiment: 'positive' | 'negative' | 'neutral';
    keywords: string[];
    entities: any[];
    intent_confidence: number;
}

/**
 * Advanced ML-based query classifier with confidence scoring
 */
class MLQueryClassifier {
    private featureExtractor: FeatureExtractor;
    private classificationModel: ClassificationModel;

    constructor() {
        this.featureExtractor = new FeatureExtractor();
        this.classificationModel = new ClassificationModel();
    }

    async classifyWithConfidence(query: string): Promise<QueryType> {
        // Extract features from the query
        const features = await this.featureExtractor.extractFeatures(query);
        
        // Get classification probabilities
        const probabilities = await this.classificationModel.predict(features);
        
        // Return the most likely class
        return this.getHighestProbabilityClass(probabilities);
    }

    private getHighestProbabilityClass(probabilities: Map<QueryType, number>): QueryType {
        let maxProb = 0;
        let bestClass = QueryType.Implementation;

        for (const [queryType, prob] of Array.from(probabilities.entries())) {
            if (prob > maxProb) {
                maxProb = prob;
                bestClass = queryType;
            }
        }

        return bestClass;
    }
}

/**
 * Feature extractor for ML classification
 */
class FeatureExtractor {
    async extractFeatures(query: string): Promise<QueryFeatures> {
        const tokens = query.toLowerCase().split(/\s+/);
        
        return {
            // Lexical features
            word_count: tokens.length,
            has_question_mark: query.includes('?'),
            has_code_entities: this.hasCodeEntities(query),
            
            // Semantic features
            implementation_keywords: this.countKeywords(query, ['implement', 'create', 'build', 'make', 'write', 'add']),
            debugging_keywords: this.countKeywords(query, ['error', 'bug', 'fix', 'issue', 'problem', 'debug']),
            explanation_keywords: this.countKeywords(query, ['what', 'how', 'explain', 'understand', 'describe']),
            refactoring_keywords: this.countKeywords(query, ['refactor', 'improve', 'optimize', 'clean']),
            architecture_keywords: this.countKeywords(query, ['architecture', 'design', 'pattern', 'structure']),
            
            // Syntactic features
            imperative_verbs: this.countImperativeVerbs(query),
            interrogative_words: this.countInterrogativeWords(query),
            
            // Code-specific features
            camel_case_words: this.countCamelCaseWords(query),
            function_references: this.countFunctionReferences(query),
            class_references: this.countClassReferences(query),
        };
    }

    private hasCodeEntities(query: string): boolean {
        return /[A-Z][a-zA-Z0-9]*|[a-z][a-zA-Z0-9]*\(|\.[a-zA-Z]/.test(query);
    }

    private countKeywords(query: string, keywords: string[]): number {
        const lowerQuery = query.toLowerCase();
        return keywords.filter(keyword => lowerQuery.includes(keyword)).length;
    }

    private countImperativeVerbs(query: string): number {
        const imperativeVerbs = ['create', 'build', 'make', 'implement', 'fix', 'solve', 'refactor', 'optimize'];
        return this.countKeywords(query, imperativeVerbs);
    }

    private countInterrogativeWords(query: string): number {
        const interrogativeWords = ['what', 'how', 'why', 'where', 'when', 'which', 'who'];
        return this.countKeywords(query, interrogativeWords);
    }

    private countCamelCaseWords(query: string): number {
        const camelCasePattern = /\b[a-z][a-zA-Z0-9]*[A-Z][a-zA-Z0-9]*\b/g;
        return (query.match(camelCasePattern) || []).length;
    }

    private countFunctionReferences(query: string): number {
        const functionPattern = /\b[a-zA-Z_][a-zA-Z0-9_]*\s*\(/g;
        return (query.match(functionPattern) || []).length;
    }

    private countClassReferences(query: string): number {
        const classPattern = /\b[A-Z][a-zA-Z0-9]*(?:Class|Service|Component|Manager)?\b/g;
        return (query.match(classPattern) || []).length;
    }
}

/**
 * Simple classification model (placeholder for actual ML model)
 */
class ClassificationModel {
    async predict(features: QueryFeatures): Promise<Map<QueryType, number>> {
        const probabilities = new Map<QueryType, number>();

        // Simple rule-based probability calculation
        // In a real implementation, this would be a trained ML model
        
        // Implementation probability
        let implProb = 0.1;
        implProb += features.implementation_keywords * 0.3;
        implProb += features.imperative_verbs * 0.2;
        implProb += features.has_code_entities ? 0.2 : 0;
        probabilities.set(QueryType.Implementation, Math.min(implProb, 1.0));

        // Debugging probability
        let debugProb = 0.1;
        debugProb += features.debugging_keywords * 0.4;
        debugProb += features.has_code_entities ? 0.2 : 0;
        probabilities.set(QueryType.Debugging, Math.min(debugProb, 1.0));

        // Explanation probability
        let explProb = 0.1;
        explProb += features.explanation_keywords * 0.3;
        explProb += features.interrogative_words * 0.2;
        explProb += features.has_question_mark ? 0.3 : 0;
        probabilities.set(QueryType.Explanation, Math.min(explProb, 1.0));

        // Refactoring probability
        let refactorProb = 0.1;
        refactorProb += features.refactoring_keywords * 0.4;
        refactorProb += features.has_code_entities ? 0.2 : 0;
        probabilities.set(QueryType.Refactoring, Math.min(refactorProb, 1.0));

        // Architecture probability
        let archProb = 0.1;
        archProb += features.architecture_keywords * 0.5;
        probabilities.set(QueryType.Architecture, Math.min(archProb, 1.0));

        return probabilities;
    }
}

// Supporting interfaces
interface QueryFeatures {
    word_count: number;
    has_question_mark: boolean;
    has_code_entities: boolean;
    implementation_keywords: number;
    debugging_keywords: number;
    explanation_keywords: number;
    refactoring_keywords: number;
    architecture_keywords: number;
    imperative_verbs: number;
    interrogative_words: number;
    camel_case_words: number;
    function_references: number;
    class_references: number;
}
