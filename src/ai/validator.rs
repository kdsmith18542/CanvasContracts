use crate::{
    error::CanvasResult,
    types::{Graph, NodeId, NodeType},
};

use super::ValidationResult;

/// Rule-based validator for contract structure and security
pub struct RuleBasedValidator {
    validation_rules: Vec<ValidationRule>,
    security_rules: Vec<SecurityRule>,
}

/// Validation rule
#[derive(Debug, Clone)]
struct ValidationRule {
    name: String,
    description: String,
    rule_type: RuleType,
    severity: RuleSeverity,
    check: fn(&Graph) -> ValidationCheckResult,
}

/// Security rule
#[derive(Debug, Clone)]
struct SecurityRule {
    name: String,
    description: String,
    cve_reference: Option<String>,
    severity: RuleSeverity,
    check: fn(&Graph) -> SecurityCheckResult,
}

/// Rule type
#[derive(Debug, Clone)]
enum RuleType {
    Structure,
    Logic,
    Security,
    Performance,
}

/// Rule severity
#[derive(Debug, Clone)]
enum RuleSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Validation check result
#[derive(Debug, Clone)]
struct ValidationCheckResult {
    passed: bool,
    message: String,
    affected_nodes: Vec<NodeId>,
}

/// Security check result
#[derive(Debug, Clone)]
struct SecurityCheckResult {
    passed: bool,
    message: String,
    affected_nodes: Vec<NodeId>,
    cve_reference: Option<String>,
    mitigation: String,
}

impl RuleBasedValidator {
    pub fn new() -> Self {
        let validation_rules = Self::create_validation_rules();
        let security_rules = Self::create_security_rules();

        Self {
            validation_rules,
            security_rules,
        }
    }

    /// Validate contract structure
    pub fn validate(&self, graph: &Graph) -> CanvasResult<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut info = Vec::new();

        // Run validation rules
        for rule in &self.validation_rules {
            let result = (rule.check)(graph);
            if !result.passed {
                let message = format!("{}: {}", rule.name, result.message);
                match rule.severity {
                    RuleSeverity::Info => info.push(message),
                    RuleSeverity::Warning => warnings.push(message),
                    RuleSeverity::Error => errors.push(message),
                    RuleSeverity::Critical => errors.push(format!("CRITICAL: {}", message)),
                }
            }
        }

        // Run security rules
        for rule in &self.security_rules {
            let result = (rule.check)(graph);
            if !result.passed {
                let message = format!("SECURITY: {} - {}", rule.name, result.message);
                if let Some(cve) = &result.cve_reference {
                    let message = format!("{} (CVE: {})", message, cve);
                    match rule.severity {
                        RuleSeverity::Info => info.push(message),
                        RuleSeverity::Warning => warnings.push(message),
                        RuleSeverity::Error => errors.push(message),
                        RuleSeverity::Critical => errors.push(format!("CRITICAL: {}", message)),
                    }
                } else {
                    match rule.severity {
                        RuleSeverity::Info => info.push(message),
                        RuleSeverity::Warning => warnings.push(message),
                        RuleSeverity::Error => errors.push(message),
                        RuleSeverity::Critical => errors.push(format!("CRITICAL: {}", message)),
                    }
                }
            }
        }

        let is_valid = errors.is_empty();

