use crate::{
    error::CanvasResult,
    types::{Connection, NodeId, NodeType, Position, VisualGraph, VisualNode},
};
use uuid::Uuid;

use super::{OptimizationResult, OptimizationSuggestion};

/// Optimization engine for gas efficiency
pub struct OptimizationEngine {
    gas_costs: GasCostTable,
    optimization_rules: Vec<OptimizationRule>,
}

/// Gas cost table for different operations
#[derive(Debug, Clone)]
struct GasCostTable {
    base_costs: std::collections::HashMap<NodeType, u64>,
    storage_costs: std::collections::HashMap<String, u64>,
    computation_costs: std::collections::HashMap<String, u64>,
}

/// Optimization rule
#[derive(Debug, Clone)]
struct OptimizationRule {
    name: String,
    description: String,
    pattern: Vec<NodeType>,
    replacement: Vec<NodeType>,
    gas_savings: u64,
    implementation: String,
}

impl OptimizationEngine {
    pub fn new() -> Self {
        let gas_costs = Self::create_gas_cost_table();
        let optimization_rules = Self::create_optimization_rules();

        Self {
            gas_costs,
            optimization_rules,
        }
    }

    /// Optimize contract for gas efficiency
    pub fn optimize(&self, graph: &VisualGraph) -> CanvasResult<OptimizationResult> {
        let original_gas = self.estimate_gas_usage(graph);
        let suggestions = self.generate_optimization_suggestions(graph)?;

        let total_savings: u64 = suggestions.iter().map(|s| s.estimated_gas_savings).sum();
        let optimized_gas = original_gas.saturating_sub(total_savings);

        let modified_graph = if !suggestions.is_empty() {
            Some(self.apply_optimizations(graph, &suggestions))
        } else {
            None
        };

        Ok(OptimizationResult {
            original_gas_estimate: original_gas,
            optimized_gas_estimate: optimized_gas,
            gas_savings: total_savings,
            suggestions,
            modified_graph,
        })
    }

    /// Estimate gas usage for a graph
    pub fn estimate_gas_usage(&self, graph: &VisualGraph) -> u64 {
        let nodes = graph.to_nodes();
        let mut total_gas = 0u64;

        for node in &nodes {
            // Base cost for node type
            if let Some(base_cost) = self.gas_costs.base_costs.get(&node.node_type) {
                total_gas += base_cost;
            }

            // Additional costs based on node properties
            total_gas += self.calculate_node_specific_costs(node);
        }

        // Edge costs (connections between nodes)
        let edges = graph.to_edges();
        total_gas += edges.len() as u64 * 10; // Base cost per connection

        total_gas
    }

    /// Generate optimization suggestions
    pub fn generate_optimization_suggestions(
        &self,
        graph: &VisualGraph,
    ) -> CanvasResult<Vec<OptimizationSuggestion>> {
        let mut suggestions = Vec::new();

        // Apply optimization rules
        for rule in &self.optimization_rules {
            if let Some(matching_nodes) = self.find_matching_pattern(graph, &rule.pattern) {
                suggestions.push(OptimizationSuggestion {
                    title: rule.name.clone(),
                    description: format!(
                        "{} (replacement pattern size: {})",
                        rule.description,
                        rule.replacement.len()
                    ),
                    estimated_gas_savings: rule.gas_savings,
                    nodes: matching_nodes,
                    implementation: rule.implementation.clone(),
                });
            }
        }

        // Custom optimizations based on graph analysis
        suggestions.extend(self.analyze_custom_optimizations(graph)?);

        Ok(suggestions)
    }

    /// Calculate node-specific gas costs
    fn calculate_node_specific_costs(&self, node: &crate::types::Node) -> u64 {
        let mut cost = 0u64;

        match node.node_type {
            NodeType::State => {
                // Storage operations are expensive
                cost += 20000; // SSTORE cost
                if let Some(op) = node.properties.get("operation").and_then(|v| v.as_str()) {
                    let key = op.to_ascii_lowercase();
                    if let Some(extra_cost) = self.gas_costs.storage_costs.get(&key) {
                        cost = cost.saturating_add(*extra_cost);
                    }
                }
            }
            NodeType::Arithmetic => {
                // Arithmetic operations are cheap
                cost += 3; // ADD/SUB cost
                if let Some(op) = node.properties.get("operation").and_then(|v| v.as_str()) {
                    let key = op.to_ascii_lowercase();
                    if let Some(extra_cost) = self.gas_costs.computation_costs.get(&key) {
                        cost = cost.saturating_add(*extra_cost);
                    }
                }
            }
            NodeType::Logic => {
                // Logic operations are very cheap
                cost += 1; // AND/OR cost
            }
            NodeType::External => {
                // External calls are expensive
                cost += 2600; // CALL cost
            }
            NodeType::Control => {
                // Control flow is cheap
                cost += 1; // JUMP cost
            }
            NodeType::Cryptographic => {
                // Crypto operations are moderate
                cost += 100;
            }
            NodeType::Time => {
                // Time operations are cheap
                cost += 10;
            }
            NodeType::Custom => {
                // Custom nodes vary widely
                cost += 100;
            }
            NodeType::Start | NodeType::End => {
                // Start/End have no cost
                cost += 0;
            }
            NodeType::Resurgence => {
                // Resurgence operations are moderate
                cost += 50;
            }
        }

        cost
    }

