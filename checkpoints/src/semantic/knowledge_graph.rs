//! Knowledge Graph for Code Understanding
//!
//! This module implements a graph-based representation of code relationships,
//! enabling superior context traversal compared to traditional RAG systems.

use super::types::*;
use crate::error::{CheckpointError, Result};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

/// Node types in the knowledge graph
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    Function,
    Class,
    Interface,
    Type,
    Variable,
    Constant,
    Module,
    File,
}

/// Edge types representing relationships
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EdgeType {
    Calls,
    CalledBy,
    Imports,
    ImportedBy,
    Extends,
    Implements,
    Uses,
    UsedBy,
    Defines,
    DefinedIn,
    References,
    ReferencedBy,
    DependsOn,
    DependencyOf,
    Contains,
    ContainedIn,
}

/// Node in the knowledge graph
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub node_type: NodeType,
    pub name: String,
    pub file_path: String,
    pub location: CodeLocation,
    pub metadata: HashMap<String, serde_json::Value>,
    pub checkpoint_id: Option<String>,
}

/// Edge in the knowledge graph
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
    pub weight: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Knowledge graph for code understanding
pub struct KnowledgeGraph {
    nodes: Arc<RwLock<HashMap<String, GraphNode>>>,
    edges: Arc<RwLock<Vec<GraphEdge>>>,
    adjacency_list: Arc<RwLock<HashMap<String, Vec<GraphEdge>>>>,
    reverse_adjacency_list: Arc<RwLock<HashMap<String, Vec<GraphEdge>>>>,
    node_index: Arc<RwLock<HashMap<NodeType, HashSet<String>>>>,
    file_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl KnowledgeGraph {
    /// Create a new knowledge graph
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            edges: Arc::new(RwLock::new(Vec::new())),
            adjacency_list: Arc::new(RwLock::new(HashMap::new())),
            reverse_adjacency_list: Arc::new(RwLock::new(HashMap::new())),
            node_index: Arc::new(RwLock::new(HashMap::new())),
            file_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&self, node: GraphNode) -> Result<()> {
        let node_id = node.id.clone();
        let node_type = node.node_type.clone();
        let file_path = node.file_path.clone();

        // Add to nodes
        self.nodes.write().insert(node_id.clone(), node);

        // Update node type index
        self.node_index
            .write()
            .entry(node_type)
            .or_insert_with(HashSet::new)
            .insert(node_id.clone());

        // Update file index
        self.file_index
            .write()
            .entry(file_path)
            .or_insert_with(Vec::new)
            .push(node_id);

        Ok(())
    }

    /// Add an edge to the graph
    pub fn add_edge(&self, edge: GraphEdge) -> Result<()> {
        // Validate nodes exist
        let nodes = self.nodes.read();
        if !nodes.contains_key(&edge.from) {
            return Err(CheckpointError::validation(format!(
                "Source node {} not found",
                edge.from
            )));
        }
        if !nodes.contains_key(&edge.to) {
            return Err(CheckpointError::validation(format!(
                "Target node {} not found",
                edge.to
            )));
        }
        drop(nodes);

        // Add to edges list
        self.edges.write().push(edge.clone());

        // Update adjacency list
        self.adjacency_list
            .write()
            .entry(edge.from.clone())
            .or_insert_with(Vec::new)
            .push(edge.clone());

        // Update reverse adjacency list
        self.reverse_adjacency_list
            .write()
            .entry(edge.to.clone())
            .or_insert_with(Vec::new)
            .push(edge);

        Ok(())
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: &str) -> Option<GraphNode> {
        self.nodes.read().get(node_id).cloned()
    }

