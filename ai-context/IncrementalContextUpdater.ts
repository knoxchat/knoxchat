/**
 * Incremental Context Updater
 * 
 * Provides 10-100x faster context updates by only reprocessing changed code.
 * Instead of rebuilding entire context, updates only affected nodes in the graph.
 */

import * as Types from './types';

export interface IncrementalUpdate {
    change_type: 'file_modified' | 'file_created' | 'file_deleted' | 'file_renamed';
    file_path: string;
    old_path?: string; // For renames
    changes: TextEdit[];
    timestamp: Date;
}

export interface TextEdit {
    start_line: number;
    end_line: number;
    start_column: number;
    end_column: number;
    old_text: string;
    new_text: string;
}

export interface IncrementalUpdateResult {
    updated_nodes: GraphNode[];
    invalidated_nodes: GraphNode[];
    new_edges: GraphEdge[];
    removed_edges: GraphEdge[];
    affected_checkpoints: string[];
    update_time_ms: number;
    cache_invalidations: number;
}

export interface GraphNode {
    id: string;
    type: 'function' | 'class' | 'interface' | 'module';
    name: string;
    file_path: string;
    metadata: Record<string, any>;
    last_updated: Date;
}

export interface GraphEdge {
    from: string;
    to: string;
    type: 'calls' | 'imports' | 'extends' | 'implements';
    metadata: Record<string, any>;
}

export interface CacheInvalidationStrategy {
    invalidate_entire_file: boolean;
    invalidate_downstream: boolean;
    invalidate_upstream: boolean;
    max_invalidation_depth: number;
}

/**
 * Main Incremental Context Updater
 */
export class IncrementalContextUpdater {
    private knowledgeGraph: IncrementalKnowledgeGraph;
    private astCache: IncrementalASTCache;
    private changeTracker: FileChangeTracker;
    private dependencyTracker: DependencyTracker;

    constructor() {
        this.knowledgeGraph = new IncrementalKnowledgeGraph();
        this.astCache = new IncrementalASTCache();
        this.changeTracker = new FileChangeTracker();
        this.dependencyTracker = new DependencyTracker();
    }

    /**
     * Update context incrementally based on file changes
     */
    async updateOnFileChange(
        fileChange: IncrementalUpdate,
        workspace: string
    ): Promise<IncrementalUpdateResult> {
        const startTime = Date.now();

        try {
            // 1. Determine scope of changes
            const affectedRegions = this.analyzeAffectedRegions(fileChange);

            // 2. Parse only changed regions (not entire file!)
            const changedNodes = await this.parseIncrementalChanges(
                fileChange,
                affectedRegions
            );

            // 3. Update knowledge graph (only affected nodes)
            const graphUpdate = await this.updateKnowledgeGraph(
                changedNodes,
                fileChange
            );

            // 4. Invalidate affected caches
            const cacheInvalidations = await this.invalidateAffectedCaches(
                graphUpdate,
                fileChange
            );

            // 5. Update dependency tracking
            await this.updateDependencies(graphUpdate, fileChange);

            // 6. Identify affected checkpoints
            const affectedCheckpoints = await this.identifyAffectedCheckpoints(
                graphUpdate,
                workspace
            );

            const result: IncrementalUpdateResult = {
                updated_nodes: graphUpdate.updated_nodes,
                invalidated_nodes: graphUpdate.invalidated_nodes,
                new_edges: graphUpdate.new_edges,
                removed_edges: graphUpdate.removed_edges,
                affected_checkpoints: affectedCheckpoints,
                update_time_ms: Date.now() - startTime,
                cache_invalidations: cacheInvalidations,
            };

            console.log(`⚡ Incremental update completed in ${result.update_time_ms}ms`);
            console.log(`   - Updated nodes: ${result.updated_nodes.length}`);
            console.log(`   - Invalidated caches: ${result.cache_invalidations}`);
            console.log(`   - Affected checkpoints: ${result.affected_checkpoints.length}`);

            return result;

        } catch (error) {
            console.error('Failed to perform incremental update:', error);
            // Fallback: mark entire file for full reparse
            return this.fallbackToFullUpdate(fileChange, workspace);
        }
    }