    /// Find nodes that match a pattern
    fn find_matching_pattern(
        &self,
        graph: &VisualGraph,
        pattern: &[NodeType],
    ) -> Option<Vec<NodeId>> {
        let nodes = graph.to_nodes();
        let mut matching_nodes = Vec::new();

        for window in nodes.windows(pattern.len()) {
            let window_types: Vec<NodeType> = window.iter().map(|n| n.node_type.clone()).collect();
            if window_types == pattern {
                matching_nodes.extend(window.iter().map(|n| n.id));
                return Some(matching_nodes);
            }
        }

        None
    }

    /// Analyze custom optimizations
    fn analyze_custom_optimizations(
        &self,
        graph: &VisualGraph,
    ) -> CanvasResult<Vec<OptimizationSuggestion>> {
        let mut suggestions = Vec::new();
        let nodes = graph.to_nodes();

        // Check for redundant state operations
        let state_nodes: Vec<_> = nodes
            .iter()
            .filter(|n| n.node_type == NodeType::State)
            .collect();
        if state_nodes.len() > 5 {
            suggestions.push(OptimizationSuggestion {
                title: "Reduce State Operations".to_string(),
                description: "Consider batching state operations to reduce gas costs".to_string(),
                estimated_gas_savings: (state_nodes.len() as u64 - 5) * 5000,
                nodes: state_nodes.iter().map(|n| n.id).collect(),
                implementation: "Batch multiple state updates into a single operation".to_string(),
            });
        }

        // Check for expensive external calls
        let external_nodes: Vec<_> = nodes
            .iter()
            .filter(|n| n.node_type == NodeType::External)
            .collect();
        if external_nodes.len() > 3 {
            suggestions.push(OptimizationSuggestion {
                title: "Optimize External Calls".to_string(),
                description: "Consider caching external call results".to_string(),
                estimated_gas_savings: (external_nodes.len() as u64 - 3) * 1000,
                nodes: external_nodes.iter().map(|n| n.id).collect(),
                implementation: "Cache external call results in state variables".to_string(),
            });
        }

        // Check for inefficient arithmetic patterns
        let arithmetic_nodes: Vec<_> = nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Arithmetic)
            .collect();
        if arithmetic_nodes.len() > 10 {
            suggestions.push(OptimizationSuggestion {
                title: "Optimize Arithmetic Operations".to_string(),
                description: "Consider using bit shifting for power-of-2 operations".to_string(),
                estimated_gas_savings: arithmetic_nodes.len() as u64 * 10,
                nodes: arithmetic_nodes.iter().map(|n| n.id).collect(),
                implementation: "Replace multiplication/division by powers of 2 with bit shifts"
                    .to_string(),
            });
        }

