//! Core types for Canvas Contracts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Node identifier
pub type NodeId = Uuid;

/// Edge identifier
pub type EdgeId = Uuid;

/// Port identifier
pub type PortId = String;

/// Gas amount
pub type Gas = u64;

/// Contract address
pub type ContractAddress = String;

/// Transaction hash
pub type TransactionHash = String;

/// Block number
pub type BlockNumber = u64;

/// Timestamp
pub type Timestamp = u64;

/// Value types that can flow through connections
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueType {
    /// Boolean value
    Boolean,
    /// Integer value
    Integer,
    /// Floating point value
    Float,
    /// String value
    String,
    /// Bytes value
    Bytes,
    /// Array of values
    Array(Box<ValueType>),
    /// Object with named fields
    Object(HashMap<String, ValueType>),
    /// Flow control (no data, just execution flow)
    Flow,
    /// Any type (for dynamic typing)
    Any,
}

impl ValueType {
    /// Check if this type is compatible with another
    pub fn is_compatible_with(&self, other: &ValueType) -> bool {
        match (self, other) {
            (ValueType::Any, _) | (_, ValueType::Any) => true,
            (ValueType::Flow, ValueType::Flow) => true,
            (ValueType::Boolean, ValueType::Boolean) => true,
            (ValueType::Integer, ValueType::Integer) => true,
            (ValueType::Float, ValueType::Float) => true,
            (ValueType::String, ValueType::String) => true,
            (ValueType::Bytes, ValueType::Bytes) => true,
            (ValueType::Array(inner1), ValueType::Array(inner2)) => {
                inner1.is_compatible_with(inner2)
            }
            (ValueType::Object(fields1), ValueType::Object(fields2)) => {
                fields1.len() == fields2.len()
                    && fields1.iter().all(|(k, v)| {
                        fields2.get(k).map_or(false, |v2| v.is_compatible_with(v2))
                    })
            }
            _ => false,
        }
    }
}

/// Node port (input or output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub id: PortId,
    pub name: String,
    pub value_type: ValueType,
    pub required: bool,
    pub description: Option<String>,
}

impl Port {
    pub fn new(id: impl Into<PortId>, name: impl Into<String>, value_type: ValueType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            value_type,
            required: false,
            description: None,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Node position on canvas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Node size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

/// Visual node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualNode {
    pub id: NodeId,
    pub node_type: String,
    pub position: Position,
    pub size: Size,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub properties: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, String>,
}

impl VisualNode {
    pub fn new(id: NodeId, node_type: impl Into<String>, position: Position) -> Self {
        Self {
            id,
            node_type: node_type.into(),
            position,
            size: Size::new(120.0, 80.0),
            inputs: Vec::new(),
            outputs: Vec::new(),
            properties: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn with_inputs(mut self, inputs: Vec<Port>) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn with_outputs(mut self, outputs: Vec<Port>) -> Self {
        self.outputs = outputs;
        self
    }

    pub fn with_property(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.properties.insert(key.into(), value);
        self
    }
}

/// Connection between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: EdgeId,
    pub source_node: NodeId,
    pub source_port: PortId,
    pub target_node: NodeId,
    pub target_port: PortId,
    pub metadata: HashMap<String, String>,
}

impl Connection {
    pub fn new(
        id: EdgeId,
        source_node: NodeId,
        source_port: impl Into<PortId>,
        target_node: NodeId,
        target_port: impl Into<PortId>,
    ) -> Self {
        Self {
            id,
            source_node,
            source_port: source_port.into(),
            target_node,
            target_port: target_port.into(),
            metadata: HashMap::new(),
        }
    }
}

/// Visual graph representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualGraph {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub nodes: Vec<VisualNode>,
    pub connections: Vec<Connection>,
    pub metadata: HashMap<String, String>,
}

impl VisualGraph {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            nodes: Vec::new(),
            connections: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn add_node(&mut self, node: VisualNode) {
        self.nodes.push(node);
    }