        Ok(ValidationResult {
            is_valid,
            errors,
            warnings,
            info,
        })
    }

    /// Create validation rules
    fn create_validation_rules() -> Vec<ValidationRule> {
        vec![
            // Check for cycles in the graph
            ValidationRule {
                name: "No Cycles".to_string(),
                description: "Contract should not have cycles in execution flow".to_string(),
                rule_type: RuleType::Structure,
                severity: RuleSeverity::Error,
                check: |graph| {
                    if Self::has_cycles(graph) {
                        ValidationCheckResult {
                            passed: false,
                            message: "Contract contains cycles which may cause infinite loops".to_string(),
                            affected_nodes: vec![],
                        }
                    } else {
                        ValidationCheckResult {
                            passed: true,
                            message: "No cycles detected".to_string(),
                            affected_nodes: vec![],
                        }
                    }
                },
            },
            // Check for unreachable nodes
            ValidationRule {
                name: "No Unreachable Nodes".to_string(),
                description: "All nodes should be reachable from the start node".to_string(),
                rule_type: RuleType::Structure,
                severity: RuleSeverity::Warning,
                check: |graph| {
                    let unreachable = Self::find_unreachable_nodes(graph);
                    if unreachable.is_empty() {
                        ValidationCheckResult {
                            passed: true,
                            message: "All nodes are reachable".to_string(),
                            affected_nodes: vec![],
                        }
                    } else {
                        ValidationCheckResult {
                            passed: false,
                            message: format!("Found {} unreachable nodes", unreachable.len()),
                            affected_nodes: unreachable,
                        }
                    }
                },
            },
            // Check for missing inputs
            ValidationRule {
                name: "All Inputs Connected".to_string(),
                description: "All required inputs should be connected".to_string(),
                rule_type: RuleType::Logic,
                severity: RuleSeverity::Error,
                check: |graph| {
                    let missing = Self::find_missing_inputs(graph);
                    if missing.is_empty() {
                        ValidationCheckResult {
                            passed: true,
                            message: "All required inputs are connected".to_string(),
                            affected_nodes: vec![],
                        }
                    } else {
                        ValidationCheckResult {
                            passed: false,
                            message: format!("Found {} nodes with missing inputs", missing.len()),
                            affected_nodes: missing,
                        }
                    }
                },
            },
            // Check for proper start/end nodes
            ValidationRule {
                name: "Start and End Nodes".to_string(),
                description: "Contract should have exactly one start and one end node".to_string(),
                rule_type: RuleType::Structure,
                severity: RuleSeverity::Error,
                check: |graph| {
                    let nodes = graph.get_nodes();
                    let start_nodes: Vec<_> = nodes.iter().filter(|n| n.node_type == NodeType::Start).collect();
                    let end_nodes: Vec<_> = nodes.iter().filter(|n| n.node_type == NodeType::End).collect();

                    if start_nodes.len() == 1 && end_nodes.len() == 1 {
                        ValidationCheckResult {
                            passed: true,
                            message: "Contract has proper start and end nodes".to_string(),
                            affected_nodes: vec![],
                        }
                    } else {
                        ValidationCheckResult {
                            passed: false,
                            message: format!("Expected 1 start and 1 end node, found {} start and {} end", 
                                           start_nodes.len(), end_nodes.len()),
                            affected_nodes: vec![],
                        }
                    }
                },
            },
            // Check for reasonable node count
            ValidationRule {
                name: "Reasonable Complexity".to_string(),
                description: "Contract should not be overly complex".to_string(),
                rule_type: RuleType::Performance,
                severity: RuleSeverity::Warning,
                check: |graph| {
                    let node_count = graph.get_nodes().len();
                    if node_count <= 50 {
                        ValidationCheckResult {
                            passed: true,
                            message: format!("Contract has {} nodes (within reasonable limits)", node_count),
                            affected_nodes: vec![],
                        }
                    } else {
                        ValidationCheckResult {
                            passed: false,
                            message: format!("Contract has {} nodes (consider breaking into smaller contracts)", node_count),
                            affected_nodes: vec![],
                        }
                    }
                },
            },
        ]
    }

    /// Create security rules
    fn create_security_rules() -> Vec<SecurityRule> {
        vec![
            // Check for reentrancy vulnerabilities
            SecurityRule {
                name: "Reentrancy Protection".to_string(),
                description: "External calls should not be followed by state changes".to_string(),
                cve_reference: Some("CVE-2016-10709".to_string()),
                severity: RuleSeverity::Critical,
                check: |graph| {
                    if Self::has_reentrancy_risk(graph) {
                        SecurityCheckResult {
                            passed: false,
                            message: "Potential reentrancy vulnerability detected".to_string(),
                            affected_nodes: vec![],
                            cve_reference: Some("CVE-2016-10709".to_string()),
                            mitigation: "Update state before making external calls".to_string(),
                        }
                    } else {
                        SecurityCheckResult {
                            passed: true,
                            message: "No reentrancy vulnerabilities detected".to_string(),
                            affected_nodes: vec![],
                            cve_reference: None,
                            mitigation: String::new(),
                        }
                    }
                },
            },
            // Check for access control
            SecurityRule {
                name: "Access Control".to_string(),
                description: "State modifications should have proper access controls".to_string(),
                cve_reference: None,
                severity: RuleSeverity::High,
                check: |graph| {
                    if Self::has_access_control_issues(graph) {
                        SecurityCheckResult {
                            passed: false,
                            message: "Missing access controls on state modifications".to_string(),
                            affected_nodes: vec![],
                            cve_reference: None,
                            mitigation: "Add access control checks before state modifications".to_string(),
                        }
                    } else {
                        SecurityCheckResult {
                            passed: true,
                            message: "Access controls appear to be in place".to_string(),
                            affected_nodes: vec![],
                            cve_reference: None,
                            mitigation: String::new(),
                        }
                    }
                },
            },
            // Check for unchecked arithmetic
            SecurityRule {
                name: "Arithmetic Safety".to_string(),
                description: "Arithmetic operations should have overflow checks".to_string(),
                cve_reference: Some("CVE-2018-10299".to_string()),
                severity: RuleSeverity::High,
                check: |graph| {
                    if Self::has_unchecked_arithmetic(graph) {
                        SecurityCheckResult {
                            passed: false,
                            message: "Unchecked arithmetic operations detected".to_string(),
                            affected_nodes: vec![],
                            cve_reference: Some("CVE-2018-10299".to_string()),
                            mitigation: "Add overflow checks or use SafeMath library".to_string(),
                        }
                    } else {
                        SecurityCheckResult {
                            passed: true,
                            message: "Arithmetic operations appear to be safe".to_string(),
                            affected_nodes: vec![],
                            cve_reference: None,
                            mitigation: String::new(),
                        }
                    }
                },
            },
        ]
    }

    /// Check for cycles in the graph
    fn has_cycles(graph: &Graph) -> bool {
        // Simple cycle detection using DFS
        let nodes = graph.get_nodes();
        let edges = graph.get_edges();
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        fn dfs(
            node_id: &NodeId,
            nodes: &[crate::types::Node],
            edges: &[crate::types::Edge],
            visited: &mut std::collections::HashSet<NodeId>,
            rec_stack: &mut std::collections::HashSet<NodeId>,
        ) -> bool {
            if rec_stack.contains(node_id) {
                return true; // Cycle detected
            }
            if visited.contains(node_id) {
                return false;
            }

            visited.insert(node_id.clone());
            rec_stack.insert(node_id.clone());

            // Find all outgoing edges
            for edge in edges {
                if edge.source == *node_id {
                    if dfs(&edge.target, nodes, edges, visited, rec_stack) {
                        return true;
                    }
                }
            }

            rec_stack.remove(node_id);
            false
        }

        for node in nodes {
            if !visited.contains(&node.id) {
                if dfs(&node.id, nodes, edges, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }

        false
    }

    /// Find unreachable nodes
    fn find_unreachable_nodes(graph: &Graph) -> Vec<NodeId> {
        let nodes = graph.get_nodes();
        let edges = graph.get_edges();
        let mut reachable = std::collections::HashSet::new();

        // Find start nodes
        let start_nodes: Vec<_> = nodes.iter().filter(|n| n.node_type == NodeType::Start).collect();

        // BFS from start nodes
        let mut queue = std::collections::VecDeque::new();
        for start_node in start_nodes {
            queue.push_back(start_node.id.clone());
            reachable.insert(start_node.id.clone());
        }

        while let Some(current_id) = queue.pop_front() {
            for edge in edges {
                if edge.source == current_id && !reachable.contains(&edge.target) {
                    reachable.insert(edge.target.clone());
                    queue.push_back(edge.target.clone());
                }
            }
        }

        // Find unreachable nodes
        nodes
            .iter()
            .filter(|n| !reachable.contains(&n.id))
            .map(|n| n.id.clone())
            .collect()
    }

    /// Find nodes with missing inputs
    fn find_missing_inputs(graph: &Graph) -> Vec<NodeId> {
        let nodes = graph.get_nodes();
        let edges = graph.get_edges();
        let mut missing_inputs = Vec::new();

        for node in nodes {
            if node.node_type == NodeType::Start {
                continue; // Start node doesn't need inputs
            }

            // Count incoming edges
            let incoming_count = edges.iter().filter(|e| e.target == node.id).count();
            
            // Check if node has required inputs (simplified logic)
            let required_inputs = match node.node_type {
                NodeType::Logic => 2, // AND/OR operations need 2 inputs
                NodeType::Arithmetic => 2, // Arithmetic operations need 2 inputs
                NodeType::State => 1, // State operations need at least 1 input
                NodeType::External => 1, // External calls need at least 1 input
                NodeType::Control => 1, // Control flow needs 1 input
                NodeType::End => 1, // End node needs 1 input
                _ => 0,
            };

            if incoming_count < required_inputs {
                missing_inputs.push(node.id.clone());
            }
        }

        missing_inputs
    }

    /// Check for reentrancy risk
    fn has_reentrancy_risk(graph: &Graph) -> bool {
        let nodes = graph.get_nodes();
        let edges = graph.get_edges();

        // Look for patterns: External -> State
        for edge in edges {
            if let (Some(source), Some(target)) = (
                nodes.iter().find(|n| n.id == edge.source),
                nodes.iter().find(|n| n.id == edge.target),
            ) {
                if source.node_type == NodeType::External && target.node_type == NodeType::State {
                    return true;
                }
            }
        }

        false
    }

    /// Check for access control issues
    fn has_access_control_issues(graph: &Graph) -> bool {
        let nodes = graph.get_nodes();
        
        // Check if there are state nodes without obvious access control
        let state_nodes: Vec<_> = nodes.iter().filter(|n| n.node_type == NodeType::State).collect();
        
        // Simple heuristic: if there are many state operations, assume access control might be missing
        state_nodes.len() > 3
    }

    /// Check for unchecked arithmetic
    fn has_unchecked_arithmetic(graph: &Graph) -> bool {
        let nodes = graph.get_nodes();
        
        // Look for arithmetic nodes followed by state operations
        let edges = graph.get_edges();
        
        for edge in edges {
            if let (Some(source), Some(target)) = (
                nodes.iter().find(|n| n.id == edge.source),
                nodes.iter().find(|n| n.id == edge.target),
            ) {
                if source.node_type == NodeType::Arithmetic && target.node_type == NodeType::State {
                    return true;
                }
            }
        }

        false
    }
} 