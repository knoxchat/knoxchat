/**
 * Code Understanding Engine - Core semantic analysis and code comprehension
 * 
 * This engine provides deep semantic understanding of codebases, implementing
 * the complete code analysis pipeline from the checkpoint-system-design.md.
 */

import { LanguageParser, ParseResult, Symbol as ParserSymbol, DependencyInfo } from './parsers/LanguageParser';

// For now, using mock interfaces for missing functionality
interface ExtendedLanguageParser extends LanguageParser {
    parse_file?(content: string, filePath: string): Promise<any>;
    extract_symbols?(ast: any): ParserSymbol[];
    analyze_dependencies?(ast: any): DependencyInfo[];
}

interface TypeScriptParser extends ExtendedLanguageParser {
    // TypeScript-specific parsing methods
}

export class CodeUnderstandingEngine {
    private parsers: Map<string, LanguageParser> = new Map();
    private syntaxTreeCache: Map<string, AST> = new Map();
    private callGraphCache: Map<string, CallGraph> = new Map();
    private dataFlowCache: Map<string, DataFlowGraph> = new Map();

    constructor() {
        this.initializeParsers();
    }

    /**
     * Parse and understand code semantically
     */
    async analyzeCodebase(files: FileMap): Promise<SemanticMap> {
        const results = await Promise.all([
            this.buildAST(files),
            this.buildCallGraph(files),
            this.analyzeDataFlow(files),
            this.mapDependencies(files),
            this.identifyPatterns(files),
            this.inferArchitecture(files)
        ]);

        return {
            syntaxTree: results[0],
            callGraph: results[1],
            dataFlow: results[2],
            dependencies: results[3],
            patterns: results[4],
            architecture: results[5]
        };
    }

    /**
     * Understand query intent with advanced analysis
     */
    async analyzeQuery(query: string): Promise<QueryIntent> {
        const entities = this.extractCodeEntities(query);
        const queryType = this.classifyQueryType(query);
        const scope = this.inferQueryScope(query, entities);
        const context = this.inferRequiredContext(query, queryType, scope);

        return {
            type: queryType,
            scope: scope,
            entities: entities,
            context: context,
            confidence: this.calculateConfidence(query, entities, queryType),
            priority: this.calculatePriority(queryType, scope),
            expectedComplexity: this.estimateComplexity(query, entities)
        };
    }

    /**
     * Build comprehensive AST for all files
     */
    private async buildAST(files: FileMap): Promise<SyntaxTreeMap> {
        const astMap: SyntaxTreeMap = new Map();

        for (const [filePath, fileContent] of files) {
            const cacheKey = this.generateCacheKey(filePath, fileContent);
            
            if (this.syntaxTreeCache.has(cacheKey)) {
                astMap.set(filePath, this.syntaxTreeCache.get(cacheKey)!);
                continue;
            }

            const parser = this.getParserForFile(filePath);
            if (parser) {
                try {
                    const ast = await (parser as ExtendedLanguageParser).parse_file?.(fileContent.content, filePath);
                    astMap.set(filePath, ast);
                    this.syntaxTreeCache.set(cacheKey, ast);
                } catch (error) {
                    console.warn(`Failed to parse ${filePath}:`, error);
                }
            }
        }

        return astMap;
    }

    /**
     * Build comprehensive call graph across all files
     */
    private async buildCallGraph(files: FileMap): Promise<CallGraph> {
        const callGraph: CallGraph = {
            nodes: new Map(),
            edges: [],
            clusters: new Map()
        };

        // First pass: collect all function definitions
        for (const [filePath, fileContent] of files) {
            const parser = this.getParserForFile(filePath);
            if (parser) {
                const ast = await (parser as ExtendedLanguageParser).parse_file?.(fileContent.content, filePath);
                const symbols = (parser as ExtendedLanguageParser).extract_symbols?.(ast) || [];
                
                for (const symbol of symbols) {
                    if (symbol.type === 'function') {
                        callGraph.nodes.set(symbol.name, {
                            id: symbol.name,
                            filePath,
                            location: symbol.location,
                            signature: (symbol as any).signature || '',
                            complexity: (symbol as any).complexity || 1,
                            calls: [],
                            calledBy: []
                        });
                    }
                }
            }
        }

        // Second pass: build call relationships
        for (const [filePath, fileContent] of files) {
            const parser = this.getParserForFile(filePath);
            if (parser) {
                const ast = await (parser as ExtendedLanguageParser).parse_file?.(fileContent.content, filePath);
                const dependencies = (parser as ExtendedLanguageParser).analyze_dependencies?.(ast) || [];
                
                for (const dep of dependencies) {
                    if ((dep as any).type === 'function_call') {
                        const edge: CallEdge = {
                            from: (dep as any).source || 'unknown',
                            to: (dep as any).target || 'unknown',
                            filePath,
                            location: (dep as any).location || { file: '', startLine: 0, endLine: 0, startColumn: 0, endColumn: 0 },
                            callType: (dep as any).callType || 'direct'
                        };
                        
                        callGraph.edges.push(edge);
                        
                        // Update node relationships
                        const sourceNode = callGraph.nodes.get((dep as any).source || 'unknown');
                        const targetNode = callGraph.nodes.get((dep as any).target || 'unknown');
                        
                        if (sourceNode) {
                            sourceNode.calls.push((dep as any).target || 'unknown');
                        }
                        if (targetNode) {
                            targetNode.calledBy.push((dep as any).source || 'unknown');
                        }
                    }
                }
            }
        }

        // Third pass: identify call clusters and patterns
        this.identifyCallClusters(callGraph);

        return callGraph;
    }

