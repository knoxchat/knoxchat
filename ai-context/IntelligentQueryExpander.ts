/**
 * Intelligent Query Expander
 * 
 * Automatically expands user queries to include related concepts, entities,
 * and implicit requirements for more comprehensive context retrieval.
 */

import * as Types from './types';

export interface ExpandedQuery {
    original_query: string;
    expanded_entities: ExpandedEntity[];
    related_concepts: RelatedConcept[];
    implicit_requirements: ImplicitRequirement[];
    suggested_scope_expansion: ScopeExpansion[];
    expansion_metadata: ExpansionMetadata;
}

export interface ExpandedEntity extends Types.CodeEntity {
    expansion_reason: string;
    relation_to_original: string;
    distance_from_query: number;
    expansion_confidence: number;
}

export interface RelatedConcept {
    concept: string;
    relevance: number;
    reasoning: string;
    related_to: string[];
    concept_type: 'pattern' | 'technology' | 'domain' | 'architectural';
}

export interface ImplicitRequirement {
    requirement: string;
    reason: string;
    priority: number;
    context_type: Types.ContextType;
}

export interface ScopeExpansion {
    suggested_scope: Types.QueryScope;
    reason: string;
    expected_benefit: string;
    additional_entities: string[];
}

export interface ExpansionMetadata {
    entities_added: number;
    concepts_identified: number;
    scope_suggestions: number;
    expansion_time_ms: number;
    confidence_score: number;
}

/**
 * Main Intelligent Query Expander
 */
export class IntelligentQueryExpander {
    private knowledgeBase: DomainKnowledgeBase;
    private relationshipGraph: ConceptRelationshipGraph;

    constructor() {
        this.knowledgeBase = new DomainKnowledgeBase();
        this.relationshipGraph = new ConceptRelationshipGraph();
    }

    /**
     * Expand a query with related entities and concepts
     */
    async expandQuery(
        query: string,
        baseIntent: Types.QueryIntent,
        workspaceContext: Types.WorkspaceContext
    ): Promise<ExpandedQuery> {
        const startTime = Date.now();

        // 1. Expand entities (find related code elements)
        const expandedEntities = await this.expandEntities(baseIntent, workspaceContext);

        // 2. Identify related concepts
        const relatedConcepts = await this.findRelatedConcepts(baseIntent, expandedEntities);

        // 3. Detect implicit requirements
        const implicitRequirements = await this.detectImplicitRequirements(baseIntent, expandedEntities);

        // 4. Suggest scope expansions
        const scopeExpansions = await this.suggestScopeExpansions(baseIntent, expandedEntities);

        const metadata: ExpansionMetadata = {
            entities_added: expandedEntities.length,
            concepts_identified: relatedConcepts.length,
            scope_suggestions: scopeExpansions.length,
            expansion_time_ms: Date.now() - startTime,
            confidence_score: this.calculateExpansionConfidence(expandedEntities, relatedConcepts),
        };

        return {
            original_query: query,
            expanded_entities: expandedEntities,
            related_concepts: relatedConcepts,
            implicit_requirements: implicitRequirements,
            suggested_scope_expansion: scopeExpansions,
            expansion_metadata: metadata,
        };
    }

    /**
     * Expand entities based on relationships and patterns
     */
    private async expandEntities(
        intent: Types.QueryIntent,
        context: Types.WorkspaceContext
    ): Promise<ExpandedEntity[]> {
        const expandedEntities: ExpandedEntity[] = [];

        for (const entity of intent.entities) {
            // 1. Find directly related entities (same file, same class, etc.)
            const directRelated = await this.findDirectlyRelatedEntities(entity, context);
            expandedEntities.push(...directRelated);

            // 2. Find entities in call chain
            const callChainEntities = await this.findCallChainEntities(entity, context);
            expandedEntities.push(...callChainEntities);

            // 3. Find entities with similar names or purposes
            const similarEntities = await this.findSimilarEntities(entity, context);
            expandedEntities.push(...similarEntities);

            // 4. Find dependency-related entities
            const dependencyEntities = await this.findDependencyRelatedEntities(entity, context);
            expandedEntities.push(...dependencyEntities);
        }

        // Remove duplicates and sort by confidence
        return this.deduplicateAndSort(expandedEntities);
    }

