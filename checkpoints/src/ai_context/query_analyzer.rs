//! Query Analysis Module
//!
//! Analyzes user queries to understand intent, scope, and requirements for context building

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query intent analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryIntent {
    /// Original user query
    pub original_query: String,
    /// Detected query type
    pub query_type: QueryType,
    /// Scope of the query
    pub scope: QueryScope,
    /// Extracted code entities
    pub entities: Vec<CodeEntity>,
    /// Context requirements
    pub context_requirements: Vec<ContextRequirement>,
    /// Priority indicators
    pub priority_indicators: Vec<PriorityIndicator>,
    /// Expected response type
    pub expected_response_type: ResponseType,
    /// Confidence in the analysis
    pub confidence: f64,
    /// Analysis metadata
    pub metadata: QueryAnalysisMetadata,
}

/// Types of queries the system can handle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum QueryType {
    Implementation,
    Debugging,
    Refactoring,
    Explanation,
    Architecture,
    Testing,
    Documentation,
    Performance,
    Security,
    Migration,
    Review,
    Unknown,
}

/// Scope of the query
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryScope {
    Function,
    Class,
    Module,
    Package,
    System,
    Component,
    Interface,
    Configuration,
    Database,
    API,
    Unknown,
}

/// Code entity extracted from query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEntity {
    /// Entity name
    pub name: String,
    /// Type of entity
    pub entity_type: EntityType,
    /// Confidence in detection
    pub confidence: f64,
    /// File location hint (if available)
    pub location: Option<String>,
    /// Context hint for disambiguation
    pub context_hint: Option<String>,
}

/// Types of code entities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    Function,
    Class,
    Interface,
    Type,
    Variable,
    Constant,
    Module,
    Package,
    Component,
    Service,
    File,
    Directory,
    Unknown,
}

/// Context requirement for the query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRequirement {
    /// Type of context needed
    pub requirement_type: ContextType,
    /// Priority level (0.0 to 1.0)
    pub priority: f64,
    /// Reasoning for this requirement
    pub reasoning: String,
    /// Specific scope limitations
    pub scope: Option<Vec<String>>,
}

/// Types of context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContextType {
    SemanticContext,
    ArchitecturalContext,
    EvolutionContext,
    DependencyContext,
    UsageContext,
    ExampleContext,
    TestingContext,
    DocumentationContext,
    PerformanceContext,
    SecurityContext,
}

/// Priority indicator extracted from query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityIndicator {
    /// The indicator text or pattern
    pub indicator: String,
    /// Weight of this indicator
    pub weight: f64,
    /// Reasoning for the weight
    pub reasoning: String,
    /// Category of priority
    pub category: Option<PriorityCategory>,
}

/// Priority categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PriorityCategory {
    Urgency,
    Complexity,
    Impact,
    Specificity,
    Frequency,
}

/// Expected response type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseType {
    CodeImplementation,
    Explanation,
    Debugging,
    Refactoring,
    Architecture,
    Testing,
    Documentation,
    Review,
    Migration,
    Performance,
    Security,
}

/// Query analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAnalysisMetadata {
    /// Time taken for analysis
    pub analysis_duration_ms: u64,
    /// Number of entities detected
    pub entities_detected: usize,
    /// Number of requirements identified
    pub requirements_identified: usize,
    /// Analysis strategy used
    pub analysis_strategy: String,
    /// Language patterns detected
    pub language_patterns: Vec<String>,
}

/// Query analyzer implementation
pub struct QueryAnalyzer {
    /// Pattern matchers for different query types
    query_patterns: HashMap<QueryType, Vec<String>>,
    /// Priority keyword weights
    priority_keywords: HashMap<String, f64>,
    /// Context requirement rules
    context_rules: Vec<ContextRule>,
}

/// Rule for determining context requirements
#[derive(Debug, Clone)]
struct ContextRule {
    /// Query pattern to match
    pattern: String,
    /// Context type to require
    context_type: ContextType,
    /// Priority of this context
    priority: f64,
    /// Reasoning for the rule
    reasoning: String,
}

impl QueryAnalyzer {
    /// Create a new query analyzer
    pub fn new() -> Result<Self> {
        let mut analyzer = Self {
            query_patterns: HashMap::new(),
            priority_keywords: HashMap::new(),
            context_rules: Vec::new(),
        };

        analyzer.initialize_patterns();
        analyzer.initialize_priority_keywords();
        analyzer.initialize_context_rules();

        Ok(analyzer)
    }

