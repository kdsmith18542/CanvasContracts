#[cfg(test)]
mod integration_tests {
    use canvas_contracts::{
        compiler::GraphExecutor,
        nodes::NodeRegistry,
        types::{VisualGraph, ExecutionContext},
    };

    fn setup_registry() -> NodeRegistry {
        let mut registry = NodeRegistry::new();
        for def in canvas_contracts::nodes::builtin_node_definitions() {
            registry.register_node(def);
        }
        registry
    }

    fn load_fixture(name: &str) -> VisualGraph {
        let path = format!("tests/fixtures/{}.json", name);
        let data = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path, e));
        serde_json::from_str(&data)
            .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", path, e))
    }

    // ── Fixture-based tests ──────────────────────────────────────────

    #[test]
    fn test_simple_arithmetic_fixture() {
        let graph = load_fixture("simple_arithmetic");
        let executor = GraphExecutor::new(setup_registry());
        let ctx = ExecutionContext::new(1000);
        let (trace, _) = executor.execute(&graph, ctx).unwrap();

        assert!(trace.success);
        // Start, Add, End = 3 steps
        assert_eq!(trace.steps.len(), 3);
        assert!(trace.steps.iter().any(|s| s.outputs.get("result") == Some(&serde_json::json!(30))));
    }

    #[test]
    fn test_branching_true_fixture() {
        let graph = load_fixture("branching_true");
        let executor = GraphExecutor::new(setup_registry());
        let ctx = ExecutionContext::new(1000);
        let (trace, final_ctx) = executor.execute(&graph, ctx).unwrap();

        assert!(trace.success);
        assert_eq!(
            final_ctx.storage.get("branch_result").unwrap(),
            &serde_json::json!("true_path")
        );
    }

    #[test]
    fn test_branching_false_fixture() {
        let graph = load_fixture("branching_false");
        let executor = GraphExecutor::new(setup_registry());
        let ctx = ExecutionContext::new(1000);
        let (trace, final_ctx) = executor.execute(&graph, ctx).unwrap();

        assert!(trace.success);
        assert_eq!(
            final_ctx.storage.get("branch_result").unwrap(),
            &serde_json::json!("false_path")
        );
    }

    #[test]
    fn test_multi_arithmetic_fixture() {
        let graph = load_fixture("multi_arithmetic");
        let executor = GraphExecutor::new(setup_registry());
        let ctx = ExecutionContext::new(1000);
        let (trace, _) = executor.execute(&graph, ctx).unwrap();

        assert!(trace.success);
        // Start, Add, Multiply, Subtract, End = 5 steps
        assert_eq!(trace.steps.len(), 5);
    }

    #[test]
    fn test_storage_rw_fixture() {
        let graph = load_fixture("storage_rw");
        let executor = GraphExecutor::new(setup_registry());
        let ctx = ExecutionContext::new(1000);
        let (trace, final_ctx) = executor.execute(&graph, ctx).unwrap();

        assert!(trace.success);
        assert_eq!(
            final_ctx.storage.get("stored_value").unwrap(),
            &serde_json::json!(42)
        );
    }

    // ── Edge case tests ──────────────────────────────────────────────

    #[test]
    fn test_gas_exhaustion_mid_execution() {
        let graph = load_fixture("simple_arithmetic");
        let executor = GraphExecutor::new(setup_registry());
        // Only 3 gas: Start costs 0, Add costs 3, End costs 0 — should pass
        let ctx = ExecutionContext::new(3);
        let (trace, _) = executor.execute(&graph, ctx).unwrap();
        assert!(trace.success);

        // Only 2 gas: Add costs 3 — executor returns Ok but trace shows failure
        let ctx = ExecutionContext::new(2);
        let (trace, _) = executor.execute(&graph, ctx).unwrap();
        assert!(!trace.success, "Trace should indicate failure due to gas exhaustion");
    }

    #[test]
    fn test_empty_graph() {
        let graph = VisualGraph::new("empty");
        let executor = GraphExecutor::new(setup_registry());
        let ctx = ExecutionContext::new(1000);
        let (trace, _) = executor.execute(&graph, ctx).unwrap();

        assert!(trace.success);
        assert_eq!(trace.steps.len(), 0);
    }

    #[test]
    fn test_graph_with_only_start() {
        let mut graph = VisualGraph::new("start_only");
        graph.add_node(canvas_contracts::types::VisualNode::new(
            uuid::Uuid::new_v4(), "Start",
            canvas_contracts::types::Position::new(0.0, 0.0)
        ).with_outputs(vec![canvas_contracts::types::Port::new(
            "flow_out", "Flow Out", canvas_contracts::types::ValueType::Flow
        )]));

        let executor = GraphExecutor::new(setup_registry());
        let ctx = ExecutionContext::new(1000);
        let (trace, _) = executor.execute(&graph, ctx).unwrap();

        assert!(trace.success);
        assert_eq!(trace.steps.len(), 1); // Start node alone
    }

    #[test]
    fn test_graph_missing_start_node() {
        let mut graph = VisualGraph::new("no_start");
        graph.add_node(canvas_contracts::types::VisualNode::new(
            uuid::Uuid::new_v4(), "End",
            canvas_contracts::types::Position::new(0.0, 0.0)
        ).with_inputs(vec![canvas_contracts::types::Port::new(
            "flow_in", "Flow In", canvas_contracts::types::ValueType::Flow
        ).required()]));

        let executor = GraphExecutor::new(setup_registry());
        let ctx = ExecutionContext::new(1000);
        let (trace, _) = executor.execute(&graph, ctx).unwrap();

        assert!(trace.success);
        // End node still executes via toposort
        assert_eq!(trace.steps.len(), 1);
        assert_eq!(trace.steps[0].node_type, "End");
    }
}
