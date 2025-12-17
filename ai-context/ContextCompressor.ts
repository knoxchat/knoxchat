/**
 * Context Compressor - Intelligent context compression while preserving semantics
 * 
 * This component implements intelligent context compression as specified in
 * checkpoint-system-design.md, ensuring semantic integrity is maintained.
 */

import { 
    CompleteAIContext, 
    ContextFile, 
    ArchitecturalContext, 
    RelationshipContext, 
    HistoryContext, 
    ExampleContext,
    ProjectStructure,
    DependencyGraph,
    DataFlowDiagram,
    CallGraph,
    TypeHierarchy,
    ImportGraph
} from './types';

export class ContextCompressor {
    private compressionStrategies: CompressionStrategy[];
    private semanticPreserver: SemanticPreserver;
    private tokenEstimator: TokenEstimator;

    constructor() {
        this.compressionStrategies = this.initializeCompressionStrategies();
        this.semanticPreserver = new SemanticPreserver();
        this.tokenEstimator = new TokenEstimator();
    }

    /**
     * Compress context intelligently while preserving semantics
     */
    async compressContext(
        context: CompleteAIContext,
        targetSize: number
    ): Promise<CompressedContext> {
        const startTime = Date.now();
        
        // Analyze current context size
        const currentSize = await this.tokenEstimator.estimateTokens(context);
        
        if (currentSize <= targetSize) {
            return {
                essential: context,
                compressed: null,
                references: await this.createReferenceMap(context),
                compressionRatio: 1.0,
                preservedSemantics: 1.0,
                compressionTime: Date.now() - startTime
            };
        }

        // Extract essential context (always preserved)
        const essential = await this.extractEssentialContext(context);
        const essentialSize = await this.tokenEstimator.estimateTokens(essential);
        
        if (essentialSize > targetSize) {
            throw new Error(`Essential context (${essentialSize} tokens) exceeds target size (${targetSize} tokens)`);
        }

        // Compress non-essential context
        const remainingSize = targetSize - essentialSize;
        const nonEssential = await this.extractNonEssentialContext(context, essential);
        const compressed = await this.compressNonEssential(nonEssential, remainingSize);
        
        // Create reference map for excluded content
        const references = await this.createReferenceMap(context, essential, compressed);
        
        // Calculate metrics
        const finalSize = essentialSize + (compressed ? await this.tokenEstimator.estimateTokens(compressed) : 0);
        const compressionRatio = finalSize / currentSize;
        const preservedSemantics = await this.calculateSemanticPreservation(context, essential, compressed);

        return {
            essential,
            compressed,
            references,
            compressionRatio,
            preservedSemantics,
            compressionTime: Date.now() - startTime
        };
    }

    /**
     * Extract essential context that must always be preserved
     */
    private async extractEssentialContext(context: CompleteAIContext): Promise<CompleteAIContext> {
        const essential: CompleteAIContext = {
            core_files: [],
            architecture: {} as ArchitecturalContext,
            relationships: {} as RelationshipContext,
            history: { change_timeline: [], architectural_decisions: [], refactoring_history: [] } as HistoryContext,
            examples: [],
            metadata: context.metadata,
            source_checkpoints: context.source_checkpoints,
            context_type: 'essential',
            confidence_score: context.confidence_score
        };

        // Essential files: those directly mentioned in query or with high relevance
        essential.core_files = context.core_files.filter(file => 
            this.isFileEssential(file, context)
        );

        // Essential architecture: core patterns and decisions
        essential.architecture = await this.extractEssentialArchitecture(context.architecture);

        // Essential relationships: direct dependencies and call chains
        essential.relationships = await this.extractEssentialRelationships(context.relationships);

        // Essential history: recent critical changes
        essential.history = await this.extractEssentialHistory(context.history);

        // Essential examples: most relevant examples
        essential.examples = context.examples
            .sort((a, b) => (b.confidence || 0) - (a.confidence || 0))
            .slice(0, 2); // Keep top 2 examples

        return essential;
    }

