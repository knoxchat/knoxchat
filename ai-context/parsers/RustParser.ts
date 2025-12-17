/**
 * Rust AST Parser - Implementation using regex and pattern matching
 * 
 * This parser provides semantic analysis of Rust code,
 * extracting functions, structs, traits, implementations, and relationships.
 * Note: For production use, this would ideally use a Rust AST library or service.
 */

import { LanguageParser, ParseResult, Symbol, FunctionSymbol, ClassSymbol, InterfaceSymbol, ImportSymbol, CallGraph, DependencyInfo, Location, Diagnostic, ParameterInfo, MethodInfo, PropertyInfo } from './LanguageParser';

export class RustParser implements LanguageParser {
    
    /**
     * Parse Rust file and extract semantic information
     */
    async parseFile(content: string, filePath: string): Promise<ParseResult> {
        try {
            const lines = content.split('\n');
            const symbols = this.extractSymbols(content, filePath, lines);
            const callGraph = this.buildCallGraph(content, filePath);
            const dependencies = this.analyzeDependencies(content);
            
            return {
                success: true,
                symbols,
                callGraph,
                dependencies,
                ast: { content, lines }, // Simple AST representation
                diagnostics: []
            };
        } catch (error) {
            return {
                success: false,
                symbols: [],
                callGraph: { nodes: [], edges: [] },
                dependencies: [],
                ast: null,
                diagnostics: [{
                    message: error instanceof Error ? error.message : String(error),
                    severity: 'error',
                    line: 0,
                    column: 0
                }]
            };
        }
    }

    /**
     * Extract all symbols from Rust code
     */
    private extractSymbols(content: string, filePath: string, lines: string[]): Symbol[] {
        const symbols: Symbol[] = [];

        // Extract function definitions
        symbols.push(...this.extractFunctions(content, filePath, lines));
        
        // Extract struct definitions
        symbols.push(...this.extractStructs(content, filePath, lines));
        
        // Extract trait definitions
        symbols.push(...this.extractTraits(content, filePath, lines));
        
        // Extract enum definitions
        symbols.push(...this.extractEnums(content, filePath, lines));
        
        // Extract use statements
        symbols.push(...this.extractUseStatements(content, filePath, lines));
        
        // Extract constants
        symbols.push(...this.extractConstants(content, filePath, lines));

        return symbols;
    }

    /**
     * Extract function definitions
     */
    private extractFunctions(content: string, filePath: string, lines: string[]): FunctionSymbol[] {
        const functions: FunctionSymbol[] = [];
        
        // Match function definitions: pub fn name(params) -> return_type { ... }
        const functionPattern = /^(\s*)(pub\s+)?(async\s+)?fn\s+(\w+)\s*(<[^>]*>)?\s*\(([^)]*)\)\s*(?:->\s*([^{]+))?\s*\{/gm;
        
        let match;
        while ((match = functionPattern.exec(content)) !== null) {
            const indentation = match[1];
            const isPublic = !!match[2];
            const isAsync = !!match[3];
            const functionName = match[4];
            const generics = match[5] || '';
            const parametersStr = match[6];
            const returnType = match[7] ? match[7].trim() : '()';
            
            const lineNumber = content.substring(0, match.index).split('\n').length;
            
            // Parse parameters
            const parameters = this.parseRustParameters(parametersStr);
            
            // Extract function body
            const functionBody = this.extractRustFunctionBody(content, match.index + match[0].length);
            
            // Extract function calls within the body
            const calls = this.extractFunctionCallsFromBody(functionBody);
            
            // Calculate complexity
            const complexity = this.calculateComplexity(functionBody);
            
            // Extract documentation comments
            const documentation = this.extractRustDocumentation(content, match.index);

            const location: Location = {
                file: filePath,
                startLine: lineNumber,
                endLine: lineNumber + functionBody.split('\n').length - 1,
                startColumn: indentation.length,
                endColumn: indentation.length + match[0].length
            };

            functions.push({
                type: 'function',
                name: functionName,
                parameters,
                returnType,
                isAsync,
                visibility: isPublic ? 'public' : 'private',
                location,
                body: functionBody,
                calls,
                complexity,
                isExported: isPublic,
                documentation
            });
        }

        return functions;
    }

    /**
     * Extract struct definitions
     */
    private extractStructs(content: string, filePath: string, lines: string[]): ClassSymbol[] {
        const structs: ClassSymbol[] = [];
        
        // Match struct definitions: pub struct Name { ... }
        const structPattern = /^(\s*)(pub\s+)?struct\s+(\w+)\s*(<[^>]*>)?\s*\{([^}]*)\}/gm;
        
        let match;
        while ((match = structPattern.exec(content)) !== null) {
            const indentation = match[1];
            const isPublic = !!match[2];
            const structName = match[3];
            const generics = match[4] || '';
            const fieldsStr = match[5];
            
            const lineNumber = content.substring(0, match.index).split('\n').length;
            
            // Parse fields
            const properties = this.parseRustFields(fieldsStr);
            
            // Find implementations for this struct
            const methods = this.findImplementationMethods(content, structName);
            
            // Extract documentation comments
            const documentation = this.extractRustDocumentation(content, match.index);

            const location: Location = {
                file: filePath,
                startLine: lineNumber,
                endLine: lineNumber + match[0].split('\n').length - 1,
                startColumn: indentation.length,
                endColumn: indentation.length + match[0].length
            };

            structs.push({
                type: 'class',
                name: structName,
                superClass: undefined, // Rust doesn't have inheritance
                interfaces: [], // Will be filled with trait implementations
                methods,
                properties,
                visibility: isPublic ? 'public' : 'private',
                isAbstract: false,
                location,
                isExported: isPublic,
                documentation
            });
        }

        return structs;
    }

