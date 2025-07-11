use canvas_contracts::{
    ai::{AiAssistant, PatternAnalysis, ValidationResult, OptimizationResult},
    config::Config,
    types::{Graph, Node, NodeType, Edge},
};

#[test]
fn test_ai_assistant_creation() {
    let config = Config::default();
    let ai = AiAssistant::new(&config);
    assert!(ai.is_ok());
}

#[test]
fn test_pattern_recognition() {
    let config = Config::default();
    let ai = AiAssistant::new(&config).unwrap();
    
    // Create a simple token-like graph
    let mut graph = Graph::new();
    
    // Add nodes that form a token pattern
    let start_node = Node {
        id: "start".to_string(),
        node_type: NodeType::Start,
        position: (100, 100),
        properties: Default::default(),
    };
    
    let state_node = Node {
        id: "balance".to_string(),
        node_type: NodeType::State,
        position: (200, 100),
        properties: Default::default(),
    };
    
    let logic_node = Node {
        id: "transfer".to_string(),
        node_type: NodeType::Logic,
        position: (300, 100),
        properties: Default::default(),
    };
    
    let external_node = Node {
        id: "event".to_string(),
        node_type: NodeType::External,
        position: (400, 100),
        properties: Default::default(),
    };
    
    graph.add_node(start_node);
    graph.add_node(state_node);
    graph.add_node(logic_node);
    graph.add_node(external_node);
    
    // Add edges to connect the pattern
    graph.add_edge(Edge {
        id: "edge1".to_string(),
        source: "start".to_string(),
        target: "balance".to_string(),
        source_handle: None,
        target_handle: None,
    });
    
    graph.add_edge(Edge {
        id: "edge2".to_string(),
        source: "balance".to_string(),
        target: "transfer".to_string(),
        source_handle: None,
        target_handle: None,
    });
    
    graph.add_edge(Edge {
        id: "edge3".to_string(),
        source: "transfer".to_string(),
        target: "event".to_string(),
        source_handle: None,
        target_handle: None,
    });
    
    let result = ai.analyze_patterns(&graph);
    assert!(result.is_ok());
    
    let analysis = result.unwrap();
    // Should detect token pattern
    assert!(!analysis.patterns_found.is_empty());
}

#[test]
fn test_contract_validation() {
    let config = Config::default();
    let ai = AiAssistant::new(&config).unwrap();
    
    // Create a valid graph
    let mut graph = Graph::new();
    
    let start_node = Node {
        id: "start".to_string(),
        node_type: NodeType::Start,
        position: (100, 100),
        properties: Default::default(),
    };
    
    let end_node = Node {
        id: "end".to_string(),
        node_type: NodeType::End,
        position: (200, 100),
        properties: Default::default(),
    };
    
    graph.add_node(start_node);
    graph.add_node(end_node);
    
    graph.add_edge(Edge {
        id: "edge1".to_string(),
        source: "start".to_string(),
        target: "end".to_string(),
        source_handle: None,
        target_handle: None,
    });
    
    let result = ai.validate_contract(&graph);
    assert!(result.is_ok());
    
    let validation = result.unwrap();
    assert!(validation.is_valid);
}

#[test]
fn test_contract_optimization() {
    let config = Config::default();
    let ai = AiAssistant::new(&config).unwrap();
    
    // Create a graph with optimization opportunities
    let mut graph = Graph::new();
    
    let start_node = Node {
        id: "start".to_string(),
        node_type: NodeType::Start,
        position: (100, 100),
        properties: Default::default(),
    };
    
    let arithmetic1 = Node {
        id: "add1".to_string(),
        node_type: NodeType::Arithmetic,
        position: (200, 100),
        properties: Default::default(),
    };
    
    let arithmetic2 = Node {
        id: "add2".to_string(),
        node_type: NodeType::Arithmetic,
        position: (300, 100),
        properties: Default::default(),
    };
    
    let end_node = Node {
        id: "end".to_string(),
        node_type: NodeType::End,
        position: (400, 100),
        properties: Default::default(),
    };
    
    graph.add_node(start_node);
    graph.add_node(arithmetic1);
    graph.add_node(arithmetic2);
    graph.add_node(end_node);
    
    // Connect them in sequence
    graph.add_edge(Edge {
        id: "edge1".to_string(),
        source: "start".to_string(),
        target: "add1".to_string(),
        source_handle: None,
        target_handle: None,
    });
    
    graph.add_edge(Edge {
        id: "edge2".to_string(),
        source: "add1".to_string(),
        target: "add2".to_string(),
        source_handle: None,
        target_handle: None,
    });
    
    graph.add_edge(Edge {
        id: "edge3".to_string(),
        source: "add2".to_string(),
        target: "end".to_string(),
        source_handle: None,
        target_handle: None,
    });
    
    let result = ai.optimize_contract(&graph);
    assert!(result.is_ok());
    
    let optimization = result.unwrap();
    assert!(optimization.original_gas_estimate > 0);
    assert!(optimization.gas_savings >= 0);
}

