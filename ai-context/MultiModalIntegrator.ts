/**
 * Multi-Modal Context Integrator
 * 
 * Enriches code context with insights from:
 * - Comments (TODO, FIXME, design rationale)
 * - Tests (expected behavior, edge cases, usage examples)
 * - Commit messages (architectural decisions, evolution)
 * - Documentation (README, inline docs)
 */

import * as Types from './types';

export interface EnrichedAIContext extends Types.CompleteAIContext {
    multi_modal_insights: MultiModalInsights;
}

export interface MultiModalInsights {
    comment_insights: CommentInsights;
    test_insights: TestInsights;
    commit_insights?: CommitInsights;
    documentation_insights?: DocumentationInsights;
    enrichment_metadata: EnrichmentMetadata;
}

export interface CommentInsights {
    todos: TodoComment[];
    fixmes: FixmeComment[];
    design_rationale: DesignRationale[];
    known_issues: KnownIssue[];
    deprecations: Deprecation[];
    future_improvements: FutureImprovement[];
    usage_hints: UsageHint[];
    summary: string;
}

export interface TodoComment {
    description: string;
    file_path: string;
    line_number: number;
    priority: 'low' | 'medium' | 'high';
    context: string;
    assigned_to?: string;
}

export interface FixmeComment {
    description: string;
    file_path: string;
    line_number: number;
    severity: 'minor' | 'major' | 'critical';
    context: string;
    related_code: string;
}

export interface DesignRationale {
    decision: string;
    rationale: string;
    file_path: string;
    line_number: number;
    related_pattern?: string;
    alternatives_considered?: string[];
}

export interface KnownIssue {
    description: string;
    file_path: string;
    line_number: number;
    workaround?: string;
    impact: string;
}

export interface Deprecation {
    what: string;
    reason: string;
    replacement: string;
    file_path: string;
    line_number: number;
    removal_version?: string;
}

export interface FutureImprovement {
    description: string;
    benefit: string;
    file_path: string;
    line_number: number;
    priority: 'low' | 'medium' | 'high';
}

export interface UsageHint {
    hint: string;
    applies_to: string;
    file_path: string;
    line_number: number;
    example?: string;
}

export interface TestInsights {
    test_cases: TestCase[];
    expected_behaviors: ExpectedBehavior[];
    edge_cases: EdgeCase[];
    usage_examples: TestUsageExample[];
    coverage_insights: CoverageInsights;
    summary: string;
}

export interface TestCase {
    name: string;
    description: string;
    file_path: string;
    tests_function: string;
    test_type: 'unit' | 'integration' | 'e2e';
    status: 'passing' | 'failing' | 'skipped' | 'unknown';
}

export interface ExpectedBehavior {
    behavior: string;
    tested_by: string[];
    confidence: number;
    source_file: string;
}

export interface EdgeCase {
    scenario: string;
    tested_by: string[];
    expected_outcome: string;
    source_file: string;
}

export interface TestUsageExample {
    example_code: string;
    demonstrates: string;
    file_path: string;
    relevance_score: number;
}

export interface CoverageInsights {
    well_tested_areas: string[];
    untested_areas: string[];
    coverage_percentage?: number;
    recommendations: string[];
}

export interface CommitInsights {
    recent_changes: RecentChange[];
    architectural_decisions: ArchitecturalEvolution[];
    refactoring_patterns: RefactoringPattern[];
    summary: string;
}

export interface RecentChange {
    commit_hash: string;
    message: string;
    date: Date;
    files_changed: string[];
    impact: string;
    intent: string;
}

export interface ArchitecturalEvolution {
    decision: string;
    commit_hash: string;
    date: Date;
    rationale: string;
    impact_files: string[];
}

export interface RefactoringPattern {
    pattern: string;
    occurrences: number;
    files_affected: string[];
    intent: string;
}

export interface DocumentationInsights {
    api_documentation: ApiDocumentation[];
    usage_guides: UsageGuide[];
    architectural_notes: ArchitecturalNote[];
    summary: string;
}

export interface ApiDocumentation {
    api_name: string;
    description: string;
    parameters: string[];
    return_type: string;
    examples: string[];
    source_file: string;
}

