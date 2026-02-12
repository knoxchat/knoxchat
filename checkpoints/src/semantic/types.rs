//! Types for semantic analysis

// use chrono::{DateTime, Utc}; // Will be used when implementing timestamp fields
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Complete semantic understanding of code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticContext {
    /// Function definitions and signatures
    pub functions: HashMap<String, FunctionDefinition>,
    /// Class definitions and hierarchies
    pub classes: HashMap<String, ClassDefinition>,
    /// Interface definitions
    pub interfaces: HashMap<String, InterfaceDefinition>,
    /// Type definitions
    pub types: HashMap<String, TypeDefinition>,
    /// Variable definitions
    pub variables: HashMap<String, VariableDefinition>,
    /// Constant definitions
    pub constants: HashMap<String, ConstantDefinition>,
    /// Module definitions
    pub modules: HashMap<String, ModuleDefinition>,
    /// Import statements
    pub imports: Vec<ImportStatement>,
    /// Export statements
    pub exports: Vec<ExportStatement>,
    /// Call chains and relationships
    pub call_chains: Vec<CallChain>,
    /// Inheritance hierarchies
    pub inheritance_tree: InheritanceTree,
    /// Dependency graph
    pub dependency_graph: DependencyGraph,
    /// Usage patterns
    pub usage_patterns: Vec<UsagePattern>,
}

/// Analysis of what the code change was trying to accomplish
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentAnalysis {
    /// Primary intent of the change
    pub change_intent: ChangeIntent,
    /// Features or components affected
    pub affected_features: Vec<String>,
    /// Design patterns used or modified
    pub design_patterns_used: Vec<DetectedPattern>,
    /// Architectural decisions made
    pub architectural_decisions: Vec<ArchitecturalDecision>,
    /// Type of refactoring performed
    pub refactoring_type: Option<RefactoringType>,
    /// Confidence in the analysis
    pub confidence: f64,
}

/// Types of change intents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeIntent {
    /// Adding new functionality
    FeatureAddition { feature_name: String, scope: String },
    /// Fixing a bug or issue
    BugFix {
        issue_description: String,
        affected_components: Vec<String>,
    },
    /// Refactoring existing code
    Refactoring {
        refactoring_pattern: String,
        reason: String,
    },
    /// Performance optimization
    Optimization {
        target_metric: String,
        expected_improvement: String,
    },
    /// Security improvement
    SecurityEnhancement {
        vulnerability_type: String,
        mitigation: String,
    },
    /// Code cleanup or maintenance
    Maintenance { maintenance_type: String },
    /// Configuration or setup changes
    Configuration {
        config_type: String,
        purpose: String,
    },
    /// Documentation updates
    Documentation { doc_type: String },
    /// Testing additions or improvements
    Testing {
        test_type: String,
        coverage_area: String,
    },
}

/// Impact on system architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalImpact {
    /// Layers affected by the change
    pub layers_affected: Vec<ArchitecturalLayer>,
    /// New patterns introduced
    pub patterns_introduced: Vec<DetectedPattern>,
    /// Patterns modified or removed
    pub patterns_modified: Vec<DetectedPattern>,
    /// Dependencies added or changed
    pub dependency_changes: Vec<DependencyChange>,
    /// Impact on system boundaries
    pub boundary_changes: Vec<BoundaryChange>,
    /// Overall architectural significance
    pub significance: ArchitecturalSignificance,
}

/// System architectural layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitecturalLayer {
    Presentation,
    Application,
    Domain,
    Infrastructure,
    Database,
    External,
    Configuration,
    Security,
    Logging,
    Testing,
}

/// Design patterns used in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DesignPattern {
    // Creational patterns
    Singleton,
    Factory,
    Builder,
    Prototype,
    AbstractFactory,

    // Structural patterns
    Adapter,
    Bridge,
    Composite,
    Decorator,
    Facade,
    Flyweight,
    Proxy,

    // Behavioral patterns
    Observer,
    Strategy,
    Command,
    State,
    ChainOfResponsibility,
    Mediator,
    Memento,
    Template,
    Visitor,
    Iterator,

    // Architectural patterns
    MVC,
    MVP,
    MVVM,
    Repository,
    ServiceLayer,
    DomainDrivenDesign,
    EventSourcing,
    CQRS,
    Microservices,
    Layered,
    Hexagonal,

    // Custom pattern
    Custom(String),
}