    /**
     * Extract trait definitions
     */
    private extractTraits(content: string, filePath: string, lines: string[]): InterfaceSymbol[] {
        const traits: InterfaceSymbol[] = [];
        
        // Match trait definitions: pub trait Name { ... }
        const traitPattern = /^(\s*)(pub\s+)?trait\s+(\w+)\s*(<[^>]*>)?\s*\{([^}]*)\}/gm;
        
        let match;
        while ((match = traitPattern.exec(content)) !== null) {
            const indentation = match[1];
            const isPublic = !!match[2];
            const traitName = match[3];
            const generics = match[4] || '';
            const bodyStr = match[5];
            
            const lineNumber = content.substring(0, match.index).split('\n').length;
            
            // Parse trait methods
            const methods = this.parseTraitMethods(bodyStr);
            
            // Extract documentation comments
            const documentation = this.extractRustDocumentation(content, match.index);

            const location: Location = {
                file: filePath,
                startLine: lineNumber,
                endLine: lineNumber + match[0].split('\n').length - 1,
                startColumn: indentation.length,
                endColumn: indentation.length + match[0].length
            };

            traits.push({
                type: 'interface',
                name: traitName,
                extendsInterfaces: [], // Rust traits can have supertraits
                methods,
                properties: [],
                location,
                isExported: isPublic,
                documentation
            });
        }

