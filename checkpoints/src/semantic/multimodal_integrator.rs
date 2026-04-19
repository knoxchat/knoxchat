//! Multi-Modal Context Integration
//!
//! This module combines multiple sources of information (code, comments, tests,
//! documentation, commit messages) into unified context for superior understanding.

use super::types::*;
use crate::error::Result;
use crate::types::FileChange;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Multi-modal context integrator
pub struct MultiModalIntegrator {
    config: IntegrationConfig,
    extractors: HashMap<ModalityType, Box<dyn ModalityExtractor + Send + Sync>>,
}

/// Configuration for multi-modal integration
#[derive(Debug, Clone)]
pub struct IntegrationConfig {
    pub enable_comments: bool,
    pub enable_tests: bool,
    pub enable_docs: bool,
    pub enable_commit_messages: bool,
    pub comment_weight: f64,
    pub test_weight: f64,
    pub doc_weight: f64,
    pub commit_weight: f64,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            enable_comments: true,
            enable_tests: true,
            enable_docs: true,
            enable_commit_messages: true,
            comment_weight: 0.3,
            test_weight: 0.25,
            doc_weight: 0.25,
            commit_weight: 0.2,
        }
    }
}

/// Type of modality
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModalityType {
    Code,
    Comments,
    Tests,
    Documentation,
    CommitMessages,
}

/// Unified multi-modal context
#[derive(Debug, Clone)]
pub struct MultiModalContext {
    pub code_context: CodeContext,
    pub comment_context: CommentContext,
    pub test_context: TestContext,
    pub documentation_context: DocumentationContext,
    pub commit_context: CommitContext,
    pub unified_insights: Vec<UnifiedInsight>,
    pub cross_modal_relationships: Vec<CrossModalRelationship>,
}

/// Code-specific context
#[derive(Debug, Clone)]
pub struct CodeContext {
    pub entities: Vec<EntityDefinition>,
    pub complexity_metrics: HashMap<String, f64>,
    pub patterns: Vec<String>,
}

/// Comment-extracted context
#[derive(Debug, Clone)]
pub struct CommentContext {
    pub todos: Vec<TodoComment>,
    pub fixmes: Vec<FixmeComment>,
    pub documentation_comments: Vec<DocComment>,
    pub intent_descriptions: Vec<IntentDescription>,
    pub warnings: Vec<WarningComment>,
}

/// Test-derived context
#[derive(Debug, Clone)]
pub struct TestContext {
    pub test_cases: Vec<TestCase>,
    pub expected_behaviors: Vec<ExpectedBehavior>,
    pub edge_cases: Vec<EdgeCase>,
    pub test_coverage: HashMap<String, f64>,
}

/// Documentation context
#[derive(Debug, Clone)]
pub struct DocumentationContext {
    pub readme_content: Option<String>,
    pub api_docs: Vec<ApiDocumentation>,
    pub architecture_docs: Vec<ArchitectureDoc>,
    pub usage_examples: Vec<UsageExample>,
}

/// Commit message context
#[derive(Debug, Clone)]
pub struct CommitContext {
    pub recent_commits: Vec<CommitInfo>,
    pub architectural_decisions: Vec<ArchitecturalDecision>,
    pub breaking_changes: Vec<BreakingChange>,
    pub feature_descriptions: Vec<FeatureDescription>,
}

/// TODO comment
#[derive(Debug, Clone)]
pub struct TodoComment {
    pub description: String,
    pub location: CodeLocation,
    pub priority: Priority,
    pub assignee: Option<String>,
}

/// FIXME comment
#[derive(Debug, Clone)]
pub struct FixmeComment {
    pub issue_description: String,
    pub location: CodeLocation,
    pub severity: Severity,
}

/// Documentation comment
#[derive(Debug, Clone)]
pub struct DocComment {
    pub content: String,
    pub location: CodeLocation,
    pub associated_entity: Option<String>,
    pub tags: Vec<String>,
}

/// Intent description from comments
#[derive(Debug, Clone)]
pub struct IntentDescription {
    pub intent: String,
    pub context: String,
    pub location: CodeLocation,
}

