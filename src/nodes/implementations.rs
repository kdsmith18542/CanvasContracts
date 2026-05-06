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
        let condition_bool = if let Some(val) = context.get_input(&"condition".to_string()) {
            val.as_bool()
               .ok_or_else(|| CanvasError::Node("Condition must be a boolean".to_string()))?
        } else {
            // Fallback to property
            self.condition.to_lowercase() == "true"
        };

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
        // Get the key input, fallback to self.key
        let key = if let Some(val) = context.get_input(&"key".to_string()) {
            val.as_str()
               .ok_or_else(|| CanvasError::Node("Key must be a string".to_string()))?
               .to_string()
        } else {
            self.key.clone()
        };

        // Read from storage (simulated for now)
        let value = context.execution_context.storage.get(&key).cloned()
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
        // Get the key input, fallback to self.key
        let key = if let Some(val) = context.get_input(&"key".to_string()) {
            val.as_str()
               .ok_or_else(|| CanvasError::Node("Key must be a string".to_string()))?
               .to_string()
        } else {
            self.key.clone()
        };

        // Get the value input
        let value = context
            .get_input(&"value".to_string())
            .ok_or_else(|| CanvasError::Node("Missing value input".to_string()))?;

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

/// Subtract node implementation
pub struct SubtractNode;

impl Node for SubtractNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        let a = context
            .get_input(&"a".to_string())
            .ok_or_else(|| CanvasError::Node("Missing input 'a'".to_string()))?;
        let b = context
            .get_input(&"b".to_string())
            .ok_or_else(|| CanvasError::Node("Missing input 'b'".to_string()))?;

        let a_int = a
            .as_i64()
            .ok_or_else(|| CanvasError::Node("Input 'a' must be an integer".to_string()))?;
        let b_int = b
            .as_i64()
            .ok_or_else(|| CanvasError::Node("Input 'b' must be an integer".to_string()))?;

        let result = a_int - b_int;
        context.use_gas(3)?;

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("result".to_string(), serde_json::Value::Number(result.into()));
        Ok(NodeResult::success(outputs, 3))
    }

    fn node_type(&self) -> &str { "Subtract" }
    fn name(&self) -> &str { "Subtract" }
}

/// Multiply node implementation
pub struct MultiplyNode;

impl Node for MultiplyNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        let a = context
            .get_input(&"a".to_string())
            .ok_or_else(|| CanvasError::Node("Missing input 'a'".to_string()))?;
        let b = context
            .get_input(&"b".to_string())
            .ok_or_else(|| CanvasError::Node("Missing input 'b'".to_string()))?;

        let a_int = a
            .as_i64()
            .ok_or_else(|| CanvasError::Node("Input 'a' must be an integer".to_string()))?;
        let b_int = b
            .as_i64()
            .ok_or_else(|| CanvasError::Node("Input 'b' must be an integer".to_string()))?;

        let result = a_int * b_int;
        context.use_gas(5)?;

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("result".to_string(), serde_json::Value::Number(result.into()));
        Ok(NodeResult::success(outputs, 5))
    }

    fn node_type(&self) -> &str { "Multiply" }
    fn name(&self) -> &str { "Multiply" }
}

/// Divide node implementation
pub struct DivideNode;

impl Node for DivideNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        let a = context
            .get_input(&"a".to_string())
            .ok_or_else(|| CanvasError::Node("Missing input 'a'".to_string()))?;
        let b = context
            .get_input(&"b".to_string())
            .ok_or_else(|| CanvasError::Node("Missing input 'b'".to_string()))?;

        let a_int = a
            .as_i64()
            .ok_or_else(|| CanvasError::Node("Input 'a' must be an integer".to_string()))?;
        let b_int = b
            .as_i64()
            .ok_or_else(|| CanvasError::Node("Input 'b' must be an integer".to_string()))?;

        if b_int == 0 {
            return Err(CanvasError::Node("Division by zero".to_string()));
        }

        let result = a_int / b_int;
        context.use_gas(5)?;

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("result".to_string(), serde_json::Value::Number(result.into()));
        Ok(NodeResult::success(outputs, 5))
    }

    fn node_type(&self) -> &str { "Divide" }
    fn name(&self) -> &str { "Divide" }
}