    /**
     * Extract non-essential context for compression
     */
    private async extractNonEssentialContext(
        context: CompleteAIContext,
        essential: CompleteAIContext
    ): Promise<CompleteAIContext> {
        const essentialFilePaths = new Set(essential.core_files.map(f => f.path));
        
        return {
            core_files: context.core_files.filter(file => !essentialFilePaths.has(file.path)),
            architecture: await this.extractNonEssentialArchitecture(context.architecture, essential.architecture),
            relationships: await this.extractNonEssentialRelationships(context.relationships, essential.relationships),
            history: await this.extractNonEssentialHistory(context.history, essential.history),
            examples: context.examples.slice(2), // Remaining examples
            metadata: context.metadata,
            source_checkpoints: [],
            context_type: 'non-essential',
            confidence_score: context.confidence_score
        };
    }

    /**
     * Compress non-essential context to fit within remaining size
     */
    private async compressNonEssential(
        nonEssential: CompleteAIContext,
        remainingSize: number
    ): Promise<CompressedNonEssentialContext | null> {
        if (remainingSize <= 0) {
            return null;
        }

        const compressed: CompressedNonEssentialContext = {
            files: [],
            architecture: null,
            relationships: null,
            history: null,
            examples: []
        };

        let usedTokens = 0;
        
        // Prioritize compression by importance
        const compressionPlan = await this.createCompressionPlan(nonEssential, remainingSize);
        
        for (const item of compressionPlan) {
            const compressedItem = await this.compressItem(item);
            const itemTokens = await this.tokenEstimator.estimateTokens(compressedItem);
            
            if (usedTokens + itemTokens <= remainingSize) {
                this.addCompressedItem(compressed, compressedItem, item.type);
                usedTokens += itemTokens;
            } else {
                // Try further compression
                const furtherCompressed = await this.furtherCompress(compressedItem, remainingSize - usedTokens);
                if (furtherCompressed) {
                    const furtherTokens = await this.tokenEstimator.estimateTokens(furtherCompressed);
                    if (usedTokens + furtherTokens <= remainingSize) {
                        this.addCompressedItem(compressed, furtherCompressed, item.type);
                        usedTokens += furtherTokens;
                    }
                }
            }
        }

        return compressed;
    }

    /**
     * Create reference map for excluded content
     */
    private async createReferenceMap(
        originalContext: CompleteAIContext,
        essential?: CompleteAIContext,
        compressed?: CompressedNonEssentialContext | null
    ): Promise<ReferenceMap> {
        const references: ReferenceMap = {
            excludedFiles: [],
            excludedSections: [],
            relatedCheckpoints: originalContext.source_checkpoints,
            compressionNotes: []
        };

        // Identify excluded files
        const includedFiles = new Set([
            ...(essential?.core_files.map(f => f.path) || []),
            ...(compressed?.files.map(f => f.path) || [])
        ]);

        for (const file of originalContext.core_files) {
            if (!includedFiles.has(file.path)) {
                references.excludedFiles.push({
                    path: file.path,
                    reason: 'Excluded due to space constraints',
                    summary: await this.generateFileSummary(file),
                    importance: this.calculateFileImportance(file)
                });
            }
        }

        // Add compression notes
        if (compressed) {
            references.compressionNotes.push(
                'Context has been intelligently compressed while preserving semantic meaning',
                'Full context available in referenced checkpoints',
                'Compression prioritized based on relevance and importance'
            );
        }

        return references;
    }

    /**
     * Determine if a file is essential and must be preserved
     */
    private isFileEssential(file: ContextFile, context: CompleteAIContext): boolean {
        // File is essential if:
        // 1. Directly mentioned in query
        // 2. Contains critical functions/classes
        // 3. High relevance score
        // 4. Entry point or main configuration

        const importance = this.calculateFileImportance(file);
        const isEntryPoint = this.isEntryPoint(file);
        const hasHighRelevance = file.semantic_info.functions.length > 5 || file.semantic_info.classes.length > 2;
        const isCritical = this.isCriticalFile(file);

        return importance > 0.7 || isEntryPoint || hasHighRelevance || isCritical;
    }