    /// Analyze a query to understand its intent
    pub async fn analyze_intent(&self, query: &str) -> Result<QueryIntent> {
        let start_time = std::time::Instant::now();

        // Step 1: Detect query type
        let query_type = self.detect_query_type(query);

        // Step 2: Determine scope
        let scope = self.determine_scope(query);

        // Step 3: Extract code entities
        let entities = self.extract_entities(query);

        // Step 4: Identify context requirements
        let context_requirements = self.identify_context_requirements(query, &query_type, &scope);

        // Step 5: Extract priority indicators
        let priority_indicators = self.extract_priority_indicators(query);

        // Step 6: Determine expected response type
        let expected_response_type = self.determine_response_type(&query_type);

        // Step 7: Calculate confidence
        let confidence = self.calculate_confidence(&query_type, &entities, &context_requirements);

        // Step 8: Detect language patterns
        let language_patterns = self.detect_language_patterns(query);

        let analysis_duration = start_time.elapsed();

        Ok(QueryIntent {
            original_query: query.to_string(),
            query_type,
            scope,
            entities: entities.clone(),
            context_requirements: context_requirements.clone(),
            priority_indicators,
            expected_response_type,
            confidence,
            metadata: QueryAnalysisMetadata {
                analysis_duration_ms: analysis_duration.as_millis() as u64,
                entities_detected: entities.len(),
                requirements_identified: context_requirements.len(),
                analysis_strategy: "pattern_based".to_string(),
                language_patterns,
            },
        })
    }

    /// Detect the type of query
    fn detect_query_type(&self, query: &str) -> QueryType {
        let query_lower = query.to_lowercase();

        // Check each query type pattern
        for (query_type, patterns) in &self.query_patterns {
            for pattern in patterns {
                if query_lower.contains(&pattern.to_lowercase()) {
                    return query_type.clone();
                }
            }
        }

        // Fallback logic based on common patterns
        if query_lower.contains("how to")
            || query_lower.contains("implement")
            || query_lower.contains("create")
        {
            QueryType::Implementation
        } else if query_lower.contains("debug")
            || query_lower.contains("error")
            || query_lower.contains("bug")
        {
            QueryType::Debugging
        } else if query_lower.contains("refactor")
            || query_lower.contains("improve")
            || query_lower.contains("optimize")
        {
            QueryType::Refactoring
        } else if query_lower.contains("explain")
            || query_lower.contains("what is")
            || query_lower.contains("how does")
        {
            QueryType::Explanation
        } else if query_lower.contains("architecture")
            || query_lower.contains("design")
            || query_lower.contains("structure")
        {
            QueryType::Architecture
        } else if query_lower.contains("test")
            || query_lower.contains("unit test")
            || query_lower.contains("testing")
        {
            QueryType::Testing
        } else if query_lower.contains("document")
            || query_lower.contains("comment")
            || query_lower.contains("readme")
        {
            QueryType::Documentation
        } else if query_lower.contains("performance")
            || query_lower.contains("speed")
            || query_lower.contains("slow")
        {
            QueryType::Performance
        } else if query_lower.contains("security")
            || query_lower.contains("vulnerable")
            || query_lower.contains("safe")
        {
            QueryType::Security
        } else {
            QueryType::Unknown
        }
    }

    /// Determine the scope of the query
    fn determine_scope(&self, query: &str) -> QueryScope {
        let query_lower = query.to_lowercase();

        // Check for explicit scope indicators
        if query_lower.contains("function") || query_lower.contains("method") {
            QueryScope::Function
        } else if query_lower.contains("class") || query_lower.contains("struct") {
            QueryScope::Class
        } else if query_lower.contains("module") || query_lower.contains("file") {
            QueryScope::Module
        } else if query_lower.contains("package") || query_lower.contains("library") {
            QueryScope::Package
        } else if query_lower.contains("system")
            || query_lower.contains("application")
            || query_lower.contains("entire")
        {
            QueryScope::System
        } else if query_lower.contains("component") {
            QueryScope::Component
        } else if query_lower.contains("interface") || query_lower.contains("api") {
            QueryScope::Interface
        } else if query_lower.contains("config") || query_lower.contains("setting") {
            QueryScope::Configuration
        } else if query_lower.contains("database") || query_lower.contains("db") {
            QueryScope::Database
        } else {
            // Infer scope from query length and complexity
            let word_count = query.split_whitespace().count();
            if word_count <= 5 {
                QueryScope::Function
            } else if word_count <= 10 {
                QueryScope::Module
            } else {
                QueryScope::System
            }
        }
    }

