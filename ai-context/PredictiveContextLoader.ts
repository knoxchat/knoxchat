/**
 * Predictive Context Loader
 * 
 * Predicts what context will be needed next and preloads it in background.
 * Provides instant responses by having context ready before user asks.
 */

import * as Types from './types';
import { AIContextBuilder } from './AIContextBuilder';

export interface QueryPrediction {
    predicted_query: string;
    probability: number;
    reasoning: string;
    trigger_event: string;
    priority: number;
}

export interface UserPattern {
    pattern_type: 'sequential' | 'file_based' | 'time_based' | 'task_based';
    pattern_description: string;
    confidence: number;
    occurrences: number;
    last_seen: Date;
}

export interface PreloadedContext {
    query: string;
    context: Types.CompleteAIContext;
    predicted_at: Date;
    expires_at: Date;
    hit_count: number;
    prediction_accuracy: number;
}

export interface PreloadStatistics {
    total_predictions: number;
    cache_hits: number;
    cache_misses: number;
    hit_rate: number;
    average_speedup_ms: number;
    patterns_learned: number;
}

/**
 * Main Predictive Context Loader
 */
export class PredictiveContextLoader {
    private contextBuilder: AIContextBuilder;
    private userBehaviorAnalyzer: UserBehaviorAnalyzer;
    private predictionModel: PredictionModel;
    private preloadCache: Map<string, PreloadedContext>;
    private isPreloading: boolean = false;
    private statistics: PreloadStatistics;

    constructor(contextBuilder: AIContextBuilder) {
        this.contextBuilder = contextBuilder;
        this.userBehaviorAnalyzer = new UserBehaviorAnalyzer();
        this.predictionModel = new PredictionModel();
        this.preloadCache = new Map();
        this.statistics = {
            total_predictions: 0,
            cache_hits: 0,
            cache_misses: 0,
            hit_rate: 0,
            average_speedup_ms: 0,
            patterns_learned: 0,
        };
    }

    /**
     * Start predictive preloading based on user activity
     */
    async startPredictivePreloading(workspace: string): Promise<void> {
        if (this.isPreloading) {
            console.log('📡 Predictive preloading already running');
            return;
        }

        this.isPreloading = true;
        console.log('🚀 Starting predictive context preloading...');

        // Analyze user behavior patterns
        const patterns = await this.userBehaviorAnalyzer.analyzeUserBehavior(workspace);
        console.log(`📊 Learned ${patterns.length} user behavior patterns`);

        this.statistics.patterns_learned = patterns.length;

        // Start background preloading loop
        this.backgroundPreloadLoop(workspace, patterns);
    }

    /**
     * Stop predictive preloading
     */
    stopPredictivePreloading(): void {
        this.isPreloading = false;
        console.log('⏸️ Stopped predictive context preloading');
    }

    /**
     * Try to get preloaded context for a query
     */
    async getPreloadedContext(query: string): Promise<Types.CompleteAIContext | null> {
        const cacheKey = this.normalizeCacheKey(query);
        const cached = this.preloadCache.get(cacheKey);

        if (cached && !this.isExpired(cached)) {
            // Cache hit!
            cached.hit_count++;
            this.statistics.cache_hits++;
            
            console.log(`⚡ Preloaded context hit! (${cached.hit_count} hits)`);
            console.log(`   Query: "${query}"`);
            console.log(`   Accuracy: ${(cached.prediction_accuracy * 100).toFixed(1)}%`);
            
            return cached.context;
        }

        // Cache miss
        this.statistics.cache_misses++;
        return null;
    }

    /**
     * Record prediction accuracy when user makes a query
     */
    recordQueryAccuracy(actualQuery: string, wasPreloaded: boolean): void {
        if (wasPreloaded) {
            const cacheKey = this.normalizeCacheKey(actualQuery);
            const cached = this.preloadCache.get(cacheKey);
            if (cached) {
                cached.prediction_accuracy = (cached.prediction_accuracy + 1.0) / 2;
            }
        }

        // Update learning model
        this.predictionModel.recordQuery(actualQuery, wasPreloaded);
    }

    /**
     * Get preload statistics
     */
    getStatistics(): PreloadStatistics {
        this.statistics.hit_rate = 
            this.statistics.total_predictions > 0
                ? this.statistics.cache_hits / this.statistics.total_predictions
                : 0;

        return { ...this.statistics };
    }

