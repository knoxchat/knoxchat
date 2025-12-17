/**
 * Unified Types for AI Context System
 * 
 * This file contains all shared types and interfaces used across the AI context system
 * to ensure consistency and avoid type conflicts.
 */

// Core Query and Intent Types
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
    scope?: string[];
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
    category?: PriorityCategory;
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

// Context Structure Types
export interface CompleteAIContext {
    core_files: ContextFile[];
    architecture: ArchitecturalContext;
    relationships: RelationshipContext;
    history: HistoryContext;
    examples: ExampleContext[];
    metadata: ContextMetadata;
    source_checkpoints: string[];
    context_type: string;
    confidence_score: number;
}

export interface ContextFile {
    path: string;
    complete_content: string;
    language: string;
    encoding: string;
    size?: number;
    last_modified?: Date;
    semantic_info: SemanticInfo;
}

export interface SemanticInfo {
    functions: FunctionInfo[];
    classes: ClassInfo[];
    interfaces: InterfaceInfo[];
    types: TypeInfo[];
    imports: ImportInfo[];
    exports: ExportInfo[];
}

export interface FunctionInfo {
    name: string;
    parameters: ParameterInfo[];
    return_type?: string;
    visibility: string;
    is_async: boolean;
    documentation?: string;
    complexity: number;
    calls: string[];
    called_by: string[];
}

export interface ClassInfo {
    name: string;
    extends?: string;
    implements: string[];
    methods: string[];
    properties: PropertyInfo[];
    visibility: string;
    design_patterns: string[];
}

export interface InterfaceInfo {
    name: string;
    extends: string[];
    methods: MethodSignature[];
    properties: PropertySignature[];
}

export interface TypeInfo {
    name: string;
    kind: string;
    definition: string;
    generic_parameters: string[];
}

export interface ImportInfo {
    module: string;
    imported_items: string[];
    alias?: string;
    is_default: boolean;
    is_namespace?: boolean;
    location?: CodeLocation;
}

export interface ExportInfo {
    exported_items: string[];
    is_default: boolean;
    alias?: string;
}

export interface ParameterInfo {
    name: string;
    type: string;
    is_optional: boolean;
    default_value?: string;
}

export interface PropertyInfo {
    name: string;
    type: string;
    visibility: string;
    is_static: boolean;
    is_readonly: boolean;
}

export interface MethodSignature {
    name: string;
    parameters: ParameterInfo[];
    return_type?: string;
    is_async: boolean;
}

export interface PropertySignature {
    name: string;
    type: string;
    is_optional: boolean;
    is_readonly: boolean;
}

export interface CodeLocation {
    file_path: string;
    start_line: number;
    end_line: number;
    start_column: number;
    end_column: number;
}

// Architectural Context Types
export interface ArchitecturalContext {
    project_structure: ProjectStructure;
    patterns_used: DesignPattern[];
    dependency_graph: DependencyGraph;
    data_flow_diagram: DataFlowDiagram;
    layers: ArchitecturalLayer[];
}

export interface ProjectStructure {
    root_directories: string[];
    module_structure: ModuleStructure[];
    package_dependencies: PackageDependency[];
}

export interface ModuleStructure {
    path: string;
    type: string;
    exports: string[];
    dependencies: string[];
}

export interface PackageDependency {
    name: string;
    version: string;
    type: "production" | "development" | "peer";
}

export interface DesignPattern {
    name: string;
    type: "creational" | "structural" | "behavioral" | "architectural";
    description: string;
    locations: string[];
    confidence: number;
}

export interface DependencyGraph {
    nodes: DependencyNode[];
    edges: DependencyEdge[];
    cycles: string[][];
}

export interface DependencyNode {
    id: string;
    type: string;
    metadata: Record<string, any>;
}

export interface DependencyEdge {
    from: string;
    to: string;
    type: string;
    strength: number;
}

export interface DataFlowDiagram {
    entry_points: string[];
    data_transformations: DataTransformation[];
    storage_interactions: StorageInteraction[];
}

export interface DataTransformation {
    input: string;
    output: string;
    transformation: string;
    location: string;
}

export interface StorageInteraction {
    type: "read" | "write" | "delete" | "update";
    target: string;
    location: string;
}

export interface ArchitecturalLayer {
    name: string;
    type: string;
    components: string[];
    responsibilities: string[];
}

// Relationship Context Types
export interface RelationshipContext {
    complete_call_graph: CallGraph;
    type_hierarchy: TypeHierarchy;
    import_graph: ImportGraph;
    usage_patterns: UsagePattern[];
}

export interface CallGraph {
    functions: CallGraphNode[];
    relationships: CallRelationship[];
}

export interface CallGraphNode {
    name: string;
    file: string;
    complexity: number;
    calls_count: number;
    called_by_count: number;
}

export interface CallRelationship {
    caller: string;
    called: string;
    call_type: string;
    frequency: number;
}

export interface TypeHierarchy {
    root_types: string[];
    inheritance_chains: InheritanceChain[];
    interface_implementations: InterfaceImplementation[];
}

export interface InheritanceChain {
    base: string;
    derived: string[];
    depth: number;
}

