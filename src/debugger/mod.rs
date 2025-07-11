//! Advanced debugging system for contract execution

use crate::{
    error::CanvasResult,
    types::{Graph, Node, NodeId, NodeType},
    wasm::WasmRuntime,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Debug session for contract execution
pub struct DebugSession {
    graph: Graph,
    runtime: WasmRuntime,
    breakpoints: Vec<Breakpoint>,
    execution_trace: Vec<ExecutionStep>,
    current_step: usize,
    is_paused: bool,
    variables: HashMap<String, serde_json::Value>,
    call_stack: Vec<CallStackFrame>,
}

/// Breakpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub node_id: NodeId,
    pub condition: Option<String>, // Optional condition expression
    pub enabled: bool,
    pub hit_count: u32,
}

/// Execution step in the trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_number: usize,
    pub node_id: NodeId,
    pub node_type: NodeType,
    pub timestamp: u64,
    pub inputs: HashMap<String, serde_json::Value>,
    pub outputs: HashMap<String, serde_json::Value>,
    pub gas_consumed: u64,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// Call stack frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallStackFrame {
    pub node_id: NodeId,
    pub function_name: String,
    pub line_number: Option<u32>,
    pub variables: HashMap<String, serde_json::Value>,
}

/// Debug configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub step_through: bool,
    pub log_variables: bool,
    pub log_gas: bool,
    pub log_performance: bool,
    pub max_steps: Option<usize>,
    pub timeout_ms: Option<u64>,
}

/// Debug state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugState {
    Running,
    Paused,
    Stepping,
    Finished,
    Error(String),
}

