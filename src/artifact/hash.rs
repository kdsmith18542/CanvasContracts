use std::collections::BTreeMap;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    error::{CanvasError, CanvasResult},
    types::{Connection, Port, VisualGraph, VisualNode},
};

pub const GRAPH_CANONICALIZATION: &str = "canvas-json-c14n-v1";

#[derive(Debug, Serialize)]
struct CanonicalGraph {
    schema: String,
    id: String,
    name: String,
    description: Option<String>,
    metadata: BTreeMap<String, String>,
    nodes: Vec<CanonicalNode>,
    connections: Vec<CanonicalConnection>,
}

#[derive(Debug, Serialize)]
struct CanonicalNode {
    id: String,
    node_type: String,
    position: CanonicalPosition,
    size: CanonicalSize,
    inputs: Vec<CanonicalPort>,
    outputs: Vec<CanonicalPort>,
    properties: BTreeMap<String, serde_json::Value>,
    metadata: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct CanonicalConnection {
    id: String,
    source_node: String,
    source_port: String,
    target_node: String,
    target_port: String,
    metadata: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct CanonicalPort {
    id: String,
    name: String,
    value_type: serde_json::Value,
    required: bool,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct CanonicalPosition {
    x: f64,
    y: f64,
}

#[derive(Debug, Serialize)]
struct CanonicalSize {
    width: f64,
    height: f64,
}

pub fn hash_bytes_prefixed(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

pub fn hash_value_prefixed(value: &serde_json::Value) -> CanvasResult<String> {
    let bytes = serde_json::to_vec(value).map_err(CanvasError::Serialization)?;
    Ok(hash_bytes_prefixed(&bytes))
}

pub fn canonical_graph_value(graph: &VisualGraph) -> CanvasResult<serde_json::Value> {
    let mut nodes: Vec<CanonicalNode> = graph
        .nodes
        .iter()
        .map(canonicalize_node)
        .collect::<CanvasResult<_>>()?;
    nodes.sort_by(|a, b| a.id.cmp(&b.id));

    let mut connections: Vec<CanonicalConnection> = graph
        .connections
        .iter()
        .map(canonicalize_connection)
        .collect::<CanvasResult<_>>()?;
    connections.sort_by(|a, b| {
        a.source_node
            .cmp(&b.source_node)
            .then_with(|| a.source_port.cmp(&b.source_port))
            .then_with(|| a.target_node.cmp(&b.target_node))
            .then_with(|| a.target_port.cmp(&b.target_port))
            .then_with(|| a.id.cmp(&b.id))
    });

    let canonical = CanonicalGraph {
        schema: GRAPH_CANONICALIZATION.to_string(),
        id: graph.id.to_string(),
        name: graph.name.clone(),
        description: graph.description.clone(),
        metadata: graph
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        nodes,
        connections,
    };

    let raw = serde_json::to_value(canonical).map_err(CanvasError::Serialization)?;
    Ok(canonicalize_json_value(&raw))
}

pub fn canonical_graph_json(graph: &VisualGraph) -> CanvasResult<String> {
    let value = canonical_graph_value(graph)?;
    serde_json::to_string_pretty(&value).map_err(CanvasError::Serialization)
}

pub fn canonical_graph_hash(graph: &VisualGraph) -> CanvasResult<String> {
    let value = canonical_graph_value(graph)?;
    hash_value_prefixed(&value)
}

pub fn canonicalize_json_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for key in keys {
                if let Some(v) = map.get(key) {
                    out.insert(key.clone(), canonicalize_json_value(v));
                }
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(canonicalize_json_value).collect())
        }
        _ => value.clone(),
    }
}

fn canonicalize_node(node: &VisualNode) -> CanvasResult<CanonicalNode> {
    let mut inputs: Vec<CanonicalPort> = node
        .inputs
        .iter()
        .map(canonicalize_port)
        .collect::<CanvasResult<_>>()?;
    inputs.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| a.name.cmp(&b.name)));

    let mut outputs: Vec<CanonicalPort> = node
        .outputs
        .iter()
        .map(canonicalize_port)
        .collect::<CanvasResult<_>>()?;
    outputs.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| a.name.cmp(&b.name)));

    let properties = node
        .properties
        .iter()
        .map(|(k, v)| (k.clone(), canonicalize_json_value(v)))
        .collect();

    Ok(CanonicalNode {
        id: node.id.to_string(),
        node_type: node.node_type.clone(),
        position: CanonicalPosition {
            x: node.position.x,
            y: node.position.y,
        },
        size: CanonicalSize {
            width: node.size.width,
            height: node.size.height,
        },
        inputs,
        outputs,
        properties,
        metadata: node
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    })
}

fn canonicalize_connection(connection: &Connection) -> CanvasResult<CanonicalConnection> {
    Ok(CanonicalConnection {
        id: connection.id.to_string(),
        source_node: connection.source_node.to_string(),
        source_port: connection.source_port.clone(),
        target_node: connection.target_node.to_string(),
        target_port: connection.target_port.clone(),
        metadata: connection
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    })
}

fn canonicalize_port(port: &Port) -> CanvasResult<CanonicalPort> {
    let value_type = serde_json::to_value(&port.value_type).map_err(CanvasError::Serialization)?;
    Ok(CanonicalPort {
        id: port.id.clone(),
        name: port.name.clone(),
        value_type: canonicalize_json_value(&value_type),
        required: port.required,
        description: port.description.clone(),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::types::{Connection, Position, VisualNode};

    fn sample_graph() -> VisualGraph {
        let mut graph = VisualGraph::new("Canonical Graph");

        let start_id = uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000001").unwrap();
        let add_id = uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000002").unwrap();

        let mut start = VisualNode::new(start_id, "Start", Position::new(0.0, 0.0));
        start.properties.insert("z".to_string(), json!(1));
        start
            .properties
            .insert("a".to_string(), json!({"b":1,"a":2}));

        let mut add = VisualNode::new(add_id, "Add", Position::new(100.0, 0.0));
        add.properties.insert("b".to_string(), json!(20));
        add.properties.insert("a".to_string(), json!(10));

        graph.add_node(add);
        graph.add_node(start);
        graph.add_connection(Connection::new(
            uuid::Uuid::parse_str("c0000000-0000-0000-0000-000000000001").unwrap(),
            start_id,
            "flow_out",
            add_id,
            "flow_in",
        ));

        graph
    }

    #[test]
    fn canonical_hash_is_stable_across_node_order() {
        let graph = sample_graph();
        let hash_a = canonical_graph_hash(&graph).unwrap();

        let mut reversed = graph.clone();
        reversed.nodes.reverse();
        let hash_b = canonical_graph_hash(&reversed).unwrap();

        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn nested_object_keys_are_sorted() {
        let graph = sample_graph();
        let canonical = canonical_graph_json(&graph).unwrap();

        assert!(canonical.contains("\"a\": 2"));
        assert!(canonical.find("\"a\": 2").unwrap() < canonical.find("\"b\": 1").unwrap());
    }
}
