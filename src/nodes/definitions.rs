//! Node definitions and schemas

use crate::types::{Port, ValueType};
use serde::{Deserialize, Serialize};

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
        // Crypto nodes
        create_verify_signature_node(),
        create_decode_proof_node(),
        // BaaLS runtime nodes
        create_get_sender_node(),
        create_get_contract_id_node(),
        create_get_block_timestamp_node(),
        create_get_block_height_node(),
        create_emit_event_node(),
        create_revert_node(),
        create_hash_sha256_node(),
        create_call_contract_node(),
        create_read_call_result_node(),
        create_transfer_value_node(),
        // ChronoNode proof nodes
        create_fetch_chrono_block_node(),
        create_fetch_checkpoint_node(),
        create_verify_chrono_proof_node(),
        create_extract_chrono_event_node(),
        create_extract_tx_by_sender_node(),
        create_extract_tx_by_recipient_node(),
        create_verify_archive_range_node(),
        // Resurgence DormancyOracle nodes
        create_check_token_age_node(),
        create_check_token_activity_window_node(),
        create_check_liquidity_dormancy_node(),
        create_check_governance_dormancy_node(),
        create_calculate_dormancy_score_node(),
        create_normalize_dead_coin_risk_node(),
        create_generate_dormancy_proof_node(),
        create_emit_dormancy_oracle_result_node(),
    ]
}

fn create_if_node() -> NodeDefinition {
    NodeDefinition::new(
        "If",
        "If Condition",
        "Executes different paths based on a boolean condition",
        "Logic",
    )
    .with_input(Port::new("condition", "Condition", ValueType::Boolean).required())
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_output(Port::new("true_flow", "True Flow", ValueType::Flow))
    .with_output(Port::new("false_flow", "False Flow", ValueType::Flow))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {
            "condition": {
                "type": ["boolean", "string"],
                "description": "Boolean literal or expression for the condition"
            },
            "condition_expression": {
                "type": "string",
                "description": "Legacy key for condition expression (deprecated; use 'condition')"
            }
        },
        "required": ["condition"]
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "conditional_branch".to_string(),
        expression_field: Some("condition".to_string()),
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
    NodeDefinition::new(
        "And",
        "Logical AND",
        "Performs logical AND operation",
        "Logic",
    )
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
    NodeDefinition::new(
        "Not",
        "Logical NOT",
        "Performs logical NOT operation",
        "Logic",
    )
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
    NodeDefinition::new(
        "ReadStorage",
        "Read Storage",
        "Reads a value from contract storage",
        "State",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("key", "Key", ValueType::String))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
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
    .with_visual(VisualProperties {
        width: 140.0,
        height: 100.0,
        color: "#4ECDC4".to_string(),
        icon: Some("database".to_string()),
    })
}

fn create_write_storage_node() -> NodeDefinition {
    NodeDefinition::new(
        "WriteStorage",
        "Write Storage",
        "Writes a value to contract storage",
        "State",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("key", "Key", ValueType::String))
    .with_input(Port::new("value", "Value", ValueType::Any))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
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
    .with_visual(VisualProperties {
        width: 140.0,
        height: 100.0,
        color: "#45B7D1".to_string(),
        icon: Some("save".to_string()),
    })
}

fn create_add_node() -> NodeDefinition {
    NodeDefinition::new("Add", "Add", "Adds two numbers", "Arithmetic")
        .with_input(Port::new("flow_in", "Flow In", ValueType::Flow))
        .with_input(Port::new("a", "A", ValueType::Integer).required())
        .with_input(Port::new("b", "B", ValueType::Integer).required())
        .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
        .with_output(Port::new("result", "Result", ValueType::Integer))
        .with_compiler_hint(CompilerHint {
            operation_type: "add".to_string(),
            expression_field: None,
            gas_cost: Some(3),
            optimizable: true,
        })
}

fn create_subtract_node() -> NodeDefinition {
    NodeDefinition::new(
        "Subtract",
        "Subtract",
        "Subtracts two numbers",
        "Arithmetic",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow))
    .with_input(Port::new("a", "A", ValueType::Integer).required())
    .with_input(Port::new("b", "B", ValueType::Integer).required())
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("result", "Result", ValueType::Integer))
    .with_compiler_hint(CompilerHint {
        operation_type: "subtract".to_string(),
        expression_field: None,
        gas_cost: Some(3),
        optimizable: true,
    })
}