    /**
     * Find directly related entities (same file, class, module)
     */
    private async findDirectlyRelatedEntities(
        entity: Types.CodeEntity,
        context: Types.WorkspaceContext
    ): Promise<ExpandedEntity[]> {
        const related: ExpandedEntity[] = [];

        // Example: If query mentions "login", also include:
        // - logout, authenticate, validateCredentials (related functions)
        // - AuthService, UserService (containing classes)
        // - session, token, credentials (related variables)

        const relatedNames = this.getRelatedNames(entity.name);

        for (const relatedName of relatedNames) {
            related.push({
                name: relatedName,
                type: this.inferEntityType(relatedName),
                confidence: 0.8,
                location: context.workspace_path,
                context_hint: `Related to ${entity.name}`,
                expansion_reason: 'Semantically related in same domain',
                relation_to_original: 'sibling',
                distance_from_query: 1,
                expansion_confidence: 0.8,
            });
        }

        return related;
    }

    /**
     * Find entities in the call chain
     */
    private async findCallChainEntities(
        entity: Types.CodeEntity,
        context: Types.WorkspaceContext
    ): Promise<ExpandedEntity[]> {
        const chainEntities: ExpandedEntity[] = [];

        // Example: If query mentions "login", include call chain:
        // login → validateCredentials → checkPassword → hashPassword

        const callChain = this.buildCallChain(entity.name);

        for (let i = 0; i < callChain.length; i++) {
            const funcName = callChain[i];
            if (funcName !== entity.name) {
                chainEntities.push({
                    name: funcName,
                    type: Types.EntityType.Function,
                    confidence: 0.9 - (i * 0.1), // Closer in chain = higher confidence
                    location: context.workspace_path,
                    context_hint: `Called by ${entity.name}`,
                    expansion_reason: 'Part of execution path',
                    relation_to_original: 'call-chain',
                    distance_from_query: i + 1,
                    expansion_confidence: 0.9 - (i * 0.1),
                });
            }
        }

        return chainEntities;
    }

    /**
     * Find entities with similar names or purposes
     */
    private async findSimilarEntities(
        entity: Types.CodeEntity,
        context: Types.WorkspaceContext
    ): Promise<ExpandedEntity[]> {
        const similar: ExpandedEntity[] = [];

        // Use fuzzy matching and semantic similarity
        const candidates = this.findSimilarNames(entity.name);

        for (const candidate of candidates) {
            const similarity = this.calculateNameSimilarity(entity.name, candidate);
            if (similarity > 0.6) {
                similar.push({
                    name: candidate,
                    type: entity.type,
                    confidence: similarity,
                    location: context.workspace_path,
                    context_hint: `Similar to ${entity.name}`,
                    expansion_reason: 'Name and purpose similarity',
                    relation_to_original: 'similar',
                    distance_from_query: 2,
                    expansion_confidence: similarity,
                });
            }
        }

        return similar;
    }

    /**
     * Find dependency-related entities
     */
    private async findDependencyRelatedEntities(
        entity: Types.CodeEntity,
        context: Types.WorkspaceContext
    ): Promise<ExpandedEntity[]> {
        const dependencies: ExpandedEntity[] = [];

        // Find entities that this entity depends on
        const deps = this.findDependencies(entity.name);

        for (const dep of deps) {
            dependencies.push({
                name: dep,
                type: Types.EntityType.Class,
                confidence: 0.75,
                location: context.workspace_path,
                context_hint: `Dependency of ${entity.name}`,
                expansion_reason: 'Required dependency',
                relation_to_original: 'dependency',
                distance_from_query: 2,
                expansion_confidence: 0.75,
            });
        }

        return dependencies;
    }

    /**
     * Find related concepts
     */
    private async findRelatedConcepts(
        intent: Types.QueryIntent,
        expandedEntities: ExpandedEntity[]
    ): Promise<RelatedConcept[]> {
        const concepts: RelatedConcept[] = [];

        // Extract concepts from query and entities
        const queryWords = intent.original_query.toLowerCase().split(/\s+/);
        const entityNames = [...intent.entities, ...expandedEntities].map(e => e.name.toLowerCase());

        // 1. Identify domain concepts
        concepts.push(...this.identifyDomainConcepts(queryWords, entityNames));

        // 2. Identify technology/framework concepts
        concepts.push(...this.identifyTechnologyConcepts(queryWords, entityNames));

        // 3. Identify architectural patterns
        concepts.push(...this.identifyArchitecturalPatterns(queryWords, entityNames));

        // 4. Identify cross-cutting concerns
        concepts.push(...this.identifyCrossCuttingConcerns(queryWords, entityNames));

        return concepts;
    }