export interface UsageGuide {
    title: string;
    content: string;
    code_examples: string[];
    source_file: string;
}

export interface ArchitecturalNote {
    topic: string;
    description: string;
    related_files: string[];
    source_file: string;
}

export interface EnrichmentMetadata {
    comments_analyzed: number;
    tests_analyzed: number;
    commits_analyzed: number;
    documentation_files_analyzed: number;
    enrichment_time_ms: number;
    confidence_boost: number;
}

/**
 * Main Multi-Modal Integrator
 */
export class MultiModalIntegrator {
    /**
     * Enrich context with multi-modal data
     */
    async enrichContext(
        baseContext: Types.CompleteAIContext,
        queryIntent: Types.QueryIntent
    ): Promise<EnrichedAIContext> {
        const startTime = Date.now();

        const commentInsights = await this.analyzeComments(baseContext);
        const testInsights = await this.analyzeTests(baseContext, queryIntent);
        
        // Optional: commit and documentation analysis (can be expensive)
        const commitInsights = undefined; // await this.analyzeCommits(baseContext);
        const documentationInsights = undefined; // await this.parseDocumentation(baseContext);

        const enrichmentMetadata: EnrichmentMetadata = {
            comments_analyzed: this.countComments(baseContext),
            tests_analyzed: this.countTests(baseContext),
            commits_analyzed: 0,
            documentation_files_analyzed: 0,
            enrichment_time_ms: Date.now() - startTime,
            confidence_boost: this.calculateConfidenceBoost(commentInsights, testInsights),
        };

        return {
            ...baseContext,
            multi_modal_insights: {
                comment_insights: commentInsights,
                test_insights: testInsights,
                commit_insights: commitInsights,
                documentation_insights: documentationInsights,
                enrichment_metadata: enrichmentMetadata,
            },
        };
    }