    /// Extract code entities from the query
    fn extract_entities(&self, query: &str) -> Vec<CodeEntity> {
        let mut entities = Vec::new();

        // Simple pattern-based entity extraction
        // In a real implementation, this would use NLP or more sophisticated parsing

        // Look for camelCase or PascalCase identifiers
        let camel_case_regex =
            regex::Regex::new(r"\b[a-z][a-zA-Z0-9]*[A-Z][a-zA-Z0-9]*\b").unwrap();
        for cap in camel_case_regex.find_iter(query) {
            let entity_name = cap.as_str().to_string();
            entities.push(CodeEntity {
                name: entity_name.clone(),
                entity_type: self.infer_entity_type(&entity_name, query),
                confidence: 0.7,
                location: None,
                context_hint: None,
            });
        }

        // Look for snake_case identifiers
        let snake_case_regex = regex::Regex::new(r"\b[a-z][a-z0-9_]*[a-z0-9]\b").unwrap();
        for cap in snake_case_regex.find_iter(query) {
            let entity_name = cap.as_str().to_string();
            if entity_name.len() > 3 && entity_name.contains('_') {
                entities.push(CodeEntity {
                    name: entity_name.clone(),
                    entity_type: self.infer_entity_type(&entity_name, query),
                    confidence: 0.6,
                    location: None,
                    context_hint: None,
                });
            }
        }

        // Look for file names
        let file_regex =
            regex::Regex::new(r"\b\w+\.(ts|js|rs|py|java|cpp|c|h|hpp|go|php|rb|cs|swift|kt)\b")
                .unwrap();
        for cap in file_regex.find_iter(query) {
            let file_name = cap.as_str().to_string();
            entities.push(CodeEntity {
                name: file_name,
                entity_type: EntityType::File,
                confidence: 0.9,
                location: None,
                context_hint: None,
            });
        }

        // Remove duplicates and sort by confidence
        entities.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entities.dedup_by(|a, b| a.name == b.name);

        entities
    }

    /// Infer the type of an entity based on naming patterns and context
    fn infer_entity_type(&self, entity_name: &str, query: &str) -> EntityType {
        let query_lower = query.to_lowercase();
        let entity_lower = entity_name.to_lowercase();

        // Check for explicit type indicators in query
        if query_lower.contains(&format!("function {}", entity_lower))
            || query_lower.contains(&format!("method {}", entity_lower))
        {
            return EntityType::Function;
        }

        if query_lower.contains(&format!("class {}", entity_lower)) {
            return EntityType::Class;
        }

        if query_lower.contains(&format!("interface {}", entity_lower)) {
            return EntityType::Interface;
        }

        // Infer from naming conventions
        if entity_name.chars().next().unwrap_or('a').is_uppercase() {
            // PascalCase usually indicates classes, interfaces, or types
            if entity_name.ends_with("Interface")
                || entity_name.starts_with("I") && entity_name.len() > 1
            {
                EntityType::Interface
            } else if entity_name.ends_with("Type") || entity_name.ends_with("Config") {
                EntityType::Type
            } else {
                EntityType::Class
            }
        } else {
            // camelCase or snake_case usually indicates functions or variables
            if entity_name.contains('_') {
                EntityType::Function // snake_case often used for functions
            } else {
                EntityType::Variable // camelCase often used for variables
            }
        }
    }

