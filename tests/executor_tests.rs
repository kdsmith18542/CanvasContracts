#[cfg(test)]
mod tests {
    use canvas_contracts::{
        compiler::GraphExecutor,
        nodes::NodeRegistry,
        types::{VisualGraph, VisualNode, Connection, ExecutionContext, Position, Port, ValueType},
    };
    use uuid::Uuid;

    fn setup_registry() -> NodeRegistry {
        let mut registry = NodeRegistry::new();
        for def in canvas_contracts::nodes::builtin_node_definitions() {
            registry.register_node(def);
        }
        registry
    }

    #[test]
    fn test_execute_simple_arithmetic() {
        let registry = setup_registry();
        let executor = GraphExecutor::new(registry);
        
        let mut graph = VisualGraph::new("Arithmetic Test");
        
        let start_id = Uuid::new_v4();
        let add_id = Uuid::new_v4();
        let end_id = Uuid::new_v4();
        
        graph.add_node(VisualNode::new(start_id, "Start", Position::new(0.0, 0.0))
            .with_outputs(vec![Port::new("flow_out", "Flow Out", ValueType::Flow)]));
            
        graph.add_node(VisualNode::new(add_id, "Add", Position::new(100.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow),
                Port::new("a", "A", ValueType::Integer).required(),
                Port::new("b", "B", ValueType::Integer).required()
            ])
            .with_outputs(vec![
                Port::new("flow_out", "Flow Out", ValueType::Flow),
                Port::new("result", "Result", ValueType::Integer)
            ])
            .with_property("a", serde_json::json!(10))
            .with_property("b", serde_json::json!(20)));
            
        graph.add_node(VisualNode::new(end_id, "End", Position::new(200.0, 0.0))
            .with_inputs(vec![Port::new("flow_in", "Flow In", ValueType::Flow).required()]));
            
        // Connections
        graph.add_connection(Connection::new(Uuid::new_v4(), start_id, "flow_out", add_id, "flow_in"));
        graph.add_connection(Connection::new(Uuid::new_v4(), add_id, "flow_out", end_id, "flow_in"));
        
        let context = ExecutionContext::new(1000);
        let (trace, _) = executor.execute(&graph, context).unwrap();
        
        assert!(trace.success);
        assert_eq!(trace.steps.len(), 3); // Start, Add, End
        
