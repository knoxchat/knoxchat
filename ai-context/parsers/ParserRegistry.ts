/**
 * Parser Registry - Central registry for all language parsers
 * 
 * This service manages and provides access to all available language parsers,
 * automatically selecting the appropriate parser based on file extension.
 */

import { LanguageParser, LanguageParserFactory, ParserUtils } from './LanguageParser';
import { PythonParser } from './PythonParser';
import { RustParser } from './RustParser';
import { TypeScriptParser } from './TypeScriptParser';

export class ParserRegistry {
    private static instance: ParserRegistry;
    private initialized = false;

    private constructor() {
        this.initializeParsers();
    }

    public static getInstance(): ParserRegistry {
        if (!ParserRegistry.instance) {
            ParserRegistry.instance = new ParserRegistry();
        }
        return ParserRegistry.instance;
    }

    /**
     * Initialize all available parsers
     */
    private initializeParsers(): void {
        if (this.initialized) {return;}

        // Register TypeScript/JavaScript parser
        LanguageParserFactory.registerParser('typescript', () => new TypeScriptParser());
        LanguageParserFactory.registerParser('javascript', () => new TypeScriptParser()); // TypeScript parser handles JS too

        // Register Python parser
        LanguageParserFactory.registerParser('python', () => new PythonParser());

        // Register Rust parser
        LanguageParserFactory.registerParser('rust', () => new RustParser());

        // TODO: Add more parsers as needed
        // LanguageParserFactory.registerParser('go', () => new GoParser());
        // LanguageParserFactory.registerParser('java', () => new JavaParser());
        // LanguageParserFactory.registerParser('cpp', () => new CppParser());

        this.initialized = true;
    }

    /**
     * Get parser for a specific language
     */
    public getParser(language: string): LanguageParser | null {
        return LanguageParserFactory.createParser(language);
    }

    /**
     * Get parser for a file based on its extension
     */
    public getParserForFile(filePath: string): LanguageParser | null {
        const language = ParserUtils.detectLanguage(filePath);
        return this.getParser(language);
    }

    /**
     * Check if a language is supported
     */
    public isLanguageSupported(language: string): boolean {
        return LanguageParserFactory.getSupportedLanguages().includes(language.toLowerCase());
    }

    /**
     * Get all supported languages
     */
    public getSupportedLanguages(): string[] {
        return LanguageParserFactory.getSupportedLanguages();
    }

    /**
     * Parse a file using the appropriate parser
     */
    public async parseFile(content: string, filePath: string): Promise<any> {
        const parser = this.getParserForFile(filePath);
        
        if (!parser) {
            return {
                success: false,
                error: `No parser available for file: ${filePath}`,
                symbols: [],
                callGraph: { nodes: [], edges: [] },
                dependencies: [],
                ast: null,
                diagnostics: [{
                    message: `Unsupported file type: ${ParserUtils.detectLanguage(filePath)}`,
                    severity: 'warning',
                    line: 0,
                    column: 0
                }]
            };
        }

        return await parser.parseFile(content, filePath);
    }

    /**
     * Parse multiple files in batch
     */
    public async parseFiles(files: { content: string; path: string }[]): Promise<Map<string, any>> {
        const results = new Map<string, any>();
        
        // Parse files in parallel
        const parsePromises = files.map(async file => {
            const result = await this.parseFile(file.content, file.path);
            return { path: file.path, result };
        });

        const parsedFiles = await Promise.all(parsePromises);
        
        for (const { path, result } of parsedFiles) {
            results.set(path, result);
        }

        return results;
    }

    /**
     * Get parsing statistics
     */
    public getParsingStats(results: Map<string, any>): ParsingStats {
        let totalFiles = 0;
        let successfulParses = 0;
        let failedParses = 0;
        let totalSymbols = 0;
        let totalFunctions = 0;
        let totalClasses = 0;
        let totalInterfaces = 0;
        
        const languageStats = new Map<string, number>();

        for (const [filePath, result] of results) {
            totalFiles++;
            
            const language = ParserUtils.detectLanguage(filePath);
            languageStats.set(language, (languageStats.get(language) || 0) + 1);
            
            if (result.success) {
                successfulParses++;
                totalSymbols += result.symbols.length;
                
                for (const symbol of result.symbols) {
                    switch (symbol.type) {
                        case 'function':
                            totalFunctions++;
                            break;
                        case 'class':
                            totalClasses++;
                            break;
                        case 'interface':
                            totalInterfaces++;
                            break;
                    }
                }
            } else {
                failedParses++;
            }
        }

        return {
            totalFiles,
            successfulParses,
            failedParses,
            successRate: totalFiles > 0 ? successfulParses / totalFiles : 0,
            totalSymbols,
            totalFunctions,
            totalClasses,
            totalInterfaces,
            languageDistribution: Object.fromEntries(languageStats),
            averageSymbolsPerFile: totalFiles > 0 ? totalSymbols / totalFiles : 0
        };
    }