    /**
     * Analyze data flow across the codebase
     */
    private async analyzeDataFlow(files: FileMap): Promise<DataFlowGraph> {
        const dataFlow: DataFlowGraph = {
            variables: new Map(),
            flows: [],
            transformations: new Map()
        };

        for (const [filePath, fileContent] of files) {
            const parser = this.getParserForFile(filePath);
            if (parser) {
                const ast = await (parser as ExtendedLanguageParser).parse_file?.(fileContent.content, filePath);
                
                // Analyze variable definitions and usages
                const variables = this.extractVariables(ast, filePath);
                const flows = this.traceDataFlow(ast, filePath);
                const transformations = this.identifyDataTransformations(ast, filePath);

                // Merge results
                variables.forEach((variable, name) => {
                    dataFlow.variables.set(`${filePath}:${name}`, variable);
                });
                
                dataFlow.flows.push(...flows);
                
                transformations.forEach((transformation, name) => {
                    dataFlow.transformations.set(`${filePath}:${name}`, transformation);
                });
            }
        }

        return dataFlow;
    }

    /**
     * Map all dependencies and their relationships
     */
    private async mapDependencies(files: FileMap): Promise<DependencyMap> {
        const dependencyMap: DependencyMap = {
            imports: new Map(),
            exports: new Map(),
            internal: new Map(),
            external: new Map(),
            circular: []
        };

        // Build dependency graph
        for (const [filePath, fileContent] of files) {
            const parser = this.getParserForFile(filePath);
            if (parser) {
                const ast = await (parser as ExtendedLanguageParser).parse_file?.(fileContent.content, filePath);
                const dependencies = (parser as ExtendedLanguageParser).analyze_dependencies?.(ast) || [];
                
                for (const dep of dependencies) {
                    const depType = (dep as any).type || dep.type;
                    switch (depType) {
                        case 'import':
                            this.addImportDependency(dependencyMap, filePath, dep);
                            break;
                        case 'export':
                            this.addExportDependency(dependencyMap, filePath, dep);
                            break;
                        case 'internal':
                            this.addInternalDependency(dependencyMap, filePath, dep);
                            break;
                        case 'external':
                            this.addExternalDependency(dependencyMap, filePath, dep);
                            break;
                    }
                }
            }
        }

        // Detect circular dependencies
        dependencyMap.circular = this.detectCircularDependencies(dependencyMap);

        return dependencyMap;
    }

    /**
     * Identify architectural and design patterns
     */
    private async identifyPatterns(files: FileMap): Promise<PatternMap> {
        const patterns: PatternMap = new Map();

        const patternDetectors = [
            this.detectMVCPattern.bind(this),
            this.detectRepositoryPattern.bind(this),
            this.detectFactoryPattern.bind(this),
            this.detectObserverPattern.bind(this),
            this.detectSingletonPattern.bind(this),
            this.detectDecoratorPattern.bind(this),
            this.detectStrategyPattern.bind(this),
            this.detectCommandPattern.bind(this),
            this.detectMediatorPattern.bind(this),
            this.detectBuilderPattern.bind(this)
        ];

        for (const detector of patternDetectors) {
            const detectedPatterns = await detector(files);
            detectedPatterns.forEach((pattern, name) => {
                patterns.set(name, pattern);
            });
        }

        return patterns;
    }