        // Find Add step
        let add_step = trace.steps.iter().find(|s| s.node_id == add_id).expect("Add node should have executed");
        assert_eq!(add_step.outputs["result"], serde_json::json!(30));
    }

    #[test]
    fn test_execute_branching_true() {
        let registry = setup_registry();
        let executor = GraphExecutor::new(registry);
        
        let mut graph = VisualGraph::new("Branching True Test");
        
        let start_id = Uuid::new_v4();
        let if_id = Uuid::new_v4();
        let write_true_id = Uuid::new_v4();
        let write_false_id = Uuid::new_v4();
        let end_id = Uuid::new_v4();
        
        graph.add_node(VisualNode::new(start_id, "Start", Position::new(0.0, 0.0))
            .with_outputs(vec![Port::new("flow_out", "Flow Out", ValueType::Flow)]));
            
        graph.add_node(VisualNode::new(if_id, "If", Position::new(100.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required(),
                Port::new("condition", "Condition", ValueType::Boolean).required()
            ])
            .with_outputs(vec![
                Port::new("true_flow", "True Flow", ValueType::Flow),
                Port::new("false_flow", "False Flow", ValueType::Flow)
            ])
            .with_property("condition", serde_json::json!(true)));
            
        graph.add_node(VisualNode::new(write_true_id, "WriteStorage", Position::new(200.0, -50.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required(),
                Port::new("key", "Key", ValueType::String).required(),
                Port::new("value", "Value", ValueType::Any).required()
            ])
            .with_property("key", serde_json::json!("path"))
            .with_property("value", serde_json::json!("true_branch")));

        graph.add_node(VisualNode::new(write_false_id, "WriteStorage", Position::new(200.0, 50.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required(),
                Port::new("key", "Key", ValueType::String).required(),
                Port::new("value", "Value", ValueType::Any).required()
            ])
            .with_property("key", serde_json::json!("path"))
            .with_property("value", serde_json::json!("false_branch")));
            
        graph.add_node(VisualNode::new(end_id, "End", Position::new(300.0, 0.0))
            .with_inputs(vec![Port::new("flow_in", "Flow In", ValueType::Flow).required()]));
            
        // Connections
        graph.add_connection(Connection::new(Uuid::new_v4(), start_id, "flow_out", if_id, "flow_in"));
        graph.add_connection(Connection::new(Uuid::new_v4(), if_id, "true_flow", write_true_id, "flow_in"));
        graph.add_connection(Connection::new(Uuid::new_v4(), if_id, "false_flow", write_false_id, "flow_in"));
        graph.add_connection(Connection::new(Uuid::new_v4(), write_true_id, "flow_out", end_id, "flow_in"));
        graph.add_connection(Connection::new(Uuid::new_v4(), write_false_id, "flow_out", end_id, "flow_in"));

        let context = ExecutionContext::new(1000);
        let (trace, final_ctx) = executor.execute(&graph, context).unwrap();
        
        assert!(trace.success);
        
        // Check which nodes executed
        let executed_ids: Vec<_> = trace.steps.iter().map(|s| s.node_id).collect();
        assert!(executed_ids.contains(&write_true_id));
        assert!(!executed_ids.contains(&write_false_id));
        
        // Verify storage
        assert_eq!(final_ctx.storage.get("path").unwrap(), &serde_json::json!("true_branch"));
    }

    #[test]
    fn test_execute_branching_false() {
        let registry = setup_registry();
        let executor = GraphExecutor::new(registry);
        
        let mut graph = VisualGraph::new("Branching False Test");
        
        let start_id = Uuid::new_v4();
        let if_id = Uuid::new_v4();
        let write_true_id = Uuid::new_v4();
        let write_false_id = Uuid::new_v4();
        let end_id = Uuid::new_v4();
        
        graph.add_node(VisualNode::new(start_id, "Start", Position::new(0.0, 0.0))
            .with_outputs(vec![Port::new("flow_out", "Flow Out", ValueType::Flow)]));
            
        graph.add_node(VisualNode::new(if_id, "If", Position::new(100.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required(),
                Port::new("condition", "Condition", ValueType::Boolean).required()
            ])
            .with_outputs(vec![
                Port::new("true_flow", "True Flow", ValueType::Flow),
                Port::new("false_flow", "False Flow", ValueType::Flow)
            ])
            .with_property("condition", serde_json::json!(false)));
            
        graph.add_node(VisualNode::new(write_true_id, "WriteStorage", Position::new(200.0, -50.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required(),
                Port::new("key", "Key", ValueType::String).required(),
                Port::new("value", "Value", ValueType::Any).required()
            ])
            .with_property("key", serde_json::json!("path"))
            .with_property("value", serde_json::json!("true_branch")));

        graph.add_node(VisualNode::new(write_false_id, "WriteStorage", Position::new(200.0, 50.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required(),
                Port::new("key", "Key", ValueType::String).required(),
                Port::new("value", "Value", ValueType::Any).required()
            ])
            .with_property("key", serde_json::json!("path"))
            .with_property("value", serde_json::json!("false_branch")));
            
        graph.add_node(VisualNode::new(end_id, "End", Position::new(300.0, 0.0))
            .with_inputs(vec![Port::new("flow_in", "Flow In", ValueType::Flow).required()]));
            
        // Connections
        graph.add_connection(Connection::new(Uuid::new_v4(), start_id, "flow_out", if_id, "flow_in"));
        graph.add_connection(Connection::new(Uuid::new_v4(), if_id, "true_flow", write_true_id, "flow_in"));
        graph.add_connection(Connection::new(Uuid::new_v4(), if_id, "false_flow", write_false_id, "flow_in"));
        graph.add_connection(Connection::new(Uuid::new_v4(), write_true_id, "flow_out", end_id, "flow_in"));
        graph.add_connection(Connection::new(Uuid::new_v4(), write_false_id, "flow_out", end_id, "flow_in"));

        let context = ExecutionContext::new(1000);
        let (trace, final_ctx) = executor.execute(&graph, context).unwrap();
        
        assert!(trace.success);
        
        // Check which nodes executed
        let executed_ids: Vec<_> = trace.steps.iter().map(|s| s.node_id).collect();
        assert!(!executed_ids.contains(&write_true_id));
        assert!(executed_ids.contains(&write_false_id));
        
        // Verify storage
        assert_eq!(final_ctx.storage.get("path").unwrap(), &serde_json::json!("false_branch"));
    }
}