/// Warning comment
#[derive(Debug, Clone)]
pub struct WarningComment {
    pub warning: String,
    pub location: CodeLocation,
    pub warning_type: WarningType,
}

/// Priority levels
#[derive(Debug, Clone, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Minor,
    Moderate,
    Major,
    Critical,
}

/// Warning types
#[derive(Debug, Clone, PartialEq)]
pub enum WarningType {
    Performance,
    Security,
    Deprecated,
    BestPractice,
    Other,
}

/// Test case information
#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub description: Option<String>,
    pub tested_entity: String,
    pub test_type: TestType,
    pub location: CodeLocation,
}

/// Test type
#[derive(Debug, Clone, PartialEq)]
pub enum TestType {
    Unit,
    Integration,
    E2E,
    Performance,
    Security,
}

/// Expected behavior from tests
#[derive(Debug, Clone)]
pub struct ExpectedBehavior {
    pub entity: String,
    pub behavior_description: String,
    pub inputs: Vec<String>,
    pub expected_output: String,
}

/// Edge case from tests
#[derive(Debug, Clone)]
pub struct EdgeCase {
    pub entity: String,
    pub scenario: String,
    pub expected_handling: String,
}

/// API documentation
#[derive(Debug, Clone)]
pub struct ApiDocumentation {
    pub entity_name: String,
    pub description: String,
    pub parameters: Vec<ParameterDoc>,
    pub returns: Option<String>,
    pub examples: Vec<String>,
}

/// Parameter documentation
#[derive(Debug, Clone)]
pub struct ParameterDoc {
    pub name: String,
    pub type_info: String,
    pub description: String,
    pub required: bool,
}

/// Architecture documentation
#[derive(Debug, Clone)]
pub struct ArchitectureDoc {
    pub title: String,
    pub content: String,
    pub diagrams: Vec<String>,
    pub components: Vec<String>,
}

/// Usage example
#[derive(Debug, Clone)]
pub struct UsageExample {
    pub title: String,
    pub code: String,
    pub description: String,
    pub tags: Vec<String>,
}

/// Commit information
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
    pub files_changed: Vec<String>,
}

/// Architectural decision from commits
#[derive(Debug, Clone)]
pub struct ArchitecturalDecision {
    pub decision: String,
    pub reasoning: String,
    pub alternatives: Vec<String>,
    pub trade_offs: Vec<String>,
    pub timestamp: String,
}

/// Breaking change
#[derive(Debug, Clone)]
pub struct BreakingChange {
    pub description: String,
    pub migration_guide: Option<String>,
    pub affected_apis: Vec<String>,
    pub commit: String,
}

/// Feature description
#[derive(Debug, Clone)]
pub struct FeatureDescription {
    pub name: String,
    pub description: String,
    pub implementation_files: Vec<String>,
    pub related_issues: Vec<String>,
}

/// Unified insight across modalities
#[derive(Debug, Clone)]
pub struct UnifiedInsight {
    pub insight_type: InsightType,
    pub description: String,
    pub sources: Vec<ModalityType>,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

/// Type of unified insight
#[derive(Debug, Clone, PartialEq)]
pub enum InsightType {
    TechnicalDebt,
    ArchitecturalIntent,
    PerformanceConsideration,
    SecurityConcern,
    UsagePattern,
    ImplementationNote,
}

/// Relationship between different modalities
#[derive(Debug, Clone)]
pub struct CrossModalRelationship {
    pub entity: String,
    pub code_location: CodeLocation,
    pub has_tests: bool,
    pub has_documentation: bool,
    pub has_comments: bool,
    pub completeness_score: f64,
}

/// Trait for modality extractors
pub trait ModalityExtractor {
    fn extract(&self, content: &str, file_path: &Path) -> Result<Box<dyn std::any::Any>>;
}

impl MultiModalIntegrator {
    /// Create a new multi-modal integrator
    pub fn new() -> Self {
        let mut integrator = Self {
            config: IntegrationConfig::default(),
            extractors: HashMap::new(),
        };

        // Register default extractors
        integrator.register_extractor(ModalityType::Comments, Box::new(CommentExtractor));
        integrator.register_extractor(ModalityType::Tests, Box::new(TestExtractor));
        integrator.register_extractor(ModalityType::Documentation, Box::new(DocExtractor));

        integrator
    }

