/**
 * Integration Manager - Coordinates between IDE Integration and Git Checkpoint Bridge
 * 
 * This manager provides a unified interface for IDE and Git integration,
 * ensuring proper coordination between components.
 */

import { IDEIntegration } from './IDEIntegration';
import { GitCheckpointBridge } from './GitCheckpointBridge';
import { AIContextBuilder } from '../AIContextBuilder';
import { CodeUnderstandingEngine } from '../CodeUnderstandingEngine';
import * as Types from '../types';

export interface IntegrationConfig {
    repoPath: string;
    workspacePath: string;
    enableGitIntegration?: boolean;
    enableIDEIntegration?: boolean;
    cacheTimeout?: number;
}

export class IntegrationManager {
    private ideIntegration!: IDEIntegration;
    private gitBridge!: GitCheckpointBridge;
    private contextBuilder!: AIContextBuilder;
    private codeUnderstanding!: CodeUnderstandingEngine;
    private config: IntegrationConfig;

    constructor(config: IntegrationConfig) {
        this.config = {
            enableGitIntegration: true,
            enableIDEIntegration: true,
            cacheTimeout: 300000, // 5 minutes
            ...config
        };

        // Initialize components
        this.initializeComponents();
    }

    /**
     * Initialize all integration components
     */
    private initializeComponents(): void {
        // Initialize Git bridge
        this.gitBridge = new GitCheckpointBridge(this.config.repoPath);
        
        // Initialize context builder with Git bridge as checkpoint manager
        this.contextBuilder = new AIContextBuilder(this.gitBridge);
        
        // Initialize code understanding engine
        this.codeUnderstanding = new CodeUnderstandingEngine();
        
        // Initialize IDE integration with mock AI context provider
        const mockAIContextProvider = {
            async provideContextualAssistance(position: any, partialCode: string) {
                return { suggestions: [] };
            }
        };
        
        this.ideIntegration = new IDEIntegration(
            this.contextBuilder,
            this.codeUnderstanding,
            mockAIContextProvider,
            this.gitBridge
        );
    }

    /**
     * Get comprehensive context for IDE assistance
     */
    async getContextualAssistance(
        cursorPosition: { line: number; character: number; file?: string },
        openFiles: Array<{
            path: string;
            content: string;
            language: string;
            workspacePath: string;
        }>,
        ideType: string = 'vscode'
    ): Promise<{
        codeCompletions: any[];
        refactoringSuggestions: any[];
        documentationSuggestions: any[];
        errorFixes: any[];
        gitContext: {
            recentChanges: string[];
            relatedCommits: string[];
            contextualHistory: string[];
        };
    }> {
        try {
            // Get IDE suggestions
            const suggestions = await this.ideIntegration.provideContextualAssistance(
                cursorPosition as any,
                openFiles as any,
                ideType as any
            );

            // Get Git context for debugging
            const gitContext = await this.ideIntegration.getGitContextForDebugging(
                this.config.workspacePath,
                cursorPosition.file || '',
                cursorPosition.line
            );

            return {
                codeCompletions: suggestions.codeCompletions,
                refactoringSuggestions: suggestions.refactoringSuggestions,
                documentationSuggestions: suggestions.documentationSuggestions,
                errorFixes: suggestions.errorFixes,
                gitContext
            };
        } catch (error) {
            console.error('Failed to get contextual assistance:', error);
            throw new Error(`Contextual assistance failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Sync workspace with Git and refresh all contexts
     */
    async syncWorkspace(): Promise<void> {
        try {
            if (this.config.enableGitIntegration) {
                await this.ideIntegration.syncWithGit(this.config.workspacePath);
            }
            console.log('Workspace sync completed successfully');
        } catch (error) {
            console.error('Failed to sync workspace:', error);
            throw error;
        }
    }

    /**
     * Get AI context for a specific query
     */
    async buildContextForQuery(
        query: string,
        maxTokens: number = 8000
    ): Promise<Types.CompleteAIContext> {
        return await this.contextBuilder.buildContextForQuery(
            query,
            this.config.workspacePath,
            maxTokens
        );
    }

    /**
     * Get recent Git checkpoints
     */
    async getRecentCheckpoints(limit: number = 10): Promise<Types.AIContextCheckpoint[]> {
        const checkpoints = await this.gitBridge.getCheckpointsForWorkspace(this.config.workspacePath);
        return checkpoints.slice(0, limit);
    }

    /**
     * Analyze code semantically
     */
    async analyzeCodebase(files: Map<string, any>): Promise<any> {
        return await this.codeUnderstanding.analyzeCodebase(files);
    }

    /**
     * Get integration status
     */
    getIntegrationStatus(): {
        gitIntegrationEnabled: boolean;
        ideIntegrationEnabled: boolean;
        activeConnections: number;
        repoPath: string;
        workspacePath: string;
    } {
        return {
            gitIntegrationEnabled: this.config.enableGitIntegration || false,
            ideIntegrationEnabled: this.config.enableIDEIntegration || false,
            activeConnections: this.ideIntegration.getActiveConnections().length,
            repoPath: this.config.repoPath,
            workspacePath: this.config.workspacePath
        };
    }

    /**
     * Connect to IDE
     */
    async connectToIDE(ideConfig: {
        id: string;
        type: string;
        host: string;
        port: number;
        apiKey?: string;
    }): Promise<any> {
        return await this.ideIntegration.connectToIDE(ideConfig as any);
    }

    /**
     * Disconnect from IDE
     */
    async disconnectFromIDE(ideId: string): Promise<void> {
        await this.ideIntegration.disconnectFromIDE(ideId);
    }

    /**
     * Clean up resources
     */
    async cleanup(): Promise<void> {
        // Disconnect all IDE connections
        const connections = this.ideIntegration.getActiveConnections();
        for (const connection of connections) {
            await this.ideIntegration.disconnectFromIDE(connection.id);
        }
        
        console.log('Integration manager cleanup completed');
    }
}

export default IntegrationManager;