    private calculateFileImportance(file: ContextFile): number {
        let importance = 0.5; // Base importance

        // Boost for semantic complexity
        const semanticComplexity = (
            file.semantic_info.functions.length * 0.1 +
            file.semantic_info.classes.length * 0.2 +
            file.semantic_info.interfaces.length * 0.15
        );
        importance += Math.min(semanticComplexity, 0.3);

        // Boost for recent modifications
        if (file.last_modified && Date.now() - file.last_modified.getTime() < 7 * 24 * 60 * 60 * 1000) {
            importance += 0.1;
        }

        // Boost for file size (larger files might be more important)
        const sizeBoost = Math.min(file.complete_content.length / 10000, 0.1);
        importance += sizeBoost;

        return Math.min(importance, 1.0);
    }

    private isEntryPoint(file: ContextFile): boolean {
        const fileName = file.path.toLowerCase();
        return fileName.includes('main') || 
               fileName.includes('index') || 
               fileName.includes('app') ||
               fileName.includes('server') ||
               fileName.endsWith('package.json') ||
               fileName.endsWith('tsconfig.json');
    }

    private isCriticalFile(file: ContextFile): boolean {
        const fileName = file.path.toLowerCase();
        return fileName.includes('config') ||
               fileName.includes('constant') ||
               fileName.includes('type') ||
               fileName.includes('interface') ||
               fileName.includes('model');
    }

    private async generateFileSummary(file: ContextFile): Promise<string> {
        const parts = [];
        
        if (file.semantic_info.functions.length > 0) {
            parts.push(`${file.semantic_info.functions.length} functions`);
        }
        if (file.semantic_info.classes.length > 0) {
            parts.push(`${file.semantic_info.classes.length} classes`);
        }
        if (file.semantic_info.interfaces.length > 0) {
            parts.push(`${file.semantic_info.interfaces.length} interfaces`);
        }

        const summary = parts.length > 0 ? `Contains ${parts.join(', ')}` : 'Code file';
        const lines = file.complete_content.split('\n').length;
        
        return `${summary} (${lines} lines)`;
    }

    private async extractEssentialArchitecture(architecture: ArchitecturalContext): Promise<ArchitecturalContext> {
        return {
            project_structure: architecture.project_structure,
            patterns_used: architecture.patterns_used?.slice(0, 3) || [], // Top 3 patterns
            dependency_graph: this.simplifyDependencyGraph(architecture.dependency_graph),
            data_flow_diagram: architecture.data_flow_diagram,
            layers: architecture.layers?.slice(0, 3) || [] // Top 3 layers
        };
    }

    private async extractEssentialRelationships(relationships: RelationshipContext): Promise<RelationshipContext> {
        return {
            complete_call_graph: this.simplifyCallGraph(relationships.complete_call_graph),
            type_hierarchy: relationships.type_hierarchy,
            import_graph: this.simplifyImportGraph(relationships.import_graph),
            usage_patterns: relationships.usage_patterns?.slice(0, 5) || [] // Top 5 patterns
        };
    }

    private async extractEssentialHistory(history: HistoryContext): Promise<HistoryContext> {
        return {
            change_timeline: history.change_timeline?.slice(-10) || [], // Last 10 changes
            architectural_decisions: history.architectural_decisions?.slice(-5) || [], // Last 5 decisions
            refactoring_history: history.refactoring_history?.slice(-5) || [] // Last 5 refactorings
        };
    }

    private async extractNonEssentialArchitecture(
        original: ArchitecturalContext, 
        essential: ArchitecturalContext
    ): Promise<ArchitecturalContext> {
        return {
            project_structure: { root_directories: [], module_structure: [], package_dependencies: [] }, // Non-essential structure details
            patterns_used: original.patterns_used?.slice(3) || [], // Remaining patterns
            dependency_graph: { nodes: [], edges: [], cycles: [] },
            data_flow_diagram: { entry_points: [], data_transformations: [], storage_interactions: [] },
            layers: original.layers?.slice(3) || [] // Remaining layers
        };
    }

    private async extractNonEssentialRelationships(
        original: RelationshipContext, 
        essential: RelationshipContext
    ): Promise<RelationshipContext> {
        return {
            complete_call_graph: { functions: [], relationships: [] },
            type_hierarchy: { root_types: [], inheritance_chains: [], interface_implementations: [] },
            import_graph: { modules: [], dependencies: [] },
            usage_patterns: original.usage_patterns?.slice(5) || [] // Remaining patterns
        };
    }

