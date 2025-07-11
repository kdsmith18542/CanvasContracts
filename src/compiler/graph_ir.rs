//! Graph Intermediate Representation (IR)

// TODO: Implement Graph IR generation from visual graph
// This module will convert the visual graph into an intermediate representation
// that can be used for optimization and code generation.

/// Graph IR node
#[derive(Debug, Clone)]
pub struct GraphIRNode {
    pub id: String,
    pub node_type: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub properties: std::collections::HashMap<String, String>,
}

/// Graph IR connection
#[derive(Debug, Clone)]
pub struct GraphIRConnection {
    pub id: String,
    pub source: String,
    pub target: String,
    pub data_type: String,
}

/// Graph IR representation
#[derive(Debug, Clone)]
pub struct GraphIR {
    pub nodes: Vec<GraphIRNode>,
    pub connections: Vec<GraphIRConnection>,
}

impl GraphIR {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
        }
    }
} 