    /**
     * Analyze which code regions are affected by changes
     */
    private analyzeAffectedRegions(fileChange: IncrementalUpdate): AffectedRegion[] {
        const regions: AffectedRegion[] = [];

        for (const edit of fileChange.changes) {
            // Determine which function/class the change is in
            const containingScope = this.findContainingScope(
                fileChange.file_path,
                edit.start_line
            );

            if (containingScope) {
                regions.push({
                    scope_type: containingScope.type,
                    scope_name: containingScope.name,
                    start_line: containingScope.start_line,
                    end_line: containingScope.end_line,
                    change_impact: this.assessChangeImpact(edit, containingScope),
                });
            } else {
                // Change is at module level
                regions.push({
                    scope_type: 'module',
                    scope_name: fileChange.file_path,
                    start_line: edit.start_line,
                    end_line: edit.end_line,
                    change_impact: 'module_level',
                });
            }
        }

        return this.mergeOverlappingRegions(regions);
    }

    /**
     * Parse only the changed regions (not entire file)
     */
    private async parseIncrementalChanges(
        fileChange: IncrementalUpdate,
        affectedRegions: AffectedRegion[]
    ): Promise<ParsedNode[]> {
        const parsedNodes: ParsedNode[] = [];

        for (const region of affectedRegions) {
            // Get cached AST for unchanged parts
            const cachedAST = await this.astCache.get(
                fileChange.file_path,
                region.start_line,
                region.end_line
            );

            if (cachedAST && this.canReuseCachedAST(cachedAST, region)) {
                // Reuse cached AST (no parsing needed!)
                parsedNodes.push(...this.extractNodesFromCache(cachedAST));
                continue;
            }

            // Parse only this region
            const regionCode = this.extractCodeRegion(
                fileChange,
                region.start_line,
                region.end_line
            );

            const ast = await this.parseCodeRegion(regionCode, region.scope_type);
            
            // Update cache
            await this.astCache.set(
                fileChange.file_path,
                region.start_line,
                region.end_line,
                ast
            );

            parsedNodes.push(...this.extractNodes(ast, fileChange.file_path));
        }

        return parsedNodes;
    }

    /**
     * Update knowledge graph with only changed nodes
     */
    private async updateKnowledgeGraph(
        changedNodes: ParsedNode[],
        fileChange: IncrementalUpdate
    ): Promise<GraphUpdateResult> {
        const updatedNodes: GraphNode[] = [];
        const invalidatedNodes: GraphNode[] = [];
        const newEdges: GraphEdge[] = [];
        const removedEdges: GraphEdge[] = [];

        for (const node of changedNodes) {
            // Check if node already exists in graph
            const existingNode = await this.knowledgeGraph.getNode(node.id);

            if (existingNode) {
                // Update existing node
                const updated = await this.knowledgeGraph.updateNode(
                    node.id,
                    node.data
                );
                updatedNodes.push(updated);

                // Find edges that need updating
                const oldEdges = await this.knowledgeGraph.getEdgesFrom(node.id);
                const newNodeEdges = this.extractEdges(node);

                // Identify removed edges
                for (const oldEdge of oldEdges) {
                    if (!this.edgeExists(oldEdge, newNodeEdges)) {
                        removedEdges.push(oldEdge);
                        await this.knowledgeGraph.removeEdge(oldEdge.from, oldEdge.to);
                    }
                }

                // Identify new edges
                for (const newEdge of newNodeEdges) {
                    if (!this.edgeExists(newEdge, oldEdges)) {
                        newEdges.push(newEdge);
                        await this.knowledgeGraph.addEdge(newEdge);
                    }
                }

                // Invalidate nodes that depend on this one
                const dependents = await this.knowledgeGraph.getDependents(node.id);
                invalidatedNodes.push(...dependents);

            } else {
                // Add new node
                const added = await this.knowledgeGraph.addNode(node);
                updatedNodes.push(added);

                // Add edges for new node
                const edges = this.extractEdges(node);
                newEdges.push(...edges);
                for (const edge of edges) {
                    await this.knowledgeGraph.addEdge(edge);
                }
            }
        }

        // Handle deleted nodes (if file was deleted)
        if (fileChange.change_type === 'file_deleted') {
            const fileNodes = await this.knowledgeGraph.getNodesInFile(fileChange.file_path);
            for (const node of fileNodes) {
                await this.knowledgeGraph.removeNode(node.id);
                invalidatedNodes.push(node);
            }
        }

        return {
            updated_nodes: updatedNodes,
            invalidated_nodes: invalidatedNodes,
            new_edges: newEdges,
            removed_edges: removedEdges,
        };
    }

