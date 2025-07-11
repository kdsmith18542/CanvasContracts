//! Tests for advanced features

use canvascontract::{
    nodes::custom::{CustomNodeRegistry, CustomNodeBuilder, CustomNodeDefinition},
    debugger::{DebugSession, DebuggerUtils, DebugConfig},
    types::{Graph, Node, NodeType},
    wasm::WasmRuntime,
    config::Config,
};

#[test]
fn test_custom_node_registry_operations() {
    let mut registry = CustomNodeRegistry::new();
    
    // Create a custom node
    let definition = CustomNodeBuilder::new(
        "test-custom-node".to_string(),
        "Test Custom Node".to_string(),
    )
    .description("A test custom node for testing".to_string())
    .category("Test".to_string())
    .input("input1".to_string(), "number".to_string(), true, "First input".to_string())
    .input("input2".to_string(), "string".to_string(), false, "Second input".to_string())
    .output("output1".to_string(), "boolean".to_string(), "First output".to_string())
    .property("threshold".to_string(), "number".to_string(), true, Some("100".to_string()), "Threshold value".to_string())
    .composite(r#"{"nodes": [], "edges": []}"#.to_string())
    .build();

    // Register the node
    assert!(registry.register_node(definition.clone()).is_ok());
    
    // Verify the node was registered
    assert!(registry.get_node("test-custom-node").is_some());
    assert_eq!(registry.list_nodes().len(), 1);
    
    // Test duplicate registration
    let duplicate = definition.clone();
    assert!(registry.register_node(duplicate).is_err());
    
    // Test node removal
    assert!(registry.remove_node("test-custom-node").is_ok());
    assert!(registry.get_node("test-custom-node").is_none());
    assert_eq!(registry.list_nodes().len(), 0);
}

#[test]
fn test_custom_node_builder() {
    let definition = CustomNodeBuilder::new(
        "builder-test".to_string(),
        "Builder Test Node".to_string(),
    )
    .description("Testing the builder pattern".to_string())
    .category("Builder".to_string())
    .input("x".to_string(), "number".to_string(), true, "X coordinate".to_string())
    .input("y".to_string(), "number".to_string(), true, "Y coordinate".to_string())
    .output("distance".to_string(), "number".to_string(), "Distance from origin".to_string())
    .property("precision".to_string(), "number".to_string(), false, Some("2".to_string()), "Decimal precision".to_string())
    .script("rust".to_string(), "fn calculate_distance(x: f64, y: f64) -> f64 { (x*x + y*y).sqrt() }".to_string())
    .build();

    assert_eq!(definition.id, "builder-test");
    assert_eq!(definition.name, "Builder Test Node");
    assert_eq!(definition.description, "Testing the builder pattern");
    assert_eq!(definition.category, "Builder");
    assert_eq!(definition.inputs.len(), 2);
    assert_eq!(definition.outputs.len(), 1);
    assert_eq!(definition.properties.len(), 1);
    
    // Test script implementation
    if let canvascontract::nodes::custom::CustomNodeImplementation::Script { language, code } = &definition.implementation {
        assert_eq!(language, "rust");
        assert!(code.contains("calculate_distance"));
    } else {
        panic!("Expected script implementation");
    }
}

#[test]
fn test_debug_session_creation() {
    let graph = Graph::new();
    let config = Config::default();
    let runtime = WasmRuntime::new(&config).unwrap();
    let session = DebugSession::new(graph, runtime);
    
    assert_eq!(session.get_state(), canvascontract::debugger::DebugState::Running);
    assert!(session.get_trace().is_empty());
    assert!(session.get_breakpoints().is_empty());
    assert!(session.get_variables().is_empty());
    assert!(session.get_call_stack().is_empty());
}

#[test]
fn test_breakpoint_management() {
    let graph = Graph::new();
    let config = Config::default();
    let runtime = WasmRuntime::new(&config).unwrap();
    let mut session = DebugSession::new(graph, runtime);

    // Add breakpoints
    assert!(session.add_breakpoint("node1".to_string(), None).is_ok());
    assert!(session.add_breakpoint("node2".to_string(), Some("gas_consumed > 1000".to_string())).is_ok());
    
    assert_eq!(session.get_breakpoints().len(), 2);
    
    // Toggle breakpoint
    assert!(session.toggle_breakpoint(&"node1".to_string(), false).is_ok());
    let breakpoints = session.get_breakpoints();
    assert!(!breakpoints[0].enabled);
    
    // Remove breakpoint
    assert!(session.remove_breakpoint(&"node1".to_string()).is_ok());
    assert_eq!(session.get_breakpoints().len(), 1);
    
    // Try to remove non-existent breakpoint
    assert!(session.remove_breakpoint(&"nonexistent".to_string()).is_err());
}

#[test]
fn test_debug_configurations() {
    let default_config = DebuggerUtils::default_config();
    assert!(!default_config.step_through);
    assert!(default_config.log_variables);
    assert!(default_config.log_gas);
    assert!(!default_config.log_performance);
    assert_eq!(default_config.max_steps, Some(1000));
    assert_eq!(default_config.timeout_ms, Some(30000));

    let step_config = DebuggerUtils::step_through_config();
    assert!(step_config.step_through);
    assert!(step_config.log_variables);
    assert!(step_config.log_gas);
    assert!(step_config.log_performance);
    assert_eq!(step_config.max_steps, None);
    assert_eq!(step_config.timeout_ms, None);
}

#[test]
fn test_performance_analysis() {
    use canvascontract::debugger::ExecutionStep;
    
    let trace = vec![
        ExecutionStep {
            step_number: 0,
            node_id: "node1".to_string(),
            node_type: NodeType::Start,
            timestamp: 1000,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
            gas_consumed: 100,
            duration_ms: 50,
            error: None,
        },
        ExecutionStep {
            step_number: 1,
            node_id: "node2".to_string(),
            node_type: NodeType::Logic,
            timestamp: 1050,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
            gas_consumed: 2000,
            duration_ms: 200,
            error: None,
        },
        ExecutionStep {
            step_number: 2,
            node_id: "node3".to_string(),
            node_type: NodeType::End,
            timestamp: 1250,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
            gas_consumed: 50,
            duration_ms: 10,
            error: None,
        },
    ];

    let analysis = DebuggerUtils::analyze_performance(&trace);
    
    assert_eq!(analysis.total_gas, 2150);
    assert_eq!(analysis.total_time, 260);
    assert_eq!(analysis.slowest_nodes.len(), 3);
    assert_eq!(analysis.most_expensive_nodes.len(), 3);
    assert_eq!(analysis.bottlenecks.len(), 1); // node2 is both slow and expensive
}

#[test]
fn test_custom_node_validation() {
    let mut registry = CustomNodeRegistry::new();
    
    // Test empty ID
    let invalid_node = CustomNodeBuilder::new(
        "".to_string(),
        "Invalid Node".to_string(),
    )
    .composite("{}".to_string())
    .build();
    
    assert!(registry.register_node(invalid_node).is_err());
    
    // Test empty name
    let invalid_node2 = CustomNodeBuilder::new(
        "valid-id".to_string(),
        "".to_string(),
    )
    .composite("{}".to_string())
    .build();
    
    assert!(registry.register_node(invalid_node2).is_ok()); // Name can be empty
    
    // Test empty input name
    let invalid_node3 = CustomNodeBuilder::new(
        "test3".to_string(),
        "Test 3".to_string(),
    )
    .input("".to_string(), "number".to_string(), true, "Empty input".to_string())
    .composite("{}".to_string())
    .build();
    
    assert!(registry.register_node(invalid_node3).is_err());
}

#[test]
fn test_debug_variable_management() {
    let graph = Graph::new();
    let config = Config::default();
    let runtime = WasmRuntime::new(&config).unwrap();
    let mut session = DebugSession::new(graph, runtime);
    
    // Set variables
    session.set_variable("x".to_string(), serde_json::json!(42));
    session.set_variable("y".to_string(), serde_json::json!("hello"));
    session.set_variable("z".to_string(), serde_json::json!({"nested": true}));
    
    let variables = session.get_variables();
    assert_eq!(variables.len(), 3);
    assert_eq!(variables.get("x"), Some(&serde_json::json!(42)));
    assert_eq!(variables.get("y"), Some(&serde_json::json!("hello")));
    assert_eq!(variables.get("z"), Some(&serde_json::json!({"nested": true})));
}

#[test]
fn test_custom_node_execution() {
    let mut registry = CustomNodeRegistry::new();
    
    let definition = CustomNodeBuilder::new(
        "test-exec".to_string(),
        "Test Execution".to_string(),
    )
    .input("a".to_string(), "number".to_string(), true, "First number".to_string())
    .input("b".to_string(), "number".to_string(), true, "Second number".to_string())
    .output("sum".to_string(), "number".to_string(), "Sum of inputs".to_string())
    .composite(r#"{"nodes": [], "edges": []}"#.to_string())
    .build();
    
    registry.register_node(definition).unwrap();
    
    let mut inputs = std::collections::HashMap::new();
    inputs.insert("a".to_string(), serde_json::json!(5));
    inputs.insert("b".to_string(), serde_json::json!(3));
    
    let properties = std::collections::HashMap::new();
    
    let result = registry.execute_node("test-exec", inputs, properties);
    assert!(result.is_ok());
    
    let outputs = result.unwrap();
    assert!(outputs.contains_key("sum"));
}

#[test]
fn test_debug_session_state_transitions() {
    let graph = Graph::new();
    let config = Config::default();
    let runtime = WasmRuntime::new(&config).unwrap();
    let mut session = DebugSession::new(graph, runtime);
    
    // Initial state should be Running
    assert_eq!(session.get_state(), canvascontract::debugger::DebugState::Running);
    
    // Test state transitions (these would be more comprehensive with actual execution)
    // For now, we just verify the state management works
    session.set_variable("test".to_string(), serde_json::json!("value"));
    assert_eq!(session.get_variables().len(), 1);
}

#[test]
fn test_custom_node_categories() {
    let mut registry = CustomNodeRegistry::new();
    
    let categories = vec!["Logic", "Math", "Storage", "External", "Utility"];
    
    for (i, category) in categories.iter().enumerate() {
        let definition = CustomNodeBuilder::new(
            format!("node-{}", i),
            format!("Node {}", i),
        )
        .category(category.to_string())
        .composite("{}".to_string())
        .build();
        
        assert!(registry.register_node(definition).is_ok());
    }
    
    let nodes = registry.list_nodes();
    assert_eq!(nodes.len(), categories.len());
    
    for (i, node) in nodes.iter().enumerate() {
        assert_eq!(node.category, categories[i]);
    }
}

#[test]
fn test_debug_configuration_validation() {
    let config = DebugConfig {
        step_through: true,
        log_variables: true,
        log_gas: true,
        log_performance: true,
        max_steps: Some(100),
        timeout_ms: Some(5000),
    };
    
    assert!(config.step_through);
    assert!(config.log_variables);
    assert!(config.log_gas);
    assert!(config.log_performance);
    assert_eq!(config.max_steps, Some(100));
    assert_eq!(config.timeout_ms, Some(5000));
}

#[test]
fn test_custom_node_property_validation() {
    let mut registry = CustomNodeRegistry::new();
    
    // Test required property
    let definition = CustomNodeBuilder::new(
        "test-props".to_string(),
        "Test Properties".to_string(),
    )
    .property("required_prop".to_string(), "string".to_string(), true, None, "Required property".to_string())
    .property("optional_prop".to_string(), "number".to_string(), false, Some("42".to_string()), "Optional property".to_string())
    .composite("{}".to_string())
    .build();
    
    assert!(registry.register_node(definition).is_ok());
    
    let node = registry.get_node("test-props").unwrap();
    assert_eq!(node.properties.len(), 2);
    
    let required_prop = &node.properties[0];
    assert_eq!(required_prop.name, "required_prop");
    assert!(required_prop.required);
    assert_eq!(required_prop.default_value, None);
    
    let optional_prop = &node.properties[1];
    assert_eq!(optional_prop.name, "optional_prop");
    assert!(!optional_prop.required);
    assert_eq!(optional_prop.default_value, Some("42".to_string()));
} 