    /**
     * Identify domain concepts (business logic)
     */
    private identifyDomainConcepts(queryWords: string[], entityNames: string[]): RelatedConcept[] {
        const concepts: RelatedConcept[] = [];
        const allWords = [...queryWords, ...entityNames];

        // Authentication domain
        if (this.containsAny(allWords, ['auth', 'login', 'user', 'password', 'token', 'session'])) {
            concepts.push({
                concept: 'authentication',
                relevance: 0.95,
                reasoning: 'Query involves user authentication and authorization',
                related_to: ['login', 'user', 'session'],
                concept_type: 'domain',
            });

            // Related concepts
            concepts.push({
                concept: 'authorization',
                relevance: 0.85,
                reasoning: 'Often paired with authentication',
                related_to: ['permission', 'role', 'access'],
                concept_type: 'domain',
            });

            concepts.push({
                concept: 'session_management',
                relevance: 0.80,
                reasoning: 'Critical for maintaining authenticated state',
                related_to: ['session', 'token', 'cookie'],
                concept_type: 'domain',
            });
        }

        // Data persistence domain
        if (this.containsAny(allWords, ['database', 'db', 'save', 'store', 'persist', 'repository'])) {
            concepts.push({
                concept: 'data_persistence',
                relevance: 0.90,
                reasoning: 'Query involves data storage and retrieval',
                related_to: ['database', 'repository', 'orm'],
                concept_type: 'domain',
            });
        }

        // API/HTTP domain
        if (this.containsAny(allWords, ['api', 'endpoint', 'request', 'response', 'http', 'rest'])) {
            concepts.push({
                concept: 'api_communication',
                relevance: 0.90,
                reasoning: 'Query involves API endpoints and HTTP communication',
                related_to: ['request', 'response', 'middleware'],
                concept_type: 'domain',
            });
        }

        return concepts;
    }

    /**
     * Identify technology/framework concepts
     */
    private identifyTechnologyConcepts(queryWords: string[], entityNames: string[]): RelatedConcept[] {
        const concepts: RelatedConcept[] = [];
        const allWords = [...queryWords, ...entityNames];

        // React concepts
        if (this.containsAny(allWords, ['react', 'component', 'hook', 'usestate', 'useeffect', 'jsx'])) {
            concepts.push({
                concept: 'react',
                relevance: 0.90,
                reasoning: 'Query involves React framework',
                related_to: ['component', 'hook', 'state'],
                concept_type: 'technology',
            });
        }

        // Node.js/Express concepts
        if (this.containsAny(allWords, ['express', 'middleware', 'router', 'app.get', 'app.post'])) {
            concepts.push({
                concept: 'express',
                relevance: 0.90,
                reasoning: 'Query involves Express.js framework',
                related_to: ['middleware', 'router', 'controller'],
                concept_type: 'technology',
            });
        }

        return concepts;
    }

    /**
     * Identify architectural patterns
     */
    private identifyArchitecturalPatterns(queryWords: string[], entityNames: string[]): RelatedConcept[] {
        const concepts: RelatedConcept[] = [];
        const allWords = [...queryWords, ...entityNames];

        // MVC pattern
        if (this.containsAny(allWords, ['controller', 'model', 'view', 'service'])) {
            concepts.push({
                concept: 'mvc_pattern',
                relevance: 0.85,
                reasoning: 'Code structure suggests MVC pattern',
                related_to: ['controller', 'service', 'repository'],
                concept_type: 'architectural',
            });
        }

        // Repository pattern
        if (this.containsAny(allWords, ['repository', 'dao', 'database'])) {
            concepts.push({
                concept: 'repository_pattern',
                relevance: 0.80,
                reasoning: 'Data access suggests repository pattern',
                related_to: ['repository', 'entity', 'database'],
                concept_type: 'pattern',
            });
        }

        return concepts;
    }

    /**
     * Identify cross-cutting concerns
     */
    private identifyCrossCuttingConcerns(queryWords: string[], entityNames: string[]): RelatedConcept[] {
        const concepts: RelatedConcept[] = [];
        const allWords = [...queryWords, ...entityNames];

        // Error handling
        if (this.containsAny(allWords, ['error', 'exception', 'try', 'catch', 'throw'])) {
            concepts.push({
                concept: 'error_handling',
                relevance: 0.75,
                reasoning: 'Query involves error handling',
                related_to: ['exception', 'validation', 'logging'],
                concept_type: 'architectural',
            });
        }

        // Logging
        if (this.containsAny(allWords, ['log', 'logger', 'debug', 'console'])) {
            concepts.push({
                concept: 'logging',
                relevance: 0.70,
                reasoning: 'Query involves logging and debugging',
                related_to: ['log', 'debug', 'monitoring'],
                concept_type: 'architectural',
            });
        }

        // Validation
        if (this.containsAny(allWords, ['validate', 'validation', 'check', 'verify'])) {
            concepts.push({
                concept: 'validation',
                relevance: 0.75,
                reasoning: 'Query involves input validation',
                related_to: ['validation', 'schema', 'sanitization'],
                concept_type: 'architectural',
            });
        }

        return concepts;
    }