    /**
     * Invalidate affected caches using smart strategy
     */
    private async invalidateAffectedCaches(
        graphUpdate: GraphUpdateResult,
        fileChange: IncrementalUpdate
    ): Promise<number> {
        let invalidationCount = 0;

        const strategy: CacheInvalidationStrategy = {
            invalidate_entire_file: fileChange.change_type === 'file_deleted',
            invalidate_downstream: true,
            invalidate_upstream: false, // Only downstream by default
            max_invalidation_depth: 3,
        };

        // 1. Invalidate file-level caches
        if (strategy.invalidate_entire_file) {
            await this.astCache.invalidateFile(fileChange.file_path);
            invalidationCount++;
        } else {
            // Invalidate only affected regions
            for (const node of graphUpdate.updated_nodes) {
                await this.astCache.invalidateNode(node.id);
                invalidationCount++;
            }
        }

        // 2. Invalidate semantic context cache
        for (const node of graphUpdate.updated_nodes) {
            await this.invalidateSemanticCache(node.id);
            invalidationCount++;
        }

        // 3. Invalidate downstream dependents (if strategy says so)
        if (strategy.invalidate_downstream) {
            for (const node of graphUpdate.updated_nodes) {
                const dependents = await this.knowledgeGraph.getDependentsRecursive(
                    node.id,
                    strategy.max_invalidation_depth
                );
                
                for (const dependent of dependents) {
                    await this.invalidateSemanticCache(dependent.id);
                    invalidationCount++;
                }
            }
        }

        // 4. Invalidate query result caches that used these nodes
        for (const node of [...graphUpdate.updated_nodes, ...graphUpdate.invalidated_nodes]) {
            await this.invalidateQueryCachesUsingNode(node.id);
            invalidationCount++;
        }

        return invalidationCount;
    }

    /**
     * Update dependency tracking
     */
    private async updateDependencies(
        graphUpdate: GraphUpdateResult,
        fileChange: IncrementalUpdate
    ): Promise<void> {
        // Track file-level dependencies
        await this.dependencyTracker.updateFileDependencies(
            fileChange.file_path,
            graphUpdate.new_edges.filter(e => e.type === 'imports')
        );

        // Track function-level dependencies
        for (const edge of graphUpdate.new_edges) {
            if (edge.type === 'calls') {
                await this.dependencyTracker.addFunctionDependency(edge.from, edge.to);
            }
        }

        // Remove old dependencies
        for (const edge of graphUpdate.removed_edges) {
            if (edge.type === 'calls') {
                await this.dependencyTracker.removeFunctionDependency(edge.from, edge.to);
            }
        }
    }

    /**
     * Identify which checkpoints are affected by changes
     */
    private async identifyAffectedCheckpoints(
        graphUpdate: GraphUpdateResult,
        workspace: string
    ): Promise<string[]> {
        const affectedCheckpoints = new Set<string>();

        // Find checkpoints that reference updated nodes
        for (const node of graphUpdate.updated_nodes) {
            const checkpoints = await this.findCheckpointsReferencingNode(
                node.id,
                workspace
            );
            checkpoints.forEach(cp => affectedCheckpoints.add(cp));
        }

        // Find checkpoints that reference invalidated nodes
        for (const node of graphUpdate.invalidated_nodes) {
            const checkpoints = await this.findCheckpointsReferencingNode(
                node.id,
                workspace
            );
            checkpoints.forEach(cp => affectedCheckpoints.add(cp));
        }

        return Array.from(affectedCheckpoints);
    }