fn create_multiply_node() -> NodeDefinition {
    NodeDefinition::new(
        "Multiply",
        "Multiply",
        "Multiplies two numbers",
        "Arithmetic",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow))
    .with_input(Port::new("a", "A", ValueType::Integer).required())
    .with_input(Port::new("b", "B", ValueType::Integer).required())
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
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
        .with_input(Port::new("flow_in", "Flow In", ValueType::Flow))
        .with_input(Port::new("a", "A", ValueType::Integer).required())
        .with_input(Port::new("b", "B", ValueType::Integer).required())
        .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
        .with_output(Port::new("result", "Result", ValueType::Integer))
        .with_compiler_hint(CompilerHint {
            operation_type: "divide".to_string(),
            expression_field: None,
            gas_cost: Some(5),
            optimizable: true,
        })
}

fn create_start_node() -> NodeDefinition {
    NodeDefinition::new(
        "Start",
        "Start",
        "Entry point for contract execution",
        "Control Flow",
    )
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_compiler_hint(CompilerHint {
        operation_type: "start".to_string(),
        expression_field: None,
        gas_cost: Some(0),
        optimizable: false,
    })
}

fn create_end_node() -> NodeDefinition {
    NodeDefinition::new(
        "End",
        "End",
        "Exit point for contract execution",
        "Control Flow",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_compiler_hint(CompilerHint {
        operation_type: "end".to_string(),
        expression_field: None,
        gas_cost: Some(0),
        optimizable: false,
    })
}

fn create_verify_signature_node() -> NodeDefinition {
    NodeDefinition::new(
        "VerifySignature",
        "Verify Signature",
        "Verifies an ed25519 cryptographic signature",
        "Crypto",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("message", "Message", ValueType::Bytes))
    .with_input(Port::new("signature", "Signature", ValueType::Bytes))
    .with_input(Port::new("public_key", "Public Key", ValueType::Bytes))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("result", "Result", ValueType::Boolean))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {},
        "required": []
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "verify_signature".to_string(),
        expression_field: None,
        gas_cost: Some(100),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 120.0,
        height: 80.0,
        color: "#9B59B6".to_string(),
        icon: Some("shield".to_string()),
    })
}

fn create_decode_proof_node() -> NodeDefinition {
    NodeDefinition::new(
        "DecodeProof",
        "Decode Proof",
        "Deserializes a DormancyProof JSON payload",
        "Crypto",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("proof_json", "Proof JSON", ValueType::String).required())
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("chain_id", "Chain ID", ValueType::String))
    .with_output(Port::new("address", "Address", ValueType::String))
    .with_output(Port::new(
        "dormant_since_block",
        "Dormant Since Block",
        ValueType::Integer,
    ))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {},
        "required": []
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "decode_proof".to_string(),
        expression_field: None,
        gas_cost: Some(50),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 120.0,
        height: 80.0,
        color: "#8E44AD".to_string(),
        icon: Some("file".to_string()),
    })
}

// BaaLS Runtime Nodes

fn create_get_sender_node() -> NodeDefinition {
    NodeDefinition::new(
        "GetSender",
        "Get Sender",
        "Reads the transaction sender address",
        "BaaLS Runtime",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("sender", "Sender", ValueType::String))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {},
        "required": []
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "get_sender".to_string(),
        expression_field: None,
        gas_cost: Some(10),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 120.0,
        height: 80.0,
        color: "#E74C3C".to_string(),
        icon: Some("user".to_string()),
    })
}

fn create_get_contract_id_node() -> NodeDefinition {
    NodeDefinition::new(
        "GetContractId",
        "Get Contract ID",
        "Reads the current contract's ID",
        "BaaLS Runtime",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("contract_id", "Contract ID", ValueType::String))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {},
        "required": []
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "get_contract_id".to_string(),
        expression_field: None,
        gas_cost: Some(10),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 120.0,
        height: 80.0,
        color: "#E74C3C".to_string(),
        icon: Some("file-code".to_string()),
    })
}

