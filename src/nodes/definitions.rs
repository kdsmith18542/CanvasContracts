//! Node definitions and schemas

use serde::{Deserialize, Serialize};
use crate::types::{Port, ValueType};

/// Node definition schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
    /// Unique identifier for the node type
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what the node does
    pub description: String,
    /// Category for organization
    pub category: String,
    /// Input ports
    pub inputs: Vec<Port>,
    /// Output ports
    pub outputs: Vec<Port>,
    /// Configuration schema (JSON Schema)
    pub config_schema: serde_json::Value,
    /// Compiler hints for code generation
    pub compiler_hint: CompilerHint,
    /// Visual properties
    pub visual: VisualProperties,
}

/// Compiler hints for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerHint {
    /// Type of operation this node represents
    pub operation_type: String,
    /// Expression field name (if applicable)
    pub expression_field: Option<String>,
    /// Gas cost estimation
    pub gas_cost: Option<u64>,
    /// Whether this node can be optimized
    pub optimizable: bool,
}

/// Visual properties for the node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualProperties {
    /// Default width
    pub width: f64,
    /// Default height
    pub height: f64,
    /// Color theme
    pub color: String,
    /// Icon name
    pub icon: Option<String>,
}

impl NodeDefinition {
    /// Create a new node definition
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            category: category.into(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            config_schema: serde_json::Value::Object(serde_json::Map::new()),
            compiler_hint: CompilerHint {
                operation_type: "unknown".to_string(),
                expression_field: None,
                gas_cost: None,
                optimizable: true,
            },
            visual: VisualProperties {
                width: 120.0,
                height: 80.0,
                color: "#4A90E2".to_string(),
                icon: None,
            },
        }
    }

    /// Add an input port
    pub fn with_input(mut self, port: Port) -> Self {
        self.inputs.push(port);
        self
    }

    /// Add an output port
    pub fn with_output(mut self, port: Port) -> Self {
        self.outputs.push(port);
        self
    }

    /// Set the configuration schema
    pub fn with_config_schema(mut self, schema: serde_json::Value) -> Self {
        self.config_schema = schema;
        self
    }

    /// Set compiler hints
    pub fn with_compiler_hint(mut self, hint: CompilerHint) -> Self {
        self.compiler_hint = hint;
        self
    }

    /// Set visual properties
    pub fn with_visual(mut self, visual: VisualProperties) -> Self {
        self.visual = visual;
        self
    }
}

/// Built-in node definitions
pub fn builtin_node_definitions() -> Vec<NodeDefinition> {
    vec![
        // Logic nodes
        create_if_node(),
        create_and_node(),
        create_or_node(),
        create_not_node(),
        
        // State nodes
        create_read_storage_node(),
        create_write_storage_node(),
        
        // Arithmetic nodes
        create_add_node(),
        create_subtract_node(),
        create_multiply_node(),
        create_divide_node(),
        
        // Control flow nodes
        create_start_node(),
        create_end_node(),
    ]
}

fn create_if_node() -> NodeDefinition {
    NodeDefinition::new("If", "If Condition", "Executes different paths based on a boolean condition", "Logic")
        .with_input(Port::new("condition", "Condition", ValueType::Boolean).required())
        .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
        .with_output(Port::new("true_flow", "True Flow", ValueType::Flow))
        .with_output(Port::new("false_flow", "False Flow", ValueType::Flow))
        .with_config_schema(serde_json::json!({
            "type": "object",
            "properties": {
                "condition_expression": {
                    "type": "string",
                    "description": "Boolean expression for the condition"
                }
            },
            "required": ["condition_expression"]
        }))
        .with_compiler_hint(CompilerHint {
            operation_type: "conditional_branch".to_string(),
            expression_field: Some("condition_expression".to_string()),
            gas_cost: Some(10),
            optimizable: true,
        })
        .with_visual(VisualProperties {
            width: 100.0,
            height: 60.0,
            color: "#FF6B6B".to_string(),
            icon: Some("if".to_string()),
        })
}

fn create_and_node() -> NodeDefinition {
    NodeDefinition::new("And", "Logical AND", "Performs logical AND operation", "Logic")
        .with_input(Port::new("a", "A", ValueType::Boolean).required())
        .with_input(Port::new("b", "B", ValueType::Boolean).required())
        .with_output(Port::new("result", "Result", ValueType::Boolean))
        .with_compiler_hint(CompilerHint {
            operation_type: "logical_and".to_string(),
            expression_field: None,
            gas_cost: Some(5),
            optimizable: true,
        })
}

