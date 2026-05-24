//! Graph validation

use crate::{
    config::Config,
    error::CanvasResult,
    types::{Connection, ValueType, VisualGraph, VisualNode},
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
                let is_connected = self.has_input_connection(graph, node.id, &input.id);
                if is_connected {
                    continue;
                }

                // Required flow ports must be connected.
                if matches!(input.value_type, ValueType::Flow) {
                    *result = result.clone().with_error(format!(
                        "Node {} has unconnected required flow input: {}",
                        node.id, input.name
                    ));
                    continue;
                }

                // Non-flow required ports can be provided either via connection or property.
                let has_property_value = node.properties.contains_key(&input.id);
                if !has_property_value {
                    *result = result.clone().with_error(format!(
                        "Node {} missing required input '{}' (provide connection or property '{}')",
                        node.id, input.name, input.id
                    ));
                }
            }
        }

        // Validate node properties
        self.validate_node_properties(node, graph, result);
    }

    /// Validate node properties based on node type
    fn validate_node_properties(
        &self,
        node: &VisualNode,
        graph: &VisualGraph,
        result: &mut ValidationResult,
    ) {
        match node.node_type.as_str() {
            "If" => {
                // Accept current key (`condition`) and legacy key (`condition_expression`),
                // or a connection into the `condition` input.
                let has_condition_property = node.properties.contains_key("condition")
                    || node.properties.contains_key("condition_expression");
                let has_condition_connection =
                    self.has_input_connection(graph, node.id, "condition");
                if !has_condition_property && !has_condition_connection {
                    *result = result.clone().with_error(format!(
                        "If node {} missing condition (provide input connection or property 'condition')",
                        node.id
                    ));
                }
            }
            "WriteStorage" => {
                let has_key_property = node.properties.contains_key("key");
                let has_key_connection = self.has_input_connection(graph, node.id, "key");
                if !has_key_property && !has_key_connection {
                    *result = result.clone().with_error(format!(
                        "WriteStorage node {} missing key (provide input connection or property 'key')",
                        node.id
                    ));
                }
            }
            "ReadStorage" => {
                let has_key_property = node.properties.contains_key("key");
                let has_key_connection = self.has_input_connection(graph, node.id, "key");
                if !has_key_property && !has_key_connection {
                    *result = result.clone().with_error(format!(
                        "ReadStorage node {} missing key (provide input connection or property 'key')",
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
            // BaaLS, ChronoNode, and Resurgence nodes
            "GetSender"
            | "GetContractId"
            | "GetBlockTimestamp"
            | "GetBlockHeight"
            | "EmitEvent"
            | "Revert"
            | "HashSha256"
            | "CallContract"
            | "ReadCallResult"
            | "TransferValue"
            | "FetchChronoBlock"
            | "FetchCheckpoint"
            | "VerifyChronoProof"
            | "ExtractChronoEvent"
            | "ExtractTxBySender"
            | "ExtractTxByRecipient"
            | "VerifyArchiveRange"
            | "CheckTokenAge"
            | "CheckTokenActivityWindow"
            | "CheckLiquidityDormancy"
            | "CheckGovernanceDormancy"
            | "CalculateDormancyScore"
            | "NormalizeDeadCoinRisk"
            | "GenerateDormancyProof"
            | "EmitDormancyOracleResult" => {}
            _ => {
                // Truly unknown node type
                *result = result
                    .clone()
                    .with_warning(format!("Unknown node type: {}", node.node_type));
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
        let source_port = source_node
            .outputs
            .iter()
            .find(|p| p.id == connection.source_port);
        if source_port.is_none() {
            *result = result.clone().with_error(format!(
                "Connection {} references non-existent source port: {}",
                connection.id, connection.source_port
            ));
            return;
        }

        // Check if target port exists
        let target_port = target_node
            .inputs
            .iter()
            .find(|p| p.id == connection.target_port);
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
        if !source_port
            .value_type
            .is_compatible_with(&target_port.value_type)
        {
            *result = result.clone().with_error(format!(
                "Type mismatch in connection {}: {:?} -> {:?}",
                connection.id, source_port.value_type, target_port.value_type,
            ));
        }
    }

    /// Validate graph structure
    fn validate_graph_structure(&self, graph: &VisualGraph, result: &mut ValidationResult) {
        let ir = crate::compiler::graph_ir::GraphIR::from_visual_graph(graph);

        // Check for cycles
        if petgraph::algo::is_cyclic_directed(&ir.graph) {
            *result = result
                .clone()
                .with_error("Graph contains cycles".to_string());
        }

        // Check for unreachable nodes
        let unreachable = self.find_unreachable_nodes(&ir, graph);
        if !unreachable.is_empty() {
            *result = result
                .clone()
                .with_warning(format!("Unreachable nodes found: {:?}", unreachable));
        }

        // Check for disconnected components
        // Weakly-connected components are the right signal for disconnected subgraphs.
        let components = petgraph::algo::connected_components(&ir.graph);
        if components > 1 && !graph.nodes.is_empty() {
            *result = result
                .clone()
                .with_warning(format!("Graph has {} disconnected components", components));
        }
    }

    fn has_input_connection(
        &self,
        graph: &VisualGraph,
        node_id: crate::types::NodeId,
        port_id: &str,
    ) -> bool {
        graph
            .connections
            .iter()
            .any(|c| c.target_node == node_id && c.target_port == port_id)
    }

    /// Find unreachable nodes starting from 'Start' nodes
    fn find_unreachable_nodes(
        &self,
        ir: &crate::compiler::graph_ir::GraphIR,
        graph: &VisualGraph,
    ) -> Vec<crate::types::NodeId> {
        let start_nodes: Vec<_> = ir
            .nodes
            .values()
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

        graph
            .nodes
            .iter()
            .map(|n| n.id)
            .filter(|id| !reachable.contains(id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Connection, Port, Position, ValueType, VisualNode};
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
        let mut node = VisualNode::new(Uuid::new_v4(), "If", Position::new(0.0, 0.0));
        node = node.with_property(
            "condition".to_string(),
            serde_json::Value::String("true".to_string()),
        );

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
        let node = VisualNode::new(Uuid::new_v4(), "If", Position::new(0.0, 0.0));

        let mut result = ValidationResult::valid();
        let graph = VisualGraph::new("test");
        validator.validate_node(&node, &graph, &mut result);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_required_non_flow_input_can_be_satisfied_by_property() {
        let config = Config::default();
        let validator = Validator::new(&config).unwrap();

        let add_id = Uuid::new_v4();
        let node = VisualNode::new(add_id, "Add", Position::new(0.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required(),
                Port::new("a", "A", ValueType::Integer).required(),
                Port::new("b", "B", ValueType::Integer).required(),
            ])
            .with_property("a", serde_json::json!(10))
            .with_property("b", serde_json::json!(20));

        let start_id = Uuid::new_v4();
        let start_node = VisualNode::new(start_id, "Start", Position::new(-100.0, 0.0))
            .with_outputs(vec![Port::new("flow_out", "Flow Out", ValueType::Flow)]);

        let mut graph = VisualGraph::new("property-inputs");
        graph.add_node(start_node);
        graph.add_node(node);
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            start_id,
            "flow_out",
            add_id,
            "flow_in",
        ));

        let validation = validator.validate(&graph).unwrap();
        assert!(
            validation.is_valid,
            "Validation should accept required non-flow inputs from properties: {:?}",
            validation.errors
        );
    }

    #[test]
    fn test_required_flow_input_still_requires_connection() {
        let config = Config::default();
        let validator = Validator::new(&config).unwrap();

        let add_node = VisualNode::new(Uuid::new_v4(), "Add", Position::new(0.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required(),
                Port::new("a", "A", ValueType::Integer).required(),
                Port::new("b", "B", ValueType::Integer).required(),
            ])
            .with_property("a", serde_json::json!(10))
            .with_property("b", serde_json::json!(20));

        let mut graph = VisualGraph::new("missing-flow");
        graph.add_node(add_node);

        let validation = validator.validate(&graph).unwrap();
        assert!(!validation.is_valid);
        assert!(validation
            .errors
            .iter()
            .any(|e| e.contains("required flow input")));
    }

    #[test]
    fn test_linear_graph_not_flagged_as_disconnected() {
        let config = Config::default();
        let validator = Validator::new(&config).unwrap();

        let start_id = Uuid::new_v4();
        let add_id = Uuid::new_v4();
        let end_id = Uuid::new_v4();

        let start = VisualNode::new(start_id, "Start", Position::new(0.0, 0.0))
            .with_outputs(vec![Port::new("flow_out", "Flow Out", ValueType::Flow)]);
        let add = VisualNode::new(add_id, "Add", Position::new(200.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required(),
                Port::new("a", "A", ValueType::Integer).required(),
                Port::new("b", "B", ValueType::Integer).required(),
            ])
            .with_outputs(vec![
                Port::new("flow_out", "Flow Out", ValueType::Flow),
                Port::new("result", "Result", ValueType::Integer),
            ])
            .with_property("a", serde_json::json!(10))
            .with_property("b", serde_json::json!(20));
        let end =
            VisualNode::new(end_id, "End", Position::new(400.0, 0.0)).with_inputs(vec![Port::new(
                "flow_in",
                "Flow In",
                ValueType::Flow,
            )
            .required()]);

        let mut graph = VisualGraph::new("linear");
        graph.add_node(start);
        graph.add_node(add);
        graph.add_node(end);
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            start_id,
            "flow_out",
            add_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            add_id,
            "flow_out",
            end_id,
            "flow_in",
        ));

        let validation = validator.validate(&graph).unwrap();
        assert!(validation.is_valid, "{:?}", validation.errors);
        assert!(!validation
            .warnings
            .iter()
            .any(|w| w.contains("disconnected components")));
    }
}
