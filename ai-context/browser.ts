/**
 * Browser-compatible AI Context System Entry Point
 * 
 * This provides a browser-compatible version of the AI context system
 * that doesn't rely on Node.js APIs.
 */

// Core AI Context components
export { AIContextBuilder } from './AIContextBuilder';
export { CodeUnderstandingEngine } from './CodeUnderstandingEngine';
export { ContextOptimizer } from './ContextOptimizer';
export { QueryIntentAnalyzer } from './QueryIntentAnalyzer';
export { RelevanceEngine } from './RelevanceEngine';
export { ContextTreeBuilder } from './ContextTreeBuilder';

// Browser-compatible integrations
export { BrowserGitCheckpointBridge as GitCheckpointBridge } from './integrations/BrowserGitCheckpointBridge';

// Cache and optimization
export { ContextCache } from './cache/ContextCache';

// Types
export * from './types';

// Browser-specific utilities
export class BrowserAIContextManager {
    private aiContextBuilder: any;
    private checkpointBridge: any;

    constructor() {
        // Import components dynamically to avoid Node.js dependencies
        this.initializeComponents();
    }

    private async initializeComponents() {
        try {
            const { AIContextBuilder } = await import('./AIContextBuilder');
            const { BrowserGitCheckpointBridge } = await import('./integrations/BrowserGitCheckpointBridge');
            const { CodeUnderstandingEngine } = await import('./CodeUnderstandingEngine');

            const codeEngine = new CodeUnderstandingEngine();
            this.checkpointBridge = new BrowserGitCheckpointBridge(codeEngine);
            this.aiContextBuilder = new AIContextBuilder(this.checkpointBridge);
        } catch (error) {
            console.error('Failed to initialize AI Context components:', error);
        }
    }

    /**
     * Build AI context for a query in browser environment
     */
    async buildContextForQuery(
        query: string,
        workspace: string = '/',
        maxTokens: number = 8000
    ) {
        if (!this.aiContextBuilder) {
            await this.initializeComponents();
        }

        try {
            return await this.aiContextBuilder.buildContextForQuery(query, workspace, maxTokens);
        } catch (error) {
            console.error('Failed to build AI context:', error);
            throw error;
        }
    }

    /**
     * Check if AI context system is ready
     */
    isReady(): boolean {
        return !!(this.aiContextBuilder && this.checkpointBridge);
    }

    /**
     * Get system status for debugging
     */
    getStatus() {
        return {
            ready: this.isReady(),
            components: {
                aiContextBuilder: !!this.aiContextBuilder,
                checkpointBridge: !!this.checkpointBridge,
            },
            environment: 'browser'
        };
    }
}

// Create default instance
export const browserAIContextManager = new BrowserAIContextManager();

export default browserAIContextManager;