    /**
     * Background loop for predictive preloading
     */
    private async backgroundPreloadLoop(
        workspace: string,
        patterns: UserPattern[]
    ): Promise<void> {
        while (this.isPreloading) {
            try {
                // Predict next queries
                const predictions = await this.predictNextQueries(workspace, patterns);

                // Preload high-priority predictions
                await this.preloadPredictions(predictions, workspace);

                // Clean up expired cache
                this.cleanupExpiredCache();

                // Wait before next iteration (avoid overwhelming CPU)
                await this.sleep(5000); // 5 seconds

            } catch (error) {
                console.error('Error in predictive preload loop:', error);
                await this.sleep(10000); // Wait longer on error
            }
        }
    }

    /**
     * Predict next queries based on patterns and current context
     */
    private async predictNextQueries(
        workspace: string,
        patterns: UserPattern[]
    ): Promise<QueryPrediction[]> {
        const predictions: QueryPrediction[] = [];

        // 1. Predict based on file activity
        const fileBasedPredictions = await this.predictFromFileActivity(workspace);
        predictions.push(...fileBasedPredictions);

        // 2. Predict based on sequential patterns
        const sequentialPredictions = await this.predictFromSequentialPatterns(patterns);
        predictions.push(...sequentialPredictions);

        // 3. Predict based on time patterns
        const timeBasedPredictions = await this.predictFromTimePatterns(patterns);
        predictions.push(...timeBasedPredictions);

        // 4. Predict based on task context
        const taskBasedPredictions = await this.predictFromTaskContext(workspace);
        predictions.push(...taskBasedPredictions);

        // Sort by priority and filter top predictions
        return predictions
            .sort((a, b) => b.priority - a.priority)
            .slice(0, 5); // Top 5 predictions
    }

    /**
     * Predict queries from current file activity
     */
    private async predictFromFileActivity(workspace: string): Promise<QueryPrediction[]> {
        const predictions: QueryPrediction[] = [];
        
        const currentFile = await this.getCurrentFile(workspace);
        if (!currentFile) return predictions;

        // If user opens auth file → likely to ask about authentication
        if (currentFile.includes('auth') || currentFile.includes('login')) {
            predictions.push({
                predicted_query: 'How does authentication work?',
                probability: 0.85,
                reasoning: 'User opened authentication-related file',
                trigger_event: `file_opened:${currentFile}`,
                priority: 0.9,
            });
            predictions.push({
                predicted_query: 'Show login flow',
                probability: 0.75,
                reasoning: 'Common follow-up for auth files',
                trigger_event: `file_opened:${currentFile}`,
                priority: 0.8,
            });
        }

        // If user opens test file → likely to ask about tested function
        if (currentFile.includes('.test.') || currentFile.includes('.spec.')) {
            const testedFunction = this.extractTestedFunction(currentFile);
            if (testedFunction) {
                predictions.push({
                    predicted_query: `How does ${testedFunction} work?`,
                    probability: 0.80,
                    reasoning: 'User opened test file for specific function',
                    trigger_event: `test_file_opened:${currentFile}`,
                    priority: 0.85,
                });
            }
        }

        // If user opens component file → likely to ask about usage
        if (currentFile.includes('Component') || currentFile.endsWith('.tsx')) {
            const componentName = this.extractComponentName(currentFile);
            predictions.push({
                predicted_query: `How is ${componentName} used?`,
                probability: 0.70,
                reasoning: 'User opened React component',
                trigger_event: `component_opened:${currentFile}`,
                priority: 0.75,
            });
        }

        return predictions;
    }

    /**
     * Predict queries from sequential patterns
     */
    private async predictFromSequentialPatterns(
        patterns: UserPattern[]
    ): Promise<QueryPrediction[]> {
        const predictions: QueryPrediction[] = [];

        const sequentialPatterns = patterns.filter(p => p.pattern_type === 'sequential');

        for (const pattern of sequentialPatterns) {
            // Example: User often asks about Model after asking about Controller
            // Pattern: "controller" → "model" (confidence: 0.85)
            
            const lastQuery = await this.getLastQuery();
            if (lastQuery && this.matchesPattern(lastQuery, pattern)) {
                const nextQuery = this.predictNextFromPattern(pattern);
                predictions.push({
                    predicted_query: nextQuery,
                    probability: pattern.confidence,
                    reasoning: `Sequential pattern: ${pattern.pattern_description}`,
                    trigger_event: `sequential_pattern:${lastQuery}`,
                    priority: pattern.confidence * 0.9,
                });
            }
        }

        return predictions;
    }