fn create_or_node() -> NodeDefinition {
    NodeDefinition::new("Or", "Logical OR", "Performs logical OR operation", "Logic")
        .with_input(Port::new("a", "A", ValueType::Boolean).required())
        .with_input(Port::new("b", "B", ValueType::Boolean).required())
        .with_output(Port::new("result", "Result", ValueType::Boolean))
        .with_compiler_hint(CompilerHint {
            operation_type: "logical_or".to_string(),
            expression_field: None,
            gas_cost: Some(5),
            optimizable: true,
        })
}

fn create_not_node() -> NodeDefinition {
    NodeDefinition::new("Not", "Logical NOT", "Performs logical NOT operation", "Logic")
        .with_input(Port::new("input", "Input", ValueType::Boolean).required())
        .with_output(Port::new("result", "Result", ValueType::Boolean))
        .with_compiler_hint(CompilerHint {
            operation_type: "logical_not".to_string(),
            expression_field: None,
            gas_cost: Some(3),
            optimizable: true,
        })
}

fn create_read_storage_node() -> NodeDefinition {
    NodeDefinition::new("ReadStorage", "Read Storage", "Reads a value from contract storage", "State")
        .with_input(Port::new("key", "Key", ValueType::String).required())
        .with_output(Port::new("value", "Value", ValueType::Any))
        .with_config_schema(serde_json::json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "Storage key to read"
                }
            },
            "required": ["key"]
        }))
        .with_compiler_hint(CompilerHint {
            operation_type: "read_storage".to_string(),
            expression_field: Some("key".to_string()),
            gas_cost: Some(100),
            optimizable: false,
        })
}

fn create_write_storage_node() -> NodeDefinition {
    NodeDefinition::new("WriteStorage", "Write Storage", "Writes a value to contract storage", "State")
        .with_input(Port::new("key", "Key", ValueType::String).required())
        .with_input(Port::new("value", "Value", ValueType::Any).required())
        .with_config_schema(serde_json::json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "Storage key to write"
                }
            },
            "required": ["key"]
        }))
        .with_compiler_hint(CompilerHint {
            operation_type: "write_storage".to_string(),
            expression_field: Some("key".to_string()),
            gas_cost: Some(200),
            optimizable: false,
        })
}

fn create_add_node() -> NodeDefinition {
    NodeDefinition::new("Add", "Add", "Adds two numbers", "Arithmetic")
        .with_input(Port::new("a", "A", ValueType::Integer).required())
        .with_input(Port::new("b", "B", ValueType::Integer).required())
        .with_output(Port::new("result", "Result", ValueType::Integer))
        .with_compiler_hint(CompilerHint {
            operation_type: "add".to_string(),
            expression_field: None,
            gas_cost: Some(3),
            optimizable: true,
        })
}

fn create_subtract_node() -> NodeDefinition {
    NodeDefinition::new("Subtract", "Subtract", "Subtracts two numbers", "Arithmetic")
        .with_input(Port::new("a", "A", ValueType::Integer).required())
        .with_input(Port::new("b", "B", ValueType::Integer).required())
        .with_output(Port::new("result", "Result", ValueType::Integer))
        .with_compiler_hint(CompilerHint {
            operation_type: "subtract".to_string(),
            expression_field: None,
            gas_cost: Some(3),
            optimizable: true,
        })
}

fn create_multiply_node() -> NodeDefinition {
    NodeDefinition::new("Multiply", "Multiply", "Multiplies two numbers", "Arithmetic")
        .with_input(Port::new("a", "A", ValueType::Integer).required())
        .with_input(Port::new("b", "B", ValueType::Integer).required())
        .with_output(Port::new("result", "Result", ValueType::Integer))
        .with_compiler_hint(CompilerHint {
            operation_type: "multiply".to_string(),
            expression_field: None,
            gas_cost: Some(5),
            optimizable: true,
        })
}

fn create_divide_node() -> NodeDefinition {
    NodeDefinition::new("Divide", "Divide", "Divides two numbers", "Arithmetic")
        .with_input(Port::new("a", "A", ValueType::Integer).required())
        .with_input(Port::new("b", "B", ValueType::Integer).required())
        .with_output(Port::new("result", "Result", ValueType::Integer))
        .with_compiler_hint(CompilerHint {
            operation_type: "divide".to_string(),
            expression_field: None,
            gas_cost: Some(5),
            optimizable: true,
        })
}

fn create_start_node() -> NodeDefinition {
    NodeDefinition::new("Start", "Start", "Entry point for contract execution", "Control Flow")
        .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
        .with_compiler_hint(CompilerHint {
            operation_type: "start".to_string(),
            expression_field: None,
            gas_cost: Some(0),
            optimizable: false,
        })
}

fn create_end_node() -> NodeDefinition {
    NodeDefinition::new("End", "End", "Exit point for contract execution", "Control Flow")
        .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
        .with_compiler_hint(CompilerHint {
            operation_type: "end".to_string(),
            expression_field: None,
            gas_cost: Some(0),
            optimizable: false,
        })
} 