    /// Create with custom configuration
    pub fn with_config(config: IntegrationConfig) -> Self {
        let mut integrator = Self::new();
        integrator.config = config;
        integrator
    }

    /// Register a modality extractor
    pub fn register_extractor(
        &mut self,
        modality: ModalityType,
        extractor: Box<dyn ModalityExtractor + Send + Sync>,
    ) {
        self.extractors.insert(modality, extractor);
    }

    /// Build unified multi-modal context
    pub fn build_unified_context(
        &self,
        file_changes: &[FileChange],
        commit_messages: &[String],
    ) -> Result<MultiModalContext> {
        // Extract from each modality
        let code_context = self.extract_code_context(file_changes)?;
        let comment_context = if self.config.enable_comments {
            self.extract_comment_context(file_changes)?
        } else {
            CommentContext {
                todos: Vec::new(),
                fixmes: Vec::new(),
                documentation_comments: Vec::new(),
                intent_descriptions: Vec::new(),
                warnings: Vec::new(),
            }
        };

        let test_context = if self.config.enable_tests {
            self.extract_test_context(file_changes)?
        } else {
            TestContext {
                test_cases: Vec::new(),
                expected_behaviors: Vec::new(),
                edge_cases: Vec::new(),
                test_coverage: HashMap::new(),
            }
        };

        let documentation_context = if self.config.enable_docs {
            self.extract_documentation_context(file_changes)?
        } else {
            DocumentationContext {
                readme_content: None,
                api_docs: Vec::new(),
                architecture_docs: Vec::new(),
                usage_examples: Vec::new(),
            }
        };

        let commit_context = if self.config.enable_commit_messages {
            self.extract_commit_context(commit_messages)?
        } else {
            CommitContext {
                recent_commits: Vec::new(),
                architectural_decisions: Vec::new(),
                breaking_changes: Vec::new(),
                feature_descriptions: Vec::new(),
            }
        };

        // Generate unified insights
        let unified_insights = self.generate_unified_insights(
            &code_context,
            &comment_context,
            &test_context,
            &documentation_context,
            &commit_context,
        )?;

        // Build cross-modal relationships
        let cross_modal_relationships = self.build_cross_modal_relationships(
            &code_context,
            &comment_context,
            &test_context,
            &documentation_context,
        )?;

        Ok(MultiModalContext {
            code_context,
            comment_context,
            test_context,
            documentation_context,
            commit_context,
            unified_insights,
            cross_modal_relationships,
        })
    }

    /// Extract code context
    fn extract_code_context(&self, _file_changes: &[FileChange]) -> Result<CodeContext> {
        Ok(CodeContext {
            entities: Vec::new(), // Would extract from actual parsing
            complexity_metrics: HashMap::new(),
            patterns: Vec::new(),
        })
    }

    /// Extract comment context
    fn extract_comment_context(&self, file_changes: &[FileChange]) -> Result<CommentContext> {
        let mut todos = Vec::new();
        let mut fixmes = Vec::new();
        let mut doc_comments = Vec::new();

        for file_change in file_changes {
            if let Some(content) = &file_change.new_content {
                // Extract TODO comments
                for (line_num, line) in content.lines().enumerate() {
                    if line.contains("TODO") {
                        todos.push(TodoComment {
                            description: line.trim().to_string(),
                            location: CodeLocation {
                                file_path: file_change.path.clone(),
                                start_line: (line_num + 1) as u32,
                                start_column: 1,
                                end_line: (line_num + 1) as u32,
                                end_column: line.len() as u32,
                            },
                            priority: Priority::Medium,
                            assignee: None,
                        });
                    }

                    if line.contains("FIXME") {
                        fixmes.push(FixmeComment {
                            issue_description: line.trim().to_string(),
                            location: CodeLocation {
                                file_path: file_change.path.clone(),
                                start_line: (line_num + 1) as u32,
                                start_column: 1,
                                end_line: (line_num + 1) as u32,
                                end_column: line.len() as u32,
                            },
                            severity: Severity::Moderate,
                        });
                    }

                    // Extract JSDoc/docstring style comments
                    if line.contains("/**") || line.contains("///") || line.contains("\"\"\"") {
                        doc_comments.push(DocComment {
                            content: line.trim().to_string(),
                            location: CodeLocation {
                                file_path: file_change.path.clone(),
                                start_line: (line_num + 1) as u32,
                                start_column: 1,
                                end_line: (line_num + 1) as u32,
                                end_column: line.len() as u32,
                            },
                            associated_entity: None,
                            tags: Vec::new(),
                        });
                    }
                }
            }
        }

        Ok(CommentContext {
            todos,
            fixmes,
            documentation_comments: doc_comments,
            intent_descriptions: Vec::new(),
            warnings: Vec::new(),
        })
    }