    /**
     * Analyze comments for insights
     */
    private async analyzeComments(context: Types.CompleteAIContext): Promise<CommentInsights> {
        const todos: TodoComment[] = [];
        const fixmes: FixmeComment[] = [];
        const designRationale: DesignRationale[] = [];
        const knownIssues: KnownIssue[] = [];
        const deprecations: Deprecation[] = [];
        const futureImprovements: FutureImprovement[] = [];
        const usageHints: UsageHint[] = [];

        for (const file of context.core_files) {
            const lines = file.complete_content.split('\n');
            
            for (let i = 0; i < lines.length; i++) {
                const line = lines[i];
                const lineNumber = i + 1;

                // Extract TODO comments
                const todoMatch = line.match(/\/\/\s*TODO:?\s*(.+)/i) || line.match(/\/\*\s*TODO:?\s*(.+?)\*\//i);
                if (todoMatch) {
                    todos.push({
                        description: todoMatch[1].trim(),
                        file_path: file.path,
                        line_number: lineNumber,
                        priority: this.determineTodoPriority(todoMatch[1]),
                        context: this.extractContext(lines, i),
                        assigned_to: this.extractAssignee(todoMatch[1]),
                    });
                }

                // Extract FIXME comments
                const fixmeMatch = line.match(/\/\/\s*FIXME:?\s*(.+)/i) || line.match(/\/\*\s*FIXME:?\s*(.+?)\*\//i);
                if (fixmeMatch) {
                    fixmes.push({
                        description: fixmeMatch[1].trim(),
                        file_path: file.path,
                        line_number: lineNumber,
                        severity: this.determineFixmeSeverity(fixmeMatch[1]),
                        context: this.extractContext(lines, i),
                        related_code: this.extractRelatedCode(lines, i),
                    });
                }

                // Extract design rationale (e.g., "// Using singleton pattern for performance")
                const rationaleMatch = line.match(/\/\/\s*(?:Using|Chose|Preferred|Pattern:|Design:)\s*(.+)/i);
                if (rationaleMatch) {
                    designRationale.push({
                        decision: rationaleMatch[1].trim(),
                        rationale: this.extractRationale(lines, i),
                        file_path: file.path,
                        line_number: lineNumber,
                        related_pattern: this.detectPattern(rationaleMatch[1]),
                    });
                }

                // Extract known issues (e.g., "// Known issue: race condition in concurrent access")
                const issueMatch = line.match(/\/\/\s*(?:Known issue|Issue|Bug):\s*(.+)/i);
                if (issueMatch) {
                    knownIssues.push({
                        description: issueMatch[1].trim(),
                        file_path: file.path,
                        line_number: lineNumber,
                        workaround: this.extractWorkaround(lines, i),
                        impact: 'medium',
                    });
                }

                // Extract deprecations (e.g., "@deprecated Use newFunction instead")
                const deprecationMatch = line.match(/@deprecated\s*(.+)/i) || line.match(/\/\/\s*DEPRECATED:?\s*(.+)/i);
                if (deprecationMatch) {
                    deprecations.push({
                        what: this.extractDeprecatedEntity(lines, i),
                        reason: deprecationMatch[1].trim(),
                        replacement: this.extractReplacement(deprecationMatch[1]),
                        file_path: file.path,
                        line_number: lineNumber,
                    });
                }

                // Extract usage hints (e.g., "// Note: Always call cleanup() after use")
                const hintMatch = line.match(/\/\/\s*(?:Note|Hint|Important|Warning):\s*(.+)/i);
                if (hintMatch) {
                    usageHints.push({
                        hint: hintMatch[1].trim(),
                        applies_to: this.extractApplicableEntity(lines, i),
                        file_path: file.path,
                        line_number: lineNumber,
                        example: this.extractHintExample(lines, i),
                    });
                }
            }
        }

        const summary = this.generateCommentInsightsSummary(
            todos,
            fixmes,
            designRationale,
            knownIssues,
            deprecations
        );

        return {
            todos,
            fixmes,
            design_rationale: designRationale,
            known_issues: knownIssues,
            deprecations,
            future_improvements: futureImprovements,
            usage_hints: usageHints,
            summary,
        };
    }

    /**
     * Analyze tests for insights
     */
    private async analyzeTests(
        context: Types.CompleteAIContext,
        queryIntent: Types.QueryIntent
    ): Promise<TestInsights> {
        const testCases: TestCase[] = [];
        const expectedBehaviors: ExpectedBehavior[] = [];
        const edgeCases: EdgeCase[] = [];
        const usageExamples: TestUsageExample[] = [];

        for (const file of context.core_files) {
            // Check if this is a test file
            if (!this.isTestFile(file.path)) {
                continue;
            }

            const lines = file.complete_content.split('\n');
            
            // Parse test cases (Jest/Mocha/Vitest style)
            for (let i = 0; i < lines.length; i++) {
                const line = lines[i];

                // Match: describe('ComponentName', () => { ... })
                // Match: it('should do something', () => { ... })
                // Match: test('should do something', () => { ... })
                const describeMatch = line.match(/describe\s*\(\s*['"`](.+?)['"`]/);
                const itMatch = line.match(/(?:it|test)\s*\(\s*['"`](.+?)['"`]/);

                if (itMatch) {
                    const testName = itMatch[1];
                    const testCode = this.extractTestCode(lines, i);
                    
                    testCases.push({
                        name: testName,
                        description: this.cleanTestDescription(testName),
                        file_path: file.path,
                        tests_function: this.extractTestedFunction(testCode),
                        test_type: this.detectTestType(file.path, testCode),
                        status: 'unknown',
                    });

                    // Extract expected behavior
                    if (testName.match(/should|must|will/i)) {
                        expectedBehaviors.push({
                            behavior: this.extractBehavior(testName),
                            tested_by: [testName],
                            confidence: 0.9,
                            source_file: file.path,
                        });
                    }

                    // Detect edge cases
                    if (testName.match(/edge|boundary|null|undefined|empty|zero|negative|invalid/i)) {
                        edgeCases.push({
                            scenario: this.extractScenario(testName),
                            tested_by: [testName],
                            expected_outcome: this.extractExpectedOutcome(testCode),
                            source_file: file.path,
                        });
                    }

                    // Extract usage examples
                    const usageExample = this.extractUsageExample(testCode);
                    if (usageExample) {
                        usageExamples.push({
                            example_code: usageExample,
                            demonstrates: this.extractDemonstration(testName),
                            file_path: file.path,
                            relevance_score: this.calculateExampleRelevance(usageExample, queryIntent),
                        });
                    }
                }
            }
        }

        const coverageInsights = this.analyzeCoverage(context, testCases);
        const summary = this.generateTestInsightsSummary(testCases, expectedBehaviors, edgeCases);

        return {
            test_cases: testCases,
            expected_behaviors: expectedBehaviors,
            edge_cases: edgeCases,
            usage_examples: usageExamples,
            coverage_insights: coverageInsights,
            summary,
        };
    }

    // Helper methods for comment analysis

    private determineTodoPriority(todoText: string): 'low' | 'medium' | 'high' {
        if (todoText.match(/urgent|asap|critical|important/i)) return 'high';
        if (todoText.match(/soon|needed|required/i)) return 'medium';
        return 'low';
    }

    private determineFixmeSeverity(fixmeText: string): 'minor' | 'major' | 'critical' {
        if (fixmeText.match(/critical|severe|urgent|security|vulnerability/i)) return 'critical';
        if (fixmeText.match(/major|important|bug|error/i)) return 'major';
        return 'minor';
    }

    private extractContext(lines: string[], lineIndex: number): string {
        const start = Math.max(0, lineIndex - 2);
        const end = Math.min(lines.length, lineIndex + 3);
        return lines.slice(start, end).join('\n');
    }

    private extractRelatedCode(lines: string[], lineIndex: number): string {
        // Get the next non-comment line
        for (let i = lineIndex + 1; i < Math.min(lineIndex + 5, lines.length); i++) {
            const line = lines[i].trim();
            if (line && !line.startsWith('//') && !line.startsWith('/*')) {
                return line;
            }
        }
        return '';
    }

    private extractAssignee(todoText: string): string | undefined {
        const match = todoText.match(/@([a-zA-Z0-9_-]+)/);
        return match ? match[1] : undefined;
    }

    private extractRationale(lines: string[], lineIndex: number): string {
        // Look for explanation in surrounding lines
        let rationale = lines[lineIndex];
        for (let i = lineIndex + 1; i < Math.min(lineIndex + 3, lines.length); i++) {
            const line = lines[i].trim();
            if (line.startsWith('//')) {
                rationale += ' ' + line.substring(2).trim();
            } else {
                break;
            }
        }
        return rationale;
    }

    private detectPattern(text: string): string | undefined {
        const patterns = ['singleton', 'factory', 'observer', 'repository', 'mvc', 'mvvm', 'strategy'];
        for (const pattern of patterns) {
            if (text.toLowerCase().includes(pattern)) {
                return pattern;
            }
        }
        return undefined;
    }

    private extractWorkaround(lines: string[], lineIndex: number): string | undefined {
        for (let i = lineIndex + 1; i < Math.min(lineIndex + 3, lines.length); i++) {
            const line = lines[i];
            if (line.match(/workaround|fix|solution/i)) {
                return line.substring(line.indexOf(':') + 1).trim();
            }
        }
        return undefined;
    }

    private extractDeprecatedEntity(lines: string[], lineIndex: number): string {
        // Look for function/class name in next few lines
        for (let i = lineIndex + 1; i < Math.min(lineIndex + 3, lines.length); i++) {
            const line = lines[i];
            const match = line.match(/(?:function|class|const|let|var)\s+([a-zA-Z0-9_]+)/);
            if (match) {
                return match[1];
            }
        }
        return 'unknown';
    }

    private extractReplacement(deprecationText: string): string {
        const match = deprecationText.match(/use\s+([a-zA-Z0-9_]+)/i);
        return match ? match[1] : 'see documentation';
    }

    private extractApplicableEntity(lines: string[], lineIndex: number): string {
        // Look for function/class in surrounding lines
        for (let i = lineIndex - 1; i >= Math.max(0, lineIndex - 5); i--) {
            const line = lines[i];
            const match = line.match(/(?:function|class|const)\s+([a-zA-Z0-9_]+)/);
            if (match) {
                return match[1];
            }
        }
        return 'code';
    }

    private extractHintExample(lines: string[], lineIndex: number): string | undefined {
        // Look for example in comment or code
        for (let i = lineIndex + 1; i < Math.min(lineIndex + 3, lines.length); i++) {
            const line = lines[i];
            if (line.match(/example|e\.g\.|for instance/i)) {
                return line.trim();
            }
        }
        return undefined;
    }

    private generateCommentInsightsSummary(
        todos: TodoComment[],
        fixmes: FixmeComment[],
        designRationale: DesignRationale[],
        knownIssues: KnownIssue[],
        deprecations: Deprecation[]
    ): string {
        const parts: string[] = [];

        if (todos.length > 0) {
            parts.push(`${todos.length} TODO item${todos.length > 1 ? 's' : ''}`);
        }
        if (fixmes.length > 0) {
            const criticalCount = fixmes.filter(f => f.severity === 'critical').length;
            if (criticalCount > 0) {
                parts.push(`${criticalCount} critical FIXME${criticalCount > 1 ? 's' : ''}`);
            } else {
                parts.push(`${fixmes.length} FIXME item${fixmes.length > 1 ? 's' : ''}`);
            }
        }
        if (designRationale.length > 0) {
            parts.push(`${designRationale.length} design decision${designRationale.length > 1 ? 's' : ''} documented`);
        }
        if (knownIssues.length > 0) {
            parts.push(`${knownIssues.length} known issue${knownIssues.length > 1 ? 's' : ''}`);
        }
        if (deprecations.length > 0) {
            parts.push(`${deprecations.length} deprecation${deprecations.length > 1 ? 's' : ''}`);
        }

        if (parts.length === 0) {
            return 'No significant comment insights found';
        }

        return 'Found ' + parts.join(', ') + ' in code comments';
    }

    // Helper methods for test analysis

    private isTestFile(filePath: string): boolean {
        return (
            filePath.includes('.test.') ||
            filePath.includes('.spec.') ||
            filePath.includes('__tests__') ||
            filePath.includes('/test/') ||
            filePath.includes('/tests/')
        );
    }

    private extractTestCode(lines: string[], startIndex: number): string {
        const code: string[] = [];
        let braceCount = 0;
        let started = false;

        for (let i = startIndex; i < Math.min(startIndex + 50, lines.length); i++) {
            const line = lines[i];
            code.push(line);

            // Count braces to find end of test
            for (const char of line) {
                if (char === '{') {
                    braceCount++;
                    started = true;
                } else if (char === '}') {
                    braceCount--;
                }
            }

            if (started && braceCount === 0) {
                break;
            }
        }

        return code.join('\n');
    }

    private extractTestedFunction(testCode: string): string {
        // Look for function calls in test
        const match = testCode.match(/([a-zA-Z0-9_]+)\s*\(/);
        return match ? match[1] : 'unknown';
    }

    private detectTestType(filePath: string, testCode: string): 'unit' | 'integration' | 'e2e' {
        if (filePath.includes('e2e') || testCode.includes('browser') || testCode.includes('page.')) {
            return 'e2e';
        }
        if (testCode.includes('request') || testCode.includes('api') || testCode.match(/\bget\(|post\(|put\(|delete\(/)) {
            return 'integration';
        }
        return 'unit';
    }

    private cleanTestDescription(testName: string): string {
        return testName.replace(/^should\s+/i, '').trim();
    }

    private extractBehavior(testName: string): string {
        return testName.replace(/^should\s+/i, '').replace(/^it\s+/i, '').trim();
    }

    private extractScenario(testName: string): string {
        return testName;
    }

    private extractExpectedOutcome(testCode: string): string {
        // Look for expect() statements
        const match = testCode.match(/expect\(.*?\)\.(toBe|toEqual|toContain|toThrow)\((.*?)\)/);
        if (match) {
            return `${match[1]}: ${match[2]}`;
        }
        return 'see test code';
    }

    private extractUsageExample(testCode: string): string | null {
        // Extract meaningful setup/usage code (skip assertions)
        const lines = testCode.split('\n');
        const usageLines: string[] = [];

        for (const line of lines) {
            const trimmed = line.trim();
            if (
                trimmed &&
                !trimmed.startsWith('//') &&
                !trimmed.startsWith('expect') &&
                !trimmed.startsWith('it(') &&
                !trimmed.startsWith('test(') &&
                !trimmed.startsWith('describe(') &&
                trimmed !== '{' &&
                trimmed !== '}' &&
                trimmed !== '});'
            ) {
                usageLines.push(trimmed);
            }
        }

        return usageLines.length > 0 ? usageLines.join('\n') : null;
    }

    private extractDemonstration(testName: string): string {
        return this.cleanTestDescription(testName);
    }

    private calculateExampleRelevance(example: string, queryIntent: Types.QueryIntent): number {
        let relevance = 0.5;

        // Check if example mentions query entities
        for (const entity of queryIntent.entities) {
            if (example.toLowerCase().includes(entity.name.toLowerCase())) {
                relevance += 0.2;
            }
        }

        return Math.min(relevance, 1.0);
    }

    private analyzeCoverage(
        context: Types.CompleteAIContext,
        testCases: TestCase[]
    ): CoverageInsights {
        const testedFunctions = new Set(testCases.map(tc => tc.tests_function));
        const allFunctions = new Set<string>();

        // Collect all functions from context
        for (const file of context.core_files) {
            for (const func of file.semantic_info.functions) {
                allFunctions.add(func.name);
            }
        }

        const wellTestedAreas: string[] = [];
        const untestedAreas: string[] = [];

        for (const func of Array.from(allFunctions)) {
            if (testedFunctions.has(func)) {
                wellTestedAreas.push(func);
            } else {
                untestedAreas.push(func);
            }
        }

        const coveragePercentage = allFunctions.size > 0
            ? (wellTestedAreas.length / allFunctions.size) * 100
            : 0;

        const recommendations: string[] = [];
        if (coveragePercentage < 50) {
            recommendations.push('Consider adding tests for critical functions');
        }
        if (untestedAreas.length > 0) {
            recommendations.push(`${untestedAreas.length} functions lack test coverage`);
        }

        return {
            well_tested_areas: wellTestedAreas,
            untested_areas: untestedAreas,
            coverage_percentage: Math.round(coveragePercentage),
            recommendations,
        };
    }

    private generateTestInsightsSummary(
        testCases: TestCase[],
        expectedBehaviors: ExpectedBehavior[],
        edgeCases: EdgeCase[]
    ): string {
        if (testCases.length === 0) {
            return 'No test files found in context';
        }

        const parts: string[] = [
            `${testCases.length} test case${testCases.length > 1 ? 's' : ''} found`,
        ];

        if (expectedBehaviors.length > 0) {
            parts.push(`${expectedBehaviors.length} expected behavior${expectedBehaviors.length > 1 ? 's' : ''} documented`);
        }

        if (edgeCases.length > 0) {
            parts.push(`${edgeCases.length} edge case${edgeCases.length > 1 ? 's' : ''} tested`);
        }

        return parts.join(', ');
    }

    // Utility methods

    private countComments(context: Types.CompleteAIContext): number {
        let count = 0;
        for (const file of context.core_files) {
            count += (file.complete_content.match(/\/\//g) || []).length;
            count += (file.complete_content.match(/\/\*/g) || []).length;
        }
        return count;
    }

    private countTests(context: Types.CompleteAIContext): number {
        let count = 0;
        for (const file of context.core_files) {
            if (this.isTestFile(file.path)) {
                count += (file.complete_content.match(/\b(?:it|test)\s*\(/g) || []).length;
            }
        }
        return count;
    }

    private calculateConfidenceBoost(
        commentInsights: CommentInsights,
        testInsights: TestInsights
    ): number {
        let boost = 0;

        // More design rationale = higher confidence
        boost += Math.min(commentInsights.design_rationale.length * 0.02, 0.1);

        // More tests = higher confidence
        boost += Math.min(testInsights.test_cases.length * 0.01, 0.15);

        // Edge cases tested = higher confidence
        boost += Math.min(testInsights.edge_cases.length * 0.02, 0.1);

        return Math.min(boost, 0.35); // Max 35% boost
    }
}

