/**
 * Language Parser Interface - Common interface for all language parsers
 * 
 * This interface defines the contract that all language-specific parsers must implement
 * to provide consistent semantic analysis across different programming languages.
 */

export interface LanguageParser {
    /**
     * Parse a source file and extract semantic information
     */
    parseFile(content: string, filePath: string): Promise<ParseResult>;
}

export interface ParseResult {
    success: boolean;
    symbols: Symbol[];
    callGraph: CallGraph;
    dependencies: DependencyInfo[];
    ast: any; // Language-specific AST
    diagnostics: Diagnostic[];
}

export interface Diagnostic {
    message: string;
    severity: 'error' | 'warning' | 'info';
    line: number;
    column: number;
}

export interface Location {
    file: string;
    startLine: number;
    endLine: number;
    startColumn: number;
    endColumn: number;
}

// Base symbol interface
export interface Symbol {
    type: 'function' | 'class' | 'interface' | 'type' | 'variable' | 'constant' | 'import' | 'export';
    name: string;
    location: Location;
    isExported?: boolean;
    documentation?: string;
}

// Function symbol
export interface FunctionSymbol extends Symbol {
    type: 'function';
    parameters: ParameterInfo[];
    returnType: string;
    isAsync: boolean;
    visibility: 'public' | 'private' | 'protected';
    body: string;
    calls: string[];
    complexity: number;
}

// Class symbol
export interface ClassSymbol extends Symbol {
    type: 'class';
    superClass?: string;
    interfaces: string[];
    methods: MethodInfo[];
    properties: PropertyInfo[];
    visibility: 'public' | 'private' | 'protected';
    isAbstract: boolean;
}

// Interface symbol
export interface InterfaceSymbol extends Symbol {
    type: 'interface';
    extendsInterfaces: string[];
    methods: MethodSignature[];
    properties: PropertySignature[];
}

// Type symbol
export interface TypeSymbol extends Symbol {
    type: 'type';
    definition: string;
    genericParameters: string[];
}

// Import symbol
export interface ImportSymbol extends Symbol {
    type: 'import';
    module: string;
    importedItems: string[];
    defaultImport?: string;
    namespaceImport?: string;
}

// Variable symbol
export interface VariableSymbol extends Symbol {
    type: 'variable';
    variableType: string;
    initializer?: string;
    isConst: boolean;
}

// Supporting interfaces
export interface ParameterInfo {
    name: string;
    type: string;
    optional: boolean;
    defaultValue?: string;
}

export interface MethodInfo {
    name: string;
    parameters: ParameterInfo[];
    returnType: string;
    isStatic: boolean;
    isAsync: boolean;
    visibility: 'public' | 'private' | 'protected';
}

export interface PropertyInfo {
    name: string;
    type: string;
    isStatic: boolean;
    isReadonly: boolean;
    visibility: 'public' | 'private' | 'protected';
}

export interface MethodSignature {
    name: string;
    parameters: ParameterInfo[];
    returnType: string;
    optional: boolean;
}

export interface PropertySignature {
    name: string;
    type: string;
    optional: boolean;
    readonly: boolean;
}

export interface CallGraph {
    nodes: CallGraphNode[];
    edges: CallGraphEdge[];
}

export interface CallGraphNode {
    name: string;
    type: string;
    file: string;
}

export interface CallGraphEdge {
    from: string;
    to: string;
    type: string;
}

export interface DependencyInfo {
    type: 'import' | 'require' | 'include';
    target: string;
    isExternal: boolean;
    usage: string;
}

/**
 * Language parser factory
 */
export class LanguageParserFactory {
    private static parsers: Map<string, () => LanguageParser> = new Map();

    static registerParser(language: string, factory: () => LanguageParser): void {
        this.parsers.set(language.toLowerCase(), factory);
    }

    static createParser(language: string): LanguageParser | null {
        const factory = this.parsers.get(language.toLowerCase());
        return factory ? factory() : null;
    }

    static getSupportedLanguages(): string[] {
        return Array.from(this.parsers.keys());
    }
}

/**
 * Utility functions for parsing
 */