        Ok(suggestions)
    }

    /// Create gas cost table
    fn create_gas_cost_table() -> GasCostTable {
        let mut base_costs = std::collections::HashMap::new();
        base_costs.insert(NodeType::Start, 0);
        base_costs.insert(NodeType::End, 0);
        base_costs.insert(NodeType::State, 20000); // SSTORE
        base_costs.insert(NodeType::Logic, 1); // AND/OR
        base_costs.insert(NodeType::Arithmetic, 3); // ADD/SUB
        base_costs.insert(NodeType::External, 2600); // CALL
        base_costs.insert(NodeType::Control, 1); // JUMP

        let mut storage_costs = std::collections::HashMap::new();
        storage_costs.insert("sstore".to_string(), 20000);
        storage_costs.insert("sload".to_string(), 100);
        storage_costs.insert("balance".to_string(), 400);

        let mut computation_costs = std::collections::HashMap::new();
        computation_costs.insert("add".to_string(), 3);
        computation_costs.insert("sub".to_string(), 3);
        computation_costs.insert("mul".to_string(), 5);
        computation_costs.insert("div".to_string(), 5);
        computation_costs.insert("mod".to_string(), 5);

        GasCostTable {
            base_costs,
            storage_costs,
            computation_costs,
        }
    }

    /// Create optimization rules
    fn create_optimization_rules() -> Vec<OptimizationRule> {
        vec![
            // Replace multiple additions with single operation
            OptimizationRule {
                name: "Batch Arithmetic Operations".to_string(),
                description: "Combine multiple arithmetic operations into a single operation"
                    .to_string(),
                pattern: vec![NodeType::Arithmetic, NodeType::Arithmetic],
                replacement: vec![NodeType::Arithmetic],
                gas_savings: 3,
                implementation:
                    "Use compound assignment operators (e.g., a += b instead of a = a + b)"
                        .to_string(),
            },
            // Optimize storage access patterns
            OptimizationRule {
                name: "Optimize Storage Access".to_string(),
                description: "Cache frequently accessed storage values".to_string(),
                pattern: vec![NodeType::State, NodeType::Logic, NodeType::State],
                replacement: vec![NodeType::State, NodeType::Logic],
                gas_savings: 100,
                implementation: "Store storage value in memory variable for multiple uses"
                    .to_string(),
            },
            // Reduce external calls
            OptimizationRule {
                name: "Reduce External Calls".to_string(),
                description: "Cache external call results to avoid repeated calls".to_string(),
                pattern: vec![NodeType::External, NodeType::Logic, NodeType::External],
                replacement: vec![NodeType::External, NodeType::Logic],
                gas_savings: 2600,
                implementation: "Store external call result in state variable".to_string(),
            },
            // Optimize control flow
            OptimizationRule {
                name: "Optimize Control Flow".to_string(),
                description: "Simplify nested control structures".to_string(),
                pattern: vec![NodeType::Control, NodeType::Control],
                replacement: vec![NodeType::Control],
                gas_savings: 1,
                implementation: "Combine multiple conditions into a single expression".to_string(),
            },
        ]
    }

    /// Apply optimizations to construct a modified graph
    fn apply_optimizations(
        &self,
        graph: &VisualGraph,
        suggestions: &[OptimizationSuggestion],
    ) -> VisualGraph {
        let mut modified_graph = graph.clone();

        for suggestion in suggestions {
            let matched_ids = &suggestion.nodes;
            if matched_ids.is_empty() {
                continue;
            }

            // Find corresponding rule to know the replacement types
            let rule = self
                .optimization_rules
                .iter()
                .find(|r| r.name == suggestion.title);
            let replacement_types = match rule {
                Some(r) => &r.replacement,
                None => continue,
            };

            // Gather the matched nodes in the current state of modified_graph
            let matched_nodes: Vec<VisualNode> = modified_graph
                .nodes
                .iter()
                .filter(|n| matched_ids.contains(&n.id))
                .cloned()
                .collect();

            if matched_nodes.is_empty() {
                continue;
            }

            // Compute average position
            let avg_x = matched_nodes.iter().map(|n| n.position.x).sum::<f64>()
                / matched_nodes.len() as f64;
            let avg_y = matched_nodes.iter().map(|n| n.position.y).sum::<f64>()
                / matched_nodes.len() as f64;

            // Create replacement nodes
            let mut replacement_nodes = Vec::new();
            for rep_type in replacement_types {
                // Try to find a matched node with the same NodeType
                let matched_of_type = matched_nodes.iter().find(|n| {
                    let nt = NodeType::from(n.node_type.as_str());
                    nt == *rep_type
                });

                let new_node = match matched_of_type {
                    Some(m) => {
                        let mut cloned = m.clone();
                        cloned.id = Uuid::new_v4();
                        cloned.position = Position::new(avg_x, avg_y);
                        cloned
                    }
                    None => {
                        let type_str = match rep_type {
                            NodeType::Start => "Start",
                            NodeType::End => "End",
                            NodeType::Logic => "If",
                            NodeType::State => "ReadStorage",
                            NodeType::Arithmetic => "Add",
                            NodeType::Cryptographic => "VerifySignature",
                            NodeType::External => "EmitEvent",
                            NodeType::Time => "FetchChronoBlock",
                            NodeType::Resurgence => "CheckTokenAge",
                            _ => "Custom",
                        };
                        VisualNode::new(Uuid::new_v4(), type_str, Position::new(avg_x, avg_y))
                    }
                };
                replacement_nodes.push(new_node);
            }

            if replacement_nodes.is_empty() {
                continue;
            }

            let first_rep_id = replacement_nodes[0].id;
            let last_rep_id = replacement_nodes[replacement_nodes.len() - 1].id;

            let mut new_connections = Vec::new();
            for mut conn in std::mem::take(&mut modified_graph.connections) {
                let source_matched = matched_ids.contains(&conn.source_node);
                let target_matched = matched_ids.contains(&conn.target_node);

                if source_matched && target_matched {
                    // Internal connection, discard
                    continue;
                } else if source_matched {
                    // Outgoing connection, redirect source to the last replacement node
                    conn.source_node = last_rep_id;
                    new_connections.push(conn);
                } else if target_matched {
                    // Incoming connection, redirect target to the first replacement node
                    conn.target_node = first_rep_id;
                    new_connections.push(conn);
                } else {
                    new_connections.push(conn);
                }
            }

            // Add connections between replacement nodes
            for i in 0..replacement_nodes.len() - 1 {
                let conn = Connection::new(
                    Uuid::new_v4(),
                    replacement_nodes[i].id,
                    "flow_out".to_string(),
                    replacement_nodes[i + 1].id,
                    "flow_in".to_string(),
                );
                new_connections.push(conn);
            }

            // Remove matched nodes from graph
            modified_graph
                .nodes
                .retain(|n| !matched_ids.contains(&n.id));

            // Add replacement nodes to graph
            for node in replacement_nodes {
                modified_graph.add_node(node);
            }

            modified_graph.connections = new_connections;
        }

        modified_graph
    }
}