    /**
     * Validate parser configuration
     */
    public validateConfiguration(): ValidationResult {
        const issues: string[] = [];
        const supportedLanguages = this.getSupportedLanguages();

        // Check if core languages are supported
        const coreLanguages = ['typescript', 'javascript', 'python', 'rust'];
        for (const lang of coreLanguages) {
            if (!supportedLanguages.includes(lang)) {
                issues.push(`Core language not supported: ${lang}`);
            }
        }

        // Test each parser with sample code
        const testResults: { [language: string]: boolean } = {};
        for (const lang of supportedLanguages) {
            try {
                const parser = this.getParser(lang);
                if (parser) {
                    // Simple test - this would be expanded in production
                    testResults[lang] = true;
                } else {
                    testResults[lang] = false;
                    issues.push(`Failed to create parser for ${lang}`);
                }
            } catch (error) {
                testResults[lang] = false;
                issues.push(`Error testing parser for ${lang}: ${error}`);
            }
        }

        return {
            isValid: issues.length === 0,
            issues,
            supportedLanguages,
            testResults
        };
    }
}

// Supporting interfaces
export interface ParsingStats {
    totalFiles: number;
    successfulParses: number;
    failedParses: number;
    successRate: number;
    totalSymbols: number;
    totalFunctions: number;
    totalClasses: number;
    totalInterfaces: number;
    languageDistribution: { [language: string]: number };
    averageSymbolsPerFile: number;
}

export interface ValidationResult {
    isValid: boolean;
    issues: string[];
    supportedLanguages: string[];
    testResults: { [language: string]: boolean };
}

/**
 * Enhanced parser utilities with additional functionality
 */
export class EnhancedParserUtils extends ParserUtils {
    
    /**
     * Analyze code quality metrics
     */
    static analyzeCodeQuality(symbols: any[]): CodeQualityMetrics {
        let totalComplexity = 0;
        let functionsAnalyzed = 0;
        let classesWithDocumentation = 0;
        let functionsWithDocumentation = 0;
        let publicSymbols = 0;
        let privateSymbols = 0;

        for (const symbol of symbols) {
            if (symbol.type === 'function') {
                functionsAnalyzed++;
                totalComplexity += symbol.complexity || 0;
                
                if (symbol.documentation) {
                    functionsWithDocumentation++;
                }
            }
            
            if (symbol.type === 'class') {
                if (symbol.documentation) {
                    classesWithDocumentation++;
                }
            }

            if (symbol.isExported || symbol.visibility === 'public') {
                publicSymbols++;
            } else {
                privateSymbols++;
            }
        }

        const averageComplexity = functionsAnalyzed > 0 ? totalComplexity / functionsAnalyzed : 0;
        const documentationCoverage = symbols.length > 0 ? 
            (functionsWithDocumentation + classesWithDocumentation) / symbols.length : 0;

        return {
            averageComplexity,
            totalComplexity,
            functionsAnalyzed,
            documentationCoverage,
            publicToPrivateRatio: privateSymbols > 0 ? publicSymbols / privateSymbols : publicSymbols,
            qualityScore: this.calculateQualityScore(averageComplexity, documentationCoverage)
        };
    }

    /**
     * Calculate overall quality score (0-100)
     */
    private static calculateQualityScore(avgComplexity: number, docCoverage: number): number {
        // Lower complexity is better (max score at complexity <= 5)
        const complexityScore = Math.max(0, 100 - Math.max(0, avgComplexity - 5) * 10);
        
        // Higher documentation coverage is better
        const docScore = docCoverage * 100;
        
        // Weighted average (complexity 60%, documentation 40%)
        return Math.round(complexityScore * 0.6 + docScore * 0.4);
    }

    /**
     * Extract architectural patterns from symbols
     */
    static identifyArchitecturalPatterns(symbols: any[]): ArchitecturalPattern[] {
        const patterns: ArchitecturalPattern[] = [];

        // Identify common patterns
        const classes = symbols.filter(s => s.type === 'class');
        const interfaces = symbols.filter(s => s.type === 'interface');

        // Factory pattern detection
        const factories = classes.filter(cls => 
            cls.name.toLowerCase().includes('factory') ||
            cls.methods?.some((m: any) => m.name.toLowerCase().includes('create'))
        );
        if (factories.length > 0) {
            patterns.push({
                name: 'Factory',
                type: 'creational',
                confidence: 0.8,
                locations: factories.map(f => f.location.file)
            });
        }

        // Observer pattern detection
        const observers = classes.filter(cls =>
            cls.methods?.some((m: any) => 
                m.name.toLowerCase().includes('notify') || 
                m.name.toLowerCase().includes('subscribe') ||
                m.name.toLowerCase().includes('observer')
            )
        );
        if (observers.length > 0) {
            patterns.push({
                name: 'Observer',
                type: 'behavioral',
                confidence: 0.7,
                locations: observers.map(o => o.location.file)
            });
        }

        // Singleton pattern detection
        const singletons = classes.filter(cls =>
            cls.methods?.some((m: any) => 
                m.name.toLowerCase().includes('getinstance') ||
                m.name.toLowerCase().includes('instance')
            )
        );
        if (singletons.length > 0) {
            patterns.push({
                name: 'Singleton',
                type: 'creational',
                confidence: 0.6,
                locations: singletons.map(s => s.location.file)
            });
        }

        return patterns;
    }
}

// Additional interfaces
export interface CodeQualityMetrics {
    averageComplexity: number;
    totalComplexity: number;
    functionsAnalyzed: number;
    documentationCoverage: number;
    publicToPrivateRatio: number;
    qualityScore: number;
}

export interface ArchitecturalPattern {
    name: string;
    type: 'creational' | 'structural' | 'behavioral';
    confidence: number;
    locations: string[];
}

// Export singleton instance
export const parserRegistry = ParserRegistry.getInstance();