    /// Get all nodes of a specific type
    pub fn get_nodes_by_type(&self, node_type: NodeType) -> Vec<GraphNode> {
        let node_index = self.node_index.read();
        let nodes = self.nodes.read();

        node_index
            .get(&node_type)
            .map(|node_ids| {
                node_ids
                    .iter()
                    .filter_map(|id| nodes.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all nodes in a file
    pub fn get_nodes_in_file(&self, file_path: &str) -> Vec<GraphNode> {
        let file_index = self.file_index.read();
        let nodes = self.nodes.read();

        file_index
            .get(file_path)
            .map(|node_ids| {
                node_ids
                    .iter()
                    .filter_map(|id| nodes.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get outgoing edges from a node
    pub fn get_outgoing_edges(&self, node_id: &str) -> Vec<GraphEdge> {
        self.adjacency_list
            .read()
            .get(node_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get incoming edges to a node
    pub fn get_incoming_edges(&self, node_id: &str) -> Vec<GraphEdge> {
        self.reverse_adjacency_list
            .read()
            .get(node_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all neighbors of a node (both incoming and outgoing)
    pub fn get_neighbors(&self, node_id: &str) -> Vec<GraphNode> {
        let mut neighbor_ids = HashSet::new();

        // Outgoing edges
        for edge in self.get_outgoing_edges(node_id) {
            neighbor_ids.insert(edge.to);
        }

        // Incoming edges
        for edge in self.get_incoming_edges(node_id) {
            neighbor_ids.insert(edge.from);
        }

        let nodes = self.nodes.read();
        neighbor_ids
            .into_iter()
            .filter_map(|id| nodes.get(&id).cloned())
            .collect()
    }

    /// Find shortest path between two nodes
    pub fn find_shortest_path(&self, from: &str, to: &str) -> Option<Vec<GraphNode>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent_map: HashMap<String, String> = HashMap::new();

        queue.push_back(from.to_string());
        visited.insert(from.to_string());

        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current = to.to_string();

                while &current != from {
                    if let Some(node) = self.get_node(&current) {
                        path.push(node);
                    }
                    if let Some(parent) = parent_map.get(&current) {
                        current = parent.clone();
                    } else {
                        break;
                    }
                }

                if let Some(node) = self.get_node(from) {
                    path.push(node);
                }

                path.reverse();
                return Some(path);
            }

            // Explore neighbors
            for edge in self.get_outgoing_edges(&current) {
                if !visited.contains(&edge.to) {
                    visited.insert(edge.to.clone());
                    parent_map.insert(edge.to.clone(), current.clone());
                    queue.push_back(edge.to);
                }
            }
        }

        None
    }

    /// Find all paths up to a certain depth
    pub fn find_paths_with_depth(&self, from: &str, depth: usize) -> Vec<Vec<GraphNode>> {
        let mut paths = Vec::new();
        let mut current_path = Vec::new();

        if let Some(start_node) = self.get_node(from) {
            current_path.push(start_node);
            self.dfs_paths(
                from,
                depth,
                &mut current_path,
                &mut paths,
                &mut HashSet::new(),
            );
        }

        paths
    }

    /// DFS helper for path finding
    fn dfs_paths(
        &self,
        current: &str,
        remaining_depth: usize,
        current_path: &mut Vec<GraphNode>,
        paths: &mut Vec<Vec<GraphNode>>,
        visited: &mut HashSet<String>,
    ) {
        if remaining_depth == 0 {
            paths.push(current_path.clone());
            return;
        }

        visited.insert(current.to_string());

        for edge in self.get_outgoing_edges(current) {
            if !visited.contains(&edge.to) {
                if let Some(node) = self.get_node(&edge.to) {
                    current_path.push(node);
                    self.dfs_paths(&edge.to, remaining_depth - 1, current_path, paths, visited);
                    current_path.pop();
                }
            }
        }

        visited.remove(current);
    }

    /// Find all nodes reachable from a starting node
    pub fn get_reachable_nodes(&self, from: &str, max_depth: Option<usize>) -> Vec<GraphNode> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((from.to_string(), 0usize));
        reachable.insert(from.to_string());

        while let Some((current, depth)) = queue.pop_front() {
            if let Some(max_depth) = max_depth {
                if depth >= max_depth {
                    continue;
                }
            }

            for edge in self.get_outgoing_edges(&current) {
                if !reachable.contains(&edge.to) {
                    reachable.insert(edge.to.clone());
                    queue.push_back((edge.to, depth + 1));
                }
            }
        }

        let nodes = self.nodes.read();
        reachable
            .into_iter()
            .filter_map(|id| nodes.get(&id).cloned())
            .collect()
    }

    /// Get all call chains from a function
    pub fn get_call_chains_from(&self, function_id: &str, max_depth: usize) -> Vec<Vec<GraphNode>> {
        let mut chains = Vec::new();
        let mut current_chain = Vec::new();

        if let Some(start_node) = self.get_node(function_id) {
            current_chain.push(start_node);
            self.dfs_call_chains(
                function_id,
                max_depth,
                &mut current_chain,
                &mut chains,
                &mut HashSet::new(),
            );
        }

        chains
    }

    /// DFS helper for call chain extraction
    fn dfs_call_chains(
        &self,
        current: &str,
        remaining_depth: usize,
        current_chain: &mut Vec<GraphNode>,
        chains: &mut Vec<Vec<GraphNode>>,
        visited: &mut HashSet<String>,
    ) {
        if remaining_depth == 0 || visited.contains(current) {
            if current_chain.len() > 1 {
                chains.push(current_chain.clone());
            }
            return;
        }

        visited.insert(current.to_string());

        let call_edges: Vec<_> = self
            .get_outgoing_edges(current)
            .into_iter()
            .filter(|e| e.edge_type == EdgeType::Calls)
            .collect();

        if call_edges.is_empty() && current_chain.len() > 1 {
            chains.push(current_chain.clone());
        }

        for edge in call_edges {
            if let Some(node) = self.get_node(&edge.to) {
                current_chain.push(node);
                self.dfs_call_chains(
                    &edge.to,
                    remaining_depth - 1,
                    current_chain,
                    chains,
                    visited,
                );
                current_chain.pop();
            }
        }

        visited.remove(current);
    }

    /// Get dependency chain for a module
    pub fn get_dependency_chain(&self, module_id: &str) -> Vec<GraphNode> {
        let mut dependencies = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(module_id.to_string());
        visited.insert(module_id.to_string());

        while let Some(current) = queue.pop_front() {
            if let Some(node) = self.get_node(&current) {
                dependencies.push(node);
            }

            for edge in self.get_outgoing_edges(&current) {
                if edge.edge_type == EdgeType::DependsOn && !visited.contains(&edge.to) {
                    visited.insert(edge.to.clone());
                    queue.push_back(edge.to);
                }
            }
        }

        dependencies
    }

    /// Find strongly connected components (for circular dependency detection)
    pub fn find_circular_dependencies(&self) -> Vec<Vec<GraphNode>> {
        let mut sccs = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        // First DFS to fill stack
        for node_id in self.nodes.read().keys() {
            if !visited.contains(node_id) {
                self.dfs_fill_stack(node_id, &mut visited, &mut stack);
            }
        }

        // Second DFS on transposed graph
        visited.clear();
        while let Some(node_id) = stack.pop() {
            if !visited.contains(&node_id) {
                let mut scc = Vec::new();
                self.dfs_collect_scc(&node_id, &mut visited, &mut scc);
                if scc.len() > 1 {
                    sccs.push(scc);
                }
            }
        }

        sccs
    }

    fn dfs_fill_stack(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) {
        visited.insert(node_id.to_string());

        for edge in self.get_outgoing_edges(node_id) {
            if !visited.contains(&edge.to) {
                self.dfs_fill_stack(&edge.to, visited, stack);
            }
        }

        stack.push(node_id.to_string());
    }

    fn dfs_collect_scc(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        scc: &mut Vec<GraphNode>,
    ) {
        visited.insert(node_id.to_string());

        if let Some(node) = self.get_node(node_id) {
            scc.push(node);
        }

        for edge in self.get_incoming_edges(node_id) {
            if !visited.contains(&edge.from) {
                self.dfs_collect_scc(&edge.from, visited, scc);
            }
        }
    }

    /// Get statistics about the graph
    pub fn get_statistics(&self) -> GraphStatistics {
        let nodes = self.nodes.read();
        let edges = self.edges.read();

        let mut node_type_counts = HashMap::new();
        for node in nodes.values() {
            *node_type_counts.entry(node.node_type.clone()).or_insert(0) += 1;
        }

        let mut edge_type_counts = HashMap::new();
        for edge in edges.iter() {
            *edge_type_counts.entry(edge.edge_type.clone()).or_insert(0) += 1;
        }

        GraphStatistics {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            node_type_counts,
            edge_type_counts,
            average_degree: if nodes.is_empty() {
                0.0
            } else {
                edges.len() as f64 / nodes.len() as f64
            },
        }
    }

    /// Clear the entire graph
    pub fn clear(&self) {
        self.nodes.write().clear();
        self.edges.write().clear();
        self.adjacency_list.write().clear();
        self.reverse_adjacency_list.write().clear();
        self.node_index.write().clear();
        self.file_index.write().clear();
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the knowledge graph
#[derive(Debug, Clone)]
pub struct GraphStatistics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub node_type_counts: HashMap<NodeType, usize>,
    pub edge_type_counts: HashMap<EdgeType, usize>,
    pub average_degree: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let graph = KnowledgeGraph::new();
        let stats = graph.get_statistics();
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.total_edges, 0);
    }

    #[test]
    fn test_add_node() {
        let graph = KnowledgeGraph::new();
        let node = GraphNode {
            id: "test_func".to_string(),
            node_type: NodeType::Function,
            name: "testFunction".to_string(),
            file_path: "test.ts".to_string(),
            location: CodeLocation {
                file_path: "test.ts".to_string(),
                start_line: 1,
                start_column: 1,
                end_line: 10,
                end_column: 1,
            },
            metadata: HashMap::new(),
            checkpoint_id: None,
        };

        assert!(graph.add_node(node).is_ok());
        assert_eq!(graph.get_statistics().total_nodes, 1);
    }

    #[test]
    fn test_add_edge() {
        let graph = KnowledgeGraph::new();

        // Add two nodes
        let node1 = GraphNode {
            id: "func1".to_string(),
            node_type: NodeType::Function,
            name: "function1".to_string(),
            file_path: "test.ts".to_string(),
            location: CodeLocation {
                file_path: "test.ts".to_string(),
                start_line: 1,
                start_column: 1,
                end_line: 5,
                end_column: 1,
            },
            metadata: HashMap::new(),
            checkpoint_id: None,
        };

        let node2 = GraphNode {
            id: "func2".to_string(),
            node_type: NodeType::Function,
            name: "function2".to_string(),
            file_path: "test.ts".to_string(),
            location: CodeLocation {
                file_path: "test.ts".to_string(),
                start_line: 7,
                start_column: 1,
                end_line: 12,
                end_column: 1,
            },
            metadata: HashMap::new(),
            checkpoint_id: None,
        };

        assert!(graph.add_node(node1).is_ok());
        assert!(graph.add_node(node2).is_ok());

        // Add edge
        let edge = GraphEdge {
            from: "func1".to_string(),
            to: "func2".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
            metadata: HashMap::new(),
        };

        assert!(graph.add_edge(edge).is_ok());
        assert_eq!(graph.get_statistics().total_edges, 1);
    }
}