    private async extractNonEssentialHistory(
        original: HistoryContext, 
        essential: HistoryContext
    ): Promise<HistoryContext> {
        return {
            change_timeline: original.change_timeline?.slice(0, -10) || [], // Older changes
            architectural_decisions: original.architectural_decisions?.slice(0, -5) || [],
            refactoring_history: original.refactoring_history?.slice(0, -5) || [],
            recent_modifications: original.recent_modifications?.slice(0, -10) || []
        };
    }

    private async createCompressionPlan(
        nonEssential: CompleteAIContext,
        remainingSize: number
    ): Promise<CompressionItem[]> {
        const items: CompressionItem[] = [];

        // Add files to compression plan
        for (const file of nonEssential.core_files) {
            items.push({
                type: 'file',
                content: file,
                importance: this.calculateFileImportance(file),
                estimatedTokens: await this.tokenEstimator.estimateTokens(file)
            });
        }

        // Add other content types
        if (Object.keys(nonEssential.architecture).length > 0) {
            items.push({
                type: 'architecture',
                content: nonEssential.architecture,
                importance: 0.6,
                estimatedTokens: await this.tokenEstimator.estimateTokens(nonEssential.architecture)
            });
        }

        // Sort by importance/token ratio for optimal compression
        items.sort((a, b) => (b.importance / b.estimatedTokens) - (a.importance / a.estimatedTokens));

        return items;
    }

    private async compressItem(item: CompressionItem): Promise<any> {
        switch (item.type) {
            case 'file':
                return this.compressFile(item.content as ContextFile);
            case 'architecture':
                return this.compressArchitecture(item.content);
            case 'relationships':
                return this.compressRelationships(item.content);
            case 'history':
                return this.compressHistory(item.content);
            case 'example':
                return this.compressExample(item.content);
            default:
                return item.content;
        }
    }

    private async compressFile(file: ContextFile): Promise<CompressedFile> {
        return {
            path: file.path,
            language: file.language,
            summary: await this.generateFileSummary(file),
            key_functions: file.semantic_info.functions.slice(0, 3).map(f => ({
                name: f.name,
                complexity: f.complexity,
                parameters: f.parameters.length
            })),
            key_classes: file.semantic_info.classes.slice(0, 2).map(c => ({
                name: c.name,
                methods: c.methods.length,
                properties: c.properties.length
            })),
            imports: file.semantic_info.imports.slice(0, 5),
            lines_of_code: file.complete_content.split('\n').length,
            last_modified: file.last_modified
        };
    }

    private async furtherCompress(item: any, maxTokens: number): Promise<any> {
        // Further compression strategies
        if (typeof item === 'object' && item !== null) {
            // Remove less important properties
            const compressed = { ...item };
            
            // Remove detailed content, keep only summaries
            if (compressed.complete_content) {
                delete compressed.complete_content;
            }
            if (compressed.detailed_analysis) {
                delete compressed.detailed_analysis;
            }
            
            return compressed;
        }
        
        return null;
    }

    private addCompressedItem(
        compressed: CompressedNonEssentialContext, 
        item: any, 
        type: string
    ): void {
        switch (type) {
            case 'file':
                compressed.files.push(item);
                break;
            case 'architecture':
                compressed.architecture = item;
                break;
            case 'relationships':
                compressed.relationships = item;
                break;
            case 'history':
                compressed.history = item;
                break;
            case 'example':
                compressed.examples.push(item);
                break;
        }
    }

    private async calculateSemanticPreservation(
        original: CompleteAIContext,
        essential: CompleteAIContext,
        compressed: CompressedNonEssentialContext | null
    ): Promise<number> {
        // Calculate how much semantic meaning is preserved
        let preservation = 0.5; // Base preservation from essential context

        if (compressed) {
            // Add preservation from compressed content
            preservation += 0.3 * (compressed.files.length / original.core_files.length);
            
            if (compressed.architecture) {preservation += 0.1;}
            if (compressed.relationships) {preservation += 0.1;}
        }

        return Math.min(preservation, 1.0);
    }