#[test]
fn test_node_suggestions() {
    let config = Config::default();
    let ai = AiAssistant::new(&config).unwrap();
    
    let mut graph = Graph::new();
    
    let start_node = Node {
        id: "start".to_string(),
        node_type: NodeType::Start,
        position: (100, 100),
        properties: Default::default(),
    };
    
    let logic_node = Node {
        id: "logic".to_string(),
        node_type: NodeType::Logic,
        position: (200, 100),
        properties: Default::default(),
    };
    
    graph.add_node(start_node);
    graph.add_node(logic_node);
    
    let suggestions = ai.suggest_next_nodes(&graph, "logic".to_string());
    assert!(suggestions.is_ok());
    
    let suggestions = suggestions.unwrap();
    // Should suggest appropriate next nodes for logic
    assert!(!suggestions.is_empty());
}

#[test]
fn test_security_issue_detection() {
    let config = Config::default();
    let ai = AiAssistant::new(&config).unwrap();
    
    // Create a graph with potential security issues
    let mut graph = Graph::new();
    
    let start_node = Node {
        id: "start".to_string(),
        node_type: NodeType::Start,
        position: (100, 100),
        properties: Default::default(),
    };
    
    let external_node = Node {
        id: "external".to_string(),
        node_type: NodeType::External,
        position: (200, 100),
        properties: Default::default(),
    };
    
    let state_node = Node {
        id: "state".to_string(),
        node_type: NodeType::State,
        position: (300, 100),
        properties: Default::default(),
    };
    
    graph.add_node(start_node);
    graph.add_node(external_node);
    graph.add_node(state_node);
    
    // Create reentrancy pattern: External -> State
    graph.add_edge(Edge {
        id: "edge1".to_string(),
        source: "start".to_string(),
        target: "external".to_string(),
        source_handle: None,
        target_handle: None,
    });
    
    graph.add_edge(Edge {
        id: "edge2".to_string(),
        source: "external".to_string(),
        target: "state".to_string(),
        source_handle: None,
        target_handle: None,
    });
    
    let result = ai.analyze_patterns(&graph);
    assert!(result.is_ok());
    
    let analysis = result.unwrap();
    // Should detect security issues
    assert!(!analysis.security_issues.is_empty() || !analysis.anti_patterns.is_empty());
}

#[test]
fn test_anti_pattern_detection() {
    let config = Config::default();
    let ai = AiAssistant::new(&config).unwrap();
    
    // Create a graph with anti-patterns
    let mut graph = Graph::new();
    
    let start_node = Node {
        id: "start".to_string(),
        node_type: NodeType::Start,
        position: (100, 100),
        properties: Default::default(),
    };
    
    let arithmetic_node = Node {
        id: "arithmetic".to_string(),
        node_type: NodeType::Arithmetic,
        position: (200, 100),
        properties: Default::default(),
    };
    
    let state_node = Node {
        id: "state".to_string(),
        node_type: NodeType::State,
        position: (300, 100),
        properties: Default::default(),
    };
    
    graph.add_node(start_node);
    graph.add_node(arithmetic_node);
    graph.add_node(state_node);
    
    // Create unchecked arithmetic pattern
    graph.add_edge(Edge {
        id: "edge1".to_string(),
        source: "start".to_string(),
        target: "arithmetic".to_string(),
        source_handle: None,
        target_handle: None,
    });
    
    graph.add_edge(Edge {
        id: "edge2".to_string(),
        source: "arithmetic".to_string(),
        target: "state".to_string(),
        source_handle: None,
        target_handle: None,
    });
    
    let result = ai.analyze_patterns(&graph);
    assert!(result.is_ok());
    
    let analysis = result.unwrap();
    // Should detect anti-patterns
    assert!(!analysis.anti_patterns.is_empty());
} 