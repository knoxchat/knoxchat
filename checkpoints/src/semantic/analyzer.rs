//! Main semantic analyzer that orchestrates code understanding

use super::ast_parser::{ASTParser, LanguageParser};
use super::relationship_mapper::RelationshipMapper;
use super::symbol_extractor::SymbolExtractor;
use super::types::*;
use crate::error::{CheckpointError, Result};
use crate::types::{ChangeType, FileChange};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Main semantic analyzer that provides complete code understanding
pub struct SemanticAnalyzer {
    _ast_parser: Arc<ASTParser>,
    _symbol_extractor: Arc<SymbolExtractor>,
    relationship_mapper: Arc<RelationshipMapper>,
    language_support: HashMap<String, Box<dyn LanguageParser + Send + Sync>>,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Result<Self> {
        let _ast_parser = Arc::new(ASTParser::new()?);
        let _symbol_extractor = Arc::new(SymbolExtractor::new());
        let relationship_mapper = Arc::new(RelationshipMapper::new());

        let mut language_support: HashMap<String, Box<dyn LanguageParser + Send + Sync>> =
            HashMap::new();

        // Register language parsers
        language_support.insert("typescript".to_string(), Box::new(TypeScriptParser::new()?));
        language_support.insert("javascript".to_string(), Box::new(JavaScriptParser::new()?));
        language_support.insert("rust".to_string(), Box::new(RustParser::new()?));
        language_support.insert("python".to_string(), Box::new(PythonParser::new()?));
        language_support.insert("go".to_string(), Box::new(GoParser::new()?));
        language_support.insert("java".to_string(), Box::new(JavaParser::new()?));

        Ok(Self {
            _ast_parser,
            _symbol_extractor,
            relationship_mapper,
            language_support,
        })
    }

    /// Analyze a complete codebase to build semantic understanding
    pub fn analyze_codebase(&self, file_changes: &[FileChange]) -> Result<SemanticContext> {
        let mut semantic_context = SemanticContext::new();

        // Parse all files and extract symbols
        let mut all_asts = HashMap::new();
        let mut all_symbols = HashMap::new();

        for file_change in file_changes {
            if let Some(content) = &file_change.new_content {
                // Determine language from file extension
                let language = self.detect_language(&file_change.path)?;

                if let Some(parser) = self.language_support.get(&language) {
                    // Parse the file to get AST
                    let ast = parser.parse_file(content, &file_change.path)?;
                    all_asts.insert(file_change.path.clone(), ast.clone());

                    // Extract symbols from AST
                    let symbols = parser.extract_symbols(&ast)?;
                    all_symbols.insert(file_change.path.clone(), symbols.clone());

                    // Add symbols to semantic context
                    self.add_symbols_to_context(&mut semantic_context, symbols)?;

                    // Extract imports and exports
                    let imports = parser.extract_imports(&ast)?;
                    let exports = parser.extract_exports(&ast)?;

                    semantic_context.imports.extend(imports);
                    semantic_context.exports.extend(exports);
                }
            }
        }

        // Build relationships between symbols
        self.build_relationships(&mut semantic_context, &all_asts)?;

        // Analyze usage patterns
        self.analyze_usage_patterns(&mut semantic_context, &all_asts)?;

        // Build dependency graph
        self.build_dependency_graph(&mut semantic_context, file_changes)?;

        Ok(semantic_context)
    }