        return traits;
    }

    /**
     * Extract enum definitions
     */
    private extractEnums(content: string, filePath: string, lines: string[]): Symbol[] {
        const enums: Symbol[] = [];
        
        // Match enum definitions: pub enum Name { ... }
        const enumPattern = /^(\s*)(pub\s+)?enum\s+(\w+)\s*(<[^>]*>)?\s*\{([^}]*)\}/gm;
        
        let match;
        while ((match = enumPattern.exec(content)) !== null) {
            const indentation = match[1];
            const isPublic = !!match[2];
            const enumName = match[3];
            const generics = match[4] || '';
            const variantsStr = match[5];
            
            const lineNumber = content.substring(0, match.index).split('\n').length;
            
            // Extract documentation comments
            const documentation = this.extractRustDocumentation(content, match.index);

            const location: Location = {
                file: filePath,
                startLine: lineNumber,
                endLine: lineNumber + match[0].split('\n').length - 1,
                startColumn: indentation.length,
                endColumn: indentation.length + match[0].length
            };

            enums.push({
                type: 'type',
                name: enumName,
                location,
                isExported: isPublic,
                documentation,
                definition: `enum ${enumName} { ${variantsStr.trim()} }`,
                genericParameters: generics ? [generics] : []
            } as Symbol);
        }

        return enums;
    }

    /**
     * Extract use statements (imports)
     */
    private extractUseStatements(content: string, filePath: string, lines: string[]): ImportSymbol[] {
        const imports: ImportSymbol[] = [];
        
        // Match use statements: use path::to::item;
        const usePattern = /^(\s*)use\s+([^;]+);/gm;
        
        let match;
        while ((match = usePattern.exec(content)) !== null) {
            const indentation = match[1];
            const usePath = match[2].trim();
            
            const lineNumber = content.substring(0, match.index).split('\n').length;
            
            // Parse the use path
            const { module, importedItems } = this.parseRustUsePath(usePath);

            const location: Location = {
                file: filePath,
                startLine: lineNumber,
                endLine: lineNumber,
                startColumn: indentation.length,
                endColumn: indentation.length + match[0].length
            };

            imports.push({
                type: 'import',
                name: usePath,
                module,
                importedItems,
                location
            });
        }

        return imports;
    }

    /**
     * Extract constants
     */
    private extractConstants(content: string, filePath: string, lines: string[]): Symbol[] {
        const constants: Symbol[] = [];
        
        // Match const definitions: pub const NAME: Type = value;
        const constPattern = /^(\s*)(pub\s+)?const\s+(\w+)\s*:\s*([^=]+)\s*=\s*([^;]+);/gm;
        
        let match;
        while ((match = constPattern.exec(content)) !== null) {
            const indentation = match[1];
            const isPublic = !!match[2];
            const constName = match[3];
            const constType = match[4].trim();
            const value = match[5].trim();
            
            const lineNumber = content.substring(0, match.index).split('\n').length;

            const location: Location = {
                file: filePath,
                startLine: lineNumber,
                endLine: lineNumber,
                startColumn: indentation.length,
                endColumn: indentation.length + match[0].length
            };

            constants.push({
                type: 'constant',
                name: constName,
                location,
                isExported: isPublic,
                variableType: constType,
                initializer: value,
                isConst: true
            } as Symbol);
        }

        return constants;
    }

    /**
     * Build call graph from Rust code
     */
    private buildCallGraph(content: string, filePath: string): CallGraph {
        const nodes: any[] = [];
        const edges: any[] = [];

        // Extract all function definitions first
        const functionPattern = /fn\s+(\w+)\s*\(/g;
        let match;
        while ((match = functionPattern.exec(content)) !== null) {
            nodes.push({
                name: match[1],
                type: 'function',
                file: filePath
            });
        }

        // Extract function calls and create edges
        for (const node of nodes) {
            const functionCalls = this.findFunctionCallsInFunction(content, node.name);
            for (const call of functionCalls) {
                edges.push({
                    from: node.name,
                    to: call,
                    type: 'call'
                });
            }
        }

        return { nodes, edges };
    }

    /**
     * Analyze dependencies
     */
    private analyzeDependencies(content: string): DependencyInfo[] {
        const dependencies: DependencyInfo[] = [];
        
        const usePattern = /use\s+([^;]+);/g;
        
        let match;
        while ((match = usePattern.exec(content)) !== null) {
            const usePath = match[1].trim();
            const module = usePath.split('::')[0];
            
            dependencies.push({
                type: 'import',
                target: module,
                isExternal: !module.startsWith('crate') && !module.startsWith('self') && !module.startsWith('super'),
                usage: 'use'
            });
        }

        return dependencies;
    }

    // Helper methods

    private parseRustParameters(parametersStr: string): ParameterInfo[] {
        if (!parametersStr.trim()) {return [];}
        
        return parametersStr.split(',').map(param => {
            const trimmed = param.trim();
            
            // Handle patterns like "mut self", "&self", "&mut self"
            if (trimmed === 'self' || trimmed === 'mut self' || trimmed === '&self' || trimmed === '&mut self') {
                return {
                    name: 'self',
                    type: 'Self',
                    optional: false
                };
            }
            
            // Handle regular parameters: name: type
            const colonIndex = trimmed.lastIndexOf(':');
            if (colonIndex === -1) {
                return {
                    name: trimmed,
                    type: 'unknown',
                    optional: false
                };
            }
            
            const name = trimmed.substring(0, colonIndex).trim();
            const type = trimmed.substring(colonIndex + 1).trim();
            
            return {
                name,
                type,
                optional: false // Rust doesn't have optional parameters like TypeScript
            };
        });
    }

    private extractRustFunctionBody(content: string, startIndex: number): string {
        let braceCount = 1;
        let i = startIndex;
        
        while (i < content.length && braceCount > 0) {
            if (content[i] === '{') {braceCount++;}
            if (content[i] === '}') {braceCount--;}
            i++;
        }
        
        return content.substring(startIndex, i - 1);
    }

    private parseRustFields(fieldsStr: string): PropertyInfo[] {
        const fields: PropertyInfo[] = [];
        
        // Split by comma and parse each field
        const fieldLines = fieldsStr.split(',');
        
        for (const line of fieldLines) {
            const trimmed = line.trim();
            if (!trimmed) {continue;}
            
            // Match: pub field_name: Type
            const fieldMatch = trimmed.match(/^(pub\s+)?(\w+)\s*:\s*(.+)$/);
            if (fieldMatch) {
                const isPublic = !!fieldMatch[1];
                const fieldName = fieldMatch[2];
                const fieldType = fieldMatch[3].trim();
                
                fields.push({
                    name: fieldName,
                    type: fieldType,
                    isStatic: false,
                    isReadonly: false, // Rust mutability is different
                    visibility: isPublic ? 'public' : 'private'
                });
            }
        }
        
        return fields;
    }

    private findImplementationMethods(content: string, structName: string): MethodInfo[] {
        const methods: MethodInfo[] = [];
        
        // Find impl blocks for this struct
        const implPattern = new RegExp(`impl(?:\\s*<[^>]*>)?\\s+${structName}(?:\\s*<[^>]*>)?\\s*\\{([^}]*)\\}`, 'g');
        
        let implMatch;
        while ((implMatch = implPattern.exec(content)) !== null) {
            const implBody = implMatch[1];
            
            // Find methods within the impl block
            const methodPattern = /fn\s+(\w+)\s*\(([^)]*)\)\s*(?:->\s*([^{]+))?\s*\{/g;
            
            let methodMatch;
            while ((methodMatch = methodPattern.exec(implBody)) !== null) {
                const methodName = methodMatch[1];
                const parametersStr = methodMatch[2];
                const returnType = methodMatch[3] ? methodMatch[3].trim() : '()';
                
                const parameters = this.parseRustParameters(parametersStr);
                const isStatic = !parameters.some(p => p.name === 'self');
                
                methods.push({
                    name: methodName,
                    parameters,
                    returnType,
                    isStatic,
                    isAsync: false, // Would need to check for async keyword
                    visibility: 'public' // Would need to check for pub keyword
                });
            }
        }
        
        return methods;
    }

    private parseTraitMethods(bodyStr: string): any[] {
        const methods: any[] = [];
        
        const methodPattern = /fn\s+(\w+)\s*\(([^)]*)\)\s*(?:->\s*([^;{]+))?[;{]/g;
        
        let match;
        while ((match = methodPattern.exec(bodyStr)) !== null) {
            const methodName = match[1];
            const parametersStr = match[2];
            const returnType = match[3] ? match[3].trim() : '()';
            
            const parameters = this.parseRustParameters(parametersStr);
            
            methods.push({
                name: methodName,
                parameters,
                returnType,
                optional: false
            });
        }
        
        return methods;
    }

    private parseRustUsePath(usePath: string): { module: string; importedItems: string[] } {
        // Handle different use patterns:
        // use std::collections::HashMap;
        // use std::collections::{HashMap, HashSet};
        // use std::collections::*;
        
        if (usePath.includes('{')) {
            // Multiple imports: use module::{item1, item2}
            const parts = usePath.split('{');
            const module = parts[0].replace('::', '').trim();
            const itemsPart = parts[1].replace('}', '');
            const items = itemsPart.split(',').map(item => item.trim());
            return { module, importedItems: items };
        } else if (usePath.endsWith('::*')) {
            // Glob import: use module::*
            const module = usePath.replace('::*', '');
            return { module, importedItems: ['*'] };
        } else {
            // Single import: use module::item
            const parts = usePath.split('::');
            const item = parts[parts.length - 1];
            const module = parts.slice(0, -1).join('::');
            return { module, importedItems: [item] };
        }
    }

    private extractFunctionCallsFromBody(body: string): string[] {
        const calls: string[] = [];
        
        // Match function calls: identifier(
        const callPattern = /(\w+)\s*\(/g;
        
        let match;
        while ((match = callPattern.exec(body)) !== null) {
            calls.push(match[1]);
        }
        
        return [...new Set(calls)];
    }

    private findFunctionCallsInFunction(content: string, functionName: string): string[] {
        const functionPattern = new RegExp(`fn\\s+${functionName}\\s*\\([^)]*\\)(?:\\s*->\\s*[^{]+)?\\s*\\{`, 'g');
        const match = functionPattern.exec(content);
        
        if (!match) {return [];}
        
        const functionBody = this.extractRustFunctionBody(content, match.index + match[0].length);
        return this.extractFunctionCallsFromBody(functionBody);
    }

    private calculateComplexity(body: string): number {
        let complexity = 1; // Base complexity
        
        const complexityPatterns = [
            /\bif\b/g,
            /\belse\s+if\b/g,
            /\bwhile\b/g,
            /\bfor\b/g,
            /\bmatch\b/g,
            /=>/g, // Match arms
            /\bloop\b/g,
            /&&/g,
            /\|\|/g
        ];
        
        for (const pattern of complexityPatterns) {
            const matches = body.match(pattern);
            if (matches) {
                complexity += matches.length;
            }
        }
        
        return complexity;
    }

    private extractRustDocumentation(content: string, position: number): string | undefined {
        // Look for /// comments before the position
        const beforeContent = content.substring(0, position);
        const lines = beforeContent.split('\n');
        
        const docLines: string[] = [];
        
        // Go backwards from the current line
        for (let i = lines.length - 1; i >= 0; i--) {
            const line = lines[i].trim();
            
            if (line.startsWith('///')) {
                docLines.unshift(line.substring(3).trim());
            } else if (line === '' || line.startsWith('//')) {
                // Skip empty lines and regular comments
                continue;
            } else {
                // Stop at non-doc content
                break;
            }
        }
        
        return docLines.length > 0 ? docLines.join('\n') : undefined;
    }
}
