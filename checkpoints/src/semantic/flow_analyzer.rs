//! Control Flow and Data Flow Analysis
//!
//! This module builds Control Flow Graphs (CFG) and Data Flow Graphs (DFG) for
//! deep understanding of program execution and data movement.

use super::types::*;
use crate::error::Result;
use std::collections::{HashMap, HashSet, VecDeque};

/// Flow analyzer for CFG and DFG
pub struct FlowAnalyzer {
    config: FlowAnalysisConfig,
}

/// Configuration for flow analysis
#[derive(Debug, Clone)]
pub struct FlowAnalysisConfig {
    pub build_cfg: bool,
    pub build_dfg: bool,
    pub detect_dead_code: bool,
    pub detect_null_pointer: bool,
    pub detect_uninitialized: bool,
    pub max_depth: usize,
}

impl Default for FlowAnalysisConfig {
    fn default() -> Self {
        Self {
            build_cfg: true,
            build_dfg: true,
            detect_dead_code: true,
            detect_null_pointer: true,
            detect_uninitialized: true,
            max_depth: 100,
        }
    }
}

/// Control Flow Graph
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub entry_node: String,
    pub exit_nodes: Vec<String>,
    pub nodes: HashMap<String, CFGNode>,
    pub edges: Vec<CFGEdge>,
    pub basic_blocks: Vec<BasicBlock>,
}

/// CFG Node
#[derive(Debug, Clone)]
pub struct CFGNode {
    pub id: String,
    pub node_type: CFGNodeType,
    pub statements: Vec<Statement>,
    pub predecessors: Vec<String>,
    pub successors: Vec<String>,
}

/// CFG Node Type
#[derive(Debug, Clone, PartialEq)]
pub enum CFGNodeType {
    Entry,
    Exit,
    Statement,
    Condition,
    Loop,
    Branch,
    Call,
    Return,
}

/// CFG Edge
#[derive(Debug, Clone)]
pub struct CFGEdge {
    pub from: String,
    pub to: String,
    pub edge_type: CFGEdgeType,
    pub condition: Option<String>,
}

/// CFG Edge Type
#[derive(Debug, Clone, PartialEq)]
pub enum CFGEdgeType {
    Unconditional,
    ConditionalTrue,
    ConditionalFalse,
    LoopBack,
    Exception,
}

/// Basic Block
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: String,
    pub statements: Vec<Statement>,
    pub predecessors: Vec<String>,
    pub successors: Vec<String>,
}

/// Statement representation
#[derive(Debug, Clone)]
pub struct Statement {
    pub id: String,
    pub statement_type: StatementType,
    pub variables_read: Vec<String>,
    pub variables_written: Vec<String>,
    pub location: CodeLocation,
}

/// Statement type
#[derive(Debug, Clone, PartialEq)]
pub enum StatementType {
    Assignment,
    Declaration,
    IfStatement,
    WhileLoop,
    ForLoop,
    FunctionCall,
    Return,
    Break,
    Continue,
}

/// Data Flow Graph
#[derive(Debug, Clone)]
pub struct DataFlowGraph {
    pub nodes: HashMap<String, DFGNode>,
    pub edges: Vec<DFGEdge>,
    pub definitions: HashMap<String, Vec<Definition>>,
    pub uses: HashMap<String, Vec<Use>>,
}

/// DFG Node
#[derive(Debug, Clone)]
pub struct DFGNode {
    pub id: String,
    pub variable: String,
    pub node_type: DFGNodeType,
    pub location: CodeLocation,
}

/// DFG Node Type
#[derive(Debug, Clone, PartialEq)]
pub enum DFGNodeType {
    Definition,
    Use,
    Kill,
}

/// DFG Edge
#[derive(Debug, Clone)]
pub struct DFGEdge {
    pub from: String,
    pub to: String,
    pub variable: String,
    pub edge_type: DFGEdgeType,
}

/// DFG Edge Type
#[derive(Debug, Clone, PartialEq)]
pub enum DFGEdgeType {
    DefUse, // Definition to use
    UseDef, // Use to definition
    DefDef, // Definition to definition (kill)
}

/// Variable definition
#[derive(Debug, Clone)]
pub struct Definition {
    pub variable: String,
    pub statement_id: String,
    pub location: CodeLocation,
    pub value: Option<String>,
}

