/**
 * Python AST Parser - Real implementation using regex and pattern matching
 * 
 * This parser provides semantic analysis of Python code,
 * extracting functions, classes, imports, and relationships.
 * Note: For production use, this would ideally use a Python AST library or service.
 */

import { LanguageParser, ParseResult, Symbol, FunctionSymbol, ClassSymbol, ImportSymbol, CallGraph, DependencyInfo, Location, Diagnostic, ParameterInfo, MethodInfo, PropertyInfo } from './LanguageParser';

export class PythonParser implements LanguageParser {
    
    /**
     * Parse Python file and extract semantic information
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
     * Extract all symbols from Python code
     */
    private extractSymbols(content: string, filePath: string, lines: string[]): Symbol[] {
        const symbols: Symbol[] = [];

        // Extract function definitions
        symbols.push(...this.extractFunctions(content, filePath, lines));
        
        // Extract class definitions
        symbols.push(...this.extractClasses(content, filePath, lines));
        
        // Extract imports
        symbols.push(...this.extractImports(content, filePath, lines));
        
        // Extract global variables
        symbols.push(...this.extractGlobalVariables(content, filePath, lines));

        return symbols;
    }

    /**
     * Extract function definitions
     */
    private extractFunctions(content: string, filePath: string, lines: string[]): FunctionSymbol[] {
        const functions: FunctionSymbol[] = [];
        const functionPattern = /^(\s*)def\s+(\w+)\s*\(([^)]*)\)\s*(?:->\s*([^:]+))?\s*:/gm;
        
