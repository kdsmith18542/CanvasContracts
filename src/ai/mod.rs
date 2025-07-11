//! AI Assistant for pattern recognition and optimization

use crate::{
    config::Config,
    error::{CanvasError, CanvasResult},
    types::{Graph, NodeId, NodeType},
};

mod pattern_recognition;
mod optimization;
mod validator;

use pattern_recognition::PatternRecognitionEngine;
use optimization::OptimizationEngine;
use validator::RuleBasedValidator;

/// AI Assistant for analyzing and optimizing contracts
pub struct AiAssistant {
    config: Config,
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
    pub modified_graph: Option<Graph>,
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
            config: config.clone(),
            pattern_engine: PatternRecognitionEngine::new(),
            validator: RuleBasedValidator::new(),
            optimizer: OptimizationEngine::new(),
        })
    }

    /// Analyze contract patterns
    pub fn analyze_patterns(&self, graph: &Graph) -> CanvasResult<PatternAnalysis> {
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
    pub fn validate_contract(&self, graph: &Graph) -> CanvasResult<ValidationResult> {
        log::info!("Validating contract structure");
        
        self.validator.validate(graph)
    }

    /// Optimize contract for gas efficiency
    pub fn optimize_contract(&self, graph: &Graph) -> CanvasResult<OptimizationResult> {
        log::info!("Optimizing contract for gas efficiency");
        
        self.optimizer.optimize(graph)
    }

    /// Suggest next nodes based on context
    pub fn suggest_next_nodes(&self, graph: &Graph, current_node: NodeId) -> CanvasResult<Vec<NodeSuggestion>> {
        log::info!("Suggesting next nodes for node {}", current_node);
        
        let context = self.analyze_context(graph, current_node)?;
        let suggestions = self.generate_node_suggestions(&context)?;
        
        Ok(suggestions)
    }

    /// Generate suggestions based on analysis
    fn generate_suggestions(
        &self,
        graph: &Graph,
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
    fn analyze_context(&self, graph: &Graph, node_id: NodeId) -> CanvasResult<NodeContext> {
        // TODO: Implement context analysis
        // For now, return a basic context
        
        Ok(NodeContext {
            node_type: NodeType::Logic,
            connected_nodes: vec![],
            input_types: vec![],
            output_types: vec![],
            execution_path: vec![],
        })
    }

    /// Generate node suggestions based on context
    fn generate_node_suggestions(&self, context: &NodeContext) -> CanvasResult<Vec<NodeSuggestion>> {
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
        let graph = Graph::new();
        let result = ai.analyze_patterns(&graph);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contract_validation() {
        let config = Config::default();
        let ai = AiAssistant::new(&config).unwrap();
        let graph = Graph::new();
        let result = ai.validate_contract(&graph);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contract_optimization() {
        let config = Config::default();
        let ai = AiAssistant::new(&config).unwrap();
        let graph = Graph::new();
        let result = ai.optimize_contract(&graph);
        assert!(result.is_ok());
    }
} 