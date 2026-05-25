//! AI Assistant for pattern recognition and optimization

use crate::{
    config::Config,
    error::CanvasResult,
    types::{NodeId, NodeType, VisualGraph},
};

mod optimization;
mod pattern_recognition;
mod validator;

use optimization::OptimizationEngine;
use pattern_recognition::PatternRecognitionEngine;
use validator::RuleBasedValidator;

/// AI Assistant for analyzing and optimizing contracts
pub struct AiAssistant {
    _config: Config,
    pattern_engine: PatternRecognitionEngine,
    validator: RuleBasedValidator,
    optimizer: OptimizationEngine,
}

/// Pattern recognition result
#[derive(Debug, Clone)]
pub struct PatternAnalysis {
    pub patterns_found: Vec<ContractPattern>,
    pub anti_patterns: Vec<AntiPattern>,
    pub security_issues: Vec<SecurityIssue>,
    pub suggestions: Vec<String>,
}

/// Contract pattern
#[derive(Debug, Clone)]
pub struct ContractPattern {
    pub name: String,
    pub description: String,
    pub confidence: f64,
    pub nodes: Vec<NodeId>,
    pub category: PatternCategory,
}

/// Anti-pattern
#[derive(Debug, Clone)]
pub struct AntiPattern {
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub nodes: Vec<NodeId>,
    pub suggestion: String,
}

/// Security issue
#[derive(Debug, Clone)]
pub struct SecurityIssue {
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub nodes: Vec<NodeId>,
    pub cve_reference: Option<String>,
    pub mitigation: String,
}

/// Pattern category
#[derive(Debug, Clone)]
pub enum PatternCategory {
    Token,
    Voting,
    Escrow,
    Marketplace,
    Governance,
    Custom,
}

/// Severity level
#[derive(Debug, Clone)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Optimization suggestion
#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    pub title: String,
    pub description: String,
    pub estimated_gas_savings: u64,
    pub nodes: Vec<NodeId>,
    pub implementation: String,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

/// Optimization result
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub original_gas_estimate: u64,
    pub optimized_gas_estimate: u64,
    pub gas_savings: u64,
    pub suggestions: Vec<OptimizationSuggestion>,
    pub modified_graph: Option<VisualGraph>,
}

/// Node context for suggestions
#[derive(Debug, Clone)]
pub struct NodeContext {
    pub node_type: NodeType,
    pub connected_nodes: Vec<NodeId>,
    pub input_types: Vec<String>,
    pub output_types: Vec<String>,
    pub execution_path: Vec<NodeId>,
}

/// Node suggestion
#[derive(Debug, Clone)]
pub struct NodeSuggestion {
    pub node_type: NodeType,
    pub name: String,
    pub description: String,
    pub confidence: f64,
}

impl AiAssistant {
    /// Create a new AI assistant
    pub fn new(config: &Config) -> CanvasResult<Self> {
        Ok(Self {
            _config: config.clone(),
            pattern_engine: PatternRecognitionEngine::new(),
            validator: RuleBasedValidator::new(),
            optimizer: OptimizationEngine::new(),
        })
    }

    /// Analyze contract patterns
    pub fn analyze_patterns(&self, graph: &VisualGraph) -> CanvasResult<PatternAnalysis> {
        log::info!("Analyzing contract patterns");

        let patterns_found = self.pattern_engine.recognize_patterns(graph)?;
        let anti_patterns = self.pattern_engine.detect_anti_patterns(graph)?;
        let security_issues = self.pattern_engine.detect_security_issues(graph)?;
        let suggestions = self.generate_suggestions(graph, &patterns_found, &anti_patterns)?;

        Ok(PatternAnalysis {
            patterns_found,
            anti_patterns,
            security_issues,
            suggestions,
        })
    }

    /// Validate contract structure
    pub fn validate_contract(&self, graph: &VisualGraph) -> CanvasResult<ValidationResult> {
        log::info!("Validating contract structure");

        self.validator.validate(graph)
    }

    /// Optimize contract for gas efficiency
    pub fn optimize_contract(&self, graph: &VisualGraph) -> CanvasResult<OptimizationResult> {
        log::info!("Optimizing contract for gas efficiency");

        self.optimizer.optimize(graph)
    }

    /// Suggest next nodes based on context
    pub fn suggest_next_nodes(
        &self,
        graph: &VisualGraph,
        current_node: NodeId,
    ) -> CanvasResult<Vec<NodeSuggestion>> {
        log::info!("Suggesting next nodes for node {}", current_node);

        let context = self.analyze_context(graph, current_node)?;
        let suggestions = self.generate_node_suggestions(&context)?;

        Ok(suggestions)
    }

    /// Generate suggestions based on analysis
    fn generate_suggestions(
        &self,
        _graph: &VisualGraph,
        patterns: &[ContractPattern],
        anti_patterns: &[AntiPattern],
    ) -> CanvasResult<Vec<String>> {
        let mut suggestions = Vec::new();

        // Pattern-based suggestions
        for pattern in patterns {
            match pattern.category {
                PatternCategory::Token => {
                    suggestions.push("Consider adding transfer validation".to_string());
                    suggestions.push("Add balance checking before transfers".to_string());
                }
                PatternCategory::Voting => {
                    suggestions.push("Add vote deadline checking".to_string());
                    suggestions.push("Consider vote weight validation".to_string());
                }
                PatternCategory::Escrow => {
                    suggestions.push("Add timeout mechanism".to_string());
                    suggestions.push("Consider dispute resolution".to_string());
                }
                _ => {}
            }
        }

        // Anti-pattern based suggestions
        for anti_pattern in anti_patterns {
            suggestions.push(anti_pattern.suggestion.clone());
        }

        Ok(suggestions)
    }