        let match;
        while ((match = functionPattern.exec(content)) !== null) {
            const indentation = match[1];
            const functionName = match[2];
            const parametersStr = match[3];
            const returnType = match[4] || 'Any';
            
            const lineNumber = content.substring(0, match.index).split('\n').length;
            const isMethod = this.isInsideClass(match.index, content);
            
            // Parse parameters
            const parameters = this.parseParameters(parametersStr);
            
            // Extract function body
            const functionBody = this.extractFunctionBody(lines, lineNumber - 1, indentation.length);
            
            // Extract function calls within the body
            const calls = this.extractFunctionCallsFromBody(functionBody);
            
            // Calculate complexity
            const complexity = this.calculateComplexity(functionBody);
            
            // Extract docstring
            const documentation = this.extractDocstring(functionBody);

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
                isAsync: this.isAsyncFunction(match[0]),
                visibility: this.inferVisibility(functionName),
                location,
                body: functionBody,
                calls,
                complexity,
                isExported: !isMethod && !functionName.startsWith('_'),
                documentation
            });
        }

        return functions;
    }

    /**
     * Extract class definitions
     */
    private extractClasses(content: string, filePath: string, lines: string[]): ClassSymbol[] {
        const classes: ClassSymbol[] = [];
        const classPattern = /^(\s*)class\s+(\w+)(?:\(([^)]*)\))?\s*:/gm;
        
        let match;
        while ((match = classPattern.exec(content)) !== null) {
            const indentation = match[1];
            const className = match[2];
            const inheritanceStr = match[3] || '';
            
            const lineNumber = content.substring(0, match.index).split('\n').length;
            
            // Parse inheritance
            const superClasses = inheritanceStr ? 
                inheritanceStr.split(',').map(s => s.trim()).filter(s => s) : [];
            
            // Extract class body
            const classBody = this.extractClassBody(lines, lineNumber - 1, indentation.length);
            
            // Extract methods and properties
            const methods = this.extractMethodsFromClass(classBody, filePath, lineNumber);
            const properties = this.extractPropertiesFromClass(classBody);
            
            // Extract docstring
            const documentation = this.extractDocstring(classBody);

            const location: Location = {
                file: filePath,
                startLine: lineNumber,
                endLine: lineNumber + classBody.split('\n').length - 1,
                startColumn: indentation.length,
                endColumn: indentation.length + match[0].length
            };

            classes.push({
                type: 'class',
                name: className,
                superClass: superClasses[0],
                interfaces: superClasses.slice(1), // In Python, multiple inheritance
                methods,
                properties,
                visibility: this.inferVisibility(className),
                isAbstract: this.isAbstractClass(classBody),
                location,
                isExported: !className.startsWith('_'),
                documentation
            });
        }

        return classes;
    }

    /**
     * Extract import statements
     */
    private extractImports(content: string, filePath: string, lines: string[]): ImportSymbol[] {
        const imports: ImportSymbol[] = [];
        
        // Match "import module" and "from module import items"
        const importPattern = /^(\s*)(?:from\s+(\S+)\s+)?import\s+([^#\n]+)/gm;
        
        let match;
        while ((match = importPattern.exec(content)) !== null) {
            const fromModule = match[2];
            const importedItems = match[3].split(',').map(item => item.trim());
            
            const lineNumber = content.substring(0, match.index).split('\n').length;
            
            const location: Location = {
                file: filePath,
                startLine: lineNumber,
                endLine: lineNumber,
                startColumn: match[1].length,
                endColumn: match[1].length + match[0].length
            };

            if (fromModule) {
                // from module import items
                imports.push({
                    type: 'import',
                    name: `from ${fromModule}`,
                    module: fromModule,
                    importedItems: importedItems,
                    location
                });
            } else {
                // import module
                for (const item of importedItems) {
                    const [moduleName, alias] = item.split(' as ').map(s => s.trim());
                    imports.push({
                        type: 'import',
                        name: alias || moduleName,
                        module: moduleName,
                        importedItems: [alias || moduleName],
                        location
                    });
                }
            }
        }

        return imports;
    }

    /**
     * Extract global variables
     */
    private extractGlobalVariables(content: string, filePath: string, lines: string[]): Symbol[] {
        const variables: Symbol[] = [];
        
        // Match variable assignments at module level
        const variablePattern = /^([A-Z_][A-Z0-9_]*)\s*=\s*(.+)$/gm;
        
        let match;
        while ((match = variablePattern.exec(content)) !== null) {
            const variableName = match[1];
            const value = match[2];
            
            const lineNumber = content.substring(0, match.index).split('\n').length;
            
            const location: Location = {
                file: filePath,
                startLine: lineNumber,
                endLine: lineNumber,
                startColumn: 0,
                endColumn: match[0].length
            };

            variables.push({
                type: 'variable',
                name: variableName,
                location,
                isExported: !variableName.startsWith('_'),
                variableType: this.inferType(value),
                initializer: value,
                isConst: true // Python constants are by convention uppercase
            } as Symbol);
        }

        return variables;
    }

    /**
     * Build call graph from Python code
     */
    private buildCallGraph(content: string, filePath: string): CallGraph {
        const nodes: any[] = [];
        const edges: any[] = [];

        // Extract all function definitions first
        const functionPattern = /def\s+(\w+)\s*\(/g;
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
        
        const importPattern = /^(?:from\s+(\S+)\s+)?import\s+([^#\n]+)/gm;
        
        let match;
        while ((match = importPattern.exec(content)) !== null) {
            const module = match[1] || match[2].split(',')[0].trim().split(' ')[0];
            
            dependencies.push({
                type: 'import',
                target: module,
                isExternal: !module.startsWith('.'),
                usage: 'import'
            });
        }

        return dependencies;
    }

    // Helper methods

    private parseParameters(parametersStr: string): ParameterInfo[] {
        if (!parametersStr.trim()) {return [];}
        
        return parametersStr.split(',').map(param => {
            const trimmed = param.trim();
            const parts = trimmed.split(':');
            const nameAndDefault = parts[0].trim();
            const type = parts[1] ? parts[1].trim() : 'Any';
            
            const [name, defaultValue] = nameAndDefault.split('=').map(s => s.trim());
            
            return {
                name,
                type,
                optional: !!defaultValue,
                defaultValue
            };
        });
    }

    private extractFunctionBody(lines: string[], startLine: number, baseIndentation: number): string {
        const bodyLines: string[] = [];
        
        for (let i = startLine + 1; i < lines.length; i++) {
            const line = lines[i];
            
            // Empty line or comment
            if (!line.trim() || line.trim().startsWith('#')) {
                bodyLines.push(line);
                continue;
            }
            
            // Check indentation
            const lineIndentation = line.length - line.trimStart().length;
            
            // If indentation is less than or equal to base, we've reached the end
            if (lineIndentation <= baseIndentation && line.trim()) {
                break;
            }
            
            bodyLines.push(line);
        }
        
        return bodyLines.join('\n');
    }

    private extractClassBody(lines: string[], startLine: number, baseIndentation: number): string {
        return this.extractFunctionBody(lines, startLine, baseIndentation);
    }

    private extractMethodsFromClass(classBody: string, filePath: string, classStartLine: number): MethodInfo[] {
        const methods: MethodInfo[] = [];
        const methodPattern = /def\s+(\w+)\s*\(([^)]*)\)\s*(?:->\s*([^:]+))?\s*:/g;
        
        let match;
        while ((match = methodPattern.exec(classBody)) !== null) {
            const methodName = match[1];
            const parametersStr = match[2];
            const returnType = match[3] || 'Any';
            
            const parameters = this.parseParameters(parametersStr);
            
            methods.push({
                name: methodName,
                parameters,
                returnType,
                isStatic: this.isStaticMethod(match[0]),
                isAsync: this.isAsyncFunction(match[0]),
                visibility: this.inferVisibility(methodName)
            });
        }
        
        return methods;
    }

    private extractPropertiesFromClass(classBody: string): PropertyInfo[] {
        const properties: PropertyInfo[] = [];
        
        // Look for self.property = value patterns
        const propertyPattern = /self\.(\w+)\s*=\s*(.+)/g;
        
        let match;
        while ((match = propertyPattern.exec(classBody)) !== null) {
            const propertyName = match[1];
            const value = match[2];
            
            properties.push({
                name: propertyName,
                type: this.inferType(value),
                isStatic: false,
                isReadonly: false,
                visibility: this.inferVisibility(propertyName)
            });
        }
        
        return properties;
    }

    private extractFunctionCallsFromBody(body: string): string[] {
        const calls: string[] = [];
        const callPattern = /(\w+)\s*\(/g;
        
        let match;
        while ((match = callPattern.exec(body)) !== null) {
            calls.push(match[1]);
        }
        
        return [...new Set(calls)];
    }

    private findFunctionCallsInFunction(content: string, functionName: string): string[] {
        const functionPattern = new RegExp(`def\\s+${functionName}\\s*\\([^)]*\\)\\s*:([\\s\\S]*?)(?=^\\S|$)`, 'm');
        const match = functionPattern.exec(content);
        
        if (!match) {return [];}
        
        return this.extractFunctionCallsFromBody(match[1]);
    }

    private calculateComplexity(body: string): number {
        let complexity = 1; // Base complexity
        
        const complexityPatterns = [
            /\bif\b/g,
            /\belif\b/g,
            /\bwhile\b/g,
            /\bfor\b/g,
            /\btry\b/g,
            /\bexcept\b/g,
            /\band\b/g,
            /\bor\b/g
        ];
        
        for (const pattern of complexityPatterns) {
            const matches = body.match(pattern);
            if (matches) {
                complexity += matches.length;
            }
        }
        
        return complexity;
    }

    private extractDocstring(body: string): string | undefined {
        const docstringPattern = /^\s*"""([\s\S]*?)"""/;
        const match = docstringPattern.exec(body);
        return match ? match[1].trim() : undefined;
    }

    private isInsideClass(position: number, content: string): boolean {
        const beforePosition = content.substring(0, position);
        const classPattern = /class\s+\w+(?:\([^)]*\))?\s*:/g;
        const functionPattern = /def\s+\w+\s*\([^)]*\)\s*:/g;
        
        let lastClass = -1;
        let lastFunction = -1;
        
        let match;
        while ((match = classPattern.exec(beforePosition)) !== null) {
            lastClass = match.index;
        }
        
        while ((match = functionPattern.exec(beforePosition)) !== null) {
            lastFunction = match.index;
        }
        
        return lastClass > lastFunction && lastClass !== -1;
    }

    private isAsyncFunction(functionDef: string): boolean {
        return functionDef.includes('async def');
    }

    private isStaticMethod(methodDef: string): boolean {
        // Look for @staticmethod decorator
        return false; // Would need to check preceding lines for decorators
    }

    private isAbstractClass(classBody: string): boolean {
        return classBody.includes('@abstractmethod') || classBody.includes('ABC');
    }

    private inferVisibility(name: string): 'public' | 'private' | 'protected' {
        if (name.startsWith('__')) {return 'private';}
        if (name.startsWith('_')) {return 'protected';}
        return 'public';
    }

    private inferType(value: string): string {
        value = value.trim();
        
        if (value.startsWith('"') || value.startsWith("'")) {return 'str';}
        if (/^\d+$/.test(value)) {return 'int';}
        if (/^\d+\.\d+$/.test(value)) {return 'float';}
        if (value === 'True' || value === 'False') {return 'bool';}
        if (value.startsWith('[')) {return 'list';}
        if (value.startsWith('{')) {return 'dict';}
        if (value.startsWith('(')) {return 'tuple';}
        
        return 'Any';
    }
}
