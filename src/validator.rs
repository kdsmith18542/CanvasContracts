//! Contract validation module

use crate::{
    error::{CanvasError, CanvasResult},
    types::{VisualGraph, VisualNode, Connection},
};

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        }
    }

    /// Add an error
    pub fn with_error(mut self, error: String) -> Self {
        self.errors.push(error);
        self.is_valid = false;
        self
    }

    /// Add a warning
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    /// Add info
    pub fn with_info(mut self, info: String) -> Self {
        self.info.push(info);
        self
    }
}

/// Contract validator
pub struct ContractValidator;

impl ContractValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self
    }

    /// Validate a visual graph
    pub fn validate(&self, graph: &VisualGraph) -> CanvasResult<ValidationResult> {
        let mut result = ValidationResult::new();

        // Validate nodes
        for node in &graph.nodes {
            self.validate_node(node, &mut result);
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
    fn validate_node(&self, node: &VisualNode, result: &mut ValidationResult) {
        // Check for required inputs
        for input in &node.inputs {
            if input.required && !self.is_input_connected(node, input, &[]) {
                *result = result.clone().with_error(format!(
                    "Node {} has unconnected required input: {}",
                    node.id, input.name
                ));
            }
        }

        // Validate node properties
        self.validate_node_properties(node, result);
    }

    /// Validate node properties
    fn validate_node_properties(&self, node: &VisualNode, result: &mut ValidationResult) {
        match node.node_type.as_str() {
            "If" => {
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
            _ => {
                *result = result.clone().with_warning(format!(
                    "Unknown node type: {}",
                    node.node_type
                ));
            }
        }
    }

    /// Check if an input is connected
    fn is_input_connected(
        &self,
        node: &VisualNode,
        input: &crate::types::Port,
        connections: &[Connection],
    ) -> bool {
        connections.iter().any(|conn| {
            conn.target_node == node.id && conn.target_port == input.name
        })
    }

    /// Validate a connection
    fn validate_connection(
        &self,
        connection: &Connection,
        graph: &VisualGraph,
        result: &mut ValidationResult,
    ) {
        // Check if source node exists
        if !graph.nodes.iter().any(|n| n.id == connection.source_node) {
            *result = result.clone().with_error(format!(
                "Connection {} references non-existent source node: {}",
                connection.id, connection.source_node
            ));
        }

        // Check if target node exists
        if !graph.nodes.iter().any(|n| n.id == connection.target_node) {
            *result = result.clone().with_error(format!(
                "Connection {} references non-existent target node: {}",
                connection.id, connection.target_node
            ));
        }

        // Check if source port exists
        if let Some(source_node) = graph.nodes.iter().find(|n| n.id == connection.source_node) {
            if !source_node.outputs.iter().any(|p| p.name == connection.source_port) {
                *result = result.clone().with_error(format!(
                    "Connection {} references non-existent source port: {}",
                    connection.id, connection.source_port
                ));
            }
        }

        // Check if target port exists
        if let Some(target_node) = graph.nodes.iter().find(|n| n.id == connection.target_node) {
            if !target_node.inputs.iter().any(|p| p.name == connection.target_port) {
                *result = result.clone().with_error(format!(
                    "Connection {} references non-existent target port: {}",
                    connection.id, connection.target_port
                ));
            }
        }

        // Check type compatibility
        if let (Some(source_node), Some(target_node)) = (
            graph.nodes.iter().find(|n| n.id == connection.source_node),
            graph.nodes.iter().find(|n| n.id == connection.target_node),
        ) {
            if let (Some(source_port), Some(target_port)) = (
                source_node.outputs.iter().find(|p| p.name == connection.source_port),
                target_node.inputs.iter().find(|p| p.name == connection.target_port),
            ) {
                if source_port.value_type != target_port.value_type {
                    *result = result.clone().with_error(format!(
                        "Type mismatch in connection {}: {} -> {}",
                        connection.id,
                        format!("{:?}", source_port.value_type),
                        format!("{:?}", target_port.value_type)
                    ));
                }
            }
        }
    }

    /// Validate graph structure
    fn validate_graph_structure(&self, graph: &VisualGraph, result: &mut ValidationResult) {
        // Check for cycles
        if self.has_cycles(graph) {
            *result = result.clone().with_error("Graph contains cycles".to_string());
        }

        // Check for unreachable nodes
        let unreachable = self.find_unreachable_nodes(graph);
        if !unreachable.is_empty() {
            *result = result.clone().with_warning(format!(
                "Unreachable nodes found: {:?}",
                unreachable
            ));
        }

        // Check for disconnected components
        let components = self.find_connected_components(graph);
        if components.len() > 1 {
            *result = result.clone().with_warning(format!(
                "Graph has {} disconnected components",
                components.len()
            ));
        }
    }

    /// Check if graph has cycles
    fn has_cycles(&self, _graph: &VisualGraph) -> bool {
        // TODO: Implement cycle detection
        false
    }

    /// Find unreachable nodes
    fn find_unreachable_nodes(&self, _graph: &VisualGraph) -> Vec<String> {
        // TODO: Implement unreachable node detection
        vec![]
    }

    /// Find connected components
    fn find_connected_components(&self, _graph: &VisualGraph) -> Vec<Vec<String>> {
        // TODO: Implement connected component detection
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid);

        result = result.with_error("Test error".to_string());
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);

        result = result.with_warning("Test warning".to_string());
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_validator_creation() {
        let validator = ContractValidator::new();
        assert!(validator.validate(&VisualGraph::new("test")).is_ok());
    }
} 