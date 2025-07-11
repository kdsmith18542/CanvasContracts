//! Node implementations

use crate::{
    error::{CanvasError, CanvasResult},
    types::{NodeResult, PortId},
};

use super::NodeContext;

/// Trait for implementing nodes
pub trait Node: Send + Sync {
    /// Execute the node
    fn execute(&self, context: &mut NodeContext) -> CanvasResult<NodeResult>;
    
    /// Get the node type name
    fn node_type(&self) -> &str;
    
    /// Validate the node configuration
    fn validate(&self) -> CanvasResult<()> {
        Ok(())
    }
}

/// Simple pass-through node
pub struct PassThroughNode {
    node_type: String,
}

impl PassThroughNode {
    pub fn new(node_type: impl Into<String>) -> Self {
        Self {
            node_type: node_type.into(),
        }
    }
}

impl Node for PassThroughNode {
    fn execute(&self, context: &mut NodeContext) -> CanvasResult<NodeResult> {
        // Simply pass through all inputs to outputs
        let mut outputs = std::collections::HashMap::new();
        
        for (port_id, value) in &context.inputs {
            outputs.insert(port_id.clone(), value.clone());
        }
        
        Ok(NodeResult::success(outputs, 1))
    }
    
    fn node_type(&self) -> &str {
        &self.node_type
    }
}

/// If condition node
pub struct IfNode {
    condition_expression: String,
}

impl IfNode {
    pub fn new(condition_expression: impl Into<String>) -> Self {
        Self {
            condition_expression: condition_expression.into(),
        }
    }
}

impl Node for IfNode {
    fn execute(&self, context: &mut NodeContext) -> CanvasResult<NodeResult> {
        // TODO: Implement condition evaluation
        // For now, always take the true branch
        let mut outputs = std::collections::HashMap::new();
        
        // Set the true flow output
        outputs.insert("true_flow".to_string(), serde_json::Value::Bool(true));
        
        Ok(NodeResult::success(outputs, 10))
    }
    
    fn node_type(&self) -> &str {
        "If"
    }
}

/// Add node
pub struct AddNode;

impl Node for AddNode {
    fn execute(&self, context: &mut NodeContext) -> CanvasResult<NodeResult> {
        let a = context.get_input("a")
            .ok_or_else(|| CanvasError::Node("Missing input 'a'".to_string()))?;
        let b = context.get_input("b")
            .ok_or_else(|| CanvasError::Node("Missing input 'b'".to_string()))?;
        
        let result = match (a, b) {
            (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
                if let (Some(a_val), Some(b_val)) = (a.as_i64(), b.as_i64()) {
                    serde_json::Value::Number((a_val + b_val).into())
                } else {
                    return Err(CanvasError::Node("Invalid number format".to_string()));
                }
            }
            _ => return Err(CanvasError::Node("Inputs must be numbers".to_string())),
        };
        
        let mut outputs = std::collections::HashMap::new();
        outputs.insert("result".to_string(), result);
        
        Ok(NodeResult::success(outputs, 3))
    }
    
    fn node_type(&self) -> &str {
        "Add"
    }
}

/// Read storage node
pub struct ReadStorageNode {
    key: String,
}

impl ReadStorageNode {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
        }
    }
}

impl Node for ReadStorageNode {
    fn execute(&self, context: &mut NodeContext) -> CanvasResult<NodeResult> {
        // Read from storage
        let value = context.execution_context.storage.get(&self.key)
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        
        let mut outputs = std::collections::HashMap::new();
        outputs.insert("value".to_string(), value);
        
        // Use gas for storage read
        context.use_gas(100)?;
        
        Ok(NodeResult::success(outputs, 100))
    }
    
    fn node_type(&self) -> &str {
        "ReadStorage"
    }
}

/// Write storage node
pub struct WriteStorageNode {
    key: String,
}

impl WriteStorageNode {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
        }
    }
}

impl Node for WriteStorageNode {
    fn execute(&self, context: &mut NodeContext) -> CanvasResult<NodeResult> {
        let value = context.get_input("value")
            .ok_or_else(|| CanvasError::Node("Missing input 'value'".to_string()))?;
        
        // Write to storage
        context.execution_context.storage.insert(self.key.clone(), value.clone());
        
        // Use gas for storage write
        context.use_gas(200)?;
        
        Ok(NodeResult::success(std::collections::HashMap::new(), 200))
    }
    
    fn node_type(&self) -> &str {
        "WriteStorage"
    }
}

/// Start node
pub struct StartNode;

impl Node for StartNode {
    fn execute(&self, _context: &mut NodeContext) -> CanvasResult<NodeResult> {
        let mut outputs = std::collections::HashMap::new();
        outputs.insert("flow_out".to_string(), serde_json::Value::Bool(true));
        
        Ok(NodeResult::success(outputs, 0))
    }
    
    fn node_type(&self) -> &str {
        "Start"
    }
}

/// End node
pub struct EndNode;

impl Node for EndNode {
    fn execute(&self, _context: &mut NodeContext) -> CanvasResult<NodeResult> {
        // End node doesn't produce outputs
        Ok(NodeResult::success(std::collections::HashMap::new(), 0))
    }
    
    fn node_type(&self) -> &str {
        "End"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ExecutionContext;

    #[test]
    fn test_pass_through_node() {
        let node = PassThroughNode::new("Test");
        let mut context = NodeContext::new(ExecutionContext::new(1000));
        
        context.inputs.insert("test_input".to_string(), serde_json::Value::String("test".to_string()));
        
        let result = node.execute(&mut context).unwrap();
        assert_eq!(result.outputs.get("test_input").unwrap(), &serde_json::Value::String("test".to_string()));
    }

    #[test]
    fn test_add_node() {
        let node = AddNode;
        let mut context = NodeContext::new(ExecutionContext::new(1000));
        
        context.inputs.insert("a".to_string(), serde_json::Value::Number(5.into()));
        context.inputs.insert("b".to_string(), serde_json::Value::Number(3.into()));
        
        let result = node.execute(&mut context).unwrap();
        assert_eq!(result.outputs.get("result").unwrap(), &serde_json::Value::Number(8.into()));
    }

    #[test]
    fn test_read_storage_node() {
        let node = ReadStorageNode::new("test_key");
        let mut context = NodeContext::new(ExecutionContext::new(1000));
        
        // Set up storage
        context.execution_context.storage.insert("test_key".to_string(), serde_json::Value::String("test_value".to_string()));
        
        let result = node.execute(&mut context).unwrap();
        assert_eq!(result.outputs.get("value").unwrap(), &serde_json::Value::String("test_value".to_string()));
    }

    #[test]
    fn test_write_storage_node() {
        let node = WriteStorageNode::new("test_key");
        let mut context = NodeContext::new(ExecutionContext::new(1000));
        
        context.inputs.insert("value".to_string(), serde_json::Value::String("test_value".to_string()));
        
        let result = node.execute(&mut context).unwrap();
        assert_eq!(context.execution_context.storage.get("test_key").unwrap(), &serde_json::Value::String("test_value".to_string()));
    }
} 