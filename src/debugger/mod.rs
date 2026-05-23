use crate::{
    compiler::GraphExecutor,
    error::{CanvasError, CanvasResult},
    nodes::NodeRegistry,
    types::{ExecutionTrace, ExecutionStep, NodeId, VisualGraph},
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Debug session for contract execution
pub struct DebugSession {
    graph: VisualGraph,
    executor: GraphExecutor,
    breakpoints: Vec<Breakpoint>,
    current_step: usize,
    trace: ExecutionTrace,
    variables: HashMap<String, serde_json::Value>,
    is_paused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub node_id: NodeId,
    pub condition: Option<String>,
    pub enabled: bool,
    pub hit_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallStackFrame {
    pub node_id: NodeId,
    pub function_name: String,
    pub line_number: Option<u32>,
    pub variables: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub log_gas: bool,
    pub log_performance: bool,
    pub max_steps: Option<usize>,
    pub timeout_ms: Option<u64>,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            log_gas: true,
            log_performance: false,
            max_steps: Some(1000),
            timeout_ms: Some(30000),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DebugState {
    Running,
    Paused,
    Stepping,
    Finished,
    Error(String),
}

impl DebugSession {
    pub fn new(graph: VisualGraph, registry: NodeRegistry) -> Self {
        Self {
            executor: GraphExecutor::new(registry),
            graph,
            breakpoints: Vec::new(),
            current_step: 0,
            trace: ExecutionTrace::new(),
            variables: HashMap::new(),
            is_paused: false,
        }
    }

    pub fn add_breakpoint(&mut self, node_id: NodeId, condition: Option<String>) -> CanvasResult<()> {
        if !self.graph.nodes.iter().any(|n| n.id == node_id) {
            return Err(CanvasError::NodeNotFound(node_id.to_string()));
        }
        self.breakpoints.push(Breakpoint { node_id, condition, enabled: true, hit_count: 0 });
        Ok(())
    }

    pub fn remove_breakpoint(&mut self, node_id: &NodeId) -> CanvasResult<()> {
        let idx = self.breakpoints.iter().position(|bp| &bp.node_id == node_id);
        match idx {
            Some(i) => { self.breakpoints.remove(i); Ok(()) }
            None => Err(CanvasError::BreakpointNotFound(node_id.to_string())),
        }
    }

    pub fn start_debug(&mut self, _config: &DebugConfig) -> CanvasResult<DebugState> {
        let ctx = crate::types::ExecutionContext::new(1_000_000);
        let (trace, _) = self.executor.execute(&self.graph, ctx)?;
        self.trace = trace;
        self.current_step = 0;
        self.is_paused = false;

        if self.trace.steps.is_empty() {
            return Ok(DebugState::Error("No nodes to execute".to_string()));
        }

        if self.should_pause_at_current() {
            self.is_paused = true;
            return Ok(DebugState::Paused);
        }

        Ok(DebugState::Running)
    }

    pub fn step_next(&mut self) -> CanvasResult<DebugState> {
        if self.current_step >= self.trace.steps.len() {
            return Ok(DebugState::Finished);
        }
        let step = &self.trace.steps[self.current_step];
        for (k, v) in &step.outputs {
            self.variables.insert(k.clone(), v.clone());
        }
        self.current_step += 1;

        if self.current_step >= self.trace.steps.len() {
            return Ok(DebugState::Finished);
        }

        if self.should_pause_at_current() {
            self.is_paused = true;
            return Ok(DebugState::Paused);
        }

        Ok(DebugState::Stepping)
    }

    pub fn continue_execution(&mut self) -> CanvasResult<DebugState> {
        self.is_paused = false;
        while self.current_step < self.trace.steps.len() && !self.is_paused {
            let step = &self.trace.steps[self.current_step];
            for (k, v) in &step.outputs {
                self.variables.insert(k.clone(), v.clone());
            }
            self.current_step += 1;

            if self.should_pause_at_current() {
                self.is_paused = true;
            }
        }

        if self.is_paused { Ok(DebugState::Paused) }
        else { Ok(DebugState::Finished) }
    }

    pub fn get_state(&self) -> DebugState {
        if self.is_paused { DebugState::Paused }
        else if self.trace.steps.is_empty() || self.current_step >= self.trace.steps.len() {
            DebugState::Finished
        }
        else { DebugState::Running }
    }

    pub fn get_trace(&self) -> &[ExecutionStep] { &self.trace.steps }
    pub fn get_breakpoints(&self) -> &[Breakpoint] { &self.breakpoints }
    pub fn get_variables(&self) -> &HashMap<String, serde_json::Value> { &self.variables }

    fn should_pause_at_current(&self) -> bool {
        if self.current_step >= self.trace.steps.len() { return false; }
        let step = &self.trace.steps[self.current_step];
        self.breakpoints.iter().any(|bp| bp.node_id == step.node_id && bp.enabled)
    }
}

pub struct DebuggerUtils;

impl DebuggerUtils {
    pub fn analyze_performance(trace: &[ExecutionStep]) -> PerformanceAnalysis {
        let mut total_gas = 0u64;
        let mut total_time = 0u64;

        for step in trace {
            total_gas += step.gas_consumed;
            total_time += step.duration_ms;
        }

        let mut by_time: Vec<_> = trace.iter().collect();
        by_time.sort_by_key(|s| std::cmp::Reverse(s.duration_ms));
        let slowest = by_time.iter().take(5).map(|s| s.node_id).collect();

        let mut by_gas: Vec<_> = trace.iter().collect();
        by_gas.sort_by_key(|s| std::cmp::Reverse(s.gas_consumed));
        let most_expensive = by_gas.iter().take(5).map(|s| s.node_id).collect();

        let bottlenecks: Vec<_> = trace.iter()
            .filter(|s| s.duration_ms > 100 && s.gas_consumed > 1000)
            .map(|s| s.node_id)
            .collect();

        PerformanceAnalysis { total_gas, total_time, slowest_nodes: slowest, most_expensive_nodes: most_expensive, bottlenecks }
    }
}

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
    use crate::nodes::builtin_node_definitions;

    fn test_registry() -> NodeRegistry {
        let mut r = NodeRegistry::new();
        for d in builtin_node_definitions() { r.register_node(d); }
        r
    }

    fn test_graph() -> VisualGraph {
        crate::types::VisualGraph::new("test")
    }

    #[test]
    fn test_debug_session_creation() {
        let session = DebugSession::new(test_graph(), test_registry());
        assert_eq!(session.get_state(), DebugState::Finished);
    }

    #[test]
    fn test_breakpoint_management() {
        let mut session = DebugSession::new(test_graph(), test_registry());
        let fake_id = uuid::Uuid::new_v4();
        assert!(session.add_breakpoint(fake_id, None).is_err());
        assert_eq!(session.get_breakpoints().len(), 0);
    }

    #[test]
    fn test_start_debug_empty_graph() {
        let mut session = DebugSession::new(test_graph(), test_registry());
        let state = session.start_debug(&DebugConfig::default()).unwrap();
        assert_eq!(state, DebugState::Error("No nodes to execute".to_string()));
    }
}
