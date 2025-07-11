use crate::{
    error::CanvasResult,
    types::{Graph, NodeId, NodeType, VisualGraph},
};

use super::{
    AntiPattern, ContractPattern, PatternCategory, SecurityIssue, Severity,
};

/// Pattern recognition engine using graph analysis
pub struct PatternRecognitionEngine {
    patterns: Vec<PatternDefinition>,
    anti_patterns: Vec<AntiPatternDefinition>,
    security_patterns: Vec<SecurityPatternDefinition>,
}

/// Pattern definition for recognition
#[derive(Debug, Clone)]
struct PatternDefinition {
    name: String,
    category: PatternCategory,
    description: String,
    node_sequence: Vec<NodeType>,
    required_connections: Vec<(NodeType, NodeType)>,
    optional_nodes: Vec<NodeType>,
}

/// Anti-pattern definition
#[derive(Debug, Clone)]
struct AntiPatternDefinition {
    name: String,
    description: String,
    severity: Severity,
    pattern: Vec<NodeType>,
    suggestion: String,
}

/// Security pattern definition
#[derive(Debug, Clone)]
struct SecurityPatternDefinition {
    name: String,
    description: String,
    severity: Severity,
    cve_reference: Option<String>,
    pattern: Vec<NodeType>,
    mitigation: String,
}

impl PatternRecognitionEngine {
    pub fn new() -> Self {
        let patterns = Self::define_patterns();
        let anti_patterns = Self::define_anti_patterns();
        let security_patterns = Self::define_security_patterns();

        Self {
            patterns,
            anti_patterns,
            security_patterns,
        }
    }

    /// Recognize contract patterns in the graph
    pub fn recognize_patterns(&self, graph: &Graph) -> CanvasResult<Vec<ContractPattern>> {
        let mut patterns_found = Vec::new();

        for pattern_def in &self.patterns {
            if let Some(confidence) = self.match_pattern(graph, pattern_def) {
                if confidence > 0.6 {
                    patterns_found.push(ContractPattern {
                        name: pattern_def.name.clone(),
                        description: pattern_def.description.clone(),
                        confidence,
                        nodes: self.find_pattern_nodes(graph, pattern_def),
                        category: pattern_def.category.clone(),
                    });
                }
            }
        }

        Ok(patterns_found)
    }

    /// Detect anti-patterns in the graph
    pub fn detect_anti_patterns(&self, graph: &Graph) -> CanvasResult<Vec<AntiPattern>> {
        let mut anti_patterns_found = Vec::new();

        for anti_pattern_def in &self.anti_patterns {
            if self.match_anti_pattern(graph, anti_pattern_def) {
                anti_patterns_found.push(AntiPattern {
                    name: anti_pattern_def.name.clone(),
                    description: anti_pattern_def.description.clone(),
                    severity: anti_pattern_def.severity.clone(),
                    nodes: self.find_anti_pattern_nodes(graph, anti_pattern_def),
                    suggestion: anti_pattern_def.suggestion.clone(),
                });
            }
        }

        Ok(anti_patterns_found)
    }

    /// Detect security issues in the graph
    pub fn detect_security_issues(&self, graph: &Graph) -> CanvasResult<Vec<SecurityIssue>> {
        let mut security_issues = Vec::new();

        for security_pattern_def in &self.security_patterns {
            if self.match_security_pattern(graph, security_pattern_def) {
                security_issues.push(SecurityIssue {
                    name: security_pattern_def.name.clone(),
                    description: security_pattern_def.description.clone(),
                    severity: security_pattern_def.severity.clone(),
                    nodes: self.find_security_pattern_nodes(graph, security_pattern_def),
                    cve_reference: security_pattern_def.cve_reference.clone(),
                    mitigation: security_pattern_def.mitigation.clone(),
                });
            }
        }

        Ok(security_issues)
    }

