//! Graph Intermediate Representation (IR)
use crate::types::{VisualGraph, NodeId};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// Graph IR node
#[derive(Debug, Clone)]
pub struct GraphIRNode {
    pub id: NodeId,
    pub node_type: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Graph IR connection
#[derive(Debug, Clone)]
pub struct GraphIRConnection {
    pub id: crate::types::EdgeId,
    pub source: NodeId,
    pub source_port: String,
    pub target: NodeId,
    pub target_port: String,
}

/// Graph IR representation
#[derive(Debug, Clone)]
pub struct GraphIR {
    pub nodes: HashMap<NodeId, GraphIRNode>,
    pub connections: Vec<GraphIRConnection>,
    pub graph: DiGraph<NodeId, ()>,
    pub node_map: HashMap<NodeId, NodeIndex>,
}

impl GraphIR {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn from_visual_graph(graph: &VisualGraph) -> Self {
        let mut ir = GraphIR::new();
        
        // Add nodes to petgraph and IR map
        for node in &graph.nodes {
            let node_idx = ir.graph.add_node(node.id);
            ir.node_map.insert(node.id, node_idx);
            
            ir.nodes.insert(node.id, GraphIRNode {
                id: node.id,
                node_type: node.node_type.clone(),
                inputs: node.inputs.iter().map(|p| p.id.clone()).collect(),
                outputs: node.outputs.iter().map(|p| p.id.clone()).collect(),
                properties: node.properties.clone(),
            });
        }
        
        // Add edges to petgraph and IR connections
        for conn in &graph.connections {
            if let (Some(&src_idx), Some(&tgt_idx)) = (ir.node_map.get(&conn.source_node), ir.node_map.get(&conn.target_node)) {
                ir.graph.add_edge(src_idx, tgt_idx, ());
                
                ir.connections.push(GraphIRConnection {
                    id: conn.id,
                    source: conn.source_node,
                    source_port: conn.source_port.clone(),
                    target: conn.target_node,
                    target_port: conn.target_port.clone(),
                });
            }
        }
        
        ir
    }
}

impl Default for GraphIR {
    fn default() -> Self { Self::new() }
}