fn create_get_block_timestamp_node() -> NodeDefinition {
    NodeDefinition::new(
        "GetBlockTimestamp",
        "Get Block Timestamp",
        "Reads the current block timestamp",
        "BaaLS Runtime",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("timestamp", "Timestamp", ValueType::Integer))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {},
        "required": []
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "get_block_timestamp".to_string(),
        expression_field: None,
        gas_cost: Some(10),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 120.0,
        height: 80.0,
        color: "#E74C3C".to_string(),
        icon: Some("clock".to_string()),
    })
}

fn create_get_block_height_node() -> NodeDefinition {
    NodeDefinition::new(
        "GetBlockHeight",
        "Get Block Height",
        "Reads the current block number",
        "BaaLS Runtime",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("height", "Height", ValueType::Integer))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {},
        "required": []
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "get_block_height".to_string(),
        expression_field: None,
        gas_cost: Some(10),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 120.0,
        height: 80.0,
        color: "#E74C3C".to_string(),
        icon: Some("layers".to_string()),
    })
}

fn create_emit_event_node() -> NodeDefinition {
    NodeDefinition::new(
        "EmitEvent",
        "Emit Event",
        "Emits a named event with key-value data",
        "BaaLS Runtime",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("event_name", "Event Name", ValueType::String))
    .with_input(Port::new(
        "event_data",
        "Event Data",
        ValueType::Object(std::collections::HashMap::new()),
    ))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {
            "event_name": {
                "type": "string",
                "description": "Name of the event to emit"
            }
        },
        "required": ["event_name"]
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "emit_event".to_string(),
        expression_field: Some("event_name".to_string()),
        gas_cost: Some(50),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 120.0,
        height: 80.0,
        color: "#E74C3C".to_string(),
        icon: Some("bell".to_string()),
    })
}

fn create_revert_node() -> NodeDefinition {
    NodeDefinition::new(
        "Revert",
        "Revert",
        "Aborts execution with a reason string",
        "BaaLS Runtime",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("reason", "Reason", ValueType::String))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {
            "reason": {
                "type": "string",
                "description": "Revert reason message"
            }
        },
        "required": ["reason"]
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "revert".to_string(),
        expression_field: Some("reason".to_string()),
        gas_cost: Some(10),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 120.0,
        height: 80.0,
        color: "#C0392B".to_string(),
        icon: Some("x-circle".to_string()),
    })
}

fn create_hash_sha256_node() -> NodeDefinition {
    NodeDefinition::new(
        "HashSha256",
        "Hash SHA-256",
        "Computes SHA-256 hash of input bytes",
        "BaaLS Runtime",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("input", "Input", ValueType::Bytes).required())
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("hash", "Hash", ValueType::Bytes))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {},
        "required": []
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "hash_sha256".to_string(),
        expression_field: None,
        gas_cost: Some(100),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 120.0,
        height: 80.0,
        color: "#E74C3C".to_string(),
        icon: Some("hash".to_string()),
    })
}

// ChronoNode Proof Nodes

fn create_fetch_chrono_block_node() -> NodeDefinition {
    NodeDefinition::new(
        "FetchChronoBlock",
        "Fetch Chrono Block",
        "Query ChronoNode for a block by height/chain_id",
        "ChronoNode",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("chain_id", "Chain ID", ValueType::String))
    .with_input(Port::new("height", "Height", ValueType::Integer))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new(
        "block_data",
        "Block Data",
        ValueType::Object(std::collections::HashMap::new()),
    ))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {
            "chain_id": {"type": "string"},
            "height": {"type": "integer"}
        }
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "fetch_chrono_block".to_string(),
        expression_field: None,
        gas_cost: Some(500),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#16A085".to_string(),
        icon: Some("database".to_string()),
    })
}

fn create_fetch_checkpoint_node() -> NodeDefinition {
    NodeDefinition::new(
        "FetchCheckpoint",
        "Fetch Checkpoint",
        "Query ChronoNode checkpoint for a block range",
        "ChronoNode",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("chain_id", "Chain ID", ValueType::String))
    .with_input(Port::new("from_height", "From Height", ValueType::Integer))
    .with_input(Port::new("to_height", "To Height", ValueType::Integer))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new(
        "checkpoint",
        "Checkpoint",
        ValueType::Object(std::collections::HashMap::new()),
    ))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {"chain_id": {"type": "string"}}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "fetch_checkpoint".to_string(),
        expression_field: None,
        gas_cost: Some(300),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#16A085".to_string(),
        icon: Some("bookmark".to_string()),
    })
}