    /// Analyze the intent behind code changes
    pub fn analyze_intent(
        &self,
        file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<IntentAnalysis> {
        let mut intent_analysis = IntentAnalysis {
            change_intent: self.classify_change_intent(file_changes)?,
            affected_features: self.identify_affected_features(file_changes, semantic_context)?,
            design_patterns_used: self.identify_design_patterns(semantic_context)?,
            architectural_decisions: self
                .extract_architectural_decisions(file_changes, semantic_context)?,
            refactoring_type: self.identify_refactoring_type(file_changes)?,
            confidence: 0.0,
        };

        // Calculate confidence score
        intent_analysis.confidence =
            self.calculate_intent_confidence(&intent_analysis, file_changes)?;

        Ok(intent_analysis)
    }

    /// Analyze architectural impact of changes
    pub fn analyze_architectural_impact(
        &self,
        file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<ArchitecturalImpact> {
        Ok(ArchitecturalImpact {
            layers_affected: self.identify_affected_layers(file_changes, semantic_context)?,
            patterns_introduced: self.identify_new_patterns(file_changes, semantic_context)?,
            patterns_modified: self.identify_modified_patterns(file_changes, semantic_context)?,
            dependency_changes: self.analyze_dependency_changes(file_changes, semantic_context)?,
            boundary_changes: self.analyze_boundary_changes(file_changes, semantic_context)?,
            significance: self.assess_architectural_significance(file_changes, semantic_context)?,
        })
    }

    /// Build code relationships and dependencies
    pub fn build_code_relationships(
        &self,
        file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<CodeRelationships> {
        Ok(CodeRelationships {
            direct_dependencies: self
                .extract_direct_dependencies(file_changes, semantic_context)?,
            transitive_dependencies: self
                .extract_transitive_dependencies(file_changes, semantic_context)?,
            dependents: self.find_dependents(file_changes, semantic_context)?,
            coupling_strength: self.calculate_coupling_strength(file_changes, semantic_context)?,
            cohesion_metrics: self.calculate_cohesion_metrics(file_changes, semantic_context)?,
        })
    }

    // Private helper methods

    fn detect_language(&self, file_path: &Path) -> Result<String> {
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            match extension {
                "ts" | "tsx" => Ok("typescript".to_string()),
                "js" | "jsx" => Ok("javascript".to_string()),
                "rs" => Ok("rust".to_string()),
                "py" => Ok("python".to_string()),
                "go" => Ok("go".to_string()),
                "java" => Ok("java".to_string()),
                _ => Err(CheckpointError::validation(format!(
                    "Unsupported file extension: {}",
                    extension
                ))),
            }
        } else {
            Err(CheckpointError::validation("File has no extension"))
        }
    }

    fn add_symbols_to_context(
        &self,
        context: &mut SemanticContext,
        symbols: Vec<EntityDefinition>,
    ) -> Result<()> {
        for symbol in symbols {
            match symbol {
                EntityDefinition::Function(func) => {
                    context.functions.insert(func.name.clone(), func);
                }
                EntityDefinition::Class(class) => {
                    context.classes.insert(class.name.clone(), class);
                }
                EntityDefinition::Interface(interface) => {
                    context.interfaces.insert(interface.name.clone(), interface);
                }
                EntityDefinition::Type(type_def) => {
                    context.types.insert(type_def.name.clone(), type_def);
                }
                EntityDefinition::Variable(variable) => {
                    context.variables.insert(variable.name.clone(), variable);
                }
                EntityDefinition::Constant(constant) => {
                    context.constants.insert(constant.name.clone(), constant);
                }
                EntityDefinition::Module(module) => {
                    context.modules.insert(module.name.clone(), module);
                }
            }
        }
        Ok(())
    }

    fn build_relationships(
        &self,
        context: &mut SemanticContext,
        asts: &HashMap<std::path::PathBuf, AST>,
    ) -> Result<()> {
        // Build call chains
        for (file_path, ast) in asts {
            let call_chains = self.relationship_mapper.build_call_chains(ast, file_path)?;
            context.call_chains.extend(call_chains);
        }

        // Build inheritance tree
        context.inheritance_tree = self
            .relationship_mapper
            .build_inheritance_tree(&context.classes)?;

        Ok(())
    }

    fn analyze_usage_patterns(
        &self,
        context: &mut SemanticContext,
        asts: &HashMap<std::path::PathBuf, AST>,
    ) -> Result<()> {
        for (file_path, ast) in asts {
            let patterns = self
                .relationship_mapper
                .identify_usage_patterns(ast, file_path)?;
            context.usage_patterns.extend(patterns);
        }
        Ok(())
    }

    fn build_dependency_graph(
        &self,
        context: &mut SemanticContext,
        file_changes: &[FileChange],
    ) -> Result<()> {
        context.dependency_graph = self
            .relationship_mapper
            .build_dependency_graph(file_changes, context)?;
        Ok(())
    }

    fn classify_change_intent(&self, file_changes: &[FileChange]) -> Result<ChangeIntent> {
        // Simple heuristic-based classification
        // In a real implementation, this could use ML or more sophisticated analysis

        let created_count = file_changes
            .iter()
            .filter(|c| matches!(c.change_type, crate::types::ChangeType::Created))
            .count();
        let modified_count = file_changes
            .iter()
            .filter(|c| matches!(c.change_type, crate::types::ChangeType::Modified))
            .count();
        let deleted_count = file_changes
            .iter()
            .filter(|c| matches!(c.change_type, crate::types::ChangeType::Deleted))
            .count();

        if created_count > modified_count && created_count > deleted_count {
            Ok(ChangeIntent::FeatureAddition {
                feature_name: "New functionality".to_string(),
                scope: format!("{} new files", created_count),
            })
        } else if deleted_count > 0 && modified_count > deleted_count {
            Ok(ChangeIntent::Refactoring {
                refactoring_pattern: "Code cleanup".to_string(),
                reason: "Removing unused code and improving structure".to_string(),
            })
        } else {
            Ok(ChangeIntent::Maintenance {
                maintenance_type: "Code updates".to_string(),
            })
        }
    }

    fn identify_affected_features(
        &self,
        file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<Vec<String>> {
        let mut features = Vec::new();

        for change in file_changes {
            let path_str = change.path.to_string_lossy().to_lowercase();

            // Infer feature areas from file path components
            let path_parts: Vec<&str> = path_str.split(['/', '\\']).collect();
            for part in &path_parts {
                match *part {
                    "auth" | "login" | "oauth" | "session" => {
                        features.push("authentication".to_string())
                    }
                    "api" | "routes" | "endpoints" | "handlers" => {
                        features.push("api".to_string())
                    }
                    "db" | "database" | "migrations" | "models" | "schema" => {
                        features.push("database".to_string())
                    }
                    "ui" | "components" | "views" | "pages" | "gui" => {
                        features.push("ui".to_string())
                    }
                    "test" | "tests" | "spec" | "testing" => {
                        features.push("testing".to_string())
                    }
                    "config" | "settings" | "env" => {
                        features.push("configuration".to_string())
                    }
                    "util" | "utils" | "helpers" | "lib" => {
                        features.push("utilities".to_string())
                    }
                    _ => {}
                }
            }

            // Also consider symbols in the changed file
            if let Some(content) = &change.new_content {
                let language = self.detect_language(&change.path).ok();
                if let Some(lang) = language {
                    if let Some(parser) = self.language_support.get(&lang) {
                        if let Ok(ast) = parser.parse_file(content, &change.path) {
                            if let Ok(symbols) = parser.extract_symbols(&ast) {
                                for sym in &symbols {
                                    match sym {
                                        EntityDefinition::Class(c) => {
                                            features.push(c.name.clone());
                                        }
                                        EntityDefinition::Module(m) => {
                                            features.push(m.name.clone());
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Also add any module names present in the semantic context
        for module in semantic_context.modules.keys() {
            features.push(module.clone());
        }

        // Deduplicate
        features.sort();
        features.dedup();
        if features.is_empty() {
            features.push("core".to_string());
        }
        Ok(features)
    }

    fn identify_design_patterns(
        &self,
        semantic_context: &SemanticContext,
    ) -> Result<Vec<DetectedPattern>> {
        let mut patterns = Vec::new();

        // Look for common patterns in classes
        for class in semantic_context.classes.values() {
            for pattern in &class.design_patterns {
                patterns.push(DetectedPattern {
                    name: pattern.as_str().to_string(),
                    pattern: pattern.clone(),
                    confidence: 0.7,
                    locations: vec![class.name.clone()],
                    description: None,
                });
            }
        }

        // TODO: Add more sophisticated pattern detection

        Ok(patterns)
    }

    fn extract_architectural_decisions(
        &self,
        file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<Vec<ArchitecturalDecision>> {
        let mut decisions = Vec::new();

        // Detect introduction of design patterns as architectural decisions
        for class in semantic_context.classes.values() {
            if !class.design_patterns.is_empty() {
                for pattern in &class.design_patterns {
                    decisions.push(ArchitecturalDecision {
                        decision: format!(
                            "Adopted {} pattern in {}",
                            pattern.as_str(),
                            class.name
                        ),
                        reasoning: format!(
                            "Class {} uses the {} design pattern",
                            class.name,
                            pattern.as_str()
                        ),
                        alternatives: Vec::new(),
                        tradeoffs: Vec::new(),
                        impact: "Affects code structure and extensibility".to_string(),
                    });
                }
            }
        }

        // Detect new external dependencies
        let has_new_imports = file_changes.iter().any(|c| {
            matches!(c.change_type, crate::types::ChangeType::Created) && c.new_content.is_some()
        });
        if has_new_imports && !semantic_context.imports.is_empty() {
            let external: Vec<_> = semantic_context
                .imports
                .iter()
                .filter(|i| !i.module.starts_with('.') && !i.module.starts_with('/'))
                .map(|i| i.module.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            if !external.is_empty() {
                decisions.push(ArchitecturalDecision {
                    decision: format!("Added external dependencies: {}", external.join(", ")),
                    reasoning: "New external modules introduced".to_string(),
                    alternatives: vec!["Implement functionality in-house".to_string()],
                    tradeoffs: vec![
                        "Faster development vs. additional dependency".to_string(),
                    ],
                    impact: "Increases external dependency surface".to_string(),
                });
            }
        }

        Ok(decisions)
    }

    fn identify_refactoring_type(
        &self,
        file_changes: &[FileChange],
    ) -> Result<Option<RefactoringType>> {
        // Simple heuristics for refactoring detection
        let has_renames = file_changes
            .iter()
            .any(|c| matches!(c.change_type, crate::types::ChangeType::Renamed { .. }));
        let has_moves = file_changes
            .iter()
            .any(|c| matches!(c.change_type, crate::types::ChangeType::Moved { .. }));

        if has_renames {
            Ok(Some(RefactoringType::RenameVariable)) // Could be more specific
        } else if has_moves {
            Ok(Some(RefactoringType::MoveMethod))
        } else {
            Ok(None)
        }
    }

    fn calculate_intent_confidence(
        &self,
        intent_analysis: &IntentAnalysis,
        file_changes: &[FileChange],
    ) -> Result<f64> {
        let mut confidence: f64 = 0.5; // base confidence

        // More affected features → more information → higher confidence
        if !intent_analysis.affected_features.is_empty() {
            confidence += 0.1;
        }

        // Design patterns provide strong signal
        if !intent_analysis.design_patterns_used.is_empty() {
            confidence += 0.1;
        }

        // Clear intent types boost confidence
        match &intent_analysis.change_intent {
            ChangeIntent::FeatureAddition { .. } => confidence += 0.1,
            ChangeIntent::BugFix { .. } => confidence += 0.15,
            ChangeIntent::Refactoring { .. } => confidence += 0.1,
            _ => {}
        }

        // Very few changes → clearer intent
        if file_changes.len() <= 3 {
            confidence += 0.1;
        }

        // Refactoring type identified → more signal
        if intent_analysis.refactoring_type.is_some() {
            confidence += 0.05;
        }

        Ok(confidence.min(1.0))
    }

    fn identify_affected_layers(
        &self,
        file_changes: &[FileChange],
        _semantic_context: &SemanticContext,
    ) -> Result<Vec<ArchitecturalLayer>> {
        let mut layers = std::collections::HashSet::new();

        for change in file_changes {
            let path = change.path.to_string_lossy().to_lowercase();
            let ext = change
                .path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            // Infer layer from path patterns
            if path.contains("component")
                || path.contains("view")
                || path.contains("page")
                || path.contains("gui")
                || ext == "css"
                || ext == "scss"
                || ext == "html"
                || ext == "tsx"
                || ext == "jsx"
            {
                layers.insert("Presentation");
            }
            if path.contains("service")
                || path.contains("controller")
                || path.contains("handler")
                || path.contains("route")
                || path.contains("api")
            {
                layers.insert("Application");
            }
            if path.contains("model")
                || path.contains("entity")
                || path.contains("domain")
                || path.contains("type")
            {
                layers.insert("Domain");
            }
            if path.contains("db")
                || path.contains("database")
                || path.contains("migration")
                || path.contains("schema")
                || path.contains("sql")
            {
                layers.insert("Database");
            }
            if path.contains("infra")
                || path.contains("storage")
                || path.contains("cache")
                || path.contains("queue")
            {
                layers.insert("Infrastructure");
            }
            if path.contains("config") || path.contains("env") || path.contains("setting") {
                layers.insert("Configuration");
            }
            if path.contains("auth") || path.contains("security") || path.contains("crypto") {
                layers.insert("Security");
            }
            if path.contains("log") || path.contains("monitor") || path.contains("metric") {
                layers.insert("Logging");
            }
            if path.contains("test") || path.contains("spec") || path.contains("mock") {
                layers.insert("Testing");
            }
        }

        let result: Vec<ArchitecturalLayer> = layers
            .into_iter()
            .map(|l| match l {
                "Presentation" => ArchitecturalLayer::Presentation,
                "Application" => ArchitecturalLayer::Application,
                "Domain" => ArchitecturalLayer::Domain,
                "Database" => ArchitecturalLayer::Database,
                "Infrastructure" => ArchitecturalLayer::Infrastructure,
                "Configuration" => ArchitecturalLayer::Configuration,
                "Security" => ArchitecturalLayer::Security,
                "Logging" => ArchitecturalLayer::Logging,
                "Testing" => ArchitecturalLayer::Testing,
                _ => ArchitecturalLayer::Application,
            })
            .collect();

        if result.is_empty() {
            Ok(vec![ArchitecturalLayer::Application])
        } else {
            Ok(result)
        }
    }

    fn identify_new_patterns(
        &self,
        file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<Vec<DetectedPattern>> {
        let mut patterns = Vec::new();

        // Detect Singleton pattern: classes with getInstance method
        for (name, class_def) in &semantic_context.classes {
            let methods_lower: Vec<String> = class_def.methods.iter().map(|m| m.to_lowercase()).collect();
            let has_get_instance = methods_lower.iter().any(|m| {
                m.contains("getinstance") || m.contains("get_instance")
            });
            let has_private_ctor = methods_lower.iter().any(|m| m == "constructor");

            if has_get_instance {
                let mut confidence = 0.4;
                if has_private_ctor { confidence += 0.3; }
                if has_get_instance { confidence += 0.3; }

                patterns.push(DetectedPattern {
                    name: format!("Singleton: {}", name),
                    pattern: DesignPattern::Singleton,
                    confidence,
                    locations: vec![format!("{}:{}", class_def.location.file_path.display(), class_def.location.start_line)],
                    description: Some(format!("Class '{}' implements the Singleton pattern", name)),
                });
            }
        }

        // Detect Factory pattern: functions with "factory", "create", "build" in name
        for (name, func_def) in &semantic_context.functions {
            let lower = name.to_lowercase();
            if lower.contains("factory") || lower.starts_with("create") || lower.starts_with("build") {
                patterns.push(DetectedPattern {
                    name: format!("Factory: {}", name),
                    pattern: DesignPattern::Factory,
                    confidence: 0.7,
                    locations: vec![format!("{}:{}", func_def.location.file_path.display(), func_def.location.start_line)],
                    description: Some(format!("Function '{}' appears to be a factory method", name)),
                });
            }
        }

        // Detect Observer pattern: classes with subscribe/on/emit/addEventListener
        for (name, class_def) in &semantic_context.classes {
            let methods_lower: Vec<String> = class_def.methods.iter().map(|m| m.to_lowercase()).collect();
            let has_subscribe = methods_lower.iter().any(|m| {
                m.contains("subscribe") || m.contains("addeventlistener") || m == "on"
            });
            let has_emit = methods_lower.iter().any(|m| {
                m.contains("emit") || m.contains("notify") || m.contains("publish") || m.contains("dispatch")
            });

            if has_subscribe && has_emit {
                patterns.push(DetectedPattern {
                    name: format!("Observer: {}", name),
                    pattern: DesignPattern::Observer,
                    confidence: 0.8,
                    locations: vec![format!("{}:{}", class_def.location.file_path.display(), class_def.location.start_line)],
                    description: Some(format!("Class '{}' implements the Observer/EventEmitter pattern", name)),
                });
            }
        }

        // Detect new external dependency additions from imports
        for import in &semantic_context.imports {
            let module = &import.module;
            if !module.starts_with('.') && !module.starts_with('/') {
                let is_new_file = file_changes.iter().any(|fc| {
                    fc.change_type == ChangeType::Created
                });
                if is_new_file {
                    patterns.push(DetectedPattern {
                        name: format!("External dependency: {}", module),
                        pattern: DesignPattern::Custom(format!("ExternalDep:{}", module)),
                        confidence: 0.6,
                        locations: vec![module.clone()],
                        description: Some(format!("New external dependency '{}' introduced", module)),
                    });
                }
            }
        }

        Ok(patterns)
    }

    fn identify_modified_patterns(
        &self,
        file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<Vec<DetectedPattern>> {
        let mut patterns = Vec::new();

        for fc in file_changes {
            if fc.change_type != ChangeType::Modified {
                continue;
            }

            let path_str = fc.path.to_string_lossy().to_string();

            for (name, class_def) in &semantic_context.classes {
                if class_def.location.file_path.to_string_lossy() == path_str {
                    let lower = name.to_lowercase();
                    if lower.contains("service") || lower.contains("repository") || lower.contains("controller") {
                        let pattern_type = if lower.contains("service") {
                            DesignPattern::ServiceLayer
                        } else if lower.contains("repository") {
                            DesignPattern::Repository
                        } else {
                            DesignPattern::MVC
                        };

                        patterns.push(DetectedPattern {
                            name: format!("Modified pattern: {}", name),
                            pattern: pattern_type,
                            confidence: 0.6,
                            locations: vec![format!("{}:{}", path_str, class_def.location.start_line)],
                            description: Some(format!("Architectural pattern class '{}' was modified", name)),
                        });
                    }
                }
            }
        }

        Ok(patterns)
    }

    fn analyze_dependency_changes(
        &self,
        file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<Vec<DependencyChange>> {
        let mut changes = Vec::new();

        // Analyze imports in new/modified files to detect dependency additions
        for fc in file_changes {
            if fc.change_type == ChangeType::Deleted {
                // Deleted file = potential dependency removal
                let path_str = fc.path.to_string_lossy().to_string();
                changes.push(DependencyChange {
                    change_type: DependencyChangeType::Removed,
                    dependency: path_str.clone(),
                    impact: "File deleted — dependents may break".to_string(),
                    reasoning: format!("File '{}' was deleted", path_str),
                });
                continue;
            }

            // Check imports in the current semantic context for this file
            for import in &semantic_context.imports {
                let is_external = !import.module.starts_with('.') && !import.module.starts_with('/');

                if fc.change_type == ChangeType::Created {
                    changes.push(DependencyChange {
                        change_type: DependencyChangeType::Added,
                        dependency: import.module.clone(),
                        impact: if is_external {
                            "New external dependency added".to_string()
                        } else {
                            "New internal dependency added".to_string()
                        },
                        reasoning: format!("Import '{}' introduced in new file", import.module),
                    });
                }
            }
        }

        // Check for circular dependencies
        let edges = &semantic_context.dependency_graph.edges;
        for edge in edges {
            // Simple cycle detection: A → B and B → A
            let reverse_exists = edges.iter().any(|e| {
                e.from == edge.to && e.to == edge.from
            });
            if reverse_exists && edge.from < edge.to {
                changes.push(DependencyChange {
                    change_type: DependencyChangeType::Modified,
                    dependency: format!("{} <-> {}", edge.from, edge.to),
                    impact: "Circular dependency detected".to_string(),
                    reasoning: format!("Bidirectional dependency between '{}' and '{}'", edge.from, edge.to),
                });
            }
        }

        Ok(changes)
    }

    fn analyze_boundary_changes(
        &self,
        file_changes: &[FileChange],
        _semantic_context: &SemanticContext,
    ) -> Result<Vec<BoundaryChange>> {
        let mut changes = Vec::new();

        // Detect boundary changes based on file path patterns
        let mut modules_touched: HashMap<String, Vec<&FileChange>> = HashMap::new();
        for fc in file_changes {
            let path_str = fc.path.to_string_lossy().to_string();
            // Extract module from path (first directory component)
            let parts: Vec<&str> = path_str.split('/').collect();
            if parts.len() >= 2 {
                modules_touched
                    .entry(parts[0].to_string())
                    .or_default()
                    .push(fc);
            }
        }

        // If changes span multiple modules, a module boundary may be crossed
        if modules_touched.len() > 1 {
            let module_names: Vec<String> = modules_touched.keys().cloned().collect();
            changes.push(BoundaryChange {
                boundary_type: BoundaryType::ModuleBoundary,
                change_description: format!(
                    "Changes span {} modules: {}",
                    module_names.len(),
                    module_names.join(", ")
                ),
                impact: "Cross-module changes may affect module interfaces".to_string(),
            });
        }

        // Detect layer boundary crossings
        let layers: Vec<&str> = file_changes
            .iter()
            .filter_map(|fc| {
                let p = fc.path.to_string_lossy().to_lowercase();
                if p.contains("controller") || p.contains("view") || p.contains("component") {
                    Some("presentation")
                } else if p.contains("service") || p.contains("usecase") {
                    Some("application")
                } else if p.contains("model") || p.contains("entity") || p.contains("domain") {
                    Some("domain")
                } else if p.contains("repo") || p.contains("database") || p.contains("db") {
                    Some("data")
                } else {
                    None
                }
            })
            .collect();

        let unique_layers: std::collections::HashSet<&&str> = layers.iter().collect();
        if unique_layers.len() > 1 {
            changes.push(BoundaryChange {
                boundary_type: BoundaryType::LayerBoundary,
                change_description: format!(
                    "Changes cross {} architectural layers",
                    unique_layers.len()
                ),
                impact: "Cross-layer changes may violate layered architecture constraints".to_string(),
            });
        }

        Ok(changes)
    }

    fn assess_architectural_significance(
        &self,
        file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<ArchitecturalSignificance> {
        let mut score = 0u32;

        // Number of files changed
        if file_changes.len() > 10 {
            score += 3;
        } else if file_changes.len() > 5 {
            score += 2;
        } else if file_changes.len() > 1 {
            score += 1;
        }

        // Interface / public API changes are more significant
        if !semantic_context.interfaces.is_empty() {
            score += 2;
        }

        // Dependency graph changes
        if !semantic_context.dependency_graph.cycles.is_empty() {
            score += 2; // circular deps are always significant
        }

        // Large amount of new exports → public surface area change
        if semantic_context.exports.len() > 5 {
            score += 1;
        }

        Ok(match score {
            0..=1 => ArchitecturalSignificance::Low,
            2..=3 => ArchitecturalSignificance::Medium,
            4..=5 => ArchitecturalSignificance::High,
            _ => ArchitecturalSignificance::Critical,
        })
    }

    fn extract_direct_dependencies(
        &self,
        _file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<Vec<String>> {
        Ok(semantic_context
            .imports
            .iter()
            .map(|import| import.module.clone())
            .collect())
    }

    fn extract_transitive_dependencies(
        &self,
        _file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<Vec<String>> {
        // BFS from direct imports through the dependency graph
        let direct: std::collections::HashSet<String> = semantic_context
            .imports
            .iter()
            .map(|i| i.module.clone())
            .collect();

        let mut visited: std::collections::HashSet<String> = direct.clone();
        let mut queue: std::collections::VecDeque<String> = direct.into_iter().collect();
        let mut transitive = Vec::new();

        // Build adjacency from the dependency graph edges
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &semantic_context.dependency_graph.edges {
            adj.entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
        }

        while let Some(current) = queue.pop_front() {
            if let Some(deps) = adj.get(&current) {
                for dep in deps {
                    if visited.insert(dep.clone()) {
                        transitive.push(dep.clone());
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        Ok(transitive)
    }

    fn find_dependents(
        &self,
        file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<Vec<String>> {
        // Files that import any of the changed files' exports
        let changed_paths: std::collections::HashSet<String> = file_changes
            .iter()
            .map(|c| c.path.to_string_lossy().to_string())
            .collect();

        let mut dependents = Vec::new();
        for edge in &semantic_context.dependency_graph.edges {
            if changed_paths.contains(&edge.to) && !changed_paths.contains(&edge.from) {
                dependents.push(edge.from.clone());
            }
        }

        dependents.sort();
        dependents.dedup();
        Ok(dependents)
    }

    fn calculate_coupling_strength(
        &self,
        file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<HashMap<String, f64>> {
        let mut coupling: HashMap<String, f64> = HashMap::new();

        let changed_set: std::collections::HashSet<String> = file_changes
            .iter()
            .map(|c| c.path.to_string_lossy().to_string())
            .collect();

        // Count how many edges connect each external module to the changed files
        for edge in &semantic_context.dependency_graph.edges {
            if changed_set.contains(&edge.from) && !changed_set.contains(&edge.to) {
                *coupling.entry(edge.to.clone()).or_insert(0.0) += edge.strength;
            }
        }

        // Normalise to [0, 1] based on the max coupling observed
        let max = coupling.values().cloned().fold(0.0f64, f64::max);
        if max > 0.0 {
            for val in coupling.values_mut() {
                *val /= max;
            }
        }

        Ok(coupling)
    }

    fn calculate_cohesion_metrics(
        &self,
        _file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<CohesionMetrics> {
        // Functional cohesion: ratio of functions that call at least one sibling
        let total_funcs = semantic_context.functions.len() as f64;
        let funcs_with_internal_calls = semantic_context
            .functions
            .values()
            .filter(|f| !f.calls.is_empty())
            .count() as f64;

        let functional = if total_funcs > 0.0 {
            funcs_with_internal_calls / total_funcs
        } else {
            0.0
        };

        // Sequential cohesion: approximate from call chain length
        let avg_chain_len = if !semantic_context.call_chains.is_empty() {
            1.0 / semantic_context.call_chains.len() as f64
        } else {
            0.0
        };
        let sequential = (avg_chain_len * 10.0).min(1.0);

        // Communicational: functions sharing types
        let total_types = semantic_context.types.len() as f64;
        let communicational = if total_types > 0.0 && total_funcs > 0.0 {
            (total_types / total_funcs).min(1.0)
        } else {
            0.0
        };

        Ok(CohesionMetrics {
            functional_cohesion: functional,
            sequential_cohesion: sequential,
            communicational_cohesion: communicational,
            procedural_cohesion: functional * 0.8,
            temporal_cohesion: 0.5,
            logical_cohesion: communicational * 0.7,
            coincidental_cohesion: 0.1,
        })
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to create default SemanticAnalyzer")
    }
}

// Placeholder types and implementations for AST and language parsers

#[derive(Debug, Clone)]
pub struct AST {
    pub root: ASTNode,
    pub file_path: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct ASTNode {
    pub node_type: String,
    pub text: String,
    pub children: Vec<ASTNode>,
    pub location: CodeLocation,
}

// Language parsers backed by tree-sitter
pub struct TypeScriptParser {
    parser: std::sync::Mutex<tree_sitter::Parser>,
}
pub struct JavaScriptParser {
    parser: std::sync::Mutex<tree_sitter::Parser>,
}
pub struct RustParser {
    parser: std::sync::Mutex<tree_sitter::Parser>,
}
pub struct PythonParser {
    parser: std::sync::Mutex<tree_sitter::Parser>,
}
pub struct GoParser {
    parser: std::sync::Mutex<tree_sitter::Parser>,
}
pub struct JavaParser {
    parser: std::sync::Mutex<tree_sitter::Parser>,
}

impl TypeScriptParser {
    pub fn new() -> Result<Self> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .map_err(|e| CheckpointError::generic(format!("Failed to set TS language: {}", e)))?;
        Ok(Self { parser: std::sync::Mutex::new(parser) })
    }
}

impl JavaScriptParser {
    pub fn new() -> Result<Self> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .map_err(|e| CheckpointError::generic(format!("Failed to set JS language: {}", e)))?;
        Ok(Self { parser: std::sync::Mutex::new(parser) })
    }
}

impl RustParser {
    pub fn new() -> Result<Self> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .map_err(|e| CheckpointError::generic(format!("Failed to set Rust language: {}", e)))?;
        Ok(Self { parser: std::sync::Mutex::new(parser) })
    }
}

impl PythonParser {
    pub fn new() -> Result<Self> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|e| CheckpointError::generic(format!("Failed to set Python language: {}", e)))?;
        Ok(Self { parser: std::sync::Mutex::new(parser) })
    }
}

impl GoParser {
    pub fn new() -> Result<Self> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .map_err(|e| CheckpointError::generic(format!("Failed to set Go language: {}", e)))?;
        Ok(Self { parser: std::sync::Mutex::new(parser) })
    }
}

impl JavaParser {
    pub fn new() -> Result<Self> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_java::LANGUAGE.into())
            .map_err(|e| CheckpointError::generic(format!("Failed to set Java language: {}", e)))?;
        Ok(Self { parser: std::sync::Mutex::new(parser) })
    }
}

/// Convert a tree-sitter node into our ASTNode recursively
fn ts_node_to_ast_node(node: tree_sitter::Node, source: &[u8], file_path: &Path) -> ASTNode {
    let text = node.utf8_text(source).unwrap_or("").to_string();
    let mut children = Vec::new();

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            // Only include named children to keep the tree manageable
            if child.is_named() {
                children.push(ts_node_to_ast_node(child, source, file_path));
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    ASTNode {
        node_type: node.kind().to_string(),
        text,
        children,
        location: CodeLocation {
            file_path: file_path.to_path_buf(),
            start_line: node.start_position().row as u32 + 1,
            end_line: node.end_position().row as u32 + 1,
            start_column: node.start_position().column as u32 + 1,
            end_column: node.end_position().column as u32 + 1,
        },
    }
}

/// Helper: find the first named child with a given field name or kind
fn find_child_by_kind<'a>(node: &tree_sitter::Node<'a>, kind: &str) -> Option<tree_sitter::Node<'a>> {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.kind() == kind {
                return Some(child);
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    None
}

/// Helper: get text of the first child matching `kind`
#[allow(dead_code)]
fn child_text<'a>(node: &tree_sitter::Node<'a>, kind: &str, source: &'a [u8]) -> Option<String> {
    find_child_by_kind(node, kind).map(|n| n.utf8_text(source).unwrap_or("").to_string())
}

/// Extract symbols from a tree-sitter tree for TypeScript/JavaScript
fn extract_symbols_ts_js(
    root: &tree_sitter::Node,
    source: &[u8],
    file_path: &Path,
) -> Vec<EntityDefinition> {
    let mut symbols = Vec::new();
    let mut cursor = root.walk();

    fn walk_ts_js(
        cursor: &mut tree_sitter::TreeCursor,
        source: &[u8],
        file_path: &Path,
        symbols: &mut Vec<EntityDefinition>,
    ) {
        let node = cursor.node();
        let kind = node.kind();

        match kind {
            "function_declaration" | "method_definition" | "arrow_function" => {
                let name = node
                    .child_by_field_name("name")
                    .map(|n| n.utf8_text(source).unwrap_or("").to_string())
                    .unwrap_or_else(|| "<anonymous>".to_string());

                let is_async = find_child_by_kind(&node, "async").is_some();

                let mut params = Vec::new();
                if let Some(param_list) = node.child_by_field_name("parameters") {
                    let mut pc = param_list.walk();
                    if pc.goto_first_child() {
                        loop {
                            let p = pc.node();
                            if p.is_named() && p.kind() != "(" && p.kind() != ")" && p.kind() != "," {
                                let pname = p.child_by_field_name("pattern")
                                    .or_else(|| p.child_by_field_name("name"))
                                    .map(|n| n.utf8_text(source).unwrap_or("").to_string())
                                    .unwrap_or_else(|| p.utf8_text(source).unwrap_or("").to_string());
                                let ptype = p.child_by_field_name("type")
                                    .map(|n| n.utf8_text(source).unwrap_or("").to_string())
                                    .unwrap_or_default();
                                params.push(Parameter {
                                    name: pname,
                                    param_type: ptype,
                                    is_optional: p.kind() == "optional_parameter",
                                    default_value: p.child_by_field_name("value")
                                        .map(|n| n.utf8_text(source).unwrap_or("").to_string()),
                                    documentation: None,
                                });
                            }
                            if !pc.goto_next_sibling() { break; }
                        }
                    }
                }

                let return_type = node.child_by_field_name("return_type")
                    .map(|n| n.utf8_text(source).unwrap_or("").to_string());

                let loc = CodeLocation {
                    file_path: file_path.to_path_buf(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    start_column: node.start_position().column as u32 + 1,
                    end_column: node.end_position().column as u32 + 1,
                };

                let lines = (node.end_position().row - node.start_position().row + 1) as u32;

                symbols.push(EntityDefinition::Function(FunctionDefinition {
                    name,
                    parameters: params,
                    return_type,
                    visibility: Visibility::Public,
                    is_async,
                    is_static: false,
                    documentation: None,
                    location: loc,
                    calls: Vec::new(),
                    called_by: Vec::new(),
                    complexity: 1,
                    lines_of_code: lines,
                }));
            }
            "class_declaration" => {
                let name = node
                    .child_by_field_name("name")
                    .map(|n| n.utf8_text(source).unwrap_or("").to_string())
                    .unwrap_or_default();

                let extends = node
                    .child_by_field_name("superclass")
                    .map(|n| n.utf8_text(source).unwrap_or("").to_string());

                let mut methods = Vec::new();
                if let Some(body) = node.child_by_field_name("body") {
                    let mut bc = body.walk();
                    if bc.goto_first_child() {
                        loop {
                            let child = bc.node();
                            if child.kind() == "method_definition" {
                                if let Some(mname) = child.child_by_field_name("name") {
                                    methods.push(mname.utf8_text(source).unwrap_or("").to_string());
                                }
                            }
                            if !bc.goto_next_sibling() { break; }
                        }
                    }
                }

                let loc = CodeLocation {
                    file_path: file_path.to_path_buf(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    start_column: node.start_position().column as u32 + 1,
                    end_column: node.end_position().column as u32 + 1,
                };

                symbols.push(EntityDefinition::Class(ClassDefinition {
                    name,
                    extends,
                    implements: Vec::new(),
                    properties: Vec::new(),
                    methods,
                    visibility: Visibility::Public,
                    is_abstract: false,
                    is_static: false,
                    documentation: None,
                    location: loc,
                    design_patterns: Vec::new(),
                }));
            }
            "interface_declaration" => {
                let name = node
                    .child_by_field_name("name")
                    .map(|n| n.utf8_text(source).unwrap_or("").to_string())
                    .unwrap_or_default();

                let loc = CodeLocation {
                    file_path: file_path.to_path_buf(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    start_column: node.start_position().column as u32 + 1,
                    end_column: node.end_position().column as u32 + 1,
                };

                symbols.push(EntityDefinition::Interface(InterfaceDefinition {
                    name,
                    extends: Vec::new(),
                    methods: Vec::new(),
                    properties: Vec::new(),
                    visibility: Visibility::Public,
                    documentation: None,
                    location: loc,
                }));
            }
            "type_alias_declaration" => {
                let name = node
                    .child_by_field_name("name")
                    .map(|n| n.utf8_text(source).unwrap_or("").to_string())
                    .unwrap_or_default();

                let definition = node
                    .child_by_field_name("value")
                    .map(|n| n.utf8_text(source).unwrap_or("").to_string())
                    .unwrap_or_default();

                let loc = CodeLocation {
                    file_path: file_path.to_path_buf(),
                    start_line: node.start_position().row as u32 + 1,
                    end_line: node.end_position().row as u32 + 1,
                    start_column: node.start_position().column as u32 + 1,
                    end_column: node.end_position().column as u32 + 1,
                };

                symbols.push(EntityDefinition::Type(TypeDefinition {
                    name,
                    type_kind: TypeKind::Alias,
                    definition,
                    generic_parameters: Vec::new(),
                    visibility: Visibility::Public,
                    documentation: None,
                    location: loc,
                }));
            }
            _ => {}
        }

        // Recurse into children
        if cursor.goto_first_child() {
            loop {
                walk_ts_js(cursor, source, file_path, symbols);
                if !cursor.goto_next_sibling() { break; }
            }
            cursor.goto_parent();
        }
    }

    walk_ts_js(&mut cursor, source, file_path, &mut symbols);
    symbols
}

/// Extract import statements from a TS/JS tree
fn extract_imports_ts_js(
    root: &tree_sitter::Node,
    source: &[u8],
    file_path: &Path,
) -> Vec<ImportStatement> {
    let mut imports = Vec::new();
    let mut cursor = root.walk();

    fn walk_imports(
        cursor: &mut tree_sitter::TreeCursor,
        source: &[u8],
        file_path: &Path,
        imports: &mut Vec<ImportStatement>,
    ) {
        let node = cursor.node();
        if node.kind() == "import_statement" {
            let module = node
                .child_by_field_name("source")
                .map(|n| {
                    let t = n.utf8_text(source).unwrap_or("");
                    t.trim_matches(|c| c == '\'' || c == '"').to_string()
                })
                .unwrap_or_default();

            let mut items = Vec::new();
            let mut is_default = false;

            // Walk children to find imported names
            let mut nc = node.walk();
            if nc.goto_first_child() {
                loop {
                    let child = nc.node();
                    match child.kind() {
                        "import_clause" | "named_imports" | "import_specifier" => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                items.push(name_node.utf8_text(source).unwrap_or("").to_string());
                            } else if child.kind() == "import_clause" {
                                // Default import
                                is_default = true;
                                items.push(child.utf8_text(source).unwrap_or("").to_string());
                            }
                        }
                        "identifier" => {
                            is_default = true;
                            items.push(child.utf8_text(source).unwrap_or("").to_string());
                        }
                        _ => {}
                    }
                    if !nc.goto_next_sibling() { break; }
                }
            }

            let loc = CodeLocation {
                file_path: file_path.to_path_buf(),
                start_line: node.start_position().row as u32 + 1,
                end_line: node.end_position().row as u32 + 1,
                start_column: node.start_position().column as u32 + 1,
                end_column: node.end_position().column as u32 + 1,
            };

            imports.push(ImportStatement {
                module,
                imported_items: items,
                alias: None,
                is_default,
                is_namespace: false,
                location: loc,
            });
        }

        if cursor.goto_first_child() {
            loop {
                walk_imports(cursor, source, file_path, imports);
                if !cursor.goto_next_sibling() { break; }
            }
            cursor.goto_parent();
        }
    }

    walk_imports(&mut cursor, source, file_path, &mut imports);
    imports
}

/// Extract export statements from a TS/JS tree
fn extract_exports_ts_js(
    root: &tree_sitter::Node,
    source: &[u8],
    file_path: &Path,
) -> Vec<ExportStatement> {
    let mut exports = Vec::new();
    let mut cursor = root.walk();

    fn walk_exports(
        cursor: &mut tree_sitter::TreeCursor,
        source: &[u8],
        file_path: &Path,
        exports: &mut Vec<ExportStatement>,
    ) {
        let node = cursor.node();
        if node.kind() == "export_statement" {
            let mut items = Vec::new();
            let is_default = node.utf8_text(source).unwrap_or("").contains("default");

            let mut nc = node.walk();
            if nc.goto_first_child() {
                loop {
                    let child = nc.node();
                    if let Some(name_node) = child.child_by_field_name("name") {
                        items.push(name_node.utf8_text(source).unwrap_or("").to_string());
                    }
                    if !nc.goto_next_sibling() { break; }
                }
            }

            let loc = CodeLocation {
                file_path: file_path.to_path_buf(),
                start_line: node.start_position().row as u32 + 1,
                end_line: node.end_position().row as u32 + 1,
                start_column: node.start_position().column as u32 + 1,
                end_column: node.end_position().column as u32 + 1,
            };

            exports.push(ExportStatement {
                exported_items: items,
                is_default,
                alias: None,
                location: loc,
            });
        }

        if cursor.goto_first_child() {
            loop {
                walk_exports(cursor, source, file_path, exports);
                if !cursor.goto_next_sibling() { break; }
            }
            cursor.goto_parent();
        }
    }

    walk_exports(&mut cursor, source, file_path, &mut exports);
    exports
}

/// Macro for implementing LanguageParser on tree-sitter-backed parsers
macro_rules! impl_ts_parser {
    ($parser:ty, $extract_sym:ident, $extract_imp:ident, $extract_exp:ident) => {
        impl LanguageParser for $parser {
            fn parse_file(&self, content: &str, file_path: &Path) -> Result<AST> {
                let mut parser = self.parser.lock().map_err(|e| {
                    CheckpointError::generic(format!("Parser lock poisoned: {}", e))
                })?;
                let tree = parser.parse(content, None).ok_or_else(|| {
                    CheckpointError::generic("Tree-sitter parse returned None")
                })?;
                let root = ts_node_to_ast_node(tree.root_node(), content.as_bytes(), file_path);
                Ok(AST {
                    root,
                    file_path: file_path.to_path_buf(),
                })
            }

            fn extract_symbols(&self, ast: &AST) -> Result<Vec<EntityDefinition>> {
                // Re-parse to get tree-sitter nodes (our ASTNode is a simplified copy)
                let mut parser = self.parser.lock().map_err(|e| {
                    CheckpointError::generic(format!("Parser lock poisoned: {}", e))
                })?;
                // We use the text stored in the root node for re-parsing
                let source = ast.root.text.as_bytes();
                if let Some(tree) = parser.parse(&ast.root.text, None) {
                    Ok($extract_sym(&tree.root_node(), source, &ast.file_path))
                } else {
                    Ok(Vec::new())
                }
            }

            fn extract_imports(&self, ast: &AST) -> Result<Vec<ImportStatement>> {
                let mut parser = self.parser.lock().map_err(|e| {
                    CheckpointError::generic(format!("Parser lock poisoned: {}", e))
                })?;
                let source = ast.root.text.as_bytes();
                if let Some(tree) = parser.parse(&ast.root.text, None) {
                    Ok($extract_imp(&tree.root_node(), source, &ast.file_path))
                } else {
                    Ok(Vec::new())
                }
            }

            fn extract_exports(&self, ast: &AST) -> Result<Vec<ExportStatement>> {
                let mut parser = self.parser.lock().map_err(|e| {
                    CheckpointError::generic(format!("Parser lock poisoned: {}", e))
                })?;
                let source = ast.root.text.as_bytes();
                if let Some(tree) = parser.parse(&ast.root.text, None) {
                    Ok($extract_exp(&tree.root_node(), source, &ast.file_path))
                } else {
                    Ok(Vec::new())
                }
            }
        }
    };
}

impl_ts_parser!(TypeScriptParser, extract_symbols_ts_js, extract_imports_ts_js, extract_exports_ts_js);
impl_ts_parser!(JavaScriptParser, extract_symbols_ts_js, extract_imports_ts_js, extract_exports_ts_js);

// For Rust, Python, Go, Java — use generic extraction that matches common node kinds
// These can be made more language-specific later

/// Generic symbol extraction that works with any tree-sitter grammar
fn extract_symbols_generic(
    root: &tree_sitter::Node,
    source: &[u8],
    file_path: &Path,
) -> Vec<EntityDefinition> {
    let mut symbols = Vec::new();
    let mut cursor = root.walk();

    fn walk_generic(
        cursor: &mut tree_sitter::TreeCursor,
        source: &[u8],
        file_path: &Path,
        symbols: &mut Vec<EntityDefinition>,
    ) {
        let node = cursor.node();
        let kind = node.kind();

        // Function-like declarations across languages
        let is_function = matches!(
            kind,
            "function_item" | "function_definition" | "method_declaration"
                | "function_declaration" | "method_definition"
        );

        // Class/struct-like declarations
        let is_class = matches!(
            kind,
            "struct_item" | "class_definition" | "class_declaration"
                | "type_declaration"
        );

        // Trait/interface-like declarations
        let is_interface = matches!(
            kind,
            "trait_item" | "interface_declaration" | "interface_type"
        );

        if is_function {
            let name = node
                .child_by_field_name("name")
                .map(|n| n.utf8_text(source).unwrap_or("").to_string())
                .unwrap_or_else(|| "<anonymous>".to_string());

            let loc = CodeLocation {
                file_path: file_path.to_path_buf(),
                start_line: node.start_position().row as u32 + 1,
                end_line: node.end_position().row as u32 + 1,
                start_column: node.start_position().column as u32 + 1,
                end_column: node.end_position().column as u32 + 1,
            };

            let lines = (node.end_position().row - node.start_position().row + 1) as u32;

            symbols.push(EntityDefinition::Function(FunctionDefinition {
                name,
                parameters: Vec::new(),
                return_type: node
                    .child_by_field_name("return_type")
                    .map(|n| n.utf8_text(source).unwrap_or("").to_string()),
                visibility: Visibility::Public,
                is_async: false,
                is_static: false,
                documentation: None,
                location: loc,
                calls: Vec::new(),
                called_by: Vec::new(),
                complexity: 1,
                lines_of_code: lines,
            }));
        } else if is_class {
            let name = node
                .child_by_field_name("name")
                .map(|n| n.utf8_text(source).unwrap_or("").to_string())
                .unwrap_or_default();

            let loc = CodeLocation {
                file_path: file_path.to_path_buf(),
                start_line: node.start_position().row as u32 + 1,
                end_line: node.end_position().row as u32 + 1,
                start_column: node.start_position().column as u32 + 1,
                end_column: node.end_position().column as u32 + 1,
            };

            symbols.push(EntityDefinition::Class(ClassDefinition {
                name,
                extends: None,
                implements: Vec::new(),
                properties: Vec::new(),
                methods: Vec::new(),
                visibility: Visibility::Public,
                is_abstract: false,
                is_static: false,
                documentation: None,
                location: loc,
                design_patterns: Vec::new(),
            }));
        } else if is_interface {
            let name = node
                .child_by_field_name("name")
                .map(|n| n.utf8_text(source).unwrap_or("").to_string())
                .unwrap_or_default();

            let loc = CodeLocation {
                file_path: file_path.to_path_buf(),
                start_line: node.start_position().row as u32 + 1,
                end_line: node.end_position().row as u32 + 1,
                start_column: node.start_position().column as u32 + 1,
                end_column: node.end_position().column as u32 + 1,
            };

            symbols.push(EntityDefinition::Interface(InterfaceDefinition {
                name,
                extends: Vec::new(),
                methods: Vec::new(),
                properties: Vec::new(),
                visibility: Visibility::Public,
                documentation: None,
                location: loc,
            }));
        }

        if cursor.goto_first_child() {
            loop {
                walk_generic(cursor, source, file_path, symbols);
                if !cursor.goto_next_sibling() { break; }
            }
            cursor.goto_parent();
        }
    }

    walk_generic(&mut cursor, source, file_path, &mut symbols);
    symbols
}

fn extract_imports_generic(
    _root: &tree_sitter::Node,
    _source: &[u8],
    _file_path: &Path,
) -> Vec<ImportStatement> {
    // Generic import extraction — can be improved per-language later
    Vec::new()
}

fn extract_exports_generic(
    _root: &tree_sitter::Node,
    _source: &[u8],
    _file_path: &Path,
) -> Vec<ExportStatement> {
    Vec::new()
}

impl_ts_parser!(RustParser, extract_symbols_generic, extract_imports_generic, extract_exports_generic);
impl_ts_parser!(PythonParser, extract_symbols_generic, extract_imports_generic, extract_exports_generic);
impl_ts_parser!(GoParser, extract_symbols_generic, extract_imports_generic, extract_exports_generic);
impl_ts_parser!(JavaParser, extract_symbols_generic, extract_imports_generic, extract_exports_generic);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChangeType, FileEncoding};
    use std::path::PathBuf;

    fn make_file_change(path: &str, content: &str, change_type: ChangeType) -> FileChange {
        FileChange {
            path: PathBuf::from(path),
            change_type,
            original_content: None,
            new_content: Some(content.to_string()),
            size_bytes: content.len() as u64,
            content_hash: String::new(),
            permissions: None,
            modified_at: chrono::Utc::now(),
            encoding: FileEncoding::Utf8,
            compressed: false,
        }
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = SemanticAnalyzer::new();
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_detect_language() {
        let analyzer = SemanticAnalyzer::new().unwrap();
        assert_eq!(
            analyzer.detect_language(Path::new("src/index.ts")).unwrap(),
            "typescript"
        );
        assert_eq!(
            analyzer.detect_language(Path::new("lib/utils.js")).unwrap(),
            "javascript"
        );
        assert_eq!(
            analyzer.detect_language(Path::new("main.rs")).unwrap(),
            "rust"
        );
        assert_eq!(
            analyzer.detect_language(Path::new("app.py")).unwrap(),
            "python"
        );
    }

    #[test]
    fn test_detect_language_unsupported() {
        let analyzer = SemanticAnalyzer::new().unwrap();
        assert!(analyzer.detect_language(Path::new("style.css")).is_err());
    }

    #[test]
    fn test_analyze_typescript_file() {
        let analyzer = SemanticAnalyzer::new().unwrap();
        let code = r#"
import { Request } from 'express';

export interface User {
    name: string;
    age: number;
}

export function greet(user: User): string {
    return `Hello ${user.name}`;
}

export class UserService {
    private users: User[] = [];

    addUser(user: User): void {
        this.users.push(user);
    }

    getUser(name: string): User | undefined {
        return this.users.find(u => u.name === name);
    }
}
"#;
        let changes = vec![make_file_change("src/users.ts", code, ChangeType::Created)];
        let ctx = analyzer.analyze_codebase(&changes).unwrap();

        // Should have extracted functions, classes, interfaces, or imports
        let has_symbols = !ctx.functions.is_empty()
            || !ctx.classes.is_empty()
            || !ctx.interfaces.is_empty()
            || !ctx.imports.is_empty();
        assert!(has_symbols, "Should extract some symbols from TypeScript code");
    }

    #[test]
    fn test_analyze_architectural_impact() {
        let analyzer = SemanticAnalyzer::new().unwrap();
        let changes = vec![
            make_file_change(
                "src/controllers/api.ts",
                "export function handleRequest() {}",
                ChangeType::Modified,
            ),
            make_file_change(
                "src/models/user.ts",
                "export class User {}",
                ChangeType::Modified,
            ),
        ];
        let ctx = analyzer.analyze_codebase(&changes).unwrap();
        let impact = analyzer
            .analyze_architectural_impact(&changes, &ctx)
            .unwrap();

        assert!(!impact.layers_affected.is_empty(), "Should detect affected layers");
    }

    #[test]
    fn test_identify_affected_layers() {
        let analyzer = SemanticAnalyzer::new().unwrap();
        let changes = vec![
            make_file_change("src/controllers/home.ts", "export class Home {}", ChangeType::Modified),
            make_file_change("src/models/data.ts", "export class Data {}", ChangeType::Modified),
            make_file_change("src/views/index.tsx", "export default App;", ChangeType::Modified),
        ];
        let ctx = analyzer.analyze_codebase(&changes).unwrap();
        let layers = analyzer.identify_affected_layers(&changes, &ctx).unwrap();

        assert!(layers.len() >= 2, "Should detect multiple architectural layers");
    }

    #[test]
    fn test_calculate_intent_confidence() {
        let analyzer = SemanticAnalyzer::new().unwrap();
        let changes = vec![make_file_change(
            "src/app.ts",
            "export function run() { console.log('hello'); }",
            ChangeType::Created,
        )];
        let ctx = analyzer.analyze_codebase(&changes).unwrap();
        let intent = analyzer.analyze_intent(&changes, &ctx).unwrap();

        assert!(intent.confidence >= 0.0 && intent.confidence <= 1.0,
            "Confidence should be between 0 and 1, got {}", intent.confidence);
    }

    #[test]
    fn test_analyze_rust_file() {
        let analyzer = SemanticAnalyzer::new().unwrap();
        let code = r#"
use std::collections::HashMap;

pub struct Config {
    pub name: String,
    pub value: i32,
}

pub fn process(config: &Config) -> bool {
    config.value > 0
}
"#;
        let changes = vec![make_file_change("src/config.rs", code, ChangeType::Created)];
        let ctx = analyzer.analyze_codebase(&changes);
        assert!(ctx.is_ok(), "Should parse Rust code without errors");
    }

    #[test]
    fn test_analyze_python_file() {
        let analyzer = SemanticAnalyzer::new().unwrap();
        let code = r#"
import os
from typing import List

class DataProcessor:
    def __init__(self, path: str):
        self.path = path

    def process(self, items: List[str]) -> bool:
        return len(items) > 0

def main():
    dp = DataProcessor("/tmp")
    dp.process(["a", "b"])
"#;
        let changes = vec![make_file_change("app.py", code, ChangeType::Created)];
        let ctx = analyzer.analyze_codebase(&changes);
        assert!(ctx.is_ok(), "Should parse Python code without errors");
    }

    #[test]
    fn test_empty_changes() {
        let analyzer = SemanticAnalyzer::new().unwrap();
        let changes: Vec<FileChange> = vec![];
        let ctx = analyzer.analyze_codebase(&changes).unwrap();
        assert!(ctx.functions.is_empty());
        assert!(ctx.classes.is_empty());
    }
}