    /**
     * Detect implicit requirements
     */
    private async detectImplicitRequirements(
        intent: Types.QueryIntent,
        expandedEntities: ExpandedEntity[]
    ): Promise<ImplicitRequirement[]> {
        const requirements: ImplicitRequirement[] = [];

        // Based on query type, add implicit requirements
        switch (intent.query_type) {
            case Types.QueryType.Implementation:
                requirements.push({
                    requirement: 'Need usage examples and patterns',
                    reason: 'Implementation queries benefit from seeing existing patterns',
                    priority: 0.9,
                    context_type: Types.ContextType.ExampleContext,
                });
                requirements.push({
                    requirement: 'Need type definitions and interfaces',
                    reason: 'Implementation requires understanding data structures',
                    priority: 0.85,
                    context_type: Types.ContextType.SemanticContext,
                });
                break;

            case Types.QueryType.Debugging:
                requirements.push({
                    requirement: 'Need complete call stack',
                    reason: 'Debugging requires understanding execution flow',
                    priority: 0.95,
                    context_type: Types.ContextType.DependencyContext,
                });
                requirements.push({
                    requirement: 'Need error handling logic',
                    reason: 'Debugging often involves error conditions',
                    priority: 0.80,
                    context_type: Types.ContextType.SemanticContext,
                });
                break;

            case Types.QueryType.Refactoring:
                requirements.push({
                    requirement: 'Need impact analysis',
                    reason: 'Refactoring requires understanding all usages',
                    priority: 0.95,
                    context_type: Types.ContextType.UsageContext,
                });
                requirements.push({
                    requirement: 'Need test coverage',
                    reason: 'Refactoring needs tests to verify correctness',
                    priority: 0.85,
                    context_type: Types.ContextType.TestingContext,
                });
                break;

            case Types.QueryType.Architecture:
                requirements.push({
                    requirement: 'Need system-level view',
                    reason: 'Architecture queries need broad context',
                    priority: 0.95,
                    context_type: Types.ContextType.ArchitecturalContext,
                });
                requirements.push({
                    requirement: 'Need dependency relationships',
                    reason: 'Architecture requires understanding component interactions',
                    priority: 0.90,
                    context_type: Types.ContextType.DependencyContext,
                });
                break;
        }

        // Add requirements based on detected concepts
        if (this.containsConcept(['authentication', 'authorization'], expandedEntities)) {
            requirements.push({
                requirement: 'Need security context',
                reason: 'Authentication/authorization requires security considerations',
                priority: 0.90,
                context_type: Types.ContextType.SemanticContext,
            });
        }

        return requirements;
    }

    /**
     * Suggest scope expansions
     */
    private async suggestScopeExpansions(
        intent: Types.QueryIntent,
        expandedEntities: ExpandedEntity[]
    ): Promise<ScopeExpansion[]> {
        const expansions: ScopeExpansion[] = [];

        // If query is function-level but involves many entities, suggest class-level
        if (intent.scope === Types.QueryScope.Function && expandedEntities.length > 5) {
            expansions.push({
                suggested_scope: Types.QueryScope.Class,
                reason: 'Query involves multiple related functions',
                expected_benefit: 'Better understanding of class-level context and relationships',
                additional_entities: expandedEntities.slice(0, 10).map(e => e.name),
            });
        }

        // If query involves cross-file references, suggest module-level
        if (this.hasCrossFileReferences(expandedEntities)) {
            expansions.push({
                suggested_scope: Types.QueryScope.Module,
                reason: 'Entities span multiple files',
                expected_benefit: 'Complete understanding of module interactions',
                additional_entities: this.extractCrossFileEntities(expandedEntities),
            });
        }

        // If query is about architecture, always suggest system-level
        if (intent.query_type === Types.QueryType.Architecture && intent.scope !== Types.QueryScope.System) {
            expansions.push({
                suggested_scope: Types.QueryScope.System,
                reason: 'Architecture questions benefit from system-wide view',
                expected_benefit: 'Complete architectural understanding and patterns',
                additional_entities: ['all system components'],
            });
        }

        return expansions;
    }

    // Helper methods

