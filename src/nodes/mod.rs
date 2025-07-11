//! Node system for Canvas Contracts

mod definitions;
mod implementations;

use crate::{
    error::{CanvasError, CanvasResult},
    types::{ExecutionContext, NodeResult, PortId, ValueType},
};

pub use definitions::NodeDefinition;
pub use implementations::Node;

/// Node context for execution
pub struct NodeContext {
    pub execution_context: ExecutionContext,
    pub inputs: std::collections::HashMap<PortId, serde_json::Value>,
    pub outputs: std::collections::HashMap<PortId, serde_json::Value>,
}

impl NodeContext {
    pub fn new(execution_context: ExecutionContext) -> Self {
        Self {
            execution_context,
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
        }
    }

    pub fn get_input(&self, port_id: &PortId) -> Option<&serde_json::Value> {
        self.inputs.get(port_id)
    }

    pub fn set_output(&mut self, port_id: PortId, value: serde_json::Value) {
        self.outputs.insert(port_id, value);
    }

    pub fn use_gas(&mut self, amount: u64) -> CanvasResult<()> {
        self.execution_context
            .use_gas(amount)
            .map_err(|e| CanvasError::Validation(e))
    }

    pub fn emit_event(&mut self, name: String, data: std::collections::HashMap<String, serde_json::Value>) {
        let event = crate::types::Event {
            name,
            data,
            indexed_data: Vec::new(),
        };
        self.execution_context.emit_event(event);
    }
}

/// Node registry for managing available node types
pub struct NodeRegistry {
    definitions: std::collections::HashMap<String, NodeDefinition>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            definitions: std::collections::HashMap::new(),
        }
    }

    pub fn register_node(&mut self, definition: NodeDefinition) {
        self.definitions.insert(definition.id.clone(), definition);
    }

    pub fn get_node_definition(&self, node_type: &str) -> Option<&NodeDefinition> {
        self.definitions.get(node_type)
    }

    pub fn list_node_types(&self) -> Vec<String> {
        self.definitions.keys().cloned().collect()
    }

    pub fn create_node(&self, node_type: &str) -> CanvasResult<Box<dyn Node>> {
        let definition = self
            .get_node_definition(node_type)
            .ok_or_else(|| CanvasError::Node(format!("Unknown node type: {}", node_type)))?;

        // TODO: Implement node creation based on definition
        Err(CanvasError::Node("Node creation not yet implemented".to_string()))
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::new()
    }
} 