    /// Match a pattern against the graph
    fn match_pattern(&self, graph: &Graph, pattern: &PatternDefinition) -> Option<f64> {
        let nodes = graph.get_nodes();
        let mut matches = 0;
        let mut total_required = pattern.node_sequence.len();

        // Check for required node sequence
        for (i, required_type) in pattern.node_sequence.iter().enumerate() {
            if let Some(_) = nodes.iter().find(|node| node.node_type == *required_type) {
                matches += 1;
            }
        }

        // Check for required connections
        for (source_type, target_type) in &pattern.required_connections {
            if self.has_connection(graph, source_type, target_type) {
                matches += 1;
                total_required += 1;
            }
        }

        if total_required == 0 {
            return None;
        }

        let confidence = matches as f64 / total_required as f64;
        Some(confidence)
    }

    /// Match an anti-pattern against the graph
    fn match_anti_pattern(&self, graph: &Graph, anti_pattern: &AntiPatternDefinition) -> bool {
        let nodes = graph.get_nodes();
        
        // Check if the anti-pattern sequence exists
        for window in nodes.windows(anti_pattern.pattern.len()) {
            let window_types: Vec<NodeType> = window.iter().map(|n| n.node_type.clone()).collect();
            if window_types == anti_pattern.pattern {
                return true;
            }
        }

        false
    }

    /// Match a security pattern against the graph
    fn match_security_pattern(&self, graph: &Graph, security_pattern: &SecurityPatternDefinition) -> bool {
        let nodes = graph.get_nodes();
        
        // Check if the security pattern sequence exists
        for window in nodes.windows(security_pattern.pattern.len()) {
            let window_types: Vec<NodeType> = window.iter().map(|n| n.node_type.clone()).collect();
            if window_types == security_pattern.pattern {
                return true;
            }
        }

        false
    }

    /// Check if there's a connection between two node types
    fn has_connection(&self, graph: &Graph, source_type: &NodeType, target_type: &NodeType) -> bool {
        let edges = graph.get_edges();
        let nodes = graph.get_nodes();

        for edge in edges {
            if let (Some(source), Some(target)) = (
                nodes.iter().find(|n| n.id == edge.source),
                nodes.iter().find(|n| n.id == edge.target),
            ) {
                if source.node_type == *source_type && target.node_type == *target_type {
                    return true;
                }
            }
        }

        false
    }

    /// Find nodes that match a pattern
    fn find_pattern_nodes(&self, graph: &Graph, pattern: &PatternDefinition) -> Vec<NodeId> {
        let nodes = graph.get_nodes();
        let mut pattern_nodes = Vec::new();

        for node in nodes {
            if pattern.node_sequence.contains(&node.node_type) {
                pattern_nodes.push(node.id.clone());
            }
        }

        pattern_nodes
    }

    /// Find nodes that match an anti-pattern
    fn find_anti_pattern_nodes(&self, graph: &Graph, anti_pattern: &AntiPatternDefinition) -> Vec<NodeId> {
        let nodes = graph.get_nodes();
        let mut anti_pattern_nodes = Vec::new();

        for window in nodes.windows(anti_pattern.pattern.len()) {
            let window_types: Vec<NodeType> = window.iter().map(|n| n.node_type.clone()).collect();
            if window_types == anti_pattern.pattern {
                anti_pattern_nodes.extend(window.iter().map(|n| n.id.clone()));
            }
        }

        anti_pattern_nodes
    }

    /// Find nodes that match a security pattern
    fn find_security_pattern_nodes(&self, graph: &Graph, security_pattern: &SecurityPatternDefinition) -> Vec<NodeId> {
        let nodes = graph.get_nodes();
        let mut security_pattern_nodes = Vec::new();

        for window in nodes.windows(security_pattern.pattern.len()) {
            let window_types: Vec<NodeType> = window.iter().map(|n| n.node_type.clone()).collect();
            if window_types == security_pattern.pattern {
                security_pattern_nodes.extend(window.iter().map(|n| n.id.clone()));
            }
        }

        security_pattern_nodes
    }