/// Variable use
#[derive(Debug, Clone)]
pub struct Use {
    pub variable: String,
    pub statement_id: String,
    pub location: CodeLocation,
    pub context: UseContext,
}

/// Use context
#[derive(Debug, Clone, PartialEq)]
pub enum UseContext {
    Read,
    Write,
    ReadWrite,
}

/// Flow analysis result
#[derive(Debug, Clone)]
pub struct FlowAnalysisResult {
    pub cfg: Option<ControlFlowGraph>,
    pub dfg: Option<DataFlowGraph>,
    pub dead_code: Vec<DeadCode>,
    pub null_pointer_risks: Vec<NullPointerRisk>,
    pub uninitialized_variables: Vec<UninitializedVariable>,
    pub unreachable_code: Vec<UnreachableCode>,
}

/// Dead code detection
#[derive(Debug, Clone)]
pub struct DeadCode {
    pub location: CodeLocation,
    pub reason: String,
    pub code_snippet: String,
}

/// Null pointer risk
#[derive(Debug, Clone)]
pub struct NullPointerRisk {
    pub variable: String,
    pub location: CodeLocation,
    pub risk_level: RiskLevel,
    pub reason: String,
}

/// Risk level
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Uninitialized variable
#[derive(Debug, Clone)]
pub struct UninitializedVariable {
    pub variable: String,
    pub use_location: CodeLocation,
    pub declaration_location: Option<CodeLocation>,
}

/// Unreachable code
#[derive(Debug, Clone)]
pub struct UnreachableCode {
    pub location: CodeLocation,
    pub reason: String,
    pub block_id: String,
}