impl DesignPattern {
    pub fn as_str(&self) -> &str {
        match self {
            DesignPattern::Singleton => "Singleton",
            DesignPattern::Factory => "Factory",
            DesignPattern::Builder => "Builder",
            DesignPattern::Prototype => "Prototype",
            DesignPattern::AbstractFactory => "AbstractFactory",
            DesignPattern::Adapter => "Adapter",
            DesignPattern::Bridge => "Bridge",
            DesignPattern::Composite => "Composite",
            DesignPattern::Decorator => "Decorator",
            DesignPattern::Facade => "Facade",
            DesignPattern::Flyweight => "Flyweight",
            DesignPattern::Proxy => "Proxy",
            DesignPattern::Observer => "Observer",
            DesignPattern::Strategy => "Strategy",
            DesignPattern::Command => "Command",
            DesignPattern::State => "State",
            DesignPattern::ChainOfResponsibility => "ChainOfResponsibility",
            DesignPattern::Mediator => "Mediator",
            DesignPattern::Memento => "Memento",
            DesignPattern::Template => "Template",
            DesignPattern::Visitor => "Visitor",
            DesignPattern::Iterator => "Iterator",
            DesignPattern::MVC => "MVC",
            DesignPattern::MVP => "MVP",
            DesignPattern::MVVM => "MVVM",
            DesignPattern::Repository => "Repository",
            DesignPattern::ServiceLayer => "ServiceLayer",
            DesignPattern::DomainDrivenDesign => "DomainDrivenDesign",
            DesignPattern::EventSourcing => "EventSourcing",
            DesignPattern::CQRS => "CQRS",
            DesignPattern::Microservices => "Microservices",
            DesignPattern::Layered => "Layered",
            DesignPattern::Hexagonal => "Hexagonal",
            DesignPattern::Custom(name) => name,
        }
    }
}

/// Detected design pattern with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub name: String,
    pub pattern: DesignPattern,
    pub confidence: f64,
    pub locations: Vec<String>,
    pub description: Option<String>,
}

/// Architectural decisions made
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalDecision {
    /// Decision made
    pub decision: String,
    /// Reasoning behind the decision
    pub reasoning: String,
    /// Alternatives considered
    pub alternatives: Vec<String>,
    /// Trade-offs involved
    pub tradeoffs: Vec<String>,
    /// Impact assessment
    pub impact: String,
}

/// Types of refactoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefactoringType {
    ExtractMethod,
    ExtractClass,
    InlineMethod,
    MoveMethod,
    RenameVariable,
    RenameMethod,
    RenameClass,
    ExtractInterface,
    PullUpMethod,
    PushDownMethod,
    ReplaceConditionalWithPolymorphism,
    ReplaceInheritanceWithComposition,
    SimplifyConditional,
    RemoveDuplicateCode,
    SplitLargeClass,
    SplitLargeMethod,
    Custom(String),
}

/// Function definition with semantic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub visibility: Visibility,
    pub is_async: bool,
    pub is_static: bool,
    pub documentation: Option<String>,
    pub location: CodeLocation,
    pub calls: Vec<String>,     // Functions this function calls
    pub called_by: Vec<String>, // Functions that call this function
    pub complexity: u32,
    pub lines_of_code: u32,
}

/// Class definition with semantic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDefinition {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub properties: Vec<Property>,
    pub methods: Vec<String>, // Method names
    pub visibility: Visibility,
    pub is_abstract: bool,
    pub is_static: bool,
    pub documentation: Option<String>,
    pub location: CodeLocation,
    pub design_patterns: Vec<DesignPattern>,
}

/// Interface definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceDefinition {
    pub name: String,
    pub extends: Vec<String>,
    pub methods: Vec<MethodSignature>,
    pub properties: Vec<PropertySignature>,
    pub visibility: Visibility,
    pub documentation: Option<String>,
    pub location: CodeLocation,
}

/// Type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDefinition {
    pub name: String,
    pub type_kind: TypeKind,
    pub definition: String,
    pub generic_parameters: Vec<String>,
    pub visibility: Visibility,
    pub documentation: Option<String>,
    pub location: CodeLocation,
}