    /// Extract test context
    fn extract_test_context(&self, file_changes: &[FileChange]) -> Result<TestContext> {
        let mut test_cases = Vec::new();

        for file_change in file_changes {
            // Check if it's a test file
            if self.is_test_file(file_change.path.to_string_lossy().as_ref()) {
                if let Some(content) = &file_change.new_content {
                    // Extract test cases (simplified)
                    for (line_num, line) in content.lines().enumerate() {
                        if line.contains("test(")
                            || line.contains("it(")
                            || line.contains("def test_")
                        {
                            test_cases.push(TestCase {
                                name: line.trim().to_string(),
                                description: None,
                                tested_entity: "unknown".to_string(),
                                test_type: TestType::Unit,
                                location: CodeLocation {
                                    file_path: file_change.path.clone(),
                                    start_line: (line_num + 1) as u32,
                                    start_column: 1,
                                    end_line: (line_num + 1) as u32,
                                    end_column: line.len() as u32,
                                },
                            });
                        }
                    }
                }
            }
        }

        Ok(TestContext {
            test_cases,
            expected_behaviors: Vec::new(),
            edge_cases: Vec::new(),
            test_coverage: HashMap::new(),
        })
    }

    /// Extract documentation context
    fn extract_documentation_context(
        &self,
        file_changes: &[FileChange],
    ) -> Result<DocumentationContext> {
        let mut readme_content = None;
        let api_docs = Vec::new();

        for file_change in file_changes {
            if file_change
                .path
                .to_string_lossy()
                .to_lowercase()
                .contains("readme")
            {
                readme_content = file_change.new_content.clone();
            }
        }

        Ok(DocumentationContext {
            readme_content,
            api_docs,
            architecture_docs: Vec::new(),
            usage_examples: Vec::new(),
        })
    }

    /// Extract commit context
    fn extract_commit_context(&self, commit_messages: &[String]) -> Result<CommitContext> {
        let mut architectural_decisions = Vec::new();
        let mut breaking_changes = Vec::new();

        for msg in commit_messages {
            // Look for architectural decisions
            if msg.to_lowercase().contains("architecture") || msg.to_lowercase().contains("design")
            {
                architectural_decisions.push(ArchitecturalDecision {
                    decision: msg.clone(),
                    reasoning: "From commit message".to_string(),
                    alternatives: Vec::new(),
                    trade_offs: Vec::new(),
                    timestamp: "unknown".to_string(),
                });
            }

            // Look for breaking changes
            if msg.contains("BREAKING CHANGE") || msg.contains("!:") {
                breaking_changes.push(BreakingChange {
                    description: msg.clone(),
                    migration_guide: None,
                    affected_apis: Vec::new(),
                    commit: "unknown".to_string(),
                });
            }
        }

        Ok(CommitContext {
            recent_commits: Vec::new(),
            architectural_decisions,
            breaking_changes,
            feature_descriptions: Vec::new(),
        })
    }