    /// Analyze context around a node
    fn analyze_context(&self, graph: &VisualGraph, node_id: NodeId) -> CanvasResult<NodeContext> {
        let node = graph.get_node(node_id).ok_or_else(|| {
            crate::error::CanvasError::NodeNotFound(format!("Node {} not found", node_id))
        })?;

        let node_type = NodeType::from(node.node_type.as_str());

        // Find connected nodes
        let mut connected_nodes = Vec::new();
        for conn in &graph.connections {
            if conn.source_node == node_id {
                connected_nodes.push(conn.target_node);
            } else if conn.target_node == node_id {
                connected_nodes.push(conn.source_node);
            }
        }
        connected_nodes.sort();
        connected_nodes.dedup();

        // Input and Output types
        let input_types = node
            .inputs
            .iter()
            .map(|p| format!("{:?}", p.value_type))
            .collect();
        let output_types = node
            .outputs
            .iter()
            .map(|p| format!("{:?}", p.value_type))
            .collect();

        // Trace execution path backwards from current node to a root/Start node
        let mut execution_path = Vec::new();
        let mut current = node_id;
        execution_path.push(current);
        let mut visited = std::collections::HashSet::new();
        visited.insert(current);
        while let Some(pred) = graph
            .connections
            .iter()
            .find(|c| c.target_node == current && !visited.contains(&c.source_node))
            .map(|c| c.source_node)
        {
            execution_path.push(pred);
            visited.insert(pred);
            current = pred;
        }
        execution_path.reverse();

        Ok(NodeContext {
            node_type,
            connected_nodes,
            input_types,
            output_types,
            execution_path,
        })
    }

    /// Generate node suggestions based on context
    fn generate_node_suggestions(
        &self,
        context: &NodeContext,
    ) -> CanvasResult<Vec<NodeSuggestion>> {
        let mut suggestions = Vec::new();

        match context.node_type {
            NodeType::Logic => {
                suggestions.push(NodeSuggestion {
                    node_type: NodeType::State,
                    name: "Write Storage".to_string(),
                    description: "Store the result of your logic".to_string(),
                    confidence: 0.8,
                });
                suggestions.push(NodeSuggestion {
                    node_type: NodeType::Control,
                    name: "End".to_string(),
                    description: "End the execution flow".to_string(),
                    confidence: 0.6,
                });
            }
            NodeType::State => {
                suggestions.push(NodeSuggestion {
                    node_type: NodeType::External,
                    name: "Emit Event".to_string(),
                    description: "Notify about state changes".to_string(),
                    confidence: 0.7,
                });
            }
            _ => {}
        }

        Ok(suggestions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_ai_assistant_creation() {
        let config = Config::default();
        let ai = AiAssistant::new(&config);
        assert!(ai.is_ok());
    }

    #[test]
    fn test_pattern_analysis() {
        let config = Config::default();
        let ai = AiAssistant::new(&config).unwrap();
        let graph = VisualGraph::new("test");
        let result = ai.analyze_patterns(&graph);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contract_validation() {
        let config = Config::default();
        let ai = AiAssistant::new(&config).unwrap();
        let graph = VisualGraph::new("test");
        let result = ai.validate_contract(&graph);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contract_optimization() {
        let config = Config::default();
        let ai = AiAssistant::new(&config).unwrap();
        let graph = VisualGraph::new("test");
        let result = ai.optimize_contract(&graph);
        assert!(result.is_ok());
    }

    #[test]
    fn test_suggest_next_nodes_and_context() {
        let config = Config::default();
        let ai = AiAssistant::new(&config).unwrap();
        let mut graph = VisualGraph::new("test");

        let node_id = uuid::Uuid::new_v4();
        let node =
            crate::types::VisualNode::new(node_id, "If", crate::types::Position::new(0.0, 0.0))
                .with_inputs(vec![crate::types::Port::new(
                    "condition",
                    "Condition",
                    crate::types::ValueType::Boolean,
                )]);
        graph.add_node(node);

        let result = ai.suggest_next_nodes(&graph, node_id);
        assert!(result.is_ok());
        let suggestions = result.unwrap();
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].name, "Write Storage");
    }

    #[test]
    fn test_context_analysis_missing_node() {
        let config = Config::default();
        let ai = AiAssistant::new(&config).unwrap();
        let graph = VisualGraph::new("test");
        let result = ai.suggest_next_nodes(&graph, uuid::Uuid::new_v4());
        assert!(result.is_err());
    }

    #[test]
    fn test_contract_optimization_graph_rewrite() {
        let config = Config::default();
        let ai = AiAssistant::new(&config).unwrap();
        let mut graph = VisualGraph::new("test");

        let n1_id = uuid::Uuid::new_v4();
        let n2_id = uuid::Uuid::new_v4();
        let n1 = crate::types::VisualNode::new(n1_id, "Add", crate::types::Position::new(0.0, 0.0));
        let n2 =
            crate::types::VisualNode::new(n2_id, "Subtract", crate::types::Position::new(0.0, 0.0));

        graph.add_node(n1);
        graph.add_node(n2);

        let conn = crate::types::Connection::new(
            uuid::Uuid::new_v4(),
            n1_id,
            "flow_out",
            n2_id,
            "flow_in",
        );
        graph.add_connection(conn);

        let result = ai.optimize_contract(&graph).unwrap();
        assert!(result.modified_graph.is_some());

        let modified = result.modified_graph.unwrap();
        assert_eq!(modified.nodes.len(), 1);
    }
}