fn create_verify_chrono_proof_node() -> NodeDefinition {
    NodeDefinition::new(
        "VerifyChronoProof",
        "Verify Chrono Proof",
        "Verify a Merkle proof from ChronoNode",
        "ChronoNode",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(
        Port::new(
            "proof",
            "Proof",
            ValueType::Object(std::collections::HashMap::new()),
        )
        .required(),
    )
    .with_input(Port::new("data", "Data", ValueType::Bytes).required())
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("valid", "Valid", ValueType::Boolean))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "verify_chrono_proof".to_string(),
        expression_field: None,
        gas_cost: Some(500),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#16A085".to_string(),
        icon: Some("shield-check".to_string()),
    })
}

fn create_extract_chrono_event_node() -> NodeDefinition {
    NodeDefinition::new(
        "ExtractChronoEvent",
        "Extract Chrono Event",
        "Extract an event from a ChronoBlock",
        "ChronoNode",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new(
        "block_data",
        "Block Data",
        ValueType::Object(std::collections::HashMap::new()),
    ))
    .with_input(Port::new("event_type", "Event Type", ValueType::String))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new(
        "event_data",
        "Event Data",
        ValueType::Object(std::collections::HashMap::new()),
    ))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {"event_type": {"type": "string"}}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "extract_chrono_event".to_string(),
        expression_field: Some("event_type".to_string()),
        gas_cost: Some(100),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#16A085".to_string(),
        icon: Some("file-text".to_string()),
    })
}

fn create_extract_tx_by_sender_node() -> NodeDefinition {
    NodeDefinition::new(
        "ExtractTxBySender",
        "Extract Tx By Sender",
        "Filter transactions by sender address",
        "ChronoNode",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new(
        "block_data",
        "Block Data",
        ValueType::Object(std::collections::HashMap::new()),
    ))
    .with_input(Port::new("sender", "Sender", ValueType::String))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new(
        "transactions",
        "Transactions",
        ValueType::Array(Box::new(
            ValueType::Object(std::collections::HashMap::new()),
        )),
    ))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "extract_tx_by_sender".to_string(),
        expression_field: None,
        gas_cost: Some(200),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#16A085".to_string(),
        icon: Some("search".to_string()),
    })
}

fn create_verify_archive_range_node() -> NodeDefinition {
    NodeDefinition::new(
        "VerifyArchiveRange",
        "Verify Archive Range",
        "Verify a range of archived blocks",
        "ChronoNode",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("chain_id", "Chain ID", ValueType::String).required())
    .with_input(Port::new("from_height", "From Height", ValueType::Integer).required())
    .with_input(Port::new("to_height", "To Height", ValueType::Integer).required())
    .with_input(
        Port::new(
            "proof",
            "Proof",
            ValueType::Object(std::collections::HashMap::new()),
        )
        .required(),
    )
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("valid", "Valid", ValueType::Boolean))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "verify_archive_range".to_string(),
        expression_field: None,
        gas_cost: Some(500),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#16A085".to_string(),
        icon: Some("check-square".to_string()),
    })
}

// Resurgence DormancyOracle Nodes

fn create_check_token_age_node() -> NodeDefinition {
    NodeDefinition::new(
        "CheckTokenAge",
        "Check Token Age",
        "Calculate time since last token activity",
        "Resurgence",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new(
        "token_address",
        "Token Address",
        ValueType::String,
    ))
    .with_input(Port::new(
        "current_block",
        "Current Block",
        ValueType::Integer,
    ))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("age_blocks", "Age (Blocks)", ValueType::Integer))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "check_token_age".to_string(),
        expression_field: None,
        gas_cost: Some(50),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#D35400".to_string(),
        icon: Some("clock".to_string()),
    })
}

fn create_check_token_activity_window_node() -> NodeDefinition {
    NodeDefinition::new(
        "CheckTokenActivityWindow",
        "Check Token Activity Window",
        "Check if token was active in a time window",
        "Resurgence",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new(
        "token_address",
        "Token Address",
        ValueType::String,
    ))
    .with_input(Port::new(
        "window_start",
        "Window Start",
        ValueType::Integer,
    ))
    .with_input(Port::new("window_end", "Window End", ValueType::Integer))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("active", "Active", ValueType::Boolean))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "check_token_activity_window".to_string(),
        expression_field: None,
        gas_cost: Some(100),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#D35400".to_string(),
        icon: Some("activity".to_string()),
    })
}