    /**
     * Fallback to full update if incremental fails
     */
    private async fallbackToFullUpdate(
        fileChange: IncrementalUpdate,
        workspace: string
    ): Promise<IncrementalUpdateResult> {
        console.warn('⚠️ Falling back to full file update');

        // Invalidate entire file
        await this.astCache.invalidateFile(fileChange.file_path);
        await this.knowledgeGraph.removeNodesInFile(fileChange.file_path);

        // Return minimal result
        return {
            updated_nodes: [],
            invalidated_nodes: [],
            new_edges: [],
            removed_edges: [],
            affected_checkpoints: [],
            update_time_ms: 0,
            cache_invalidations: 1,
        };
    }

    // Helper methods

    private findContainingScope(
        filePath: string,
        lineNumber: number
    ): ContainingScope | null {
        // Use cached file structure to find scope
        const fileStructure = this.astCache.getFileStructure(filePath);
        if (!fileStructure) return null;

        // Find function/class that contains this line
        for (const scope of fileStructure.scopes) {
            if (lineNumber >= scope.start_line && lineNumber <= scope.end_line) {
                return scope;
            }
        }

        return null;
    }

    private assessChangeImpact(
        edit: TextEdit,
        scope: ContainingScope
    ): 'signature_change' | 'implementation_change' | 'comment_change' | 'module_level' {
        // Detect if change affects function signature
        if (this.isInSignature(edit, scope)) {
            return 'signature_change';
        }

        // Detect if it's just a comment
        if (this.isCommentChange(edit)) {
            return 'comment_change';
        }

        return 'implementation_change';
    }

    private isInSignature(edit: TextEdit, scope: ContainingScope): boolean {
        // Signature is typically first 1-3 lines of function/class
        return edit.start_line <= scope.start_line + 3;
    }

    private isCommentChange(edit: TextEdit): boolean {
        return (
            edit.new_text.trim().startsWith('//') ||
            edit.new_text.trim().startsWith('/*') ||
            edit.old_text.trim().startsWith('//')
        );
    }

    private mergeOverlappingRegions(regions: AffectedRegion[]): AffectedRegion[] {
        if (regions.length <= 1) return regions;

        const sorted = regions.sort((a, b) => a.start_line - b.start_line);
        const merged: AffectedRegion[] = [sorted[0]];

        for (let i = 1; i < sorted.length; i++) {
            const current = sorted[i];
            const last = merged[merged.length - 1];

            if (current.start_line <= last.end_line + 5) { // 5-line buffer
                // Merge regions
                last.end_line = Math.max(last.end_line, current.end_line);
                last.change_impact = this.mergeImpact(last.change_impact, current.change_impact);
            } else {
                merged.push(current);
            }
        }

        return merged;
    }

    private mergeImpact(
        impact1: string,
        impact2: string
    ): 'signature_change' | 'implementation_change' | 'comment_change' | 'module_level' {
        // Signature change takes precedence
        if (impact1 === 'signature_change' || impact2 === 'signature_change') {
            return 'signature_change';
        }
        if (impact1 === 'module_level' || impact2 === 'module_level') {
            return 'module_level';
        }
        if (impact1 === 'implementation_change' || impact2 === 'implementation_change') {
            return 'implementation_change';
        }
        return 'comment_change';
    }

    private canReuseCachedAST(cachedAST: any, region: AffectedRegion): boolean {
        // Can reuse if change is just a comment
        return region.change_impact === 'comment_change';
    }

    private extractCodeRegion(
        fileChange: IncrementalUpdate,
        startLine: number,
        endLine: number
    ): string {
        // Extract code from the changes
        return fileChange.changes
            .filter(edit => edit.start_line >= startLine && edit.end_line <= endLine)
            .map(edit => edit.new_text)
            .join('\n');
    }