    /// Generate unified insights from multiple modalities
    fn generate_unified_insights(
        &self,
        _code: &CodeContext,
        comments: &CommentContext,
        _tests: &TestContext,
        _docs: &DocumentationContext,
        commits: &CommitContext,
    ) -> Result<Vec<UnifiedInsight>> {
        let mut insights = Vec::new();

        // Technical debt from FIXMEs
        if !comments.fixmes.is_empty() {
            insights.push(UnifiedInsight {
                insight_type: InsightType::TechnicalDebt,
                description: format!(
                    "Found {} FIXME comments indicating technical debt",
                    comments.fixmes.len()
                ),
                sources: vec![ModalityType::Comments],
                confidence: 0.8,
                evidence: comments
                    .fixmes
                    .iter()
                    .map(|f| f.issue_description.clone())
                    .take(3)
                    .collect(),
            });
        }

        // Architectural intents from commits
        if !commits.architectural_decisions.is_empty() {
            insights.push(UnifiedInsight {
                insight_type: InsightType::ArchitecturalIntent,
                description: format!(
                    "Identified {} architectural decisions from commit history",
                    commits.architectural_decisions.len()
                ),
                sources: vec![ModalityType::CommitMessages],
                confidence: 0.9,
                evidence: commits
                    .architectural_decisions
                    .iter()
                    .map(|d| d.decision.clone())
                    .take(3)
                    .collect(),
            });
        }

        Ok(insights)
    }

    /// Build cross-modal relationships
    fn build_cross_modal_relationships(
        &self,
        _code: &CodeContext,
        comments: &CommentContext,
        tests: &TestContext,
        _docs: &DocumentationContext,
    ) -> Result<Vec<CrossModalRelationship>> {
        let mut relationships = Vec::new();

        // For each code entity, check what other modalities reference it
        // This is simplified - in reality would do sophisticated matching

        // Check if entities have documentation
        let has_docs = !comments.documentation_comments.is_empty();
        let has_tests = !tests.test_cases.is_empty();
        let has_comments = !comments.todos.is_empty() || !comments.fixmes.is_empty();

        if has_docs || has_tests || has_comments {
            relationships.push(CrossModalRelationship {
                entity: "example_entity".to_string(),
                code_location: CodeLocation {
                    file_path: PathBuf::from("unknown"),
                    start_line: 0,
                    start_column: 0,
                    end_line: 0,
                    end_column: 0,
                },
                has_tests,
                has_documentation: has_docs,
                has_comments,
                completeness_score: 0.7,
            });
        }

        Ok(relationships)
    }

    /// Check if a file is a test file
    fn is_test_file(&self, path: &str) -> bool {
        path.contains("test")
            || path.contains("spec")
            || path.ends_with("_test.rs")
            || path.ends_with(".test.ts")
    }
}

impl Default for MultiModalIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

// Default extractors
struct CommentExtractor;
impl ModalityExtractor for CommentExtractor {
    fn extract(&self, _content: &str, _file_path: &Path) -> Result<Box<dyn std::any::Any>> {
        Ok(Box::new(CommentContext {
            todos: Vec::new(),
            fixmes: Vec::new(),
            documentation_comments: Vec::new(),
            intent_descriptions: Vec::new(),
            warnings: Vec::new(),
        }))
    }
}

struct TestExtractor;
impl ModalityExtractor for TestExtractor {
    fn extract(&self, _content: &str, _file_path: &Path) -> Result<Box<dyn std::any::Any>> {
        Ok(Box::new(TestContext {
            test_cases: Vec::new(),
            expected_behaviors: Vec::new(),
            edge_cases: Vec::new(),
            test_coverage: HashMap::new(),
        }))
    }
}

struct DocExtractor;
impl ModalityExtractor for DocExtractor {
    fn extract(&self, _content: &str, _file_path: &Path) -> Result<Box<dyn std::any::Any>> {
        Ok(Box::new(DocumentationContext {
            readme_content: None,
            api_docs: Vec::new(),
            architecture_docs: Vec::new(),
            usage_examples: Vec::new(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrator_creation() {
        let integrator = MultiModalIntegrator::new();
        assert!(integrator.config.enable_comments);
        assert!(integrator.config.enable_tests);
    }

    #[test]
    fn test_is_test_file() {
        let integrator = MultiModalIntegrator::new();
        assert!(integrator.is_test_file("test/unit.test.ts"));
        assert!(integrator.is_test_file("src/component.spec.tsx"));
        assert!(!integrator.is_test_file("src/component.tsx"));
    }
}