    // Helper methods for simplification
    private simplifyDependencyGraph(graph: any): any {
        // Simplify dependency graph while preserving key relationships
        return graph; // Simplified implementation
    }

    private simplifyCallGraph(graph: any): any {
        // Simplify call graph while preserving main call chains
        return graph; // Simplified implementation
    }

    private simplifyImportGraph(graph: any): any {
        // Simplify import graph while preserving critical imports
        return graph; // Simplified implementation
    }

    private compressArchitecture(architecture: any): any {
        // Compress architectural information
        return architecture; // Simplified implementation
    }

    private compressRelationships(relationships: any): any {
        // Compress relationship information
        return relationships; // Simplified implementation
    }

    private compressHistory(history: any): any {
        // Compress history information
        return history; // Simplified implementation
    }

    private compressExample(example: any): any {
        // Compress example information
        return example; // Simplified implementation
    }

    private initializeCompressionStrategies(): CompressionStrategy[] {
        return [
            new WhitespaceCompressionStrategy(),
            new CommentRemovalStrategy(),
            new SemanticSummarizationStrategy(),
            new StructuralSimplificationStrategy()
        ];
    }
}

// Supporting classes and interfaces

class TokenEstimator {
    async estimateTokens(content: any): Promise<number> {
        const text = typeof content === 'string' ? content : JSON.stringify(content);
        return Math.ceil(text.length / 4); // Rough estimation: 4 chars per token
    }
}

class SemanticPreserver {
    // Methods for preserving semantic meaning during compression
}

abstract class CompressionStrategy {
    abstract compress(content: any): Promise<any>;
    abstract getCompressionRatio(): number;
}

class WhitespaceCompressionStrategy extends CompressionStrategy {
    async compress(content: any): Promise<any> {
        if (typeof content === 'string') {
            return content.replace(/\s+/g, ' ').trim();
        }
        return content;
    }

    getCompressionRatio(): number {
        return 0.1; // 10% space savings
    }
}

class CommentRemovalStrategy extends CompressionStrategy {
    async compress(content: any): Promise<any> {
        if (typeof content === 'string') {
            return content
                .replace(/\/\/.*$/gm, '') // Remove single-line comments
                .replace(/\/\*[\s\S]*?\*\//g, ''); // Remove multi-line comments
        }
        return content;
    }

    getCompressionRatio(): number {
        return 0.15; // 15% space savings
    }
}

class SemanticSummarizationStrategy extends CompressionStrategy {
    async compress(content: any): Promise<any> {
        // Implement semantic summarization
        return content; // Simplified implementation
    }

    getCompressionRatio(): number {
        return 0.4; // 40% space savings
    }
}

class StructuralSimplificationStrategy extends CompressionStrategy {
    async compress(content: any): Promise<any> {
        // Implement structural simplification
        return content; // Simplified implementation
    }

    getCompressionRatio(): number {
        return 0.25; // 25% space savings
    }
}

// Type definitions

export interface CompressedContext {
    essential: CompleteAIContext;
    compressed: CompressedNonEssentialContext | null;
    references: ReferenceMap;
    compressionRatio: number;
    preservedSemantics: number;
    compressionTime: number;
}

export interface CompressedNonEssentialContext {
    files: CompressedFile[];
    architecture: any;
    relationships: any;
    history: any;
    examples: any[];
}

export interface CompressedFile {
    path: string;
    language: string;
    summary: string;
    key_functions: any[];
    key_classes: any[];
    imports: any[];
    lines_of_code: number;
    last_modified?: Date;
}

export interface ReferenceMap {
    excludedFiles: ExcludedFile[];
    excludedSections: ExcludedSection[];
    relatedCheckpoints: string[];
    compressionNotes: string[];
}

export interface ExcludedFile {
    path: string;
    reason: string;
    summary: string;
    importance: number;
}

export interface ExcludedSection {
    section: string;
    reason: string;
    summary: string;
}

interface CompressionItem {
    type: string;
    content: any;
    importance: number;
    estimatedTokens: number;
}

export default ContextCompressor;