    /**
     * Predict queries from time-based patterns
     */
    private async predictFromTimePatterns(
        patterns: UserPattern[]
    ): Promise<QueryPrediction[]> {
        const predictions: QueryPrediction[] = [];

        const timePatterns = patterns.filter(p => p.pattern_type === 'time_based');
        const currentHour = new Date().getHours();

        for (const pattern of timePatterns) {
            // Example: User often reviews authentication at start of day (9am)
            if (this.isTimeMatch(currentHour, pattern)) {
                const predictedQuery = this.extractQueryFromPattern(pattern);
                predictions.push({
                    predicted_query: predictedQuery,
                    probability: pattern.confidence * 0.7, // Lower confidence for time-based
                    reasoning: `Time-based pattern: ${pattern.pattern_description}`,
                    trigger_event: `time_pattern:${currentHour}:00`,
                    priority: pattern.confidence * 0.6,
                });
            }
        }

        return predictions;
    }

    /**
     * Predict queries from task context
     */
    private async predictFromTaskContext(workspace: string): Promise<QueryPrediction[]> {
        const predictions: QueryPrediction[] = [];

        // Check for error messages (user might debug)
        const hasErrors = await this.checkForErrors(workspace);
        if (hasErrors) {
            predictions.push({
                predicted_query: 'Help debug this error',
                probability: 0.90,
                reasoning: 'Error detected in workspace',
                trigger_event: 'error_detected',
                priority: 0.95, // High priority - user likely needs help
            });
        }

        // Check for TODO comments (user might work on them)
        const recentTodos = await this.getRecentTodos(workspace);
        if (recentTodos.length > 0) {
            const todo = recentTodos[0];
            predictions.push({
                predicted_query: `How to implement: ${todo.description}`,
                probability: 0.65,
                reasoning: 'Recent TODO comment found',
                trigger_event: `todo_found:${todo.line}`,
                priority: 0.70,
            });
        }

        return predictions;
    }

    /**
     * Preload contexts for predictions
     */
    private async preloadPredictions(
        predictions: QueryPrediction[],
        workspace: string
    ): Promise<void> {
        for (const prediction of predictions) {
            // Only preload if not already cached and probability is high enough
            if (prediction.probability < 0.6) continue;

            const cacheKey = this.normalizeCacheKey(prediction.predicted_query);
            if (this.preloadCache.has(cacheKey)) continue;

            // Preload in background (don't block)
            this.preloadInBackground(prediction, workspace);
        }
    }

    /**
     * Preload context in background
     */
    private async preloadInBackground(
        prediction: QueryPrediction,
        workspace: string
    ): Promise<void> {
        try {
            console.log(`🔮 Preloading context: "${prediction.predicted_query}" (${(prediction.probability * 100).toFixed(0)}% confidence)`);

            const context = await this.contextBuilder.buildContextForQuery(
                prediction.predicted_query,
                workspace,
                8000
            );

            const cacheKey = this.normalizeCacheKey(prediction.predicted_query);
            const expiresAt = new Date(Date.now() + 10 * 60 * 1000); // 10 minutes

            this.preloadCache.set(cacheKey, {
                query: prediction.predicted_query,
                context,
                predicted_at: new Date(),
                expires_at: expiresAt,
                hit_count: 0,
                prediction_accuracy: prediction.probability,
            });

            this.statistics.total_predictions++;

            console.log(`✅ Preloaded context ready for: "${prediction.predicted_query}"`);

        } catch (error) {
            console.error(`Failed to preload context for "${prediction.predicted_query}":`, error);
        }
    }

    /**
     * Clean up expired cache entries
     */
    private cleanupExpiredCache(): void {
        const now = Date.now();
        let cleanedCount = 0;

        for (const [key, cached] of Array.from(this.preloadCache.entries())) {
            if (this.isExpired(cached)) {
                this.preloadCache.delete(key);
                cleanedCount++;
            }
        }

        if (cleanedCount > 0) {
            console.log(`🗑️ Cleaned up ${cleanedCount} expired preloaded contexts`);
        }
    }

    // Helper methods

    private normalizeCacheKey(query: string): string {
        return query.toLowerCase().trim().replace(/\s+/g, ' ');
    }

