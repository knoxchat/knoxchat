/**
 * Predictive Context Loader - Advanced predictive context loading system
 * 
 * This system analyzes user patterns, predicts likely follow-up queries,
 * and preloads context to provide instant responses.
 */

import { QueryIntent, CompleteAIContext } from '../AIContextBuilder';
import { contextCache } from '../cache/ContextCache';

export class PredictiveContextLoader {
    private userPatternAnalyzer: UserPatternAnalyzer;
    private queryPredictor: QueryPredictor;
    private contextPreloader: ContextPreloader;
    private predictionCache: Map<string, PredictedQuery[]> = new Map();
    private config: PredictiveConfig;

    constructor(config: PredictiveConfig = {}) {
        this.config = {
            maxPredictions: config.maxPredictions || 5,
            minConfidence: config.minConfidence || 0.7,
            preloadDelay: config.preloadDelay || 100,
            patternWindowHours: config.patternWindowHours || 24,
            enableMachineLearning: config.enableMachineLearning !== false
        };

        this.userPatternAnalyzer = new UserPatternAnalyzer(this.config);
        this.queryPredictor = new QueryPredictor(this.config);
        this.contextPreloader = new ContextPreloader();
    }

    /**
     * Predict and preload context for likely follow-up queries
     */
    async predictAndPreloadContext(
        currentQuery: string,
        userHistory: QueryHistory[],
        workspace: string,
        userId?: string
    ): Promise<PredictionResult> {
        const startTime = Date.now();

        try {
            // Analyze user patterns
            const patterns = await this.userPatternAnalyzer.analyzePatterns(userHistory, userId);

            // Predict likely follow-up queries
            const predictions = await this.queryPredictor.predictFollowUpQueries(
                currentQuery,
                patterns,
                workspace
            );

            // Filter predictions by confidence
            const highConfidencePredictions = predictions.filter(
                p => (p.confidence ?? 0) >= (this.config.minConfidence ?? 0.7)
            );

            // Preload context for high-confidence predictions
            const preloadResults = await this.preloadPredictedContexts(
                highConfidencePredictions,
                workspace,
                userId
            );

            const endTime = Date.now();

            return {
                predictions: highConfidencePredictions,
                preloadedCount: preloadResults.length,
                executionTime: endTime - startTime,
                patterns: patterns,
                cacheHits: preloadResults.filter(r => r.fromCache).length
            };

        } catch (error) {
            console.error('Predictive context loading failed:', error);
            return {
                predictions: [],
                preloadedCount: 0,
                executionTime: Date.now() - startTime,
                patterns: { 
                    commonSequences: [], 
                    queryTypes: new Map(), 
                    timePatterns: [],
                    topicPatterns: [],
                    totalQueries: 0,
                    analyzedAt: new Date()
                },
                cacheHits: 0,
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }

    /**
     * Get preloaded context for a query if available
     */
    async getPreloadedContext(query: string, workspace: string): Promise<CompleteAIContext | null> {
        const cacheKey = this.generatePredictionCacheKey(query, workspace);
        
        // Check if we have a preloaded context
        const cached = await contextCache.getOrComputeQueryContext(
            query,
            workspace,
            8000,
            async () => null as any
        );

        return cached || null;
    }

    /**
     * Update prediction models with user feedback
     */
    async updatePredictionModels(
        originalQuery: string,
        actualFollowUp: string,
        wasHelpful: boolean,
        userId?: string
    ): Promise<void> {
        await this.queryPredictor.updateModel(originalQuery, actualFollowUp, wasHelpful, userId);
        await this.userPatternAnalyzer.updatePatterns(originalQuery, actualFollowUp, userId);
    }

    /**
     * Preload contexts for predicted queries
     */
    private async preloadPredictedContexts(
        predictions: PredictedQuery[],
        workspace: string,
        userId?: string
    ): Promise<PreloadResult[]> {
        const results: PreloadResult[] = [];

        // Preload in parallel with delay to avoid overwhelming the system
        const preloadPromises = predictions.map(async (prediction, index) => {
            // Stagger preloading to avoid resource contention
            await this.delay(index * (this.config.preloadDelay ?? 100));

            try {
                const result = await this.contextPreloader.preloadContext(
                    prediction.query,
                    workspace,
                    prediction.confidence
                );

                results.push({
                    query: prediction.query,
                    success: true,
                    fromCache: result.fromCache,
                    loadTime: result.loadTime,
                    confidence: prediction.confidence
                });

            } catch (error) {
                results.push({
                    query: prediction.query,
                    success: false,
                    fromCache: false,
                    loadTime: 0,
                    confidence: prediction.confidence,
                    error: error instanceof Error ? error.message : String(error)
                });
            }
        });

        await Promise.all(preloadPromises);
        return results;
    }

    /**
     * Generate cache key for predictions
     */
    private generatePredictionCacheKey(query: string, workspace: string): string {
        const normalizedQuery = query.toLowerCase().trim();
        const workspaceHash = workspace.split('/').pop() || 'unknown';
        return `prediction:${workspaceHash}:${this.hashString(normalizedQuery)}`;
    }

    /**
     * Simple hash function for cache keys
     */
    private hashString(str: string): string {
        let hash = 0;
        for (let i = 0; i < str.length; i++) {
            const char = str.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
            hash = hash & hash;
        }
        return Math.abs(hash).toString(36);
    }

    /**
     * Delay utility for staggered preloading
     */
    private delay(ms: number): Promise<void> {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    /**
     * Get prediction statistics
     */
    getStatistics(): PredictiveStats {
        return {
            totalPredictions: this.predictionCache.size,
            averageConfidence: this.calculateAverageConfidence(),
            hitRate: this.calculateHitRate(),
            memoryUsage: this.estimateMemoryUsage()
        };
    }

    private calculateAverageConfidence(): number {
        let totalConfidence = 0;
        let count = 0;

        for (const predictions of this.predictionCache.values()) {
            for (const prediction of predictions) {
                totalConfidence += prediction.confidence;
                count++;
            }
        }

        return count > 0 ? totalConfidence / count : 0;
    }

    private calculateHitRate(): number {
        // This would track actual hit rates in a real implementation
        return 0.75; // Placeholder
    }

    private estimateMemoryUsage(): number {
        let totalSize = 0;
        for (const [key, predictions] of this.predictionCache) {
            totalSize += key.length * 2; // String size
            totalSize += predictions.length * 100; // Estimate per prediction
        }
        return totalSize;
    }
}

/**
 * User Pattern Analyzer - Analyzes user behavior patterns
 */
class UserPatternAnalyzer {
    private config: PredictiveConfig;
    private patternCache: Map<string, UserPatterns> = new Map();

    constructor(config: PredictiveConfig) {
        this.config = config;
    }

    /**
     * Analyze user query patterns
     */
    async analyzePatterns(history: QueryHistory[], userId?: string): Promise<UserPatterns> {
        const cacheKey = userId || 'global';
        
        // Check cache first
        const cached = this.patternCache.get(cacheKey);
        if (cached && this.isCacheValid(cached)) {
            return cached;
        }

        // Analyze patterns
        const patterns = await this.extractPatterns(history);
        
        // Cache results
        this.patternCache.set(cacheKey, patterns);
        
        return patterns;
    }

    /**
     * Extract patterns from query history
     */
    private async extractPatterns(history: QueryHistory[]): Promise<UserPatterns> {
        // Filter recent history
        const recentHistory = this.filterRecentHistory(history);

        // Extract common query sequences
        const commonSequences = this.extractQuerySequences(recentHistory);

        // Analyze query type patterns
        const queryTypes = this.analyzeQueryTypes(recentHistory);

        // Extract time-based patterns
        const timePatterns = this.extractTimePatterns(recentHistory);

        // Extract topic patterns
        const topicPatterns = this.extractTopicPatterns(recentHistory);

        return {
            commonSequences,
            queryTypes,
            timePatterns,
            topicPatterns,
            totalQueries: recentHistory.length,
            analyzedAt: new Date()
        };
    }

    private filterRecentHistory(history: QueryHistory[]): QueryHistory[] {
        const cutoff = new Date(Date.now() - (this.config.patternWindowHours ?? 24) * 60 * 60 * 1000);
        return history.filter(h => h.timestamp > cutoff);
    }

    private extractQuerySequences(history: QueryHistory[]): QuerySequence[] {
        const sequences: Map<string, QuerySequence> = new Map();
        
        for (let i = 0; i < history.length - 1; i++) {
            const current = history[i];
            const next = history[i + 1];
            
            const sequenceKey = `${current.queryType}->${next.queryType}`;
            
            if (sequences.has(sequenceKey)) {
                const seq = sequences.get(sequenceKey)!;
                seq.frequency++;
                seq.examples.push({
                    from: current.query,
                    to: next.query,
                    timeDelta: next.timestamp.getTime() - current.timestamp.getTime()
                });
            } else {
                sequences.set(sequenceKey, {
                    from: current.queryType,
                    to: next.queryType,
                    frequency: 1,
                    confidence: 0.5,
                    examples: [{
                        from: current.query,
                        to: next.query,
                        timeDelta: next.timestamp.getTime() - current.timestamp.getTime()
                    }]
                });
            }
        }

        // Calculate confidence based on frequency
        for (const sequence of sequences.values()) {
            sequence.confidence = Math.min(0.9, sequence.frequency / history.length + 0.3);
        }

        return Array.from(sequences.values())
            .sort((a, b) => b.frequency - a.frequency)
            .slice(0, 10); // Top 10 sequences
    }

    private analyzeQueryTypes(history: QueryHistory[]): Map<string, number> {
        const types = new Map<string, number>();
        
        for (const query of history) {
            const current = types.get(query.queryType) || 0;
            types.set(query.queryType, current + 1);
        }

        return types;
    }

    private extractTimePatterns(history: QueryHistory[]): TimePattern[] {
        // Analyze time-based patterns (hour of day, day of week)
        const hourPatterns = new Map<number, number>();
        const dayPatterns = new Map<number, number>();

        for (const query of history) {
            const hour = query.timestamp.getHours();
            const day = query.timestamp.getDay();

            hourPatterns.set(hour, (hourPatterns.get(hour) || 0) + 1);
            dayPatterns.set(day, (dayPatterns.get(day) || 0) + 1);
        }

        return [
            {
                type: 'hourly',
                pattern: Object.fromEntries(hourPatterns),
                confidence: this.calculateTimePatternConfidence(hourPatterns)
            },
            {
                type: 'daily',
                pattern: Object.fromEntries(dayPatterns),
                confidence: this.calculateTimePatternConfidence(dayPatterns)
            }
        ];
    }

    private extractTopicPatterns(history: QueryHistory[]): TopicPattern[] {
        // Simple topic extraction based on keywords
        const topics = new Map<string, number>();
        const commonTopics = ['auth', 'user', 'api', 'database', 'error', 'test', 'performance'];

        for (const query of history) {
            const queryLower = query.query.toLowerCase();
            for (const topic of commonTopics) {
                if (queryLower.includes(topic)) {
                    topics.set(topic, (topics.get(topic) || 0) + 1);
                }
            }
        }

        return Array.from(topics.entries())
            .map(([topic, frequency]) => ({
                topic,
                frequency,
                confidence: Math.min(0.9, frequency / history.length + 0.2)
            }))
            .sort((a, b) => b.frequency - a.frequency)
            .slice(0, 5);
    }

    private calculateTimePatternConfidence(pattern: Map<number, number>): number {
        const values = Array.from(pattern.values());
        if (values.length === 0) {return 0;}
        const max = Math.max(...values);
        const avg = values.reduce((sum, v) => sum + v, 0) / values.length;
        if (avg === 0) {return 0;}
        return Math.min(0.9, max / avg / 10); // Normalize confidence
    }

    private isCacheValid(patterns: UserPatterns): boolean {
        const age = Date.now() - patterns.analyzedAt.getTime();
        const maxAge = (this.config.patternWindowHours ?? 24) * 60 * 60 * 1000 / 4; // Cache for 1/4 of window
        return age < maxAge;
    }

    async updatePatterns(originalQuery: string, followUpQuery: string, userId?: string): Promise<void> {
        // Update patterns with new data
        const cacheKey = userId || 'global';
        this.patternCache.delete(cacheKey); // Invalidate cache
    }
}

/**
 * Query Predictor - Predicts likely follow-up queries
 */
class QueryPredictor {
    private config: PredictiveConfig;
    private predictionModel: PredictionModel;

    constructor(config: PredictiveConfig) {
        this.config = config;
        this.predictionModel = new PredictionModel(config);
    }

    /**
     * Predict follow-up queries based on current query and patterns
     */
    async predictFollowUpQueries(
        currentQuery: string,
        patterns: UserPatterns,
        workspace: string
    ): Promise<PredictedQuery[]> {
        const predictions: PredictedQuery[] = [];

        // Sequence-based predictions
        const sequencePredictions = this.predictFromSequences(currentQuery, patterns);
        predictions.push(...sequencePredictions);

        // Topic-based predictions
        const topicPredictions = this.predictFromTopics(currentQuery, patterns);
        predictions.push(...topicPredictions);

        // ML-based predictions (if enabled)
        if (this.config.enableMachineLearning) {
            const mlPredictions = await this.predictionModel.predict(currentQuery, patterns);
            predictions.push(...mlPredictions);
        }

        // Deduplicate and sort by confidence
        const uniquePredictions = this.deduplicatePredictions(predictions);
        
        return uniquePredictions
            .sort((a, b) => b.confidence - a.confidence)
            .slice(0, this.config.maxPredictions);
    }

    private predictFromSequences(currentQuery: string, patterns: UserPatterns): PredictedQuery[] {
        const predictions: PredictedQuery[] = [];
        const currentType = this.inferQueryType(currentQuery);

        for (const sequence of patterns.commonSequences) {
            if (sequence.from === currentType) {
                const predictedQuery = this.generateQueryFromType(sequence.to, currentQuery);
                predictions.push({
                    query: predictedQuery,
                    confidence: sequence.confidence,
                    reasoning: `Based on common sequence: ${sequence.from} → ${sequence.to}`,
                    type: sequence.to,
                    source: 'sequence'
                });
            }
        }

        return predictions;
    }

    private predictFromTopics(currentQuery: string, patterns: UserPatterns): PredictedQuery[] {
        const predictions: PredictedQuery[] = [];
        const queryTopics = this.extractTopicsFromQuery(currentQuery);

        for (const topic of patterns.topicPatterns) {
            if (queryTopics.some(qt => qt.includes(topic.topic))) {
                const relatedQueries = this.generateRelatedQueries(topic.topic, currentQuery);
                for (const query of relatedQueries) {
                    predictions.push({
                        query,
                        confidence: topic.confidence * 0.8, // Slightly lower confidence
                        reasoning: `Related to topic: ${topic.topic}`,
                        type: this.inferQueryType(query),
                        source: 'topic'
                    });
                }
            }
        }

        return predictions;
    }

    private inferQueryType(query: string): string {
        const queryLower = query.toLowerCase();
        
        if (/how to|implement|create|build/.test(queryLower)) {return 'implementation';}
        if (/error|bug|fix|issue|problem/.test(queryLower)) {return 'debugging';}
        if (/refactor|improve|optimize/.test(queryLower)) {return 'refactoring';}
        if (/what is|explain|how does/.test(queryLower)) {return 'explanation';}
        if (/architecture|design|pattern/.test(queryLower)) {return 'architecture';}
        
        return 'general';
    }

    private generateQueryFromType(type: string, contextQuery: string): string {
        const templates = {
            implementation: [
                'How do I implement this?',
                'What\'s the best way to build this?',
                'Can you show me an example implementation?'
            ],
            debugging: [
                'Why is this not working?',
                'How do I fix this error?',
                'What could be causing this issue?'
            ],
            explanation: [
                'How does this work?',
                'Can you explain this code?',
                'What is the purpose of this?'
            ],
            testing: [
                'How do I test this?',
                'What test cases should I write?',
                'How do I mock this dependency?'
            ]
        };

        const typeTemplates = templates[type as keyof typeof templates] || templates.implementation;
        return typeTemplates[Math.floor(Math.random() * typeTemplates.length)];
    }

    private extractTopicsFromQuery(query: string): string[] {
        const commonTopics = ['auth', 'user', 'api', 'database', 'error', 'test', 'performance'];
        const queryLower = query.toLowerCase();
        return commonTopics.filter(topic => queryLower.includes(topic));
    }

    private generateRelatedQueries(topic: string, originalQuery: string): string[] {
        const relatedQueries: { [key: string]: string[] } = {
            auth: [
                'How do I implement authentication?',
                'What are the security best practices?',
                'How do I handle user sessions?'
            ],
            user: [
                'How do I manage user data?',
                'What about user permissions?',
                'How do I validate user input?'
            ],
            api: [
                'How do I design this API?',
                'What about API error handling?',
                'How do I test this API?'
            ],
            database: [
                'How do I optimize this query?',
                'What about database migrations?',
                'How do I handle database errors?'
            ]
        };

        return relatedQueries[topic] || [];
    }

    private deduplicatePredictions(predictions: PredictedQuery[]): PredictedQuery[] {
        const seen = new Set<string>();
        return predictions.filter(p => {
            const key = p.query.toLowerCase().trim();
            if (seen.has(key)) {
                return false;
            }
            seen.add(key);
            return true;
        });
    }

    async updateModel(
        originalQuery: string,
        actualFollowUp: string,
        wasHelpful: boolean,
        userId?: string
    ): Promise<void> {
        await this.predictionModel.updateWithFeedback(originalQuery, actualFollowUp, wasHelpful, userId);
    }
}

/**
 * Context Preloader - Handles actual context preloading
 */
class ContextPreloader {
    async preloadContext(query: string, workspace: string, confidence: number): Promise<PreloadContextResult> {
        const startTime = Date.now();
        
        // Check if already cached
        const cacheKey = `preload:${workspace}:${query}`;
        
        try {
            // Simulate context building (in real implementation, would call AIContextBuilder)
            await this.delay(50); // Simulate processing time
            
            const endTime = Date.now();
            
            return {
                success: true,
                fromCache: false,
                loadTime: endTime - startTime
            };
            
        } catch (error) {
            return {
                success: false,
                fromCache: false,
                loadTime: Date.now() - startTime,
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }

    private delay(ms: number): Promise<void> {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}

/**
 * Simple ML Prediction Model
 */
class PredictionModel {
    private config: PredictiveConfig;
    private trainingData: TrainingExample[] = [];

    constructor(config: PredictiveConfig) {
        this.config = config;
    }

    async predict(query: string, patterns: UserPatterns): Promise<PredictedQuery[]> {
        // Simple rule-based predictions (placeholder for actual ML)
        const predictions: PredictedQuery[] = [];
        
        // Add some common follow-up patterns
        const queryType = this.inferQueryType(query);
        
        switch (queryType) {
            case 'implementation':
                predictions.push({
                    query: 'How do I test this implementation?',
                    confidence: 0.75,
                    reasoning: 'Common follow-up to implementation questions',
                    type: 'testing',
                    source: 'ml'
                });
                break;
                
            case 'debugging':
                predictions.push({
                    query: 'How can I prevent this error in the future?',
                    confidence: 0.70,
                    reasoning: 'Common follow-up to debugging questions',
                    type: 'prevention',
                    source: 'ml'
                });
                break;
        }
        
        return predictions;
    }

    async updateWithFeedback(
        originalQuery: string,
        actualFollowUp: string,
        wasHelpful: boolean,
        userId?: string
    ): Promise<void> {
        this.trainingData.push({
            originalQuery,
            actualFollowUp,
            wasHelpful,
            userId,
            timestamp: new Date()
        });

        // Keep only recent training data
        const cutoff = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000); // 30 days
        this.trainingData = this.trainingData.filter(t => t.timestamp > cutoff);
    }

    private inferQueryType(query: string): string {
        // Same logic as QueryPredictor
        const queryLower = query.toLowerCase();
        
        if (/how to|implement|create|build/.test(queryLower)) {return 'implementation';}
        if (/error|bug|fix|issue|problem/.test(queryLower)) {return 'debugging';}
        if (/refactor|improve|optimize/.test(queryLower)) {return 'refactoring';}
        
        return 'general';
    }
}

// Supporting interfaces

export interface PredictiveConfig {
    maxPredictions?: number;
    minConfidence?: number;
    preloadDelay?: number;
    patternWindowHours?: number;
    enableMachineLearning?: boolean;
}

export interface QueryHistory {
    query: string;
    queryType: string;
    timestamp: Date;
    workspace: string;
    userId?: string;
    wasSuccessful: boolean;
}

export interface PredictedQuery {
    query: string;
    confidence: number;
    reasoning: string;
    type: string;
    source: 'sequence' | 'topic' | 'ml';
}

export interface PredictionResult {
    predictions: PredictedQuery[];
    preloadedCount: number;
    executionTime: number;
    patterns: UserPatterns;
    cacheHits: number;
    error?: string;
}

export interface UserPatterns {
    commonSequences: QuerySequence[];
    queryTypes: Map<string, number>;
    timePatterns: TimePattern[];
    topicPatterns: TopicPattern[];
    totalQueries: number;
    analyzedAt: Date;
}

interface QuerySequence {
    from: string;
    to: string;
    frequency: number;
    confidence: number;
    examples: SequenceExample[];
}

interface SequenceExample {
    from: string;
    to: string;
    timeDelta: number;
}

interface TimePattern {
    type: 'hourly' | 'daily';
    pattern: { [key: number]: number };
    confidence: number;
}

interface TopicPattern {
    topic: string;
    frequency: number;
    confidence: number;
}

interface PreloadResult {
    query: string;
    success: boolean;
    fromCache: boolean;
    loadTime: number;
    confidence: number;
    error?: string;
}

interface PreloadContextResult {
    success: boolean;
    fromCache: boolean;
    loadTime: number;
    error?: string;
}

interface PredictiveStats {
    totalPredictions: number;
    averageConfidence: number;
    hitRate: number;
    memoryUsage: number;
}

interface TrainingExample {
    originalQuery: string;
    actualFollowUp: string;
    wasHelpful: boolean;
    userId?: string;
    timestamp: Date;
}