    pub fn add_connection(&mut self, connection: Connection) {
        self.connections.push(connection);
    }

    pub fn get_node(&self, id: NodeId) -> Option<&VisualNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut VisualNode> {
        self.nodes.iter_mut().find(|node| node.id == id)
    }
}

/// Contract compilation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationResult {
    pub wasm_bytes: Vec<u8>,
    pub abi: ContractABI,
    pub gas_estimate: Gas,
    pub warnings: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Contract ABI (Application Binary Interface)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractABI {
    pub functions: Vec<FunctionABI>,
    pub events: Vec<EventABI>,
    pub errors: Vec<ErrorABI>,
    pub metadata: HashMap<String, String>,
}

/// Function ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionABI {
    pub name: String,
    pub inputs: Vec<ParameterABI>,
    pub outputs: Vec<ParameterABI>,
    pub state_mutability: StateMutability,
    pub gas_estimate: Option<Gas>,
}

/// Event ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventABI {
    pub name: String,
    pub inputs: Vec<ParameterABI>,
    pub anonymous: bool,
}

/// Error ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorABI {
    pub name: String,
    pub inputs: Vec<ParameterABI>,
}

/// Parameter ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterABI {
    pub name: String,
    pub value_type: ValueType,
    pub indexed: bool,
}

/// State mutability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StateMutability {
    Pure,
    View,
    NonPayable,
    Payable,
}

/// Execution context for nodes
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub gas_used: Gas,
    pub gas_limit: Gas,
    pub storage: HashMap<String, serde_json::Value>,
    pub events: Vec<Event>,
    pub metadata: HashMap<String, String>,
}

impl ExecutionContext {
    pub fn new(gas_limit: Gas) -> Self {
        Self {
            gas_used: 0,
            gas_limit,
            storage: HashMap::new(),
            events: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn use_gas(&mut self, amount: Gas) -> Result<(), String> {
        if self.gas_used + amount > self.gas_limit {
            return Err("Gas limit exceeded".to_string());
        }
        self.gas_used += amount;
        Ok(())
    }

    pub fn emit_event(&mut self, event: Event) {
        self.events.push(event);
    }
}

/// Event emitted during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub data: HashMap<String, serde_json::Value>,
    pub indexed_data: Vec<serde_json::Value>,
}

/// Node execution result
#[derive(Debug, Clone)]
pub struct NodeResult {
    pub outputs: HashMap<PortId, serde_json::Value>,
    pub gas_used: Gas,
    pub events: Vec<Event>,
    pub error: Option<String>,
}

impl NodeResult {
    pub fn success(outputs: HashMap<PortId, serde_json::Value>, gas_used: Gas) -> Self {
        Self {
            outputs,
            gas_used,
            events: Vec::new(),
            error: None,
        }
    }

    pub fn error(error: impl Into<String>, gas_used: Gas) -> Self {
        Self {
            outputs: HashMap::new(),
            gas_used,
            events: Vec::new(),
            error: Some(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_compatibility() {
        assert!(ValueType::Boolean.is_compatible_with(&ValueType::Boolean));
        assert!(ValueType::Any.is_compatible_with(&ValueType::Boolean));
        assert!(ValueType::Boolean.is_compatible_with(&ValueType::Any));
        assert!(!ValueType::Boolean.is_compatible_with(&ValueType::Integer));
    }

    #[test]
    fn test_visual_graph_operations() {
        let mut graph = VisualGraph::new("test graph");
        let node = VisualNode::new(Uuid::new_v4(), "test", Position::new(0.0, 0.0));
        let node_id = node.id;
        
        graph.add_node(node);
        assert!(graph.get_node(node_id).is_some());
    }

    #[test]
    fn test_execution_context_gas() {
        let mut context = ExecutionContext::new(1000);
        assert!(context.use_gas(500).is_ok());
        assert!(context.use_gas(600).is_err());
    }
} 