export interface InterfaceImplementation {
    interface: string;
    implementations: string[];
}

export interface ImportGraph {
    modules: ImportGraphNode[];
    dependencies: ImportDependency[];
}

export interface ImportGraphNode {
    module: string;
    exports: string[];
    imports: string[];
}

export interface ImportDependency {
    from: string;
    to: string;
    imported_items: string[];
}

export interface UsagePattern {
    pattern: string;
    description: string;
    frequency: number;
    locations: string[];
    confidence: number;
}

// History and Evolution Types
export interface HistoryContext {
    [x: string]: any;
    change_timeline: ChangeEvent[];
    architectural_decisions: ArchitecturalDecision[];
    refactoring_history: RefactoringEvent[];
}

// Alias for backward compatibility
export type EvolutionContext = HistoryContext;

export interface ChangeEvent {
    timestamp: Date;
    checkpoint_id: string;
    description: string;
    files_changed: string[];
    change_type: string;
    impact: string;
}

export interface ArchitecturalDecision {
    decision: string;
    reasoning: string;
    alternatives: string[];
    trade_offs: string[];
    timestamp: Date;
}

export interface RefactoringEvent {
    type: string;
    description: string;
    files_affected: string[];
    timestamp: Date;
    reasoning: string;
}

// Example Context Types
export interface ExampleContext {
    description: string;
    complete_code_example: string;
    surrounding_context: string;
    applicable_patterns: string[];
    confidence: number;
}

// Context Metadata
export interface ContextMetadata {
    checkpoints_used: string[];
    context_type: ContextType;
    confidence_score: number;
    token_count: number;
    build_time_ms: number;
    cache_hit_rate: number;
    generated_at: Date;
}

// Checkpoint Types
export interface AIContextCheckpoint {
    id: string;
    base_checkpoint: BaseCheckpoint;
    semantic_context: SemanticContext;
    intent_analysis: IntentAnalysis;
    architectural_impact: ArchitecturalImpact;
    code_relationships: CodeRelationships;
    confidence_score: number;
    created_at: Date;
    file_changes: FileChange[];
    git_metadata?: GitMetadata;
}

export interface GitMetadata {
    commit_hash: string;
    author: string;
    message: string;
    branch: string;
    parent_commits: string[];
    merge_commit: boolean;
}

export interface BaseCheckpoint {
    id: string;
    session_id: string;
    description: string;
    created_at: Date;
    file_changes: FileChange[];
    files_affected: number;
    size_bytes: number;
    tags: string[];
    metadata: Record<string, any>;
}

export interface FileChange {
    path: string;
    change_type: string;
    new_content?: string;
    original_content?: string;
    size_bytes: number;
    content_hash: string;
    permissions?: any;
    modified_at: Date;
    encoding: string;
    compressed: boolean;
}

export interface SemanticContext {
    functions: Map<string, FunctionInfo>;
    classes: Map<string, ClassInfo>;
    interfaces: Map<string, InterfaceInfo>;
    types: Map<string, TypeInfo>;
    constants: Map<string, any>;
    imports: ImportInfo[];
    exports: ExportInfo[];
    call_chains: any[];
    inheritance_tree: any;
    dependency_graph: any;
    usage_patterns: any[];
}

export interface IntentAnalysis {
    change_intent: any;
    affected_features: string[];
    design_patterns_used: DesignPattern[];
    architectural_decisions: ArchitecturalDecision[];
    refactoring_type?: string;
    confidence: number;
}

export interface ArchitecturalImpact {
    layers_affected: string[];
    patterns_introduced: string[];
    patterns_modified: string[];
    dependency_changes: any[];
    boundary_changes: any[];
    significance: "Low" | "Medium" | "High";
}

export interface CodeRelationships {
    direct_dependencies: string[];
    transitive_dependencies: string[];
    dependents: string[];
    coupling_strength: Record<string, any>;
    cohesion_metrics: CohesionMetrics;
}

export interface CohesionMetrics {
    functional_cohesion: number;
    sequential_cohesion: number;
    communicational_cohesion: number;
    procedural_cohesion: number;
    temporal_cohesion: number;
    logical_cohesion: number;
    coincidental_cohesion: number;
}

// Relevance Engine Types
export interface RelevanceScore {
    semantic: number;
    temporal: number;
    architectural: number;
    dependency: number;
    usage: number;
    composite: number;
    confidence: number;
    reasoning: string;
}

export interface RelevanceWeights {
    semantic: number;
    temporal: number;
    architectural: number;
    dependency: number;
    usage: number;
}

export interface ScoredCheckpoint {
    checkpoint: AIContextCheckpoint;
    score: RelevanceScore;
}

export interface WorkspaceContext {
    workspace_path: string;
    current_files: string[];
    recent_changes: any[];
    project_metadata: any;
}

// Context Tree Types
export interface ContextTree {
    getCoreContext(): any;
    getArchitecturalContext(): any;
    getRelationshipContext(): any;
    getEvolutionContext(): any;
    getExampleContext(): any;
}

// Optimization Types
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

// Additional helper interfaces
export interface RelevanceScores {
    semantic: number;
    temporal: number;
    architectural: number;
    dependency: number;
    usage: number;
    composite: number;
}