fn create_check_liquidity_dormancy_node() -> NodeDefinition {
    NodeDefinition::new(
        "CheckLiquidityDormancy",
        "Check Liquidity Dormancy",
        "Evaluate liquidity pool dormancy",
        "Resurgence",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("pool_address", "Pool Address", ValueType::String))
    .with_input(Port::new(
        "liquidity_threshold",
        "Liquidity Threshold",
        ValueType::Integer,
    ))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("dormant", "Dormant", ValueType::Boolean))
    .with_output(Port::new(
        "liquidity_score",
        "Liquidity Score",
        ValueType::Integer,
    ))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "check_liquidity_dormancy".to_string(),
        expression_field: None,
        gas_cost: Some(100),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#D35400".to_string(),
        icon: Some("droplet".to_string()),
    })
}

fn create_check_governance_dormancy_node() -> NodeDefinition {
    NodeDefinition::new(
        "CheckGovernanceDormancy",
        "Check Governance Dormancy",
        "Evaluate governance activity dormancy",
        "Resurgence",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new(
        "token_address",
        "Token Address",
        ValueType::String,
    ))
    .with_input(Port::new(
        "activity_window",
        "Activity Window",
        ValueType::Integer,
    ))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("dormant", "Dormant", ValueType::Boolean))
    .with_output(Port::new(
        "governance_score",
        "Governance Score",
        ValueType::Integer,
    ))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "check_governance_dormancy".to_string(),
        expression_field: None,
        gas_cost: Some(100),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#D35400".to_string(),
        icon: Some("vote".to_string()),
    })
}

fn create_calculate_dormancy_score_node() -> NodeDefinition {
    NodeDefinition::new(
        "CalculateDormancyScore",
        "Calculate Dormancy Score",
        "Weighted scoring of dormancy factors",
        "Resurgence",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("age_score", "Age Score", ValueType::Integer))
    .with_input(Port::new(
        "liquidity_score",
        "Liquidity Score",
        ValueType::Integer,
    ))
    .with_input(Port::new(
        "governance_score",
        "Governance Score",
        ValueType::Integer,
    ))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new(
        "dormancy_score",
        "Dormancy Score",
        ValueType::Integer,
    ))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {
            "age_weight": {"type": "number", "default": 0.4},
            "liquidity_weight": {"type": "number", "default": 0.3},
            "governance_weight": {"type": "number", "default": 0.3}
        }
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "calculate_dormancy_score".to_string(),
        expression_field: None,
        gas_cost: Some(50),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#D35400".to_string(),
        icon: Some("calculator".to_string()),
    })
}

fn create_normalize_dead_coin_risk_node() -> NodeDefinition {
    NodeDefinition::new(
        "NormalizeDeadCoinRisk",
        "Normalize Dead Coin Risk",
        "Normalize risk score to standard range",
        "Resurgence",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("raw_score", "Raw Score", ValueType::Integer))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new(
        "normalized_score",
        "Normalized Score",
        ValueType::Integer,
    ))
    .with_output(Port::new("risk_label", "Risk Label", ValueType::String))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "normalize_dead_coin_risk".to_string(),
        expression_field: None,
        gas_cost: Some(30),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#D35400".to_string(),
        icon: Some("gauge".to_string()),
    })
}

fn create_generate_dormancy_proof_node() -> NodeDefinition {
    NodeDefinition::new(
        "GenerateDormancyProof",
        "Generate Dormancy Proof",
        "Assemble evidence manifest and hash",
        "Resurgence",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new(
        "token_address",
        "Token Address",
        ValueType::String,
    ))
    .with_input(Port::new(
        "dormancy_score",
        "Dormancy Score",
        ValueType::Integer,
    ))
    .with_input(Port::new(
        "evidence_sources",
        "Evidence Sources",
        ValueType::Array(Box::new(ValueType::String)),
    ))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("proof_hash", "Proof Hash", ValueType::Bytes))
    .with_output(Port::new(
        "manifest",
        "Manifest",
        ValueType::Object(std::collections::HashMap::new()),
    ))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "generate_dormancy_proof".to_string(),
        expression_field: None,
        gas_cost: Some(200),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#D35400".to_string(),
        icon: Some("file-check".to_string()),
    })
}

