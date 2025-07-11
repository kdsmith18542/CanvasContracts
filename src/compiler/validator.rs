//! Graph validation

use crate::{
    config::Config,
    error::{CanvasError, CanvasResult},
    types::{VisualGraph, VisualNode, Connection, ValueType},
};

use super::ValidationResult;

/// Graph validator
pub struct Validator {
    config: Config,
}

impl Validator {
    /// Create a new validator
    pub fn new(config: &Config) -> CanvasResult<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    /// Validate a visual graph
    pub fn validate(&self, graph: &VisualGraph) -> CanvasResult<ValidationResult> {
        let mut result = ValidationResult::valid();

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
            if input.required {
                // Check if this input is connected
                let is_connected = false; // TODO: Check actual connections
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

    /// Validate node properties
    fn validate_node_properties(&self, node: &VisualNode, result: &mut ValidationResult) {
        // TODO: Implement property validation based on node type
        match node.node_type.as_str() {
            "If" => {
                // Check if condition property exists
                if !node.properties.contains_key("condition") {
                    *result = result.clone().with_error(format!(
                        "If node {} missing required 'condition' property",
                        node.id
                    ));
                }
            }
            "WriteStorage" => {
                // Check if key property exists
                if !node.properties.contains_key("key") {
                    *result = result.clone().with_error(format!(
                        "WriteStorage node {} missing required 'key' property",
                        node.id
                    ));
                }
            }
            _ => {
                // Unknown node type - warning
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
                "Type mismatch in connection {}: {} -> {}",
                connection.id,
                format!("{:?}", source_port.value_type),
                format!("{:?}", target_port.value_type)
            ));
        }
    }

    /// Validate graph structure
    fn validate_graph_structure(&self, graph: &VisualGraph, result: &mut ValidationResult) {
        // Check for cycles (basic implementation)
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
    fn has_cycles(&self, graph: &VisualGraph) -> bool {
        // TODO: Implement cycle detection using DFS
        false
    }

    /// Find unreachable nodes
    fn find_unreachable_nodes(&self, graph: &VisualGraph) -> Vec<String> {
        // TODO: Implement reachability analysis
        Vec::new()
    }

    /// Find connected components
    fn find_connected_components(&self, graph: &VisualGraph) -> Vec<Vec<String>> {
        // TODO: Implement connected components analysis
        vec![graph.nodes.iter().map(|n| n.id.to_string()).collect()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{VisualNode, Position, Port, ValueType};
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
        validator.validate_node(&node, &mut result);
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
        validator.validate_node(&node, &mut result);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }
} 