    /// Identify context requirements for the query
    fn identify_context_requirements(
        &self,
        query: &str,
        query_type: &QueryType,
        scope: &QueryScope,
    ) -> Vec<ContextRequirement> {
        let mut requirements = Vec::new();

        // Apply context rules
        for rule in &self.context_rules {
            if query.to_lowercase().contains(&rule.pattern.to_lowercase()) {
                requirements.push(ContextRequirement {
                    requirement_type: rule.context_type.clone(),
                    priority: rule.priority,
                    reasoning: rule.reasoning.clone(),
                    scope: None,
                });
            }
        }

        // Default requirements based on query type and scope
        match query_type {
            QueryType::Implementation => {
                requirements.push(ContextRequirement {
                    requirement_type: ContextType::ExampleContext,
                    priority: 0.8,
                    reasoning: "Implementation queries benefit from examples".to_string(),
                    scope: None,
                });
                requirements.push(ContextRequirement {
                    requirement_type: ContextType::SemanticContext,
                    priority: 0.9,
                    reasoning: "Need to understand existing code structure".to_string(),
                    scope: None,
                });
            }
            QueryType::Debugging => {
                requirements.push(ContextRequirement {
                    requirement_type: ContextType::EvolutionContext,
                    priority: 0.7,
                    reasoning: "Recent changes might be related to the bug".to_string(),
                    scope: None,
                });
                requirements.push(ContextRequirement {
                    requirement_type: ContextType::DependencyContext,
                    priority: 0.8,
                    reasoning: "Dependencies might be causing the issue".to_string(),
                    scope: None,
                });
            }
            QueryType::Architecture => {
                requirements.push(ContextRequirement {
                    requirement_type: ContextType::ArchitecturalContext,
                    priority: 1.0,
                    reasoning: "Architecture queries require architectural context".to_string(),
                    scope: None,
                });
                requirements.push(ContextRequirement {
                    requirement_type: ContextType::DependencyContext,
                    priority: 0.9,
                    reasoning: "Dependencies are crucial for architecture understanding"
                        .to_string(),
                    scope: None,
                });
            }
            QueryType::Performance => {
                requirements.push(ContextRequirement {
                    requirement_type: ContextType::PerformanceContext,
                    priority: 1.0,
                    reasoning: "Performance queries require performance context".to_string(),
                    scope: None,
                });
                requirements.push(ContextRequirement {
                    requirement_type: ContextType::UsageContext,
                    priority: 0.7,
                    reasoning: "Usage patterns affect performance".to_string(),
                    scope: None,
                });
            }
            _ => {
                // Default semantic context for all queries
                requirements.push(ContextRequirement {
                    requirement_type: ContextType::SemanticContext,
                    priority: 0.7,
                    reasoning: "Basic semantic understanding is always helpful".to_string(),
                    scope: None,
                });
            }
        }

        // Adjust priority based on scope
        match scope {
            QueryScope::System => {
                for req in &mut requirements {
                    if req.requirement_type == ContextType::ArchitecturalContext {
                        req.priority = (req.priority * 1.2).min(1.0);
                    }
                }
            }
            QueryScope::Function => {
                for req in &mut requirements {
                    if req.requirement_type == ContextType::ExampleContext {
                        req.priority = (req.priority * 1.1).min(1.0);
                    }
                }
            }
            _ => {}
        }

        // Remove duplicates and sort by priority
        requirements.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        requirements.dedup_by(|a, b| a.requirement_type == b.requirement_type);

        requirements
    }

    /// Extract priority indicators from the query
    fn extract_priority_indicators(&self, query: &str) -> Vec<PriorityIndicator> {
        let mut indicators = Vec::new();
        let query_lower = query.to_lowercase();

        // Check for priority keywords
        for (keyword, weight) in &self.priority_keywords {
            if query_lower.contains(&keyword.to_lowercase()) {
                indicators.push(PriorityIndicator {
                    indicator: keyword.clone(),
                    weight: *weight,
                    reasoning: format!("Keyword '{}' indicates priority", keyword),
                    category: self.categorize_priority_keyword(keyword),
                });
            }
        }

        // Check for urgency indicators
        let urgency_patterns = ["urgent", "asap", "quickly", "immediately", "fast", "now"];
        for pattern in &urgency_patterns {
            if query_lower.contains(pattern) {
                indicators.push(PriorityIndicator {
                    indicator: pattern.to_string(),
                    weight: 0.9,
                    reasoning: "Urgency indicator".to_string(),
                    category: Some(PriorityCategory::Urgency),
                });
            }
        }

        // Check for complexity indicators
        let complexity_patterns = [
            "complex",
            "complicated",
            "difficult",
            "advanced",
            "sophisticated",
        ];
        for pattern in &complexity_patterns {
            if query_lower.contains(pattern) {
                indicators.push(PriorityIndicator {
                    indicator: pattern.to_string(),
                    weight: 0.7,
                    reasoning: "Complexity indicator".to_string(),
                    category: Some(PriorityCategory::Complexity),
                });
            }
        }

        // Check for specificity indicators
        if query.len() > 100 {
            indicators.push(PriorityIndicator {
                indicator: "detailed_query".to_string(),
                weight: 0.8,
                reasoning: "Long, detailed query indicates specific need".to_string(),
                category: Some(PriorityCategory::Specificity),
            });
        }

        indicators
    }

    /// Determine the expected response type
    fn determine_response_type(&self, query_type: &QueryType) -> ResponseType {
        match query_type {
            QueryType::Implementation => ResponseType::CodeImplementation,
            QueryType::Debugging => ResponseType::Debugging,
            QueryType::Refactoring => ResponseType::Refactoring,
            QueryType::Explanation => ResponseType::Explanation,
            QueryType::Architecture => ResponseType::Architecture,
            QueryType::Testing => ResponseType::Testing,
            QueryType::Documentation => ResponseType::Documentation,
            QueryType::Performance => ResponseType::Performance,
            QueryType::Security => ResponseType::Security,
            QueryType::Migration => ResponseType::Migration,
            QueryType::Review => ResponseType::Review,
            QueryType::Unknown => ResponseType::Explanation, // Default to explanation
        }
    }