/// Logical AND node implementation
pub struct AndNode;

impl Node for AndNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        let a = context
            .get_input(&"a".to_string())
            .ok_or_else(|| CanvasError::Node("Missing input 'a'".to_string()))?;
        let b = context
            .get_input(&"b".to_string())
            .ok_or_else(|| CanvasError::Node("Missing input 'b'".to_string()))?;

        let a_bool = a
            .as_bool()
            .ok_or_else(|| CanvasError::Node("Input 'a' must be a boolean".to_string()))?;
        let b_bool = b
            .as_bool()
            .ok_or_else(|| CanvasError::Node("Input 'b' must be a boolean".to_string()))?;

        context.use_gas(5)?;

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("result".to_string(), serde_json::Value::Bool(a_bool && b_bool));
        Ok(NodeResult::success(outputs, 5))
    }

    fn node_type(&self) -> &str { "And" }
    fn name(&self) -> &str { "Logical AND" }
}

/// Logical OR node implementation
pub struct OrNode;

impl Node for OrNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        let a = context
            .get_input(&"a".to_string())
            .ok_or_else(|| CanvasError::Node("Missing input 'a'".to_string()))?;
        let b = context
            .get_input(&"b".to_string())
            .ok_or_else(|| CanvasError::Node("Missing input 'b'".to_string()))?;

        let a_bool = a
            .as_bool()
            .ok_or_else(|| CanvasError::Node("Input 'a' must be a boolean".to_string()))?;
        let b_bool = b
            .as_bool()
            .ok_or_else(|| CanvasError::Node("Input 'b' must be a boolean".to_string()))?;

        context.use_gas(5)?;

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("result".to_string(), serde_json::Value::Bool(a_bool || b_bool));
        Ok(NodeResult::success(outputs, 5))
    }

    fn node_type(&self) -> &str { "Or" }
    fn name(&self) -> &str { "Logical OR" }
}

/// Logical NOT node implementation
pub struct NotNode;

impl Node for NotNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        let input = context
            .get_input(&"input".to_string())
            .ok_or_else(|| CanvasError::Node("Missing input 'input'".to_string()))?;

        let input_bool = input
            .as_bool()
            .ok_or_else(|| CanvasError::Node("Input must be a boolean".to_string()))?;

        context.use_gas(3)?;

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("result".to_string(), serde_json::Value::Bool(!input_bool));
        Ok(NodeResult::success(outputs, 3))
    }

    fn node_type(&self) -> &str { "Not" }
    fn name(&self) -> &str { "Logical NOT" }
}

/// Node factory for creating nodes by type identifier
pub struct NodeFactory;

impl NodeFactory {
    /// Create a node by type. Supports all 12 built-in node types.
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
            "Subtract" => Ok(Box::new(SubtractNode)),
            "Multiply" => Ok(Box::new(MultiplyNode)),
            "Divide" => Ok(Box::new(DivideNode)),
            "And" => Ok(Box::new(AndNode)),
            "Or" => Ok(Box::new(OrNode)),
            "Not" => Ok(Box::new(NotNode)),
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

    /// Helper to create a context with the given gas limit
    fn ctx(gas: u64) -> crate::nodes::NodeContext {
        crate::nodes::NodeContext::new(ExecutionContext::new(gas))
    }

    // ── If ────────────────────────────────────────────────────────────

    #[test]
    fn test_if_node_true() {
        let mut c = ctx(1000);
        c.inputs.insert("condition".to_string(), serde_json::json!(true));
        let r = IfNode::new("true").execute(&mut c).unwrap();
        assert!(r.outputs.contains_key("true_flow"));
        assert!(!r.outputs.contains_key("false_flow"));
    }

    #[test]
    fn test_if_node_false() {
        let mut c = ctx(1000);
        c.inputs.insert("condition".to_string(), serde_json::json!(false));
        let r = IfNode::new("true").execute(&mut c).unwrap();
        assert!(r.outputs.contains_key("false_flow"));
        assert!(!r.outputs.contains_key("true_flow"));
    }