impl FlowAnalyzer {
    /// Create a new flow analyzer
    pub fn new() -> Self {
        Self {
            config: FlowAnalysisConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: FlowAnalysisConfig) -> Self {
        Self { config }
    }

    /// Analyze flow for a function
    pub fn analyze_function(&self, statements: &[Statement]) -> Result<FlowAnalysisResult> {
        let cfg = if self.config.build_cfg {
            Some(self.build_cfg(statements)?)
        } else {
            None
        };

        let dfg = if self.config.build_dfg {
            Some(self.build_dfg(statements)?)
        } else {
            None
        };

        let dead_code = if self.config.detect_dead_code {
            self.detect_dead_code(&cfg)?
        } else {
            Vec::new()
        };

        let null_pointer_risks = if self.config.detect_null_pointer {
            self.detect_null_pointer_risks(&dfg)?
        } else {
            Vec::new()
        };

        let uninitialized_variables = if self.config.detect_uninitialized {
            self.detect_uninitialized_variables(&dfg)?
        } else {
            Vec::new()
        };

        let unreachable_code = self.detect_unreachable_code(&cfg)?;

        Ok(FlowAnalysisResult {
            cfg,
            dfg,
            dead_code,
            null_pointer_risks,
            uninitialized_variables,
            unreachable_code,
        })
    }

    /// Build Control Flow Graph
    fn build_cfg(&self, statements: &[Statement]) -> Result<ControlFlowGraph> {
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();

        // Create entry node
        let entry_id = "entry".to_string();
        nodes.insert(
            entry_id.clone(),
            CFGNode {
                id: entry_id.clone(),
                node_type: CFGNodeType::Entry,
                statements: Vec::new(),
                predecessors: Vec::new(),
                successors: Vec::new(),
            },
        );

        // Process statements and create nodes
        let mut current_id = entry_id.clone();
        for (idx, stmt) in statements.iter().enumerate() {
            let node_id = format!("node_{}", idx);
            let node_type = match stmt.statement_type {
                StatementType::IfStatement => CFGNodeType::Condition,
                StatementType::WhileLoop | StatementType::ForLoop => CFGNodeType::Loop,
                StatementType::Return => CFGNodeType::Return,
                StatementType::FunctionCall => CFGNodeType::Call,
                _ => CFGNodeType::Statement,
            };

            nodes.insert(
                node_id.clone(),
                CFGNode {
                    id: node_id.clone(),
                    node_type,
                    statements: vec![stmt.clone()],
                    predecessors: vec![current_id.clone()],
                    successors: Vec::new(),
                },
            );

            // Add edge
            edges.push(CFGEdge {
                from: current_id.clone(),
                to: node_id.clone(),
                edge_type: CFGEdgeType::Unconditional,
                condition: None,
            });

            // Update predecessors/successors
            if let Some(current_node) = nodes.get_mut(&current_id) {
                current_node.successors.push(node_id.clone());
            }

            current_id = node_id;
        }

        // Create exit node
        let exit_id = "exit".to_string();
        nodes.insert(
            exit_id.clone(),
            CFGNode {
                id: exit_id.clone(),
                node_type: CFGNodeType::Exit,
                statements: Vec::new(),
                predecessors: vec![current_id.clone()],
                successors: Vec::new(),
            },
        );

        edges.push(CFGEdge {
            from: current_id,
            to: exit_id.clone(),
            edge_type: CFGEdgeType::Unconditional,
            condition: None,
        });

        // Build basic blocks
        let basic_blocks = self.compute_basic_blocks(&nodes, &edges)?;

        Ok(ControlFlowGraph {
            entry_node: entry_id,
            exit_nodes: vec![exit_id],
            nodes,
            edges,
            basic_blocks,
        })
    }

    /// Build Data Flow Graph
    fn build_dfg(&self, statements: &[Statement]) -> Result<DataFlowGraph> {
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();
        let mut definitions: HashMap<String, Vec<Definition>> = HashMap::new();
        let mut uses: HashMap<String, Vec<Use>> = HashMap::new();

        // Track definitions and uses
        let mut def_map: HashMap<String, String> = HashMap::new(); // variable -> node_id

        for (idx, stmt) in statements.iter().enumerate() {
            // Process definitions (writes)
            for var in &stmt.variables_written {
                let node_id = format!("def_{}_{}", var, idx);
                nodes.insert(
                    node_id.clone(),
                    DFGNode {
                        id: node_id.clone(),
                        variable: var.clone(),
                        node_type: DFGNodeType::Definition,
                        location: stmt.location.clone(),
                    },
                );

                definitions
                    .entry(var.clone())
                    .or_insert_with(Vec::new)
                    .push(Definition {
                        variable: var.clone(),
                        statement_id: stmt.id.clone(),
                        location: stmt.location.clone(),
                        value: None,
                    });

                // If there was a previous definition, add def-def edge (kill)
                if let Some(prev_def_id) = def_map.get(var) {
                    edges.push(DFGEdge {
                        from: prev_def_id.clone(),
                        to: node_id.clone(),
                        variable: var.clone(),
                        edge_type: DFGEdgeType::DefDef,
                    });
                }

                def_map.insert(var.clone(), node_id);
            }

            // Process uses (reads)
            for var in &stmt.variables_read {
                let node_id = format!("use_{}_{}", var, idx);
                nodes.insert(
                    node_id.clone(),
                    DFGNode {
                        id: node_id.clone(),
                        variable: var.clone(),
                        node_type: DFGNodeType::Use,
                        location: stmt.location.clone(),
                    },
                );

                uses.entry(var.clone()).or_insert_with(Vec::new).push(Use {
                    variable: var.clone(),
                    statement_id: stmt.id.clone(),
                    location: stmt.location.clone(),
                    context: UseContext::Read,
                });

                // Add def-use edge if there's a definition
                if let Some(def_id) = def_map.get(var) {
                    edges.push(DFGEdge {
                        from: def_id.clone(),
                        to: node_id,
                        variable: var.clone(),
                        edge_type: DFGEdgeType::DefUse,
                    });
                }
            }
        }

        Ok(DataFlowGraph {
            nodes,
            edges,
            definitions,
            uses,
        })
    }

    /// Compute basic blocks
    fn compute_basic_blocks(
        &self,
        nodes: &HashMap<String, CFGNode>,
        edges: &[CFGEdge],
    ) -> Result<Vec<BasicBlock>> {
        let mut basic_blocks = Vec::new();

        // Identify leaders (first statement of basic blocks)
        let mut leaders = HashSet::new();
        leaders.insert("entry".to_string());

        for edge in edges {
            if edge.edge_type != CFGEdgeType::Unconditional {
                leaders.insert(edge.to.clone());
            }
        }

        // Build basic blocks
        let mut current_block_id = String::new();
        let mut current_statements = Vec::new();

        for (node_id, node) in nodes.iter() {
            if leaders.contains(node_id) && !current_statements.is_empty() {
                basic_blocks.push(BasicBlock {
                    id: current_block_id.clone(),
                    statements: current_statements.clone(),
                    predecessors: Vec::new(),
                    successors: Vec::new(),
                });
                current_statements.clear();
            }

            if leaders.contains(node_id) {
                current_block_id = format!("bb_{}", basic_blocks.len());
            }

            current_statements.extend(node.statements.clone());
        }

        // Add final block
        if !current_statements.is_empty() {
            basic_blocks.push(BasicBlock {
                id: current_block_id,
                statements: current_statements,
                predecessors: Vec::new(),
                successors: Vec::new(),
            });
        }

        Ok(basic_blocks)
    }

    /// Detect dead code
    fn detect_dead_code(&self, cfg: &Option<ControlFlowGraph>) -> Result<Vec<DeadCode>> {
        let mut dead_code = Vec::new();

        if let Some(cfg) = cfg {
            // Find unreachable nodes
            let reachable = self.find_reachable_nodes(&cfg.entry_node, &cfg.edges);

            for (node_id, node) in &cfg.nodes {
                if !reachable.contains(node_id) && node.node_type != CFGNodeType::Entry {
                    for stmt in &node.statements {
                        dead_code.push(DeadCode {
                            location: stmt.location.clone(),
                            reason: "Unreachable code".to_string(),
                            code_snippet: format!("Node: {}", node_id),
                        });
                    }
                }
            }
        }

        Ok(dead_code)
    }

    /// Find reachable nodes from entry
    fn find_reachable_nodes(&self, entry: &str, edges: &[CFGEdge]) -> HashSet<String> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(entry.to_string());
        reachable.insert(entry.to_string());

        while let Some(node_id) = queue.pop_front() {
            for edge in edges {
                if edge.from == node_id && !reachable.contains(&edge.to) {
                    reachable.insert(edge.to.clone());
                    queue.push_back(edge.to.clone());
                }
            }
        }

        reachable
    }