fn create_emit_dormancy_oracle_result_node() -> NodeDefinition {
    NodeDefinition::new(
        "EmitDormancyOracleResult",
        "Emit Dormancy Oracle Result",
        "Emit standardized dormancy oracle event",
        "Resurgence",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new(
        "token_address",
        "Token Address",
        ValueType::String,
    ))
    .with_input(Port::new(
        "dormancy_score",
        "Dormancy Score",
        ValueType::Integer,
    ))
    .with_input(Port::new("risk_label", "Risk Label", ValueType::String))
    .with_input(Port::new("proof_hash", "Proof Hash", ValueType::Bytes))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "emit_dormancy_oracle_result".to_string(),
        expression_field: None,
        gas_cost: Some(30),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#D35400".to_string(),
        icon: Some("broadcast".to_string()),
    })
}

fn create_call_contract_node() -> NodeDefinition {
    NodeDefinition::new(
        "CallContract",
        "Call Contract",
        "Call another contract via WASI host function",
        "BaaLS Runtime",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("contract_address", "Contract Address", ValueType::String).required())
    .with_input(Port::new("method", "Method", ValueType::String).required())
    .with_input(Port::new(
        "arguments",
        "Arguments",
        ValueType::Array(Box::new(ValueType::Any)),
    ))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("success", "Success", ValueType::Boolean))
    .with_output(Port::new("output", "Output", ValueType::Any))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "call_contract".to_string(),
        expression_field: None,
        gas_cost: Some(500),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 100.0,
        color: "#E74C3C".to_string(),
        icon: Some("phone-call".to_string()),
    })
}

fn create_read_call_result_node() -> NodeDefinition {
    NodeDefinition::new(
        "ReadCallResult",
        "Read Call Result",
        "Extract a field value from call result object",
        "BaaLS Runtime",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("result_object", "Result Object", ValueType::Any).required())
    .with_input(Port::new("field_name", "Field Name", ValueType::String).required())
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("value", "Value", ValueType::Any))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "read_call_result".to_string(),
        expression_field: None,
        gas_cost: Some(10),
        optimizable: true,
    })
    .with_visual(VisualProperties {
        width: 120.0,
        height: 80.0,
        color: "#E74C3C".to_string(),
        icon: Some("eye".to_string()),
    })
}

fn create_transfer_value_node() -> NodeDefinition {
    NodeDefinition::new(
        "TransferValue",
        "Transfer Value",
        "Transfer ledger value/balance to recipient",
        "BaaLS Runtime",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new("recipient", "Recipient", ValueType::String).required())
    .with_input(Port::new("amount", "Amount", ValueType::Integer).required())
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new("success", "Success", ValueType::Boolean))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "transfer_value".to_string(),
        expression_field: None,
        gas_cost: Some(150),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#C0392B".to_string(),
        icon: Some("send".to_string()),
    })
}

fn create_extract_tx_by_recipient_node() -> NodeDefinition {
    NodeDefinition::new(
        "ExtractTxByRecipient",
        "Extract Tx By Recipient",
        "Filter transactions by recipient address",
        "ChronoNode",
    )
    .with_input(Port::new("flow_in", "Flow In", ValueType::Flow).required())
    .with_input(Port::new(
        "block_data",
        "Block Data",
        ValueType::Object(std::collections::HashMap::new()),
    ))
    .with_input(Port::new("recipient", "Recipient", ValueType::String))
    .with_output(Port::new("flow_out", "Flow Out", ValueType::Flow))
    .with_output(Port::new(
        "transactions",
        "Transactions",
        ValueType::Array(Box::new(
            ValueType::Object(std::collections::HashMap::new()),
        )),
    ))
    .with_config_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }))
    .with_compiler_hint(CompilerHint {
        operation_type: "extract_tx_by_recipient".to_string(),
        expression_field: None,
        gas_cost: Some(200),
        optimizable: false,
    })
    .with_visual(VisualProperties {
        width: 140.0,
        height: 80.0,
        color: "#16A085".to_string(),
        icon: Some("search".to_string()),
    })
}