/// Constant definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantDefinition {
    pub name: String,
    pub value: String,
    pub const_type: String,
    pub visibility: Visibility,
    pub documentation: Option<String>,
    pub location: CodeLocation,
}

/// Variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDefinition {
    pub name: String,
    pub var_type: String,
    pub visibility: Visibility,
    pub is_mutable: bool,
    pub documentation: Option<String>,
    pub location: CodeLocation,
}

/// Module definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDefinition {
    pub name: String,
    pub visibility: Visibility,
    pub documentation: Option<String>,
    pub location: CodeLocation,
    pub submodules: Vec<String>,
    pub exports: Vec<String>,
}

/// Import statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStatement {
    pub module: String,
    pub imported_items: Vec<String>,
    pub alias: Option<String>,
    pub is_default: bool,
    pub is_namespace: bool,
    pub location: CodeLocation,
}

/// Export statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStatement {
    pub exported_items: Vec<String>,
    pub is_default: bool,
    pub alias: Option<String>,
    pub location: CodeLocation,
}

/// Call chain representing function call relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallChain {
    pub caller: String,
    pub called: String,
    pub call_type: CallType,
    pub location: CodeLocation,
    pub is_async: bool,
    pub parameters_passed: Vec<String>,
}

/// Inheritance tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceTree {
    pub root_classes: Vec<String>,
    pub relationships: HashMap<String, Vec<String>>, // parent -> children
    pub depth_map: HashMap<String, u32>,             // class -> depth in tree
}

/// Dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
    pub cycles: Vec<Vec<String>>, // Circular dependencies
    pub external_dependencies: Vec<ExternalDependency>,
}

/// Usage patterns in the code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    pub pattern_type: PatternType,
    pub description: String,
    pub frequency: u32,
    pub locations: Vec<CodeLocation>,
    pub confidence: f64,
}

/// Code relationships and dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRelationships {
    pub direct_dependencies: Vec<String>,
    pub transitive_dependencies: Vec<String>,
    pub dependents: Vec<String>, // Code that depends on this
    pub coupling_strength: HashMap<String, f64>, // file/module -> coupling score
    pub cohesion_metrics: CohesionMetrics,
}