    #[test]
    fn test_if_node_missing_input_fallback() {
        let mut c = ctx(1000);
        // Should no longer error, but fallback to property value ("true")
        let r = IfNode::new("true").execute(&mut c).unwrap();
        assert!(r.outputs.contains_key("true_flow"));
    }

    #[test]
    fn test_if_node_wrong_type() {
        let mut c = ctx(1000);
        c.inputs.insert("condition".to_string(), serde_json::json!(42));
        assert!(IfNode::new("true").execute(&mut c).is_err());
    }

    // ── Add ───────────────────────────────────────────────────────────

    #[test]
    fn test_add_node() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(5));
        c.inputs.insert("b".to_string(), serde_json::json!(3));
        let r = AddNode.execute(&mut c).unwrap();
        assert_eq!(r.outputs["result"].as_i64().unwrap(), 8);
        assert_eq!(r.gas_used, 3);
    }

    #[test]
    fn test_add_node_negative() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(-10));
        c.inputs.insert("b".to_string(), serde_json::json!(3));
        let r = AddNode.execute(&mut c).unwrap();
        assert_eq!(r.outputs["result"].as_i64().unwrap(), -7);
    }

    // ── Subtract ──────────────────────────────────────────────────────

    #[test]
    fn test_subtract_node() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(10));
        c.inputs.insert("b".to_string(), serde_json::json!(4));
        let r = SubtractNode.execute(&mut c).unwrap();
        assert_eq!(r.outputs["result"].as_i64().unwrap(), 6);
        assert_eq!(r.gas_used, 3);
    }

    #[test]
    fn test_subtract_node_negative_result() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(3));
        c.inputs.insert("b".to_string(), serde_json::json!(10));
        let r = SubtractNode.execute(&mut c).unwrap();
        assert_eq!(r.outputs["result"].as_i64().unwrap(), -7);
    }

    // ── Multiply ──────────────────────────────────────────────────────

    #[test]
    fn test_multiply_node() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(6));
        c.inputs.insert("b".to_string(), serde_json::json!(7));
        let r = MultiplyNode.execute(&mut c).unwrap();
        assert_eq!(r.outputs["result"].as_i64().unwrap(), 42);
        assert_eq!(r.gas_used, 5);
    }

    #[test]
    fn test_multiply_by_zero() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(999));
        c.inputs.insert("b".to_string(), serde_json::json!(0));
        let r = MultiplyNode.execute(&mut c).unwrap();
        assert_eq!(r.outputs["result"].as_i64().unwrap(), 0);
    }

    // ── Divide ────────────────────────────────────────────────────────

    #[test]
    fn test_divide_node() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(20));
        c.inputs.insert("b".to_string(), serde_json::json!(4));
        let r = DivideNode.execute(&mut c).unwrap();
        assert_eq!(r.outputs["result"].as_i64().unwrap(), 5);
        assert_eq!(r.gas_used, 5);
    }

    #[test]
    fn test_divide_by_zero() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(10));
        c.inputs.insert("b".to_string(), serde_json::json!(0));
        let err = DivideNode.execute(&mut c).unwrap_err();
        assert!(err.to_string().contains("Division by zero"));
    }

    #[test]
    fn test_divide_integer_truncation() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(7));
        c.inputs.insert("b".to_string(), serde_json::json!(2));
        let r = DivideNode.execute(&mut c).unwrap();
        assert_eq!(r.outputs["result"].as_i64().unwrap(), 3); // integer division
    }

    // ── And ───────────────────────────────────────────────────────────

    #[test]
    fn test_and_node_true_true() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(true));
        c.inputs.insert("b".to_string(), serde_json::json!(true));
        let r = AndNode.execute(&mut c).unwrap();
        assert_eq!(r.outputs["result"].as_bool().unwrap(), true);
    }

    #[test]
    fn test_and_node_true_false() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(true));
        c.inputs.insert("b".to_string(), serde_json::json!(false));
        let r = AndNode.execute(&mut c).unwrap();
        assert_eq!(r.outputs["result"].as_bool().unwrap(), false);
    }

    #[test]
    fn test_and_node_wrong_type() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(1));
        c.inputs.insert("b".to_string(), serde_json::json!(true));
        assert!(AndNode.execute(&mut c).is_err());
    }

    // ── Or ────────────────────────────────────────────────────────────

    #[test]
    fn test_or_node_false_false() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(false));
        c.inputs.insert("b".to_string(), serde_json::json!(false));
        let r = OrNode.execute(&mut c).unwrap();
        assert_eq!(r.outputs["result"].as_bool().unwrap(), false);
    }

    #[test]
    fn test_or_node_true_false() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(true));
        c.inputs.insert("b".to_string(), serde_json::json!(false));
        let r = OrNode.execute(&mut c).unwrap();
        assert_eq!(r.outputs["result"].as_bool().unwrap(), true);
    }

    // ── Not ───────────────────────────────────────────────────────────

    #[test]
    fn test_not_node_true() {
        let mut c = ctx(1000);
        c.inputs.insert("input".to_string(), serde_json::json!(true));
        let r = NotNode.execute(&mut c).unwrap();
        assert_eq!(r.outputs["result"].as_bool().unwrap(), false);
    }

    #[test]
    fn test_not_node_false() {
        let mut c = ctx(1000);
        c.inputs.insert("input".to_string(), serde_json::json!(false));
        let r = NotNode.execute(&mut c).unwrap();
        assert_eq!(r.outputs["result"].as_bool().unwrap(), true);
    }

    #[test]
    fn test_not_node_wrong_type() {
        let mut c = ctx(1000);
        c.inputs.insert("input".to_string(), serde_json::json!("not_a_bool"));
        assert!(NotNode.execute(&mut c).is_err());
    }

    // ── Storage ───────────────────────────────────────────────────────

    #[test]
    fn test_write_then_read_storage() {
        let mut c = ctx(10000);
        c.inputs.insert("key".to_string(), serde_json::json!("mykey"));
        c.inputs.insert("value".to_string(), serde_json::json!(42));
        WriteStorageNode::new("mykey").execute(&mut c).unwrap();

        // Now read it back
        let mut c2 = crate::nodes::NodeContext::new(c.execution_context);
        c2.inputs.insert("key".to_string(), serde_json::json!("mykey"));
        let r = ReadStorageNode::new("mykey").execute(&mut c2).unwrap();
        assert_eq!(r.outputs["value"].as_i64().unwrap(), 42);
    }

    // ── Start / End ───────────────────────────────────────────────────

    #[test]
    fn test_start_node() {
        let mut c = ctx(1000);
        let r = StartNode.execute(&mut c).unwrap();
        assert!(r.outputs.contains_key("flow_out"));
        assert_eq!(r.gas_used, 0);
    }

    #[test]
    fn test_end_node() {
        let mut c = ctx(1000);
        let r = EndNode.execute(&mut c).unwrap();
        assert!(r.outputs.is_empty());
        assert_eq!(r.gas_used, 0);
    }

    // ── Gas exhaustion ────────────────────────────────────────────────

    #[test]
    fn test_arithmetic_gas_exhaustion() {
        let mut c = ctx(2); // only 2 gas, Add costs 3
        c.inputs.insert("a".to_string(), serde_json::json!(1));
        c.inputs.insert("b".to_string(), serde_json::json!(2));
        assert!(AddNode.execute(&mut c).is_err());
    }

    // ── Factory ───────────────────────────────────────────────────────

    #[test]
    fn test_factory_creates_all_12_types() {
        let props = std::collections::HashMap::new();
        let types = [
            "If", "Add", "Subtract", "Multiply", "Divide",
            "And", "Or", "Not", "ReadStorage", "WriteStorage",
            "Start", "End",
        ];
        for t in &types {
            let node = NodeFactory::create_node(t, &props);
            assert!(node.is_ok(), "Factory failed for type: {}", t);
            assert_eq!(node.unwrap().node_type(), *t);
        }
    }

    #[test]
    fn test_factory_unknown_type() {
        let props = std::collections::HashMap::new();
        assert!(NodeFactory::create_node("Nonexistent", &props).is_err());
    }
} 