    /**
     * Infer overall system architecture
     */
    private async inferArchitecture(files: FileMap): Promise<ArchitectureMap> {
        const architecture: ArchitectureMap = {
            layers: this.identifyArchitecturalLayers(files),
            components: this.identifyComponents(files),
            services: this.identifyServices(files),
            dataModels: this.identifyDataModels(files),
            apiEndpoints: this.identifyAPIEndpoints(files),
            configurations: this.identifyConfigurations(files),
            entryPoints: this.identifyEntryPoints(files),
            testStructure: this.identifyTestStructure(files)
        };

        return architecture;
    }

    /**
     * Extract code entities from natural language query
     */
    private extractCodeEntities(query: string): CodeEntity[] {
        const entities: CodeEntity[] = [];
        
        // Extract function names
        const functionMatches = query.match(/\b[a-zA-Z_][a-zA-Z0-9_]*\s*\(/g);
        if (functionMatches) {
            functionMatches.forEach(match => {
                const name = match.replace(/\s*\($/, '');
                entities.push({
                    type: 'function',
                    name,
                    confidence: 0.9
                });
            });
        }

        // Extract class names (PascalCase)
        const classMatches = query.match(/\b[A-Z][a-zA-Z0-9]*\b/g);
        if (classMatches) {
            classMatches.forEach(name => {
                entities.push({
                    type: 'class',
                    name,
                    confidence: 0.7
                });
            });
        }

        // Extract variable names (camelCase)
        const variableMatches = query.match(/\b[a-z][a-zA-Z0-9]*\b/g);
        if (variableMatches) {
            variableMatches.forEach(name => {
                if (!this.isCommonWord(name)) {
                    entities.push({
                        type: 'variable',
                        name,
                        confidence: 0.5
                    });
                }
            });
        }

        // Extract file paths
        const fileMatches = query.match(/[a-zA-Z0-9_-]+\.[a-zA-Z]{2,4}/g);
        if (fileMatches) {
            fileMatches.forEach(name => {
                entities.push({
                    type: 'file',
                    name,
                    confidence: 0.8
                });
            });
        }

        return entities;
    }

    /**
     * Classify query type based on content
     */
    private classifyQueryType(query: string): QueryType {
        const queryLower = query.toLowerCase();

        if (/how to|implement|create|build|add|write/.test(queryLower)) {
            return 'implementation';
        }
        if (/error|bug|fix|issue|problem|debug|not working/.test(queryLower)) {
            return 'debugging';
        }
        if (/refactor|improve|optimize|restructure|clean up|better/.test(queryLower)) {
            return 'refactoring';
        }
        if (/architecture|design|pattern|structure|organize|overview/.test(queryLower)) {
            return 'architecture';
        }
        if (/what is|explain|how does|understand|clarify|meaning/.test(queryLower)) {
            return 'explanation';
        }
        if (/test|testing|spec|mock|unit test|integration/.test(queryLower)) {
            return 'testing';
        }
        if (/performance|slow|fast|optimize|speed|memory/.test(queryLower)) {
            return 'performance';
        }
        if (/security|auth|permission|secure|vulnerability/.test(queryLower)) {
            return 'security';
        }

        return 'general';
    }

    /**
     * Infer query scope based on entities and context
     */
    private inferQueryScope(query: string, entities: CodeEntity[]): QueryScope {
        const queryLower = query.toLowerCase();

        if (/system|application|entire|whole|all|global/.test(queryLower)) {
            return 'system';
        }
        if (/module|package|namespace|library/.test(queryLower)) {
            return 'module';
        }
        if (/class|interface|type|object/.test(queryLower) || 
            entities.some(e => e.type === 'class')) {
            return 'class';
        }
        if (/function|method|procedure/.test(queryLower) || 
            entities.some(e => e.type === 'function')) {
            return 'function';
        }
        if (/variable|property|field|attribute/.test(queryLower) || 
            entities.some(e => e.type === 'variable')) {
            return 'variable';
        }

        return 'module'; // Default scope
    }

    /**
     * Infer required context based on query analysis
     */
    private inferRequiredContext(
        query: string, 
        queryType: QueryType, 
        scope: QueryScope
    ): RequiredContext {
        const context: RequiredContext = {
            includeDefinitions: false,
            includeUsages: false,
            includeDependencies: false,
            includeTests: false,
            includeDocumentation: false,
            includeHistory: false,
            includeArchitecture: false,
            includePerformance: false,
            includeSecurity: false,
            depth: 1
        };

        // Base requirements by query type
        switch (queryType) {
            case 'implementation':
                context.includeDefinitions = true;
                context.includeUsages = true;
                context.includeDependencies = true;
                context.includeTests = true;
                context.depth = 2;
                break;
            case 'debugging':
                context.includeDefinitions = true;
                context.includeUsages = true;
                context.includeDependencies = true;
                context.includeHistory = true;
                context.depth = 3;
                break;
            case 'refactoring':
                context.includeDefinitions = true;
                context.includeUsages = true;
                context.includeDependencies = true;
                context.includeTests = true;
                context.includeArchitecture = true;
                context.depth = 2;
                break;
            case 'architecture':
                context.includeArchitecture = true;
                context.includeDependencies = true;
                context.includeDocumentation = true;
                context.depth = 1;
                break;
            case 'performance':
                context.includePerformance = true;
                context.includeUsages = true;
                context.depth = 2;
                break;
            case 'security':
                context.includeSecurity = true;
                context.includeUsages = true;
                context.includeDependencies = true;
                context.depth = 2;
                break;
        }

        // Adjust based on scope
        if (scope === 'system') {
            context.includeArchitecture = true;
            context.depth = Math.max(context.depth, 1);
        } else if (scope === 'function') {
            context.depth = Math.max(context.depth, 2);
        }

        return context;
    }

    // Helper methods
    private initializeParsers(): void {
        // Mock parsers for now - would be replaced with actual implementations
        const mockParser: LanguageParser = {
            async parseFile(content: string, filePath: string): Promise<any> {
                return { type: 'mock_ast', content: content.slice(0, 100) };
            }
        };
        
        this.parsers.set('typescript', mockParser);
        this.parsers.set('javascript', mockParser);
        // Add more parsers as needed
    }

    private getParserForFile(filePath: string): LanguageParser | undefined {
        const extension = filePath.split('.').pop()?.toLowerCase();
        
        switch (extension) {
            case 'ts':
            case 'tsx':
                return this.parsers.get('typescript');
            case 'js':
            case 'jsx':
                return this.parsers.get('javascript');
            default:
                return undefined;
        }
    }

    private generateCacheKey(filePath: string, fileContent: FileContent): string {
        // Simple hash based on file path and content length
        return `${filePath}:${fileContent.content.length}:${fileContent.lastModified || 0}`;
    }

    private calculateConfidence(query: string, entities: CodeEntity[], queryType: QueryType): number {
        let confidence = 0.5; // Base confidence

        // Boost confidence based on entities found
        confidence += entities.length * 0.1;

        // Boost confidence based on query clarity
        if (queryType !== 'general') {
            confidence += 0.2;
        }

        // Boost confidence based on specificity
        if (entities.some(e => e.confidence > 0.8)) {
            confidence += 0.2;
        }

        return Math.min(confidence, 1.0);
    }

    private calculatePriority(queryType: QueryType, scope: QueryScope): number {
        const typePriorities: Record<QueryType, number> = {
            debugging: 0.9,
            security: 0.8,
            performance: 0.7,
            implementation: 0.6,
            refactoring: 0.5,
            architecture: 0.4,
            testing: 0.4,
            explanation: 0.3,
            general: 0.2
        };

        const scopePriorities: Record<QueryScope, number> = {
            system: 0.9,
            module: 0.7,
            class: 0.5,
            function: 0.4,
            variable: 0.2
        };

        return (typePriorities[queryType] + scopePriorities[scope]) / 2;
    }

    private estimateComplexity(query: string, entities: CodeEntity[]): ComplexityLevel {
        let complexity = 0;

        // Base complexity from query length
        complexity += Math.min(query.length / 100, 0.3);

        // Complexity from number of entities
        complexity += entities.length * 0.1;

        // Complexity from query type
        if (/system|architecture|refactor|debug/.test(query.toLowerCase())) {
            complexity += 0.4;
        }

        if (complexity > 0.7) {return 'high';}
        if (complexity > 0.4) {return 'medium';}
        return 'low';
    }

    private isCommonWord(word: string): boolean {
        const commonWords = new Set([
            'the', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for', 'of', 'with', 'by',
            'from', 'up', 'about', 'into', 'through', 'during', 'before', 'after', 'above',
            'below', 'over', 'under', 'again', 'further', 'then', 'once', 'here', 'there',
            'when', 'where', 'why', 'how', 'all', 'any', 'both', 'each', 'few', 'more',
            'most', 'other', 'some', 'such', 'only', 'own', 'same', 'so', 'than', 'too',
            'very', 'can', 'will', 'just', 'should', 'now'
        ]);
        
        return commonWords.has(word.toLowerCase());
    }

    // Pattern detection methods (simplified implementations)
    private async detectMVCPattern(files: FileMap): Promise<Map<string, Pattern>> {
        const patterns = new Map<string, Pattern>();
        // Implementation would analyze file structure for MVC pattern
        return patterns;
    }

    private async detectRepositoryPattern(files: FileMap): Promise<Map<string, Pattern>> {
        const patterns = new Map<string, Pattern>();
        // Implementation would look for repository pattern indicators
        return patterns;
    }

    private async detectFactoryPattern(files: FileMap): Promise<Map<string, Pattern>> {
        const patterns = new Map<string, Pattern>();
        // Implementation would detect factory pattern usage
        return patterns;
    }

    private async detectObserverPattern(files: FileMap): Promise<Map<string, Pattern>> {
        const patterns = new Map<string, Pattern>();
        // Implementation would detect observer pattern
        return patterns;
    }

    private async detectSingletonPattern(files: FileMap): Promise<Map<string, Pattern>> {
        const patterns = new Map<string, Pattern>();
        // Implementation would detect singleton pattern
        return patterns;
    }

    private async detectDecoratorPattern(files: FileMap): Promise<Map<string, Pattern>> {
        const patterns = new Map<string, Pattern>();
        // Implementation would detect decorator pattern
        return patterns;
    }

    private async detectStrategyPattern(files: FileMap): Promise<Map<string, Pattern>> {
        const patterns = new Map<string, Pattern>();
        // Implementation would detect strategy pattern
        return patterns;
    }

    private async detectCommandPattern(files: FileMap): Promise<Map<string, Pattern>> {
        const patterns = new Map<string, Pattern>();
        // Implementation would detect command pattern
        return patterns;
    }

    private async detectMediatorPattern(files: FileMap): Promise<Map<string, Pattern>> {
        const patterns = new Map<string, Pattern>();
        // Implementation would detect mediator pattern
        return patterns;
    }

    private async detectBuilderPattern(files: FileMap): Promise<Map<string, Pattern>> {
        const patterns = new Map<string, Pattern>();
        // Implementation would detect builder pattern
        return patterns;
    }

    // Architecture analysis methods (simplified)
    private identifyArchitecturalLayers(files: FileMap): ArchitecturalLayer[] {
        // Implementation would analyze directory structure and naming patterns
        return [];
    }

    private identifyComponents(files: FileMap): Component[] {
        // Implementation would identify major components
        return [];
    }

    private identifyServices(files: FileMap): Service[] {
        // Implementation would identify service classes
        return [];
    }

    private identifyDataModels(files: FileMap): DataModel[] {
        // Implementation would identify data models/entities
        return [];
    }

    private identifyAPIEndpoints(files: FileMap): APIEndpoint[] {
        // Implementation would identify API endpoints
        return [];
    }

    private identifyConfigurations(files: FileMap): Configuration[] {
        // Implementation would identify configuration files
        return [];
    }

    private identifyEntryPoints(files: FileMap): EntryPoint[] {
        // Implementation would identify application entry points
        return [];
    }

    private identifyTestStructure(files: FileMap): TestStructure {
        // Implementation would analyze test organization
        return { testFiles: [], coverage: 0, frameworks: [] };
    }

    // Additional helper methods would be implemented here...
    private identifyCallClusters(callGraph: CallGraph): void {
        // Implementation for call cluster identification
    }

    private extractVariables(ast: AST, filePath: string): Map<string, Variable> {
        // Implementation for variable extraction
        return new Map();
    }

    private traceDataFlow(ast: AST, filePath: string): DataFlow[] {
        // Implementation for data flow tracing
        return [];
    }

    private identifyDataTransformations(ast: AST, filePath: string): Map<string, DataTransformation> {
        // Implementation for data transformation identification
        return new Map();
    }

    private addImportDependency(dependencyMap: DependencyMap, filePath: string, dep: any): void {
        // Implementation for adding import dependencies
    }

    private addExportDependency(dependencyMap: DependencyMap, filePath: string, dep: any): void {
        // Implementation for adding export dependencies
    }

    private addInternalDependency(dependencyMap: DependencyMap, filePath: string, dep: any): void {
        // Implementation for adding internal dependencies
    }

    private addExternalDependency(dependencyMap: DependencyMap, filePath: string, dep: any): void {
        // Implementation for adding external dependencies
    }

    private detectCircularDependencies(dependencyMap: DependencyMap): CircularDependency[] {
        // Implementation for circular dependency detection
        return [];
    }
}

// Supporting types and interfaces
export type FileMap = Map<string, FileContent>;
export type SyntaxTreeMap = Map<string, AST>;
export type PatternMap = Map<string, Pattern>;

export interface FileContent {
    content: string;
    lastModified?: number;
    encoding?: string;
}

export interface SemanticMap {
    syntaxTree: SyntaxTreeMap;
    callGraph: CallGraph;
    dataFlow: DataFlowGraph;
    dependencies: DependencyMap;
    patterns: PatternMap;
    architecture: ArchitectureMap;
}

export interface QueryIntent {
    type: QueryType;
    scope: QueryScope;
    entities: CodeEntity[];
    context: RequiredContext;
    confidence: number;
    priority: number;
    expectedComplexity: ComplexityLevel;
}

export type QueryType = 
    | 'implementation' 
    | 'debugging' 
    | 'refactoring' 
    | 'architecture' 
    | 'explanation'
    | 'testing'
    | 'performance'
    | 'security'
    | 'general';

export type QueryScope = 'function' | 'class' | 'module' | 'system' | 'variable';

export type ComplexityLevel = 'low' | 'medium' | 'high';

export interface CodeEntity {
    type: 'function' | 'class' | 'variable' | 'file' | 'module';
    name: string;
    confidence: number;
}

export interface RequiredContext {
    includeDefinitions: boolean;
    includeUsages: boolean;
    includeDependencies: boolean;
    includeTests: boolean;
    includeDocumentation: boolean;
    includeHistory: boolean;
    includeArchitecture: boolean;
    includePerformance: boolean;
    includeSecurity: boolean;
    depth: number;
}

export interface CallGraph {
    nodes: Map<string, CallNode>;
    edges: CallEdge[];
    clusters: Map<string, CallCluster>;
}

export interface CallNode {
    id: string;
    filePath: string;
    location: any;
    signature: string;
    complexity: number;
    calls: string[];
    calledBy: string[];
}

export interface CallEdge {
    from: string;
    to: string;
    filePath: string;
    location: any;
    callType: string;
}

export interface CallCluster {
    id: string;
    nodes: string[];
    cohesion: number;
}

export interface DataFlowGraph {
    variables: Map<string, Variable>;
    flows: DataFlow[];
    transformations: Map<string, DataTransformation>;
}

export interface Variable {
    name: string;
    type: string;
    scope: string;
    location: any;
    usages: any[];
}

export interface DataFlow {
    from: string;
    to: string;
    transformation?: string;
    location: any;
}

export interface DataTransformation {
    name: string;
    input: string;
    output: string;
    location: any;
}

export interface DependencyMap {
    imports: Map<string, any>;
    exports: Map<string, any>;
    internal: Map<string, any>;
    external: Map<string, any>;
    circular: CircularDependency[];
}

export interface CircularDependency {
    files: string[];
    severity: 'low' | 'medium' | 'high';
}

export interface Pattern {
    name: string;
    type: string;
    confidence: number;
    files: string[];
    description: string;
}

export interface ArchitectureMap {
    layers: ArchitecturalLayer[];
    components: Component[];
    services: Service[];
    dataModels: DataModel[];
    apiEndpoints: APIEndpoint[];
    configurations: Configuration[];
    entryPoints: EntryPoint[];
    testStructure: TestStructure;
}

export interface ArchitecturalLayer {
    name: string;
    type: string;
    files: string[];
    dependencies: string[];
}

export interface Component {
    name: string;
    type: string;
    files: string[];
    interfaces: string[];
}

export interface Service {
    name: string;
    type: string;
    methods: string[];
    dependencies: string[];
}

export interface DataModel {
    name: string;
    fields: any[];
    relationships: any[];
}

export interface APIEndpoint {
    path: string;
    method: string;
    handler: string;
    parameters: any[];
}

export interface Configuration {
    name: string;
    type: string;
    settings: any;
}

export interface EntryPoint {
    name: string;
    file: string;
    type: string;
}

export interface TestStructure {
    testFiles: string[];
    coverage: number;
    frameworks: string[];
}

// Re-export AST type from parser
export type AST = any; // Would import from parser module

export default CodeUnderstandingEngine;
