use crate::{
    error::CanvasResult,
    types::{Graph, NodeId, NodeType},
};

use super::{OptimizationSuggestion, OptimizationResult};

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
    pub fn optimize(&self, graph: &Graph) -> CanvasResult<OptimizationResult> {
        let original_gas = self.estimate_gas_usage(graph);
        let suggestions = self.generate_optimization_suggestions(graph)?;
        
        let total_savings: u64 = suggestions.iter().map(|s| s.estimated_gas_savings).sum();
        let optimized_gas = original_gas.saturating_sub(total_savings);

        Ok(OptimizationResult {
            original_gas_estimate: original_gas,
            optimized_gas_estimate: optimized_gas,
            gas_savings: total_savings,
            suggestions,
            modified_graph: None, // TODO: Implement graph modification
        })
    }

    /// Estimate gas usage for a graph
    pub fn estimate_gas_usage(&self, graph: &Graph) -> u64 {
        let nodes = graph.get_nodes();
        let mut total_gas = 0u64;

        for node in nodes {
            // Base cost for node type
            if let Some(base_cost) = self.gas_costs.base_costs.get(&node.node_type) {
                total_gas += base_cost;
            }

            // Additional costs based on node properties
            total_gas += self.calculate_node_specific_costs(node);
        }

        // Edge costs (connections between nodes)
        let edges = graph.get_edges();
        total_gas += edges.len() as u64 * 10; // Base cost per connection

        total_gas
    }

    /// Generate optimization suggestions
    pub fn generate_optimization_suggestions(&self, graph: &Graph) -> CanvasResult<Vec<OptimizationSuggestion>> {
        let mut suggestions = Vec::new();

        // Apply optimization rules
        for rule in &self.optimization_rules {
            if let Some(matching_nodes) = self.find_matching_pattern(graph, &rule.pattern) {
                suggestions.push(OptimizationSuggestion {
                    title: rule.name.clone(),
                    description: rule.description.clone(),
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
            }
            NodeType::Arithmetic => {
                // Arithmetic operations are cheap
                cost += 3; // ADD/SUB cost
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
        }

        cost
    }

    /// Find nodes that match a pattern
    fn find_matching_pattern(&self, graph: &Graph, pattern: &[NodeType]) -> Option<Vec<NodeId>> {
        let nodes = graph.get_nodes();
        let mut matching_nodes = Vec::new();

        for window in nodes.windows(pattern.len()) {
            let window_types: Vec<NodeType> = window.iter().map(|n| n.node_type.clone()).collect();
            if window_types == pattern {
                matching_nodes.extend(window.iter().map(|n| n.id.clone()));
                return Some(matching_nodes);
            }
        }

        None
    }

    /// Analyze custom optimizations
    fn analyze_custom_optimizations(&self, graph: &Graph) -> CanvasResult<Vec<OptimizationSuggestion>> {
        let mut suggestions = Vec::new();
        let nodes = graph.get_nodes();

        // Check for redundant state operations
        let state_nodes: Vec<_> = nodes.iter().filter(|n| n.node_type == NodeType::State).collect();
        if state_nodes.len() > 5 {
            suggestions.push(OptimizationSuggestion {
                title: "Reduce State Operations".to_string(),
                description: "Consider batching state operations to reduce gas costs".to_string(),
                estimated_gas_savings: (state_nodes.len() as u64 - 5) * 5000,
                nodes: state_nodes.iter().map(|n| n.id.clone()).collect(),
                implementation: "Batch multiple state updates into a single operation".to_string(),
            });
        }

        // Check for expensive external calls
        let external_nodes: Vec<_> = nodes.iter().filter(|n| n.node_type == NodeType::External).collect();
        if external_nodes.len() > 3 {
            suggestions.push(OptimizationSuggestion {
                title: "Optimize External Calls".to_string(),
                description: "Consider caching external call results".to_string(),
                estimated_gas_savings: (external_nodes.len() as u64 - 3) * 1000,
                nodes: external_nodes.iter().map(|n| n.id.clone()).collect(),
                implementation: "Cache external call results in state variables".to_string(),
            });
        }

        // Check for inefficient arithmetic patterns
        let arithmetic_nodes: Vec<_> = nodes.iter().filter(|n| n.node_type == NodeType::Arithmetic).collect();
        if arithmetic_nodes.len() > 10 {
            suggestions.push(OptimizationSuggestion {
                title: "Optimize Arithmetic Operations".to_string(),
                description: "Consider using bit shifting for power-of-2 operations".to_string(),
                estimated_gas_savings: arithmetic_nodes.len() as u64 * 10,
                nodes: arithmetic_nodes.iter().map(|n| n.id.clone()).collect(),
                implementation: "Replace multiplication/division by powers of 2 with bit shifts".to_string(),
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
                description: "Combine multiple arithmetic operations into a single operation".to_string(),
                pattern: vec![NodeType::Arithmetic, NodeType::Arithmetic],
                replacement: vec![NodeType::Arithmetic],
                gas_savings: 3,
                implementation: "Use compound assignment operators (e.g., a += b instead of a = a + b)".to_string(),
            },
            // Optimize storage access patterns
            OptimizationRule {
                name: "Optimize Storage Access".to_string(),
                description: "Cache frequently accessed storage values".to_string(),
                pattern: vec![NodeType::State, NodeType::Logic, NodeType::State],
                replacement: vec![NodeType::State, NodeType::Logic],
                gas_savings: 100,
                implementation: "Store storage value in memory variable for multiple uses".to_string(),
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
} 