    /// Define common contract patterns
    fn define_patterns() -> Vec<PatternDefinition> {
        vec![
            // ERC-20 Token Pattern
            PatternDefinition {
                name: "ERC-20 Token".to_string(),
                category: PatternCategory::Token,
                description: "Standard fungible token contract".to_string(),
                node_sequence: vec![
                    NodeType::State, // balance storage
                    NodeType::Logic, // transfer logic
                    NodeType::External, // transfer event
                ],
                required_connections: vec![
                    (NodeType::State, NodeType::Logic),
                    (NodeType::Logic, NodeType::External),
                ],
                optional_nodes: vec![NodeType::Control],
            },
            // Voting Pattern
            PatternDefinition {
                name: "Voting Mechanism".to_string(),
                category: PatternCategory::Voting,
                description: "Decentralized voting system".to_string(),
                node_sequence: vec![
                    NodeType::State, // vote storage
                    NodeType::Logic, // vote counting
                    NodeType::Control, // deadline check
                ],
                required_connections: vec![
                    (NodeType::State, NodeType::Logic),
                    (NodeType::Control, NodeType::Logic),
                ],
                optional_nodes: vec![NodeType::External],
            },
            // Escrow Pattern
            PatternDefinition {
                name: "Escrow Contract".to_string(),
                category: PatternCategory::Escrow,
                description: "Conditional payment system".to_string(),
                node_sequence: vec![
                    NodeType::State, // escrow storage
                    NodeType::Logic, // release logic
                    NodeType::Control, // timeout check
                ],
                required_connections: vec![
                    (NodeType::State, NodeType::Logic),
                    (NodeType::Control, NodeType::Logic),
                ],
                optional_nodes: vec![NodeType::External],
            },
        ]
    }

    /// Define anti-patterns
    fn define_anti_patterns() -> Vec<AntiPatternDefinition> {
        vec![
            // Unchecked arithmetic
            AntiPatternDefinition {
                name: "Unchecked Arithmetic".to_string(),
                description: "Arithmetic operations without overflow checks".to_string(),
                severity: Severity::High,
                pattern: vec![NodeType::Arithmetic, NodeType::State],
                suggestion: "Add overflow checks before arithmetic operations".to_string(),
            },
            // Reentrancy risk
            AntiPatternDefinition {
                name: "Reentrancy Risk".to_string(),
                description: "External calls before state updates".to_string(),
                severity: Severity::Critical,
                pattern: vec![NodeType::External, NodeType::State],
                suggestion: "Update state before making external calls".to_string(),
            },
            // Missing access control
            AntiPatternDefinition {
                name: "Missing Access Control".to_string(),
                description: "State modifications without permission checks".to_string(),
                severity: Severity::High,
                pattern: vec![NodeType::State],
                suggestion: "Add access control checks before state modifications".to_string(),
            },
        ]
    }

    /// Define security patterns
    fn define_security_patterns() -> Vec<SecurityPatternDefinition> {
        vec![
            // Integer overflow
            SecurityPatternDefinition {
                name: "Integer Overflow".to_string(),
                description: "Potential integer overflow in arithmetic operations".to_string(),
                severity: Severity::High,
                cve_reference: Some("CVE-2018-10299".to_string()),
                pattern: vec![NodeType::Arithmetic, NodeType::State],
                mitigation: "Use checked arithmetic operations or SafeMath library".to_string(),
            },
            // Reentrancy attack
            SecurityPatternDefinition {
                name: "Reentrancy Attack".to_string(),
                description: "Vulnerable to reentrancy attacks".to_string(),
                severity: Severity::Critical,
                cve_reference: Some("CVE-2016-10709".to_string()),
                pattern: vec![NodeType::External, NodeType::State],
                mitigation: "Follow checks-effects-interactions pattern".to_string(),
            },
            // Access control bypass
            SecurityPatternDefinition {
                name: "Access Control Bypass".to_string(),
                description: "Missing or insufficient access controls".to_string(),
                severity: Severity::High,
                cve_reference: None,
                pattern: vec![NodeType::State],
                mitigation: "Implement proper access control mechanisms".to_string(),
            },
        ]
    }
} 