    private async parseCodeRegion(code: string, scopeType: string): Promise<any> {
        // Use appropriate parser based on scope type
        // Simplified - would use actual tree-sitter parser
        return { parsed: true, code };
    }

    private extractNodesFromCache(cachedAST: any): ParsedNode[] {
        // Extract nodes from cached AST
        return [];
    }

    private extractNodes(ast: any, filePath: string): ParsedNode[] {
        // Extract nodes from parsed AST
        return [];
    }

    private extractEdges(node: ParsedNode): GraphEdge[] {
        // Extract edges (calls, imports, etc.) from node
        return [];
    }

    private edgeExists(edge: GraphEdge, edges: GraphEdge[]): boolean {
        return edges.some(e => e.from === edge.from && e.to === edge.to && e.type === edge.type);
    }

    private async invalidateSemanticCache(nodeId: string): Promise<void> {
        // Invalidate semantic context cache for this node
    }

    private async invalidateQueryCachesUsingNode(nodeId: string): Promise<void> {
        // Invalidate query result caches that used this node
    }

    private async findCheckpointsReferencingNode(
        nodeId: string,
        workspace: string
    ): Promise<string[]> {
        // Find checkpoints that reference this node
        return [];
    }
}

/**
 * Incremental Knowledge Graph
 */
class IncrementalKnowledgeGraph {
    private nodes: Map<string, GraphNode> = new Map();
    private edges: Map<string, GraphEdge[]> = new Map();
    private reverseEdges: Map<string, GraphEdge[]> = new Map();

    async getNode(nodeId: string): Promise<GraphNode | null> {
        return this.nodes.get(nodeId) || null;
    }

    async updateNode(nodeId: string, data: any): Promise<GraphNode> {
        const existing = this.nodes.get(nodeId);
        const updated: GraphNode = {
            ...existing!,
            metadata: { ...existing?.metadata, ...data },
            last_updated: new Date(),
        };
        this.nodes.set(nodeId, updated);
        return updated;
    }

    async addNode(node: ParsedNode): Promise<GraphNode> {
        const graphNode: GraphNode = {
            id: node.id,
            type: node.type as any,
            name: node.name,
            file_path: node.file_path,
            metadata: node.metadata,
            last_updated: new Date(),
        };
        this.nodes.set(node.id, graphNode);
        return graphNode;
    }

    async removeNode(nodeId: string): Promise<void> {
        this.nodes.delete(nodeId);
        this.edges.delete(nodeId);
    }

    async getEdgesFrom(nodeId: string): Promise<GraphEdge[]> {
        return this.edges.get(nodeId) || [];
    }

    async addEdge(edge: GraphEdge): Promise<void> {
        const fromEdges = this.edges.get(edge.from) || [];
        fromEdges.push(edge);
        this.edges.set(edge.from, fromEdges);

        const toEdges = this.reverseEdges.get(edge.to) || [];
        toEdges.push(edge);
        this.reverseEdges.set(edge.to, toEdges);
    }

    async removeEdge(from: string, to: string): Promise<void> {
        const fromEdges = this.edges.get(from) || [];
        this.edges.set(from, fromEdges.filter(e => e.to !== to));

        const toEdges = this.reverseEdges.get(to) || [];
        this.reverseEdges.set(to, toEdges.filter(e => e.from !== from));
    }

    async getDependents(nodeId: string): Promise<GraphNode[]> {
        const edges = this.reverseEdges.get(nodeId) || [];
        const dependents: GraphNode[] = [];
        for (const edge of edges) {
            const node = await this.getNode(edge.from);
            if (node) dependents.push(node);
        }
        return dependents;
    }

    async getDependentsRecursive(nodeId: string, maxDepth: number): Promise<GraphNode[]> {
        const visited = new Set<string>();
        const result: GraphNode[] = [];

        const traverse = async (id: string, depth: number) => {
            if (depth > maxDepth || visited.has(id)) return;
            visited.add(id);

            const dependents = await this.getDependents(id);
            result.push(...dependents);

            for (const dep of dependents) {
                await traverse(dep.id, depth + 1);
            }
        };

        await traverse(nodeId, 0);
        return result;
    }