impl DebugSession {
    /// Create a new debug session
    pub fn new(graph: Graph, runtime: WasmRuntime) -> Self {
        Self {
            graph,
            runtime,
            breakpoints: Vec::new(),
            execution_trace: Vec::new(),
            current_step: 0,
            is_paused: false,
            variables: HashMap::new(),
            call_stack: Vec::new(),
        }
    }

    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, node_id: NodeId, condition: Option<String>) -> CanvasResult<()> {
        // Validate that the node exists
        if !self.graph.has_node(&node_id) {
            return Err(crate::error::CanvasError::NodeNotFound(node_id));
        }

        let breakpoint = Breakpoint {
            node_id,
            condition,
            enabled: true,
            hit_count: 0,
        };

        self.breakpoints.push(breakpoint);
        Ok(())
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, node_id: &NodeId) -> CanvasResult<()> {
        let index = self.breakpoints.iter().position(|bp| &bp.node_id == node_id);
        if let Some(idx) = index {
            self.breakpoints.remove(idx);
            Ok(())
        } else {
            Err(crate::error::CanvasError::BreakpointNotFound(node_id.clone()))
        }
    }

    /// Enable/disable a breakpoint
    pub fn toggle_breakpoint(&mut self, node_id: &NodeId, enabled: bool) -> CanvasResult<()> {
        if let Some(breakpoint) = self.breakpoints.iter_mut().find(|bp| &bp.node_id == node_id) {
            breakpoint.enabled = enabled;
            Ok(())
        } else {
            Err(crate::error::CanvasError::BreakpointNotFound(node_id.clone()))
        }
    }

    /// Start debugging with configuration
    pub fn start_debug(&mut self, config: DebugConfig) -> CanvasResult<DebugState> {
        self.execution_trace.clear();
        self.current_step = 0;
        self.is_paused = false;
        self.variables.clear();
        self.call_stack.clear();

        // Find start node
        let start_nodes: Vec<_> = self.graph.get_nodes()
            .iter()
            .filter(|n| n.node_type == NodeType::Start)
            .collect();

        if start_nodes.is_empty() {
            return Ok(DebugState::Error("No start node found".to_string()));
        }

        let start_node = start_nodes[0];
        self.execute_node(start_node, &config)?;

        Ok(DebugState::Running)
    }

    /// Continue execution
    pub fn continue_execution(&mut self, config: &DebugConfig) -> CanvasResult<DebugState> {
        self.is_paused = false;
        self.execute_remaining(config)
    }

    /// Step to next node
    pub fn step_next(&mut self, config: &DebugConfig) -> CanvasResult<DebugState> {
        if self.current_step >= self.execution_trace.len() {
            return Ok(DebugState::Finished);
        }

        let current_node = self.get_current_node()?;
        self.execute_node(current_node, config)?;
        self.current_step += 1;

        Ok(DebugState::Stepping)
    }

    /// Step into function (for composite nodes)
    pub fn step_into(&mut self, config: &DebugConfig) -> CanvasResult<DebugState> {
        let current_node = self.get_current_node()?;
        
        // Check if current node is a composite node
        if let Some(composite_data) = self.get_composite_node_data(current_node) {
            // Push current frame to call stack
            let frame = CallStackFrame {
                node_id: current_node.id.clone(),
                function_name: "composite".to_string(),
                line_number: None,
                variables: self.variables.clone(),
            };
            self.call_stack.push(frame);

            // Start debugging the composite node
            // TODO: Implement composite node debugging
            return Ok(DebugState::Stepping);
        }

        // If not composite, just step next
        self.step_next(config)
    }

    /// Step out of current function
    pub fn step_out(&mut self, config: &DebugConfig) -> CanvasResult<DebugState> {
        if let Some(frame) = self.call_stack.pop() {
            // Restore variables from the frame
            self.variables = frame.variables;
            return Ok(DebugState::Stepping);
        }

        // If no call stack, just continue
        self.continue_execution(config)
    }

    /// Get current execution state
    pub fn get_state(&self) -> DebugState {
        if self.is_paused {
            DebugState::Paused
        } else if self.current_step >= self.execution_trace.len() {
            DebugState::Finished
        } else {
            DebugState::Running
        }
    }

    /// Get execution trace
    pub fn get_trace(&self) -> &[ExecutionStep] {
        &self.execution_trace
    }

    /// Get current variables
    pub fn get_variables(&self) -> &HashMap<String, serde_json::Value> {
        &self.variables
    }

    /// Set variable value
    pub fn set_variable(&mut self, name: String, value: serde_json::Value) {
        self.variables.insert(name, value);
    }

    /// Get call stack
    pub fn get_call_stack(&self) -> &[CallStackFrame] {
        &self.call_stack
    }

    /// Get breakpoints
    pub fn get_breakpoints(&self) -> &[Breakpoint] {
        &self.breakpoints
    }

    /// Execute a single node
    fn execute_node(&mut self, node: &Node, config: &DebugConfig) -> CanvasResult<()> {
        let start_time = std::time::Instant::now();
        let start_gas = self.runtime.get_gas_consumed();

        // Check breakpoints
        if self.should_break_at_node(&node.id)? {
            self.is_paused = true;
            return Ok(());
        }

        // Execute the node
        let inputs = self.get_node_inputs(node)?;
        let outputs = self.execute_node_logic(node, &inputs)?;
        
        let end_time = std::time::Instant::now();
        let end_gas = self.runtime.get_gas_consumed();
        let duration = end_time.duration_since(start_time).as_millis() as u64;
        let gas_consumed = end_gas.saturating_sub(start_gas);

        // Record execution step
        let step = ExecutionStep {
            step_number: self.execution_trace.len(),
            node_id: node.id.clone(),
            node_type: node.node_type.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            inputs,
            outputs: outputs.clone(),
            gas_consumed,
            duration_ms: duration,
            error: None,
        };

        self.execution_trace.push(step);

        // Update variables
        for (key, value) in outputs {
            self.variables.insert(key, value);
        }

        // Log if configured
        if config.log_variables {
            log::debug!("Variables after node {}: {:?}", node.id, self.variables);
        }

        if config.log_gas {
            log::debug!("Gas consumed by node {}: {}", node.id, gas_consumed);
        }

        if config.log_performance {
            log::debug!("Node {} took {}ms", node.id, duration);
        }

        Ok(())
    }

    /// Execute remaining nodes
    fn execute_remaining(&mut self, config: &DebugConfig) -> CanvasResult<DebugState> {
        while self.current_step < self.execution_trace.len() && !self.is_paused {
            let current_node = self.get_current_node()?;
            self.execute_node(current_node, config)?;
            self.current_step += 1;

            // Check for timeout
            if let Some(timeout) = config.timeout_ms {
                if self.execution_trace.len() > timeout as usize {
                    return Ok(DebugState::Error("Execution timeout".to_string()));
                }
            }

            // Check for max steps
            if let Some(max_steps) = config.max_steps {
                if self.execution_trace.len() >= max_steps {
                    return Ok(DebugState::Error("Maximum steps exceeded".to_string()));
                }
            }
        }

        if self.is_paused {
            Ok(DebugState::Paused)
        } else {
            Ok(DebugState::Finished)
        }
    }

    /// Check if execution should break at a node
    fn should_break_at_node(&mut self, node_id: &NodeId) -> CanvasResult<bool> {
        for breakpoint in &mut self.breakpoints {
            if breakpoint.node_id == *node_id && breakpoint.enabled {
                breakpoint.hit_count += 1;

                // Check condition if specified
                if let Some(condition) = &breakpoint.condition {
                    if self.evaluate_condition(condition)? {
                        return Ok(true);
                    }
                } else {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Evaluate a breakpoint condition
    fn evaluate_condition(&self, condition: &str) -> CanvasResult<bool> {
        // TODO: Implement condition evaluation
        // This would parse and evaluate expressions like "gas_consumed > 1000"
        // For now, always return true
        Ok(true)
    }

    /// Get current node
    fn get_current_node(&self) -> CanvasResult<&Node> {
        if self.current_step >= self.execution_trace.len() {
            return Err(crate::error::CanvasError::ExecutionError(
                "No more nodes to execute".to_string()
            ));
        }

        let step = &self.execution_trace[self.current_step];
        self.graph.get_node(&step.node_id)
            .ok_or_else(|| crate::error::CanvasError::NodeNotFound(step.node_id.clone()))
    }

    /// Get node inputs
    fn get_node_inputs(&self, node: &Node) -> CanvasResult<HashMap<String, serde_json::Value>> {
        let mut inputs = HashMap::new();

        // Get inputs from connected nodes
        let edges = self.graph.get_edges();
        for edge in edges {
            if edge.target == node.id {
                if let Some(source_node) = self.graph.get_node(&edge.source) {
                    // Get output from source node
                    if let Some(output_value) = self.variables.get(&format!("{}_output", source_node.id)) {
                        inputs.insert(edge.source.clone(), output_value.clone());
                    }
                }
            }
        }

        Ok(inputs)
    }

    /// Execute node logic
    fn execute_node_logic(
        &self,
        node: &Node,
        inputs: &HashMap<String, serde_json::Value>,
    ) -> CanvasResult<HashMap<String, serde_json::Value>> {
        // TODO: Implement actual node execution logic
        // This would delegate to the appropriate node implementation
        
        let mut outputs = HashMap::new();
        outputs.insert(format!("{}_output", node.id), serde_json::Value::Null);
        
        Ok(outputs)
    }

    /// Get composite node data
    fn get_composite_node_data(&self, node: &Node) -> Option<String> {
        // TODO: Implement composite node data extraction
        // This would check if the node has composite data and return it
        None
    }
}

/// Debugger utilities
pub struct DebuggerUtils;

impl DebuggerUtils {
    /// Create a default debug configuration
    pub fn default_config() -> DebugConfig {
        DebugConfig {
            step_through: false,
            log_variables: true,
            log_gas: true,
            log_performance: false,
            max_steps: Some(1000),
            timeout_ms: Some(30000), // 30 seconds
        }
    }

    /// Create a step-through debug configuration
    pub fn step_through_config() -> DebugConfig {
        DebugConfig {
            step_through: true,
            log_variables: true,
            log_gas: true,
            log_performance: true,
            max_steps: None,
            timeout_ms: None,
        }
    }

    /// Analyze execution trace for performance issues
    pub fn analyze_performance(trace: &[ExecutionStep]) -> PerformanceAnalysis {
        let mut analysis = PerformanceAnalysis {
            total_gas: 0,
            total_time: 0,
            slowest_nodes: Vec::new(),
            most_expensive_nodes: Vec::new(),
            bottlenecks: Vec::new(),
        };

        for step in trace {
            analysis.total_gas += step.gas_consumed;
            analysis.total_time += step.duration_ms;
        }

        // Find slowest nodes
        let mut nodes_by_time: Vec<_> = trace.iter().collect();
        nodes_by_time.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms));
        analysis.slowest_nodes = nodes_by_time.iter().take(5).map(|s| s.node_id.clone()).collect();

        // Find most expensive nodes
        let mut nodes_by_gas: Vec<_> = trace.iter().collect();
        nodes_by_gas.sort_by(|a, b| b.gas_consumed.cmp(&a.gas_consumed));
        analysis.most_expensive_nodes = nodes_by_gas.iter().take(5).map(|s| s.node_id.clone()).collect();

        // Identify bottlenecks (nodes that are both slow and expensive)
        for step in trace {
            if step.duration_ms > 100 && step.gas_consumed > 1000 {
                analysis.bottlenecks.push(step.node_id.clone());
            }
        }

        analysis
    }
}

/// Performance analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    pub total_gas: u64,
    pub total_time: u64,
    pub slowest_nodes: Vec<NodeId>,
    pub most_expensive_nodes: Vec<NodeId>,
    pub bottlenecks: Vec<NodeId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Graph, Node, NodeType};

    #[test]
    fn test_debug_session_creation() {
        let graph = Graph::new();
        let runtime = WasmRuntime::new(&crate::config::Config::default()).unwrap();
        let session = DebugSession::new(graph, runtime);
        
        assert_eq!(session.get_state(), DebugState::Running);
        assert!(session.get_trace().is_empty());
    }

    #[test]
    fn test_breakpoint_management() {
        let graph = Graph::new();
        let runtime = WasmRuntime::new(&crate::config::Config::default()).unwrap();
        let mut session = DebugSession::new(graph, runtime);

        // Add breakpoint
        assert!(session.add_breakpoint("test-node".to_string(), None).is_ok());
        assert_eq!(session.get_breakpoints().len(), 1);

        // Remove breakpoint
        assert!(session.remove_breakpoint(&"test-node".to_string()).is_ok());
        assert_eq!(session.get_breakpoints().len(), 0);
    }

    #[test]
    fn test_debug_configurations() {
        let default_config = DebuggerUtils::default_config();
        assert!(!default_config.step_through);
        assert!(default_config.log_variables);

        let step_config = DebuggerUtils::step_through_config();
        assert!(step_config.step_through);
        assert!(step_config.log_variables);
    }
} 