    private getRelatedNames(entityName: string): string[] {
        const related: string[] = [];
        const lowerName = entityName.toLowerCase();

        // Common patterns for related names
        const patterns: Record<string, string[]> = {
            login: ['logout', 'authenticate', 'validateCredentials', 'checkPassword', 'createSession'],
            user: ['account', 'profile', 'credentials', 'authentication'],
            create: ['read', 'update', 'delete', 'save', 'find'],
            get: ['set', 'update', 'delete', 'fetch'],
            validate: ['sanitize', 'check', 'verify', 'process'],
            auth: ['session', 'token', 'permission', 'role'],
        };

        for (const [key, values] of Object.entries(patterns)) {
            if (lowerName.includes(key)) {
                related.push(...values);
            }
        }

        return related;
    }

    private inferEntityType(name: string): Types.EntityType {
        if (name.match(/^[A-Z]/)) return Types.EntityType.Class;
        if (name.match(/^I[A-Z]/)) return Types.EntityType.Interface;
        return Types.EntityType.Function;
    }

    private buildCallChain(functionName: string): string[] {
        // Simplified - in reality, would use knowledge graph
        const chains: Record<string, string[]> = {
            login: ['validateCredentials', 'checkPassword', 'hashPassword', 'createSession'],
            authenticate: ['validateToken', 'verifySignature', 'checkExpiration'],
        };
        return chains[functionName] || [];
    }

    private findSimilarNames(name: string): string[] {
        // Simplified - in reality, would use fuzzy matching algorithm
        return [];
    }

    private calculateNameSimilarity(name1: string, name2: string): number {
        // Simplified Levenshtein distance
        const longer = name1.length > name2.length ? name1 : name2;
        const shorter = name1.length > name2.length ? name2 : name1;
        const longerLength = longer.length;
        
        if (longerLength === 0) return 1.0;
        
        return (longerLength - this.levenshteinDistance(longer, shorter)) / longerLength;
    }

    private levenshteinDistance(str1: string, str2: string): number {
        const matrix: number[][] = [];

        for (let i = 0; i <= str2.length; i++) {
            matrix[i] = [i];
        }

        for (let j = 0; j <= str1.length; j++) {
            matrix[0][j] = j;
        }

        for (let i = 1; i <= str2.length; i++) {
            for (let j = 1; j <= str1.length; j++) {
                if (str2.charAt(i - 1) === str1.charAt(j - 1)) {
                    matrix[i][j] = matrix[i - 1][j - 1];
                } else {
                    matrix[i][j] = Math.min(
                        matrix[i - 1][j - 1] + 1,
                        matrix[i][j - 1] + 1,
                        matrix[i - 1][j] + 1
                    );
                }
            }
        }

        return matrix[str2.length][str1.length];
    }

    private findDependencies(entityName: string): string[] {
        // Simplified - would use actual dependency graph
        return [];
    }

    private deduplicateAndSort(entities: ExpandedEntity[]): ExpandedEntity[] {
        const seen = new Set<string>();
        const unique: ExpandedEntity[] = [];

        for (const entity of entities) {
            const key = `${entity.name}:${entity.type}`;
            if (!seen.has(key)) {
                seen.add(key);
                unique.push(entity);
            }
        }

        return unique.sort((a, b) => b.expansion_confidence - a.expansion_confidence);
    }

    private containsAny(words: string[], keywords: string[]): boolean {
        return words.some(word => keywords.some(keyword => word.includes(keyword)));
    }

    private containsConcept(concepts: string[], entities: ExpandedEntity[]): boolean {
        const entityNames = entities.map(e => e.name.toLowerCase()).join(' ');
        return concepts.some(concept => entityNames.includes(concept));
    }

    private hasCrossFileReferences(entities: ExpandedEntity[]): boolean {
        const files = new Set(entities.map(e => e.location).filter(l => l));
        return files.size > 1;
    }

    private extractCrossFileEntities(entities: ExpandedEntity[]): string[] {
        return entities
            .filter(e => e.location)
            .map(e => e.name)
            .slice(0, 5);
    }

    private calculateExpansionConfidence(
        entities: ExpandedEntity[],
        concepts: RelatedConcept[]
    ): number {
        const entityConfidence = entities.length > 0
            ? entities.reduce((sum, e) => sum + e.expansion_confidence, 0) / entities.length
            : 0;

        const conceptConfidence = concepts.length > 0
            ? concepts.reduce((sum, c) => sum + c.relevance, 0) / concepts.length
            : 0;

        return (entityConfidence + conceptConfidence) / 2;
    }
}

/**
 * Domain knowledge base (simplified)
 */
class DomainKnowledgeBase {
    // In reality, this would be loaded from a knowledge graph or ML model
}

/**
 * Concept relationship graph (simplified)
 */
class ConceptRelationshipGraph {
    // In reality, this would use a graph database or ML embeddings
}