    /// Detect null pointer risks
    fn detect_null_pointer_risks(
        &self,
        _dfg: &Option<DataFlowGraph>,
    ) -> Result<Vec<NullPointerRisk>> {
        // Would analyze data flow for potential null pointer dereferences
        Ok(Vec::new())
    }

    /// Detect uninitialized variables
    fn detect_uninitialized_variables(
        &self,
        dfg: &Option<DataFlowGraph>,
    ) -> Result<Vec<UninitializedVariable>> {
        let mut uninitialized = Vec::new();

        if let Some(dfg) = dfg {
            for (var, uses) in &dfg.uses {
                if !dfg.definitions.contains_key(var) {
                    for use_site in uses {
                        uninitialized.push(UninitializedVariable {
                            variable: var.clone(),
                            use_location: use_site.location.clone(),
                            declaration_location: None,
                        });
                    }
                }
            }
        }

        Ok(uninitialized)
    }

    /// Detect unreachable code
    fn detect_unreachable_code(
        &self,
        cfg: &Option<ControlFlowGraph>,
    ) -> Result<Vec<UnreachableCode>> {
        let mut unreachable = Vec::new();

        if let Some(cfg) = cfg {
            let reachable = self.find_reachable_nodes(&cfg.entry_node, &cfg.edges);

            for (node_id, node) in &cfg.nodes {
                if !reachable.contains(node_id) && node.node_type != CFGNodeType::Entry {
                    if let Some(first_stmt) = node.statements.first() {
                        unreachable.push(UnreachableCode {
                            location: first_stmt.location.clone(),
                            reason: "Never executed".to_string(),
                            block_id: node_id.clone(),
                        });
                    }
                }
            }
        }

        Ok(unreachable)
    }
}

impl Default for FlowAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = FlowAnalyzer::new();
        assert!(analyzer.config.build_cfg);
        assert!(analyzer.config.build_dfg);
    }

    #[test]
    fn test_risk_levels() {
        assert!(RiskLevel::Low < RiskLevel::High);
        assert!(RiskLevel::Medium < RiskLevel::Critical);
    }
}
