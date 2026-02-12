//! Semantic analysis module for AI context system
//!
//! This module provides semantic understanding of code through AST parsing,
//! symbol extraction, and relationship mapping.

pub mod analyzer;
pub mod ast_parser;
pub mod checkpoint_context_integrator;
pub mod clone_detector;
pub mod context_explainer;
pub mod context_ranker;
pub mod flow_analyzer;
pub mod incremental_updater;
pub mod knowledge_graph;
pub mod multimodal_integrator;
pub mod pattern_detector;
pub mod query_expander;
pub mod relationship_mapper;
pub mod symbol_extractor;
pub mod symbol_resolver;
pub mod temporal_analyzer;
pub mod type_inferencer;
pub mod types;

#[cfg(feature = "tree-sitter-support")]
pub mod tree_sitter_integration;

pub use analyzer::SemanticAnalyzer;
pub use ast_parser::{ASTParser, LanguageParser};
pub use checkpoint_context_integrator::{
    ArchitecturalContext, CheckpointContext, CheckpointContextIntegrator, CheckpointQueryResult,
    TemporalInsight,
};
pub use clone_detector::{CloneDetectionResult, CloneDetector, CodeClone};
pub use context_explainer::{ContextExplainer, ContextExplanation, QueryExplanation};
pub use context_ranker::{ContextRanker, PrunedContext, RankedContext, RankingConfig};
pub use flow_analyzer::{ControlFlowGraph, DataFlowGraph, FlowAnalysisResult, FlowAnalyzer};
pub use incremental_updater::{IncrementalUpdater, IncrementalUpdaterBuilder, UpdateResult};
pub use knowledge_graph::{EdgeType, GraphEdge, GraphNode, KnowledgeGraph, NodeType};
pub use multimodal_integrator::{MultiModalContext, MultiModalIntegrator};
pub use pattern_detector::{DetectedPattern, PatternDetector, PatternType};
pub use query_expander::{ExpandedEntity, ExpandedQuery, QueryExpander};
pub use relationship_mapper::RelationshipMapper;
pub use symbol_extractor::SymbolExtractor;
pub use symbol_resolver::SymbolResolver;
pub use temporal_analyzer::{EntityEvolution, PatternEvolution, TemporalAnalyzer};
pub use type_inferencer::{TypeInferenceResult, TypeInferencer, TypeInfo, TypeKind};
pub use types::*;

#[cfg(feature = "tree-sitter-support")]
pub use tree_sitter_integration::{TreeSitterParser, TreeSitterParserFactory};