    private isExpired(cached: PreloadedContext): boolean {
        return Date.now() > cached.expires_at.getTime();
    }

    private async sleep(ms: number): Promise<void> {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    private async getCurrentFile(workspace: string): Promise<string | null> {
        // Would integrate with VS Code API to get active file
        return null;
    }

    private extractTestedFunction(filePath: string): string | null {
        // Extract function name from test file path
        // Example: "login.test.ts" → "login"
        const match = filePath.match(/([^/]+)\.(?:test|spec)\./);
        return match ? match[1] : null;
    }

    private extractComponentName(filePath: string): string {
        // Extract component name from file path
        const match = filePath.match(/([^/]+)\.tsx?$/);
        return match ? match[1] : 'Component';
    }

    private async getLastQuery(): Promise<string | null> {
        // Would track recent queries
        return null;
    }

    private matchesPattern(query: string, pattern: UserPattern): boolean {
        return pattern.pattern_description.toLowerCase().includes(query.toLowerCase());
    }

    private predictNextFromPattern(pattern: UserPattern): string {
        // Extract predicted query from pattern description
        // Example: "user asks about model after controller" → "How does the model work?"
        return "Related query based on pattern";
    }

    private isTimeMatch(currentHour: number, pattern: UserPattern): boolean {
        // Check if current time matches pattern
        return true; // Simplified
    }

    private extractQueryFromPattern(pattern: UserPattern): string {
        return pattern.pattern_description;
    }

    private async checkForErrors(workspace: string): Promise<boolean> {
        // Would check for linter/compiler errors
        return false;
    }

    private async getRecentTodos(workspace: string): Promise<Array<{ description: string; line: number }>> {
        // Would scan for TODO comments
        return [];
    }
}

/**
 * User Behavior Analyzer
 */
class UserBehaviorAnalyzer {
    private queryHistory: string[] = [];
    private fileHistory: string[] = [];
    private timePatterns: Map<number, string[]> = new Map();

    async analyzeUserBehavior(workspace: string): Promise<UserPattern[]> {
        const patterns: UserPattern[] = [];

        // Analyze sequential patterns
        patterns.push(...this.analyzeSequentialPatterns());

        // Analyze file-based patterns
        patterns.push(...this.analyzeFilePatterns());

        // Analyze time-based patterns
        patterns.push(...this.analyzeTimePatterns());

        return patterns;
    }

    private analyzeSequentialPatterns(): UserPattern[] {
        const patterns: UserPattern[] = [];

        // Find common sequences in query history
        // Example: "controller" often followed by "model"
        for (let i = 0; i < this.queryHistory.length - 1; i++) {
            const current = this.queryHistory[i];
            const next = this.queryHistory[i + 1];

            patterns.push({
                pattern_type: 'sequential',
                pattern_description: `After "${current}", user often asks "${next}"`,
                confidence: 0.75,
                occurrences: 1,
                last_seen: new Date(),
            });
        }

        return patterns;
    }

    private analyzeFilePatterns(): UserPattern[] {
        // Analyze which files lead to which queries
        return [];
    }

    private analyzeTimePatterns(): UserPattern[] {
        // Analyze time-of-day patterns
        return [];
    }

    recordQuery(query: string): void {
        this.queryHistory.push(query);
        // Keep only recent history
        if (this.queryHistory.length > 50) {
            this.queryHistory.shift();
        }
    }

    recordFileOpen(filePath: string): void {
        this.fileHistory.push(filePath);
        if (this.fileHistory.length > 50) {
            this.fileHistory.shift();
        }
    }
}

/**
 * Prediction Model (ML-based, simplified)
 */
class PredictionModel {
    private accuracy: Map<string, number> = new Map();

    recordQuery(query: string, wasPreloaded: boolean): void {
        const key = this.normalizeCacheKey(query);
        const current = this.accuracy.get(key) || 0.5;
        
        // Update accuracy based on whether prediction was correct
        const newAccuracy = wasPreloaded
            ? Math.min(current + 0.1, 1.0)
            : Math.max(current - 0.05, 0.0);
        
        this.accuracy.set(key, newAccuracy);
    }

    getPredictionConfidence(query: string): number {
        const key = this.normalizeCacheKey(query);
        return this.accuracy.get(key) || 0.5;
    }

    private normalizeCacheKey(query: string): string {
        return query.toLowerCase().trim().replace(/\s+/g, ' ');
    }
}