    /// Calculate confidence in the analysis
    fn calculate_confidence(
        &self,
        query_type: &QueryType,
        entities: &[CodeEntity],
        requirements: &[ContextRequirement],
    ) -> f64 {
        let mut confidence = 0.5; // Base confidence

        // Boost confidence if query type is not unknown
        if *query_type != QueryType::Unknown {
            confidence += 0.2;
        }

        // Boost confidence based on number of entities detected
        confidence += (entities.len() as f64 * 0.1).min(0.2);

        // Boost confidence based on number of requirements identified
        confidence += (requirements.len() as f64 * 0.05).min(0.1);

        // Boost confidence based on entity confidence
        let avg_entity_confidence = if entities.is_empty() {
            0.0
        } else {
            entities.iter().map(|e| e.confidence).sum::<f64>() / entities.len() as f64
        };
        confidence += avg_entity_confidence * 0.1;

        confidence.min(1.0)
    }

    /// Detect language patterns in the query
    fn detect_language_patterns(&self, query: &str) -> Vec<String> {
        let mut patterns = Vec::new();

        // Check for file extensions mentioned
        let file_extensions = [
            "ts", "js", "rs", "py", "java", "cpp", "c", "go", "php", "rb", "cs",
        ];
        for ext in &file_extensions {
            if query.contains(&format!(".{}", ext)) {
                patterns.push(format!("language_{}", ext));
            }
        }

        // Check for language-specific keywords
        let language_keywords = [
            ("typescript", vec!["interface", "type", "namespace", "enum"]),
            (
                "javascript",
                vec!["var", "let", "const", "function", "async"],
            ),
            ("rust", vec!["fn", "struct", "enum", "impl", "trait"]),
            ("python", vec!["def", "class", "import", "from", "__init__"]),
            (
                "java",
                vec!["public", "private", "class", "interface", "extends"],
            ),
        ];

        for (language, keywords) in &language_keywords {
            for keyword in keywords {
                if query.to_lowercase().contains(&keyword.to_lowercase()) {
                    patterns.push(format!("language_{}", language));
                    break;
                }
            }
        }

        patterns.sort();
        patterns.dedup();
        patterns
    }

    /// Categorize priority keywords
    fn categorize_priority_keyword(&self, keyword: &str) -> Option<PriorityCategory> {
        match keyword.to_lowercase().as_str() {
            "urgent" | "asap" | "quickly" | "fast" | "now" => Some(PriorityCategory::Urgency),
            "complex" | "complicated" | "difficult" => Some(PriorityCategory::Complexity),
            "critical" | "important" | "essential" => Some(PriorityCategory::Impact),
            "specific" | "exact" | "precise" => Some(PriorityCategory::Specificity),
            _ => None,
        }
    }

    /// Initialize query type patterns
    fn initialize_patterns(&mut self) {
        // Implementation patterns
        self.query_patterns.insert(
            QueryType::Implementation,
            vec![
                "how to implement".to_string(),
                "create a".to_string(),
                "build a".to_string(),
                "write a".to_string(),
                "implement".to_string(),
                "add".to_string(),
                "generate".to_string(),
            ],
        );

        // Debugging patterns
        self.query_patterns.insert(
            QueryType::Debugging,
            vec![
                "debug".to_string(),
                "error".to_string(),
                "bug".to_string(),
                "fix".to_string(),
                "not working".to_string(),
                "broken".to_string(),
                "issue".to_string(),
            ],
        );

        // Add more patterns for other query types...
    }

    /// Initialize priority keywords
    fn initialize_priority_keywords(&mut self) {
        self.priority_keywords.insert("urgent".to_string(), 0.9);
        self.priority_keywords.insert("important".to_string(), 0.8);
        self.priority_keywords.insert("critical".to_string(), 0.95);
        self.priority_keywords.insert("asap".to_string(), 0.9);
        self.priority_keywords.insert("quickly".to_string(), 0.7);
        self.priority_keywords.insert("fast".to_string(), 0.6);
        // Add more keywords...
    }

    /// Initialize context rules
    fn initialize_context_rules(&mut self) {
        self.context_rules.push(ContextRule {
            pattern: "architecture".to_string(),
            context_type: ContextType::ArchitecturalContext,
            priority: 1.0,
            reasoning: "Architecture keyword requires architectural context".to_string(),
        });

        self.context_rules.push(ContextRule {
            pattern: "example".to_string(),
            context_type: ContextType::ExampleContext,
            priority: 0.8,
            reasoning: "Request for examples".to_string(),
        });

        // Add more context rules...
    }
}
