//! Graph validation

use crate::{
    config::Config,
    error::CanvasResult,
    types::{VisualGraph, VisualNode, Connection},
};

use super::ValidationResult;

/// Graph validator
pub struct Validator;

impl Validator {
    /// Create a new validator
    pub fn new(_config: &Config) -> CanvasResult<Self> {
        Ok(Self)
    }

    /// Validate a visual graph
    pub fn validate(&self, graph: &VisualGraph) -> CanvasResult<ValidationResult> {
        let mut result = ValidationResult::valid();

        // Validate nodes
        for node in &graph.nodes {
            self.validate_node(node, graph, &mut result);
        }

        // Validate connections
        for connection in &graph.connections {
            self.validate_connection(connection, graph, &mut result);
        }

        // Validate graph structure
        self.validate_graph_structure(graph, &mut result);

        Ok(result)
    }

    /// Validate a single node
    fn validate_node(&self, node: &VisualNode, graph: &VisualGraph, result: &mut ValidationResult) {
        // Check for required inputs
        for input in &node.inputs {
            if input.required {
                // Check if this input is connected
                let is_connected = graph.connections.iter().any(|c| c.target_node == node.id && c.target_port == input.id);
                if !is_connected {
                    *result = result.clone().with_error(format!(
                        "Node {} has unconnected required input: {}",
                        node.id, input.name
                    ));
                }
            }
        }

        // Validate node properties
        self.validate_node_properties(node, result);
    }

    /// Validate node properties based on node type
    fn validate_node_properties(&self, node: &VisualNode, result: &mut ValidationResult) {
        match node.node_type.as_str() {
            "If" => {
                // If nodes require a condition property
                if !node.properties.contains_key("condition") {
                    *result = result.clone().with_error(format!(
                        "If node {} missing required 'condition' property",
                        node.id
                    ));
                }
            }
            "WriteStorage" => {
                if !node.properties.contains_key("key") {
                    *result = result.clone().with_error(format!(
                        "WriteStorage node {} missing required 'key' property",
                        node.id
                    ));
                }
            }
            "ReadStorage" => {
                if !node.properties.contains_key("key") {
                    *result = result.clone().with_error(format!(
                        "ReadStorage node {} missing required 'key' property",
                        node.id
                    ));
                }
            }
            // Arithmetic nodes — no required properties
            "Add" | "Subtract" | "Multiply" | "Divide" => {}
            // Logic nodes — no required properties
            "And" | "Or" | "Not" => {}
            // Control flow nodes — no required properties
            "Start" | "End" => {}
            // Crypto nodes
            "VerifySignature" | "DecodeProof" => {}
            _ => {
                // Truly unknown node type
                *result = result.clone().with_warning(format!(
                    "Unknown node type: {}",
                    node.node_type
                ));
            }
        }
    }

    /// Validate a connection
    fn validate_connection(
        &self,
        connection: &Connection,
        graph: &VisualGraph,
        result: &mut ValidationResult,
    ) {
        // Check if source node exists
        let source_node = graph.get_node(connection.source_node);
        if source_node.is_none() {
            *result = result.clone().with_error(format!(
                "Connection {} references non-existent source node: {}",
                connection.id, connection.source_node
            ));
            return;
        }

        // Check if target node exists
        let target_node = graph.get_node(connection.target_node);
        if target_node.is_none() {
            *result = result.clone().with_error(format!(
                "Connection {} references non-existent target node: {}",
                connection.id, connection.target_node
            ));
            return;
        }

        let source_node = source_node.unwrap();
        let target_node = target_node.unwrap();

        // Check if source port exists
        let source_port = source_node.outputs.iter().find(|p| p.id == connection.source_port);
        if source_port.is_none() {
            *result = result.clone().with_error(format!(
                "Connection {} references non-existent source port: {}",
                connection.id, connection.source_port
            ));
            return;
        }

        // Check if target port exists
        let target_port = target_node.inputs.iter().find(|p| p.id == connection.target_port);
        if target_port.is_none() {
            *result = result.clone().with_error(format!(
                "Connection {} references non-existent target port: {}",
                connection.id, connection.target_port
            ));
            return;
        }

        let source_port = source_port.unwrap();
        let target_port = target_port.unwrap();

        // Check type compatibility
        if !source_port.value_type.is_compatible_with(&target_port.value_type) {
            *result = result.clone().with_error(format!(
                "Type mismatch in connection {}: {:?} -> {:?}",
                connection.id,
                source_port.value_type,
                target_port.value_type,
            ));
        }
    }

    /// Validate graph structure
    fn validate_graph_structure(&self, graph: &VisualGraph, result: &mut ValidationResult) {
        let ir = crate::compiler::graph_ir::GraphIR::from_visual_graph(graph);

        // Check for cycles
        if petgraph::algo::is_cyclic_directed(&ir.graph) {
            *result = result.clone().with_error("Graph contains cycles".to_string());
        }

        // Check for unreachable nodes
        let unreachable = self.find_unreachable_nodes(&ir, graph);
        if !unreachable.is_empty() {
            *result = result.clone().with_warning(format!(
                "Unreachable nodes found: {:?}",
                unreachable
            ));
        }

        // Check for disconnected components
        let components = petgraph::algo::tarjan_scc(&ir.graph);
        if components.len() > 1 && !graph.nodes.is_empty() {
            *result = result.clone().with_warning(format!(
                "Graph has {} disconnected components",
                components.len()
            ));
        }
    }

    /// Find unreachable nodes starting from 'Start' nodes
    fn find_unreachable_nodes(&self, ir: &crate::compiler::graph_ir::GraphIR, graph: &VisualGraph) -> Vec<crate::types::NodeId> {
        let start_nodes: Vec<_> = ir.nodes.values()
            .filter(|n| n.node_type == "Start")
            .filter_map(|n| ir.node_map.get(&n.id))
            .cloned()
            .collect();
        
        if start_nodes.is_empty() {
            if graph.nodes.is_empty() {
                return Vec::new();
            }
            return graph.nodes.iter().map(|n| n.id).collect();
        }

        let mut reachable = std::collections::HashSet::new();
        for start_idx in start_nodes {
            let mut bfs = petgraph::visit::Bfs::new(&ir.graph, start_idx);
            while let Some(nx) = bfs.next(&ir.graph) {
                reachable.insert(ir.graph[nx]);
            }
        }

        graph.nodes.iter()
            .map(|n| n.id)
            .filter(|id| !reachable.contains(id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{VisualNode, Position};
    use uuid::Uuid;

    #[test]
    fn test_validator_creation() {
        let config = Config::default();
        let validator = Validator::new(&config);
        assert!(validator.is_ok());
    }

    #[test]
    fn test_node_validation() {
        let config = Config::default();
        let validator = Validator::new(&config).unwrap();

        // Create a valid node
        let mut node = VisualNode::new(
            Uuid::new_v4(),
            "If",
            Position::new(0.0, 0.0),
        );
        node = node.with_property("condition".to_string(), serde_json::Value::String("true".to_string()));

        let mut result = ValidationResult::valid();
        let graph = VisualGraph::new("test");
        validator.validate_node(&node, &graph, &mut result);
        assert!(result.is_valid);
    }

    #[test]
    fn test_invalid_node_validation() {
        let config = Config::default();
        let validator = Validator::new(&config).unwrap();

        // Create an invalid If node without condition
        let node = VisualNode::new(
            Uuid::new_v4(),
            "If",
            Position::new(0.0, 0.0),
        );

        let mut result = ValidationResult::valid();
        let graph = VisualGraph::new("test");
        validator.validate_node(&node, &graph, &mut result);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }
} 