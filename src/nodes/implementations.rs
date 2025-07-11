//! Node implementations

use crate::{
    error::{CanvasError, CanvasResult},
    types::{ExecutionContext, NodeResult, PortId},
};

/// Node trait that all nodes must implement
pub trait Node: Send + Sync {
    /// Execute the node with given context
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult>;
    
    /// Get the node type identifier
    fn node_type(&self) -> &str;
    
    /// Get the node name
    fn name(&self) -> &str;
}

/// Basic node implementation
pub struct BasicNode {
    node_type: String,
    name: String,
    executor: Box<dyn Fn(&mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> + Send + Sync>,
}

impl BasicNode {
    pub fn new(
        node_type: impl Into<String>,
        name: impl Into<String>,
        executor: impl Fn(&mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> + Send + Sync + 'static,
    ) -> Self {
        Self {
            node_type: node_type.into(),
            name: name.into(),
            executor: Box::new(executor),
        }
    }
}

impl Node for BasicNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        (self.executor)(context)
    }

    fn node_type(&self) -> &str {
        &self.node_type
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// If node implementation
pub struct IfNode {
    condition: String,
}

impl IfNode {
    pub fn new(condition: impl Into<String>) -> Self {
        Self {
            condition: condition.into(),
        }
    }
}

impl Node for IfNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        // Get the condition input
        let condition_value = context
            .get_input(&"condition".to_string())
            .ok_or_else(|| CanvasError::Node("Missing condition input".to_string()))?;

        // Parse condition as boolean
        let condition_bool = condition_value
            .as_bool()
            .ok_or_else(|| CanvasError::Node("Condition must be a boolean".to_string()))?;

        // Use gas for condition evaluation
        context.use_gas(10)?;

        let mut outputs = std::collections::HashMap::new();
        
        if condition_bool {
            outputs.insert("true_flow".to_string(), serde_json::Value::Bool(true));
        } else {
            outputs.insert("false_flow".to_string(), serde_json::Value::Bool(true));
        }

        Ok(NodeResult::success(outputs, 10))
    }

    fn node_type(&self) -> &str {
        "If"
    }

    fn name(&self) -> &str {
        "If Condition"
    }
}

/// Add node implementation
pub struct AddNode;

impl Node for AddNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        // Get input values
        let a = context
            .get_input(&"a".to_string())
            .ok_or_else(|| CanvasError::Node("Missing input 'a'".to_string()))?;
        let b = context
            .get_input(&"b".to_string())
            .ok_or_else(|| CanvasError::Node("Missing input 'b'".to_string()))?;

        // Parse as integers
        let a_int = a
            .as_i64()
            .ok_or_else(|| CanvasError::Node("Input 'a' must be an integer".to_string()))?;
        let b_int = b
            .as_i64()
            .ok_or_else(|| CanvasError::Node("Input 'b' must be an integer".to_string()))?;

        // Perform addition
        let result = a_int + b_int;

        // Use gas for arithmetic operation
        context.use_gas(3)?;

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("result".to_string(), serde_json::Value::Number(result.into()));

        Ok(NodeResult::success(outputs, 3))
    }

    fn node_type(&self) -> &str {
        "Add"
    }

    fn name(&self) -> &str {
        "Add"
    }
}

/// Read Storage node implementation
pub struct ReadStorageNode {
    key: String,
}

impl ReadStorageNode {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}

impl Node for ReadStorageNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        // Get the key input
        let key_value = context
            .get_input(&"key".to_string())
            .ok_or_else(|| CanvasError::Node("Missing key input".to_string()))?;

        let key = key_value
            .as_str()
            .ok_or_else(|| CanvasError::Node("Key must be a string".to_string()))?;

        // Read from storage (simulated for now)
        let value = context.execution_context.storage.get(key).cloned()
            .unwrap_or(serde_json::Value::Null);

        // Use gas for storage read
        context.use_gas(100)?;

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("value".to_string(), value);

        Ok(NodeResult::success(outputs, 100))
    }

    fn node_type(&self) -> &str {
        "ReadStorage"
    }

    fn name(&self) -> &str {
        "Read Storage"
    }
}

