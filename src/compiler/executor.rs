//! Graph execution engine
use crate::{
    error::{CanvasError, CanvasResult},
    nodes::{NodeContext, NodeRegistry},
    types::{
        ExecutionContext, ExecutionStep, ExecutionTrace, NodeId, PortId, ValueType, VisualGraph,
    },
};
use petgraph::algo::toposort;
use std::collections::HashMap;

/// Executor for visual graphs
pub struct GraphExecutor {
    registry: NodeRegistry,
}

impl GraphExecutor {
    /// Create a new graph executor with a node registry
    pub fn new(registry: NodeRegistry) -> Self {
        Self { registry }
    }

    /// Execute a visual graph end-to-end
    pub fn execute(
        &self,
        graph: &VisualGraph,
        mut context: ExecutionContext,
    ) -> CanvasResult<(ExecutionTrace, ExecutionContext)> {
        let ir = crate::compiler::graph_ir::GraphIR::from_visual_graph(graph);

        // Topological sort for execution order
        // This ensures data dependencies are met before a node executes.
        let order = toposort(&ir.graph, None)
            .map_err(|_| CanvasError::ExecutionError("Graph contains cycles".to_string()))?;

        let mut trace = ExecutionTrace::new();
        let mut node_outputs: HashMap<(NodeId, PortId), serde_json::Value> = HashMap::new();
        let mut step_counter = 0;

        for node_idx in order {
            let node_id = ir.graph[node_idx];
            let ir_node = ir.nodes.get(&node_id).ok_or_else(|| {
                CanvasError::ExecutionError(format!("Node not found in IR: {}", node_id))
            })?;

            let mut node_ctx = NodeContext::new(context.clone());

            // Map inputs from previous node outputs via connections
            let mut has_flow_inputs = false;
            let mut flow_signal_received = false;

            // First, check if this node has ANY flow connections in the graph
            for conn in &ir.connections {
                if conn.target == node_id {
                    if let Some(v_node) = graph.get_node(node_id) {
                        if let Some(port) = v_node.inputs.iter().find(|p| p.id == conn.target_port)
                        {
                            if port.value_type == ValueType::Flow {
                                has_flow_inputs = true;
                            }
                        }
                    }
                }
            }

            // Then, map all inputs and check for active flow signals
            for conn in &ir.connections {
                if conn.target == node_id {
                    if let Some(val) = node_outputs.get(&(conn.source, conn.source_port.clone())) {
                        node_ctx
                            .inputs
                            .insert(conn.target_port.clone(), val.clone());

                        // Check if this is an active flow signal
                        if let Some(v_node) = graph.get_node(node_id) {
                            if let Some(port) =
                                v_node.inputs.iter().find(|p| p.id == conn.target_port)
                            {
                                if port.value_type == ValueType::Flow && val.as_bool() == Some(true)
                                {
                                    flow_signal_received = true;
                                }
                            }
                        }
                    }
                }
            }

            // Fallback: Map properties to inputs if not already connected
            for (prop_key, prop_val) in &ir_node.properties {
                if !node_ctx.inputs.contains_key(prop_key) {
                    node_ctx.inputs.insert(prop_key.clone(), prop_val.clone());
                }
            }

            // Execution logic:
            // 1. Start nodes always execute.
            // 2. Nodes without flow inputs (pure data nodes) always execute.
            // 3. Nodes with flow inputs execute only if they receive a 'true' flow signal.
            let should_execute =
                ir_node.node_type == "Start" || !has_flow_inputs || flow_signal_received;

            if should_execute {
                let node_impl = self
                    .registry
                    .create_node(&ir_node.node_type, &ir_node.properties)?;

                let start_time = std::time::Instant::now();
                let result_res = node_impl.execute(&mut node_ctx);
                let duration = start_time.elapsed().as_millis() as u64;

                match result_res {
                    Ok(result) => {
                        // Record successful step
                        let step = ExecutionStep {
                            step_number: step_counter,
                            node_id,
                            node_type: ir_node.node_type.clone(),
                            inputs: node_ctx.inputs.clone(),
                            outputs: result.outputs.clone(),
                            gas_consumed: result.gas_used,
                            duration_ms: duration,
                            error: result.error.clone(),
                        };
                        trace.add_step(step);
                        step_counter += 1;

                        if let Some(err_msg) = result.error {
                            trace.success = false;
                            trace.error = Some(err_msg);
                            return Ok((trace, context));
                        }

                        // Store outputs for subsequent nodes
                        for (port_id, val) in result.outputs {
                            node_outputs.insert((node_id, port_id), val);
                        }

                        // Propagate flow_out for non-If nodes if not already present
                        if ir_node.node_type != "If" {
                            if let Some(v_node) = graph.get_node(node_id) {
                                for port in &v_node.outputs {
                                    if port.value_type == ValueType::Flow {
                                        node_outputs
                                            .entry((node_id, port.id.clone()))
                                            .or_insert(serde_json::Value::Bool(true));
                                    }
                                }
                            }
                        }

                        // Update execution context state
                        context = node_ctx.execution_context;
                    }
                    Err(e) => {
                        // Record failed step
                        let step = ExecutionStep {
                            step_number: step_counter,
                            node_id,
                            node_type: ir_node.node_type.clone(),
                            inputs: node_ctx.inputs.clone(),
                            outputs: HashMap::new(),
                            gas_consumed: 0,
                            duration_ms: duration,
                            error: Some(e.to_string()),
                        };
                        trace.add_step(step);
                        return Ok((trace, context));
                    }
                }
            } else {
                // Propagate false to all flow outputs if node is skipped
                if let Some(v_node) = graph.get_node(node_id) {
                    for port in &v_node.outputs {
                        if port.value_type == ValueType::Flow {
                            node_outputs
                                .insert((node_id, port.id.clone()), serde_json::Value::Bool(false));
                        }
                    }
                }
            }
        }

        Ok((trace, context))
    }
}