// Supporting types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub is_optional: bool,
    pub default_value: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub prop_type: String,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_readonly: bool,
    pub default_value: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodSignature {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySignature {
    pub name: String,
    pub prop_type: String,
    pub is_optional: bool,
    pub is_readonly: bool,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file_path: PathBuf,
    pub start_line: u32,
    pub end_line: u32,
    pub start_column: u32,
    pub end_column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub id: String,
    pub node_type: DependencyNodeType,
    pub location: Option<CodeLocation>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub edge_type: DependencyType,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDependency {
    pub name: String,
    pub version: Option<String>,
    pub source: DependencySource,
    pub usage_locations: Vec<CodeLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohesionMetrics {
    pub functional_cohesion: f64,
    pub sequential_cohesion: f64,
    pub communicational_cohesion: f64,
    pub procedural_cohesion: f64,
    pub temporal_cohesion: f64,
    pub logical_cohesion: f64,
    pub coincidental_cohesion: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyChange {
    pub change_type: DependencyChangeType,
    pub dependency: String,
    pub impact: String,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryChange {
    pub boundary_type: BoundaryType,
    pub change_description: String,
    pub impact: String,
}

// Enums

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Package,
}

impl Visibility {
    pub fn as_str(&self) -> &str {
        match self {
            Visibility::Public => "public",
            Visibility::Private => "private",
            Visibility::Protected => "protected",
            Visibility::Internal => "internal",
            Visibility::Package => "package",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallType {
    Direct,
    Indirect,
    Virtual,
    Interface,
    Callback,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    Alias,
    Union,
    Intersection,
    Generic,
    Enum,
    Tuple,
    Function,
    Object,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    CreationalPattern,
    StructuralPattern,
    BehavioralPattern,
    ArchitecturalPattern,
    CodeSmell,
    AntiPattern,
    BestPractice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyNodeType {
    File,
    Module,
    Package,
    Class,
    Function,
    Interface,
    Type,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Import,
    Inheritance,
    Composition,
    Aggregation,
    Usage,
    Call,
    Reference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencySource {
    NPM,
    Cargo,
    PyPI,
    Maven,
    NuGet,
    Go,
    Local,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyChangeType {
    Added,
    Removed,
    Updated,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoundaryType {
    ModuleBoundary,
    PackageBoundary,
    ServiceBoundary,
    LayerBoundary,
    ComponentBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArchitecturalSignificance {
    Low,
    Medium,
    High,
    Critical,
}

impl SemanticContext {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            classes: HashMap::new(),
            interfaces: HashMap::new(),
            types: HashMap::new(),
            variables: HashMap::new(),
            constants: HashMap::new(),
            modules: HashMap::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            call_chains: Vec::new(),
            inheritance_tree: InheritanceTree {
                root_classes: Vec::new(),
                relationships: HashMap::new(),
                depth_map: HashMap::new(),
            },
            dependency_graph: DependencyGraph {
                nodes: Vec::new(),
                edges: Vec::new(),
                cycles: Vec::new(),
                external_dependencies: Vec::new(),
            },
            usage_patterns: Vec::new(),
        }
    }

    /// Get definition for a code entity
    pub fn get_definition(&self, entity_name: &str) -> Option<EntityDefinition> {
        if let Some(func) = self.functions.get(entity_name) {
            return Some(EntityDefinition::Function(func.clone()));
        }
        if let Some(class) = self.classes.get(entity_name) {
            return Some(EntityDefinition::Class(class.clone()));
        }
        if let Some(interface) = self.interfaces.get(entity_name) {
            return Some(EntityDefinition::Interface(interface.clone()));
        }
        if let Some(type_def) = self.types.get(entity_name) {
            return Some(EntityDefinition::Type(type_def.clone()));
        }
        if let Some(variable) = self.variables.get(entity_name) {
            return Some(EntityDefinition::Variable(variable.clone()));
        }
        if let Some(constant) = self.constants.get(entity_name) {
            return Some(EntityDefinition::Constant(constant.clone()));
        }
        if let Some(module) = self.modules.get(entity_name) {
            return Some(EntityDefinition::Module(module.clone()));
        }
        None
    }

    /// Update symbols for a specific file
    pub fn update_symbols_for_file(&mut self, file_path: &PathBuf, symbols: Vec<EntityDefinition>) {
        // Remove existing symbols for this file
        self.remove_symbols_for_file(file_path);

        // Add new symbols
        for symbol in symbols {
            match symbol {
                EntityDefinition::Function(func) => {
                    self.functions.insert(func.name.clone(), func);
                }
                EntityDefinition::Class(class) => {
                    self.classes.insert(class.name.clone(), class);
                }
                EntityDefinition::Interface(interface) => {
                    self.interfaces.insert(interface.name.clone(), interface);
                }
                EntityDefinition::Type(type_def) => {
                    self.types.insert(type_def.name.clone(), type_def);
                }
                EntityDefinition::Variable(variable) => {
                    self.variables.insert(variable.name.clone(), variable);
                }
                EntityDefinition::Constant(constant) => {
                    self.constants.insert(constant.name.clone(), constant);
                }
                EntityDefinition::Module(module) => {
                    self.modules.insert(module.name.clone(), module);
                }
            }
        }
    }

    /// Remove symbols for a specific file
    pub fn remove_symbols_for_file(&mut self, file_path: &PathBuf) {
        self.functions
            .retain(|_, func| func.location.file_path != *file_path);
        self.classes
            .retain(|_, class| class.location.file_path != *file_path);
        self.interfaces
            .retain(|_, interface| interface.location.file_path != *file_path);
        self.types
            .retain(|_, type_def| type_def.location.file_path != *file_path);
        self.variables
            .retain(|_, variable| variable.location.file_path != *file_path);
        self.constants
            .retain(|_, constant| constant.location.file_path != *file_path);
        self.modules
            .retain(|_, module| module.location.file_path != *file_path);

        // Also remove imports/exports from this file
        self.imports
            .retain(|import| import.location.file_path != *file_path);
        self.exports
            .retain(|export| export.location.file_path != *file_path);

        // Remove call chains involving this file
        self.call_chains
            .retain(|call| !call.location.file_path.eq(file_path));
    }
}

/// Union type for different kinds of entity definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityDefinition {
    Function(FunctionDefinition),
    Class(ClassDefinition),
    Interface(InterfaceDefinition),
    Type(TypeDefinition),
    Variable(VariableDefinition),
    Constant(ConstantDefinition),
    Module(ModuleDefinition),
}

impl EntityDefinition {
    /// Get the name of the entity
    pub fn name(&self) -> &str {
        match self {
            EntityDefinition::Function(f) => &f.name,
            EntityDefinition::Class(c) => &c.name,
            EntityDefinition::Interface(i) => &i.name,
            EntityDefinition::Type(t) => &t.name,
            EntityDefinition::Variable(v) => &v.name,
            EntityDefinition::Constant(c) => &c.name,
            EntityDefinition::Module(m) => &m.name,
        }
    }

    /// Get the location of the entity
    pub fn location(&self) -> &CodeLocation {
        match self {
            EntityDefinition::Function(f) => &f.location,
            EntityDefinition::Class(c) => &c.location,
            EntityDefinition::Interface(i) => &i.location,
            EntityDefinition::Type(t) => &t.location,
            EntityDefinition::Variable(v) => &v.location,
            EntityDefinition::Constant(c) => &c.location,
            EntityDefinition::Module(m) => &m.location,
        }
    }

    /// Get the visibility of the entity
    pub fn visibility(&self) -> &str {
        match self {
            EntityDefinition::Function(f) => f.visibility.as_str(),
            EntityDefinition::Class(c) => c.visibility.as_str(),
            EntityDefinition::Interface(i) => i.visibility.as_str(),
            EntityDefinition::Type(t) => t.visibility.as_str(),
            EntityDefinition::Variable(v) => v.visibility.as_str(),
            EntityDefinition::Constant(c) => c.visibility.as_str(),
            EntityDefinition::Module(m) => m.visibility.as_str(),
        }
    }

    /// Get the documentation of the entity
    pub fn documentation(&self) -> &Option<String> {
        match self {
            EntityDefinition::Function(f) => &f.documentation,
            EntityDefinition::Class(c) => &c.documentation,
            EntityDefinition::Interface(i) => &i.documentation,
            EntityDefinition::Type(t) => &t.documentation,
            EntityDefinition::Variable(v) => &v.documentation,
            EntityDefinition::Constant(c) => &c.documentation,
            EntityDefinition::Module(m) => &m.documentation,
        }
    }

    /// Get the entity type as a string
    pub fn entity_type_name(&self) -> &str {
        match self {
            EntityDefinition::Function(_) => "function",
            EntityDefinition::Class(_) => "class",
            EntityDefinition::Interface(_) => "interface",
            EntityDefinition::Type(_) => "type",
            EntityDefinition::Variable(_) => "variable",
            EntityDefinition::Constant(_) => "constant",
            EntityDefinition::Module(_) => "module",
        }
    }
}

impl Default for SemanticContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for IntentAnalysis {
    fn default() -> Self {
        Self {
            change_intent: ChangeIntent::Maintenance {
                maintenance_type: "unknown".to_string(),
            },
            affected_features: Vec::new(),
            design_patterns_used: Vec::new(),
            architectural_decisions: Vec::new(),
            refactoring_type: None,
            confidence: 0.5,
        }
    }
}

impl Default for ArchitecturalImpact {
    fn default() -> Self {
        Self {
            layers_affected: Vec::new(),
            patterns_introduced: Vec::new(),
            patterns_modified: Vec::new(),
            dependency_changes: Vec::new(),
            boundary_changes: Vec::new(),
            significance: ArchitecturalSignificance::Low,
        }
    }
}

impl Default for CodeRelationships {
    fn default() -> Self {
        Self {
            direct_dependencies: Vec::new(),
            transitive_dependencies: Vec::new(),
            dependents: Vec::new(),
            coupling_strength: std::collections::HashMap::new(),
            cohesion_metrics: CohesionMetrics::default(),
        }
    }
}

impl Default for CohesionMetrics {
    fn default() -> Self {
        Self {
            functional_cohesion: 0.5,
            sequential_cohesion: 0.5,
            communicational_cohesion: 0.5,
            procedural_cohesion: 0.5,
            temporal_cohesion: 0.5,
            logical_cohesion: 0.5,
            coincidental_cohesion: 0.5,
        }
    }
}