/// Write Storage node implementation
pub struct WriteStorageNode {
    key: String,
}

impl WriteStorageNode {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}

impl Node for WriteStorageNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        // Get the key and value inputs
        let key_value = context
            .get_input(&"key".to_string())
            .ok_or_else(|| CanvasError::Node("Missing key input".to_string()))?;
        let value = context
            .get_input(&"value".to_string())
            .ok_or_else(|| CanvasError::Node("Missing value input".to_string()))?;

        let key = key_value
            .as_str()
            .ok_or_else(|| CanvasError::Node("Key must be a string".to_string()))?;

        // Write to storage
        context.execution_context.storage.insert(key.to_string(), value.clone());

        // Use gas for storage write
        context.use_gas(200)?;

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("success".to_string(), serde_json::Value::Bool(true));

        Ok(NodeResult::success(outputs, 200))
    }

    fn node_type(&self) -> &str {
        "WriteStorage"
    }

    fn name(&self) -> &str {
        "Write Storage"
    }
}

/// Start node implementation
pub struct StartNode;

impl Node for StartNode {
    fn execute(&self, _context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        // Start node just initiates flow
        let mut outputs = std::collections::HashMap::new();
        outputs.insert("flow_out".to_string(), serde_json::Value::Bool(true));

        Ok(NodeResult::success(outputs, 0))
    }

    fn node_type(&self) -> &str {
        "Start"
    }

    fn name(&self) -> &str {
        "Start"
    }
}

/// End node implementation
pub struct EndNode;

impl Node for EndNode {
    fn execute(&self, _context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        // End node terminates flow
        Ok(NodeResult::success(std::collections::HashMap::new(), 0))
    }

    fn node_type(&self) -> &str {
        "End"
    }

    fn name(&self) -> &str {
        "End"
    }
}

/// Node factory for creating nodes
pub struct NodeFactory;

impl NodeFactory {
    /// Create a node by type
    pub fn create_node(node_type: &str, properties: &std::collections::HashMap<String, serde_json::Value>) -> CanvasResult<Box<dyn Node>> {
        match node_type {
            "If" => {
                let condition = properties
                    .get("condition_expression")
                    .and_then(|v| v.as_str())
                    .unwrap_or("true")
                    .to_string();
                Ok(Box::new(IfNode::new(condition)))
            }
            "Add" => Ok(Box::new(AddNode)),
            "ReadStorage" => {
                let key = properties
                    .get("key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default_key")
                    .to_string();
                Ok(Box::new(ReadStorageNode::new(key)))
            }
            "WriteStorage" => {
                let key = properties
                    .get("key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default_key")
                    .to_string();
                Ok(Box::new(WriteStorageNode::new(key)))
            }
            "Start" => Ok(Box::new(StartNode)),
            "End" => Ok(Box::new(EndNode)),
            _ => Err(CanvasError::Node(format!("Unknown node type: {}", node_type))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ExecutionContext;

    #[test]
    fn test_if_node() {
        let mut context = crate::nodes::NodeContext::new(ExecutionContext::new(1000));
        context.inputs.insert("condition".to_string(), serde_json::Value::Bool(true));
        
        let node = IfNode::new("true");
        let result = node.execute(&mut context);
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert!(result.outputs.contains_key("true_flow"));
    }

    #[test]
    fn test_add_node() {
        let mut context = crate::nodes::NodeContext::new(ExecutionContext::new(1000));
        context.inputs.insert("a".to_string(), serde_json::Value::Number(5.into()));
        context.inputs.insert("b".to_string(), serde_json::Value::Number(3.into()));
        
        let node = AddNode;
        let result = node.execute(&mut context);
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert_eq!(result.outputs.get("result").unwrap().as_i64().unwrap(), 8);
    }

    #[test]
    fn test_node_factory() {
        let mut properties = std::collections::HashMap::new();
        properties.insert("condition_expression".to_string(), serde_json::Value::String("true".to_string()));
        
        let node = NodeFactory::create_node("If", &properties);
        assert!(node.is_ok());
    }
} 