export class ParserUtils {
    /**
     * Detect programming language from file extension
     */
    static detectLanguage(filePath: string): string {
        const extension = filePath.split('.').pop()?.toLowerCase();
        
        switch (extension) {
            case 'ts':
            case 'tsx':
                return 'typescript';
            case 'js':
            case 'jsx':
                return 'javascript';
            case 'py':
                return 'python';
            case 'rs':
                return 'rust';
            case 'go':
                return 'go';
            case 'java':
                return 'java';
            case 'cpp':
            case 'cc':
            case 'cxx':
                return 'cpp';
            case 'c':
                return 'c';
            case 'cs':
                return 'csharp';
            case 'rb':
                return 'ruby';
            case 'php':
                return 'php';
            case 'kt':
                return 'kotlin';
            case 'swift':
                return 'swift';
            default:
                return 'text';
        }
    }

    /**
     * Calculate cyclomatic complexity for a code block
     */
    static calculateCyclomaticComplexity(code: string): number {
        // Simple regex-based complexity calculation
        // In a real implementation, this would use proper AST analysis
        let complexity = 1; // Base complexity

        const complexityPatterns = [
            /\bif\b/g,
            /\belse\s+if\b/g,
            /\bwhile\b/g,
            /\bfor\b/g,
            /\bswitch\b/g,
            /\bcase\b/g,
            /\bcatch\b/g,
            /\b\?\s*:/g, // Ternary operator
            /&&/g,
            /\|\|/g
        ];

        for (const pattern of complexityPatterns) {
            const matches = code.match(pattern);
            if (matches) {
                complexity += matches.length;
            }
        }

        return complexity;
    }

    /**
     * Extract function calls from code
     */
    static extractFunctionCalls(code: string): string[] {
        const functionCallPattern = /(\w+)\s*\(/g;
        const calls: string[] = [];
        let match;

        while ((match = functionCallPattern.exec(code)) !== null) {
            calls.push(match[1]);
        }

        return [...new Set(calls)]; // Remove duplicates
    }

    /**
     * Extract imports from code
     */
    static extractImports(code: string, language: string): string[] {
        const imports: string[] = [];

        switch (language.toLowerCase()) {
            case 'typescript':
            case 'javascript':
                const tsImportPattern = /import\s+.*?\s+from\s+['"]([^'"]+)['"]/g;
                const requirePattern = /require\(['"]([^'"]+)['"]\)/g;
                
                let match;
                while ((match = tsImportPattern.exec(code)) !== null) {
                    imports.push(match[1]);
                }
                while ((match = requirePattern.exec(code)) !== null) {
                    imports.push(match[1]);
                }
                break;

            case 'python':
                const pythonImportPattern = /(?:from\s+(\S+)\s+import|import\s+(\S+))/g;
                while ((match = pythonImportPattern.exec(code)) !== null) {
                    imports.push(match[1] || match[2]);
                }
                break;

            case 'rust':
                const rustUsePattern = /use\s+([^;]+);/g;
                while ((match = rustUsePattern.exec(code)) !== null) {
                    imports.push(match[1]);
                }
                break;
        }

        return imports;
    }

    /**
     * Normalize symbol names for consistency
     */
    static normalizeSymbolName(name: string): string {
        return name.trim().replace(/\s+/g, '_');
    }

    /**
     * Extract documentation comments
     */
    static extractDocumentation(code: string, language: string): string[] {
        const docs: string[] = [];

        switch (language.toLowerCase()) {
            case 'typescript':
            case 'javascript':
                // JSDoc comments
                const jsDocPattern = /\/\*\*([\s\S]*?)\*\//g;
                let match;
                while ((match = jsDocPattern.exec(code)) !== null) {
                    docs.push(match[1].trim());
                }
                break;

            case 'python':
                // Python docstrings
                const docstringPattern = /"""([\s\S]*?)"""/g;
                while ((match = docstringPattern.exec(code)) !== null) {
                    docs.push(match[1].trim());
                }
                break;

            case 'rust':
                // Rust doc comments
                const rustDocPattern = /\/\/\/(.*)/g;
                while ((match = rustDocPattern.exec(code)) !== null) {
                    docs.push(match[1].trim());
                }
                break;
        }

        return docs;
    }
}