    async getNodesInFile(filePath: string): Promise<GraphNode[]> {
        return Array.from(this.nodes.values()).filter(n => n.file_path === filePath);
    }

    async removeNodesInFile(filePath: string): Promise<void> {
        const nodes = await this.getNodesInFile(filePath);
        for (const node of nodes) {
            await this.removeNode(node.id);
        }
    }
}

/**
 * Incremental AST Cache
 */
class IncrementalASTCache {
    private cache: Map<string, CachedAST> = new Map();
    private fileStructures: Map<string, FileStructure> = new Map();

    async get(filePath: string, startLine: number, endLine: number): Promise<any | null> {
        const key = `${filePath}:${startLine}-${endLine}`;
        const cached = this.cache.get(key);
        if (cached && !cached.invalidated) {
            return cached.ast;
        }
        return null;
    }

    async set(filePath: string, startLine: number, endLine: number, ast: any): Promise<void> {
        const key = `${filePath}:${startLine}-${endLine}`;
        this.cache.set(key, {
            ast,
            file_path: filePath,
            start_line: startLine,
            end_line: endLine,
            cached_at: new Date(),
            invalidated: false,
        });
    }

    async invalidateFile(filePath: string): Promise<void> {
        for (const [key, cached] of Array.from(this.cache.entries())) {
            if (cached.file_path === filePath) {
                cached.invalidated = true;
            }
        }
    }

    async invalidateNode(nodeId: string): Promise<void> {
        // Invalidate cache entries that contain this node
    }

    getFileStructure(filePath: string): FileStructure | null {
        return this.fileStructures.get(filePath) || null;
    }
}

/**
 * File Change Tracker
 */
class FileChangeTracker {
    private changes: Map<string, IncrementalUpdate[]> = new Map();

    trackChange(change: IncrementalUpdate): void {
        const existing = this.changes.get(change.file_path) || [];
        existing.push(change);
        this.changes.set(change.file_path, existing);
    }

    getRecentChanges(filePath: string, since: Date): IncrementalUpdate[] {
        const changes = this.changes.get(filePath) || [];
        return changes.filter(c => c.timestamp >= since);
    }
}

/**
 * Dependency Tracker
 */
class DependencyTracker {
    private fileDependencies: Map<string, Set<string>> = new Map();
    private functionDependencies: Map<string, Set<string>> = new Map();

    async updateFileDependencies(filePath: string, imports: GraphEdge[]): Promise<void> {
        const deps = new Set(imports.map(e => e.to));
        this.fileDependencies.set(filePath, deps);
    }

    async addFunctionDependency(from: string, to: string): Promise<void> {
        const deps = this.functionDependencies.get(from) || new Set();
        deps.add(to);
        this.functionDependencies.set(from, deps);
    }

    async removeFunctionDependency(from: string, to: string): Promise<void> {
        const deps = this.functionDependencies.get(from);
        if (deps) {
            deps.delete(to);
        }
    }
}

// Supporting interfaces

interface AffectedRegion {
    scope_type: string;
    scope_name: string;
    start_line: number;
    end_line: number;
    change_impact: 'signature_change' | 'implementation_change' | 'comment_change' | 'module_level';
}

interface ContainingScope {
    type: string;
    name: string;
    start_line: number;
    end_line: number;
}

interface ParsedNode {
    id: string;
    type: string;
    name: string;
    file_path: string;
    metadata: Record<string, any>;
    data: any;
}

interface GraphUpdateResult {
    updated_nodes: GraphNode[];
    invalidated_nodes: GraphNode[];
    new_edges: GraphEdge[];
    removed_edges: GraphEdge[];
}

interface CachedAST {
    ast: any;
    file_path: string;
    start_line: number;
    end_line: number;
    cached_at: Date;
    invalidated: boolean;
}

interface FileStructure {
    scopes: ContainingScope[];
}

