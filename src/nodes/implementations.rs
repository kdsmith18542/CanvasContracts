//! Node implementations

use crate::{
    error::{CanvasError, CanvasResult},
    types::NodeResult,
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

type NodeExecutor =
    Box<dyn Fn(&mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> + Send + Sync>;

#[allow(dead_code)]
/// Basic node implementation
pub struct BasicNode {
    node_type: String,
    name: String,
    executor: NodeExecutor,
}

#[allow(dead_code)]
impl BasicNode {
    pub fn new(
        node_type: impl Into<String>,
        name: impl Into<String>,
        executor: impl Fn(&mut crate::nodes::NodeContext) -> CanvasResult<NodeResult>
            + Send
            + Sync
            + 'static,
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
        outputs.insert(
            "result".to_string(),
            serde_json::Value::Number(result.into()),
        );

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
        let value = context
            .execution_context
            .storage
            .get(&key)
            .cloned()
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
        context
            .execution_context
            .storage
            .insert(key.to_string(), value.clone());

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
        outputs.insert(
            "result".to_string(),
            serde_json::Value::Number(result.into()),
        );
        Ok(NodeResult::success(outputs, 3))
    }

    fn node_type(&self) -> &str {
        "Subtract"
    }
    fn name(&self) -> &str {
        "Subtract"
    }
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
        outputs.insert(
            "result".to_string(),
            serde_json::Value::Number(result.into()),
        );
        Ok(NodeResult::success(outputs, 5))
    }

    fn node_type(&self) -> &str {
        "Multiply"
    }
    fn name(&self) -> &str {
        "Multiply"
    }
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
        outputs.insert(
            "result".to_string(),
            serde_json::Value::Number(result.into()),
        );
        Ok(NodeResult::success(outputs, 5))
    }

    fn node_type(&self) -> &str {
        "Divide"
    }
    fn name(&self) -> &str {
        "Divide"
    }
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
        outputs.insert(
            "result".to_string(),
            serde_json::Value::Bool(a_bool && b_bool),
        );
        Ok(NodeResult::success(outputs, 5))
    }

    fn node_type(&self) -> &str {
        "And"
    }
    fn name(&self) -> &str {
        "Logical AND"
    }
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
        outputs.insert(
            "result".to_string(),
            serde_json::Value::Bool(a_bool || b_bool),
        );
        Ok(NodeResult::success(outputs, 5))
    }

    fn node_type(&self) -> &str {
        "Or"
    }
    fn name(&self) -> &str {
        "Logical OR"
    }
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

    fn node_type(&self) -> &str {
        "Not"
    }
    fn name(&self) -> &str {
        "Logical NOT"
    }
}

/// VerifySignature node — verifies ed25519 signature
pub struct VerifySignatureNode;

impl Node for VerifySignatureNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        let msg = context
            .get_input(&"message".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let sig_hex = context
            .get_input(&"signature".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let pk_hex = context
            .get_input(&"public_key".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        context.use_gas(100)?;

        let result = if msg.is_empty() || sig_hex.is_empty() || pk_hex.is_empty() {
            false
        } else {
            match verify_ed25519(&msg, &sig_hex, &pk_hex) {
                Ok(valid) => valid,
                Err(e) => {
                    log::warn!("Signature verification failed: {}", e);
                    false
                }
            }
        };

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("result".to_string(), serde_json::Value::Bool(result));

        Ok(NodeResult::success(outputs, 100))
    }

    fn node_type(&self) -> &str {
        "VerifySignature"
    }
    fn name(&self) -> &str {
        "Verify Signature"
    }
}

/// Real ed25519 signature verification using ed25519-dalek
fn verify_ed25519(message: &str, signature_hex: &str, public_key_hex: &str) -> CanvasResult<bool> {
    use ed25519_dalek::Verifier;

    let sig_bytes = hex::decode(signature_hex)
        .map_err(|_| CanvasError::Node("Invalid signature hex".to_string()))?;
    let pk_bytes = hex::decode(public_key_hex)
        .map_err(|_| CanvasError::Node("Invalid public key hex".to_string()))?;

    let sig = ed25519_dalek::Signature::from_slice(&sig_bytes)
        .map_err(|e| CanvasError::Node(format!("Invalid signature bytes: {}", e)))?;
    let pk_array: [u8; 32] = pk_bytes
        .as_slice()
        .try_into()
        .map_err(|_| CanvasError::Node("Public key must be 32 bytes".to_string()))?;
    let pubkey = ed25519_dalek::VerifyingKey::from_bytes(&pk_array)
        .map_err(|e| CanvasError::Node(format!("Invalid Ed25519 public key: {}", e)))?;

    Ok(pubkey.verify(message.as_bytes(), &sig).is_ok())
}

/// DecodeProof node — deserializes DormancyProof JSON payload
pub struct DecodeProofNode;

impl Node for DecodeProofNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        let proof_json = context
            .get_input(&"proof_json".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("{}")
            .to_string();

        context.use_gas(50)?;

        let decoded: serde_json::Value =
            serde_json::from_str(&proof_json).unwrap_or(serde_json::json!({}));

        let chain_id = decoded
            .get("chain_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let address = decoded
            .get("address")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let dormant_since_block = decoded
            .get("dormant_since_block")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "chain_id".to_string(),
            serde_json::Value::String(chain_id.to_string()),
        );
        outputs.insert(
            "address".to_string(),
            serde_json::Value::String(address.to_string()),
        );
        outputs.insert(
            "dormant_since_block".to_string(),
            serde_json::json!(dormant_since_block),
        );

        Ok(NodeResult::success(outputs, 50))
    }

    fn node_type(&self) -> &str {
        "DecodeProof"
    }
    fn name(&self) -> &str {
        "Decode Proof"
    }
}

// BaaLS Runtime Nodes

pub struct GetSenderNode;

impl Node for GetSenderNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(10)?;
        let sender = context
            .get_input(&"flow_in".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("0x0000000000000000000000000000000000000000")
            .to_string();
        let mut outputs = std::collections::HashMap::new();
        outputs.insert("sender".to_string(), serde_json::Value::String(sender));
        Ok(NodeResult::success(outputs, 10))
    }
    fn node_type(&self) -> &str {
        "GetSender"
    }
    fn name(&self) -> &str {
        "Get Sender"
    }
}

pub struct GetContractIdNode;

impl Node for GetContractIdNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(10)?;
        let contract_id = context
            .get_input(&"contract_id".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("0x0000000000000000000000000000000000000000")
            .to_string();
        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "contract_id".to_string(),
            serde_json::Value::String(contract_id),
        );
        Ok(NodeResult::success(outputs, 10))
    }
    fn node_type(&self) -> &str {
        "GetContractId"
    }
    fn name(&self) -> &str {
        "Get Contract ID"
    }
}

pub struct GetBlockHeightNode;

impl Node for GetBlockHeightNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(10)?;
        let height = context
            .get_input(&"height".to_string())
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "height".to_string(),
            serde_json::Value::Number(height.into()),
        );
        Ok(NodeResult::success(outputs, 10))
    }
    fn node_type(&self) -> &str {
        "GetBlockHeight"
    }
    fn name(&self) -> &str {
        "Get Block Height"
    }
}

pub struct GetBlockTimestampNode;

impl Node for GetBlockTimestampNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(10)?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "timestamp".to_string(),
            serde_json::Value::Number(timestamp.into()),
        );
        Ok(NodeResult::success(outputs, 10))
    }
    fn node_type(&self) -> &str {
        "GetBlockTimestamp"
    }
    fn name(&self) -> &str {
        "Get Block Timestamp"
    }
}

pub struct EmitEventNode;

impl Node for EmitEventNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(50)?;
        let event_name = context
            .get_input(&"event_name".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("UnknownEvent")
            .to_string();
        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "event_name".to_string(),
            serde_json::Value::String(event_name.clone()),
        );
        context.emit_event(event_name, std::collections::HashMap::new());
        Ok(NodeResult::success(outputs, 50))
    }
    fn node_type(&self) -> &str {
        "EmitEvent"
    }
    fn name(&self) -> &str {
        "Emit Event"
    }
}

pub struct RevertNode;

impl Node for RevertNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(10)?;
        let reason = context
            .get_input(&"reason".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("Reverted")
            .to_string();
        Err(crate::error::CanvasError::ExecutionError(reason))
    }
    fn node_type(&self) -> &str {
        "Revert"
    }
    fn name(&self) -> &str {
        "Revert"
    }
}

pub struct HashSha256Node;

impl Node for HashSha256Node {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        use sha2::{Digest, Sha256};
        context.use_gas(100)?;
        let input = context
            .get_input(&"input".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .as_bytes()
            .to_vec();
        let hash = Sha256::digest(&input);
        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "hash".to_string(),
            serde_json::Value::String(hex::encode(hash)),
        );
        Ok(NodeResult::success(outputs, 100))
    }
    fn node_type(&self) -> &str {
        "HashSha256"
    }
    fn name(&self) -> &str {
        "Hash SHA-256"
    }
}

pub struct CallContractNode;
impl Node for CallContractNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(500)?;
        let contract_address = context
            .get_input(&"contract_address".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let method = context
            .get_input(&"method".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("success".to_string(), serde_json::Value::Bool(true));
        outputs.insert(
            "output".to_string(),
            serde_json::json!({ "function": method, "address": contract_address }),
        );
        Ok(NodeResult::success(outputs, 500))
    }
    fn node_type(&self) -> &str {
        "CallContract"
    }
    fn name(&self) -> &str {
        "Call Contract"
    }
}

pub struct ReadCallResultNode;
impl Node for ReadCallResultNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(10)?;
        let result_object = context
            .get_input(&"result_object".to_string())
            .unwrap_or(&serde_json::Value::Null);
        let field_name = context
            .get_input(&"field_name".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let value = result_object
            .get(&field_name)
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("value".to_string(), value);
        Ok(NodeResult::success(outputs, 10))
    }
    fn node_type(&self) -> &str {
        "ReadCallResult"
    }
    fn name(&self) -> &str {
        "Read Call Result"
    }
}

pub struct TransferValueNode;
impl Node for TransferValueNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(150)?;
        let mut outputs = std::collections::HashMap::new();
        outputs.insert("success".to_string(), serde_json::Value::Bool(true));
        Ok(NodeResult::success(outputs, 150))
    }
    fn node_type(&self) -> &str {
        "TransferValue"
    }
    fn name(&self) -> &str {
        "Transfer Value"
    }
}

// ChronoNode Proof Node Implementations

pub struct FetchChronoBlockNode;
impl Node for FetchChronoBlockNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(500)?;
        let chain_id = context
            .get_input(&"chain_id".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("local")
            .to_string();
        let height = context
            .get_input(&"height".to_string())
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let mut outputs = std::collections::HashMap::new();
        outputs.insert("block_data".to_string(), serde_json::json!({"chain_id": chain_id, "height": height, "hash": format!("0x{:064x}", height)}));
        Ok(NodeResult::success(outputs, 500))
    }
    fn node_type(&self) -> &str {
        "FetchChronoBlock"
    }
    fn name(&self) -> &str {
        "Fetch Chrono Block"
    }
}

pub struct FetchCheckpointNode;
impl Node for FetchCheckpointNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(300)?;
        let from = context
            .get_input(&"from_height".to_string())
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let to = context
            .get_input(&"to_height".to_string())
            .and_then(|v| v.as_u64())
            .unwrap_or(100);
        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "checkpoint".to_string(),
            serde_json::json!({"from": from, "to": to, "root": format!("0x{:064x}", to)}),
        );
        Ok(NodeResult::success(outputs, 300))
    }
    fn node_type(&self) -> &str {
        "FetchCheckpoint"
    }
    fn name(&self) -> &str {
        "Fetch Checkpoint"
    }
}

pub struct VerifyChronoProofNode;
impl Node for VerifyChronoProofNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(500)?;
        let mut outputs = std::collections::HashMap::new();
        outputs.insert("valid".to_string(), serde_json::Value::Bool(true));
        Ok(NodeResult::success(outputs, 500))
    }
    fn node_type(&self) -> &str {
        "VerifyChronoProof"
    }
    fn name(&self) -> &str {
        "Verify Chrono Proof"
    }
}

pub struct ExtractChronoEventNode;
impl Node for ExtractChronoEventNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(100)?;
        let event_type = context
            .get_input(&"event_type".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("transfer");
        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "event_data".to_string(),
            serde_json::json!({"type": event_type, "data": {}}),
        );
        Ok(NodeResult::success(outputs, 100))
    }
    fn node_type(&self) -> &str {
        "ExtractChronoEvent"
    }
    fn name(&self) -> &str {
        "Extract Chrono Event"
    }
}

pub struct ExtractTxBySenderNode;
impl Node for ExtractTxBySenderNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(200)?;
        let sender = context
            .get_input(&"sender".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("0x0");
        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "transactions".to_string(),
            serde_json::json!([{"from": sender, "hash": "0x123"}]),
        );
        Ok(NodeResult::success(outputs, 200))
    }
    fn node_type(&self) -> &str {
        "ExtractTxBySender"
    }
    fn name(&self) -> &str {
        "Extract Tx By Sender"
    }
}

pub struct ExtractTxByRecipientNode;
impl Node for ExtractTxByRecipientNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(200)?;
        let recipient = context
            .get_input(&"recipient".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("0x0")
            .to_string();
        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "transactions".to_string(),
            serde_json::json!([{"to": recipient, "hash": "0x123", "value": 100}]),
        );
        Ok(NodeResult::success(outputs, 200))
    }
    fn node_type(&self) -> &str {
        "ExtractTxByRecipient"
    }
    fn name(&self) -> &str {
        "Extract Tx By Recipient"
    }
}

pub struct VerifyArchiveRangeNode;
impl Node for VerifyArchiveRangeNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(500)?;
        let mut outputs = std::collections::HashMap::new();
        outputs.insert("valid".to_string(), serde_json::Value::Bool(true));
        Ok(NodeResult::success(outputs, 500))
    }
    fn node_type(&self) -> &str {
        "VerifyArchiveRange"
    }
    fn name(&self) -> &str {
        "Verify Archive Range"
    }
}

// Resurgence DormancyOracle Node Implementations

pub struct CheckTokenAgeNode;
impl Node for CheckTokenAgeNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(50)?;
        let current = context
            .get_input(&"current_block".to_string())
            .and_then(|v| v.as_u64())
            .unwrap_or(1000);
        let last_activity = current - 100;
        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "age_blocks".to_string(),
            serde_json::Value::Number((current - last_activity).into()),
        );
        Ok(NodeResult::success(outputs, 50))
    }
    fn node_type(&self) -> &str {
        "CheckTokenAge"
    }
    fn name(&self) -> &str {
        "Check Token Age"
    }
}

pub struct CheckTokenActivityWindowNode;
impl Node for CheckTokenActivityWindowNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(100)?;
        let mut outputs = std::collections::HashMap::new();
        outputs.insert("active".to_string(), serde_json::Value::Bool(false));
        Ok(NodeResult::success(outputs, 100))
    }
    fn node_type(&self) -> &str {
        "CheckTokenActivityWindow"
    }
    fn name(&self) -> &str {
        "Check Token Activity Window"
    }
}

pub struct CheckLiquidityDormancyNode;
impl Node for CheckLiquidityDormancyNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(100)?;
        let mut outputs = std::collections::HashMap::new();
        outputs.insert("dormant".to_string(), serde_json::Value::Bool(true));
        outputs.insert(
            "liquidity_score".to_string(),
            serde_json::Value::Number(80u64.into()),
        );
        Ok(NodeResult::success(outputs, 100))
    }
    fn node_type(&self) -> &str {
        "CheckLiquidityDormancy"
    }
    fn name(&self) -> &str {
        "Check Liquidity Dormancy"
    }
}

pub struct CheckGovernanceDormancyNode;
impl Node for CheckGovernanceDormancyNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(100)?;
        let mut outputs = std::collections::HashMap::new();
        outputs.insert("dormant".to_string(), serde_json::Value::Bool(true));
        outputs.insert(
            "governance_score".to_string(),
            serde_json::Value::Number(75u64.into()),
        );
        Ok(NodeResult::success(outputs, 100))
    }
    fn node_type(&self) -> &str {
        "CheckGovernanceDormancy"
    }
    fn name(&self) -> &str {
        "Check Governance Dormancy"
    }
}

pub struct CalculateDormancyScoreNode;
impl Node for CalculateDormancyScoreNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(50)?;
        let age = context
            .get_input(&"age_score".to_string())
            .and_then(|v| v.as_u64())
            .unwrap_or(80);
        let liq = context
            .get_input(&"liquidity_score".to_string())
            .and_then(|v| v.as_u64())
            .unwrap_or(70);
        let gov = context
            .get_input(&"governance_score".to_string())
            .and_then(|v| v.as_u64())
            .unwrap_or(60);
        let score = (age * 4 + liq * 3 + gov * 3) / 10;
        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "dormancy_score".to_string(),
            serde_json::Value::Number(score.into()),
        );
        Ok(NodeResult::success(outputs, 50))
    }
    fn node_type(&self) -> &str {
        "CalculateDormancyScore"
    }
    fn name(&self) -> &str {
        "Calculate Dormancy Score"
    }
}

pub struct NormalizeDeadCoinRiskNode;
impl Node for NormalizeDeadCoinRiskNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(30)?;
        let raw = context
            .get_input(&"raw_score".to_string())
            .and_then(|v| v.as_u64())
            .unwrap_or(50);
        let normalized = raw.min(100);
        let label = if normalized > 80 {
            "Abandoned"
        } else if normalized > 50 {
            "Dormant"
        } else {
            "Active"
        };
        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "normalized_score".to_string(),
            serde_json::Value::Number(normalized.into()),
        );
        outputs.insert(
            "risk_label".to_string(),
            serde_json::Value::String(label.to_string()),
        );
        Ok(NodeResult::success(outputs, 30))
    }
    fn node_type(&self) -> &str {
        "NormalizeDeadCoinRisk"
    }
    fn name(&self) -> &str {
        "Normalize Dead Coin Risk"
    }
}

pub struct GenerateDormancyProofNode;
impl Node for GenerateDormancyProofNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        use sha2::{Digest, Sha256};
        context.use_gas(200)?;
        let token = context
            .get_input(&"token_address".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("0x0");
        let score = context
            .get_input(&"dormancy_score".to_string())
            .and_then(|v| v.as_u64())
            .unwrap_or(50);
        let hash = Sha256::digest(format!("{}:{}", token, score).as_bytes());
        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "proof_hash".to_string(),
            serde_json::Value::String(hex::encode(hash)),
        );
        outputs.insert(
            "manifest".to_string(),
            serde_json::json!({"token": token, "score": score}),
        );
        Ok(NodeResult::success(outputs, 200))
    }
    fn node_type(&self) -> &str {
        "GenerateDormancyProof"
    }
    fn name(&self) -> &str {
        "Generate Dormancy Proof"
    }
}

pub struct EmitDormancyOracleResultNode;
impl Node for EmitDormancyOracleResultNode {
    fn execute(&self, context: &mut crate::nodes::NodeContext) -> CanvasResult<NodeResult> {
        context.use_gas(30)?;
        let token = context
            .get_input(&"token_address".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("0x0");
        let _label = context
            .get_input(&"risk_label".to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let mut outputs = std::collections::HashMap::new();
        outputs.insert(
            "token".to_string(),
            serde_json::Value::String(token.to_string()),
        );
        context.emit_event(
            "DormancyOracleResult".to_string(),
            std::collections::HashMap::new(),
        );
        Ok(NodeResult::success(outputs, 30))
    }
    fn node_type(&self) -> &str {
        "EmitDormancyOracleResult"
    }
    fn name(&self) -> &str {
        "Emit Dormancy Oracle Result"
    }
}

/// Node factory for creating nodes by type identifier
pub struct NodeFactory;

impl NodeFactory {
    /// Create a node by type. Supports all 12 built-in node types.
    pub fn create_node(
        node_type: &str,
        properties: &std::collections::HashMap<String, serde_json::Value>,
    ) -> CanvasResult<Box<dyn Node>> {
        match node_type {
            "If" => {
                let condition = properties
                    .get("condition")
                    .or_else(|| properties.get("condition_expression"))
                    .and_then(|v| {
                        v.as_str()
                            .map(|s| s.to_string())
                            .or_else(|| v.as_bool().map(|b| b.to_string()))
                    })
                    .unwrap_or_else(|| "true".to_string());
                Ok(Box::new(IfNode::new(condition)))
            }
            "Add" => Ok(Box::new(AddNode)),
            "Subtract" => Ok(Box::new(SubtractNode)),
            "Multiply" => Ok(Box::new(MultiplyNode)),
            "Divide" => Ok(Box::new(DivideNode)),
            "VerifySignature" => Ok(Box::new(VerifySignatureNode)),
            "DecodeProof" => Ok(Box::new(DecodeProofNode)),
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
            // BaaLS Runtime nodes
            "GetSender" => Ok(Box::new(GetSenderNode)),
            "GetContractId" => Ok(Box::new(GetContractIdNode)),
            "GetBlockTimestamp" => Ok(Box::new(GetBlockTimestampNode)),
            "GetBlockHeight" => Ok(Box::new(GetBlockHeightNode)),
            "EmitEvent" => {
                let _event_name = properties
                    .get("event_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("UnknownEvent")
                    .to_string();
                Ok(Box::new(EmitEventNode))
            }
            "Revert" => {
                let _reason = properties
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Reverted")
                    .to_string();
                Ok(Box::new(RevertNode))
            }
            "HashSha256" => Ok(Box::new(HashSha256Node)),
            "CallContract" => Ok(Box::new(CallContractNode)),
            "ReadCallResult" => Ok(Box::new(ReadCallResultNode)),
            "TransferValue" => Ok(Box::new(TransferValueNode)),
            // ChronoNode proof nodes
            "FetchChronoBlock" => Ok(Box::new(FetchChronoBlockNode)),
            "FetchCheckpoint" => Ok(Box::new(FetchCheckpointNode)),
            "VerifyChronoProof" => Ok(Box::new(VerifyChronoProofNode)),
            "ExtractChronoEvent" => Ok(Box::new(ExtractChronoEventNode)),
            "ExtractTxBySender" => Ok(Box::new(ExtractTxBySenderNode)),
            "ExtractTxByRecipient" => Ok(Box::new(ExtractTxByRecipientNode)),
            "VerifyArchiveRange" => Ok(Box::new(VerifyArchiveRangeNode)),
            // Resurgence DormancyOracle nodes
            "CheckTokenAge" => Ok(Box::new(CheckTokenAgeNode)),
            "CheckTokenActivityWindow" => Ok(Box::new(CheckTokenActivityWindowNode)),
            "CheckLiquidityDormancy" => Ok(Box::new(CheckLiquidityDormancyNode)),
            "CheckGovernanceDormancy" => Ok(Box::new(CheckGovernanceDormancyNode)),
            "CalculateDormancyScore" => Ok(Box::new(CalculateDormancyScoreNode)),
            "NormalizeDeadCoinRisk" => Ok(Box::new(NormalizeDeadCoinRiskNode)),
            "GenerateDormancyProof" => Ok(Box::new(GenerateDormancyProofNode)),
            "EmitDormancyOracleResult" => Ok(Box::new(EmitDormancyOracleResultNode)),
            _ => Err(CanvasError::Node(format!(
                "Unknown node type: {}",
                node_type
            ))),
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
        c.inputs
            .insert("condition".to_string(), serde_json::json!(true));
        let r = IfNode::new("true").execute(&mut c).unwrap();
        assert!(r.outputs.contains_key("true_flow"));
        assert!(!r.outputs.contains_key("false_flow"));
    }

    #[test]
    fn test_if_node_false() {
        let mut c = ctx(1000);
        c.inputs
            .insert("condition".to_string(), serde_json::json!(false));
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
        c.inputs
            .insert("condition".to_string(), serde_json::json!(42));
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
        assert!(r.outputs["result"].as_bool().unwrap());
    }

    #[test]
    fn test_and_node_true_false() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(true));
        c.inputs.insert("b".to_string(), serde_json::json!(false));
        let r = AndNode.execute(&mut c).unwrap();
        assert!(!r.outputs["result"].as_bool().unwrap());
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
        assert!(!r.outputs["result"].as_bool().unwrap());
    }

    #[test]
    fn test_or_node_true_false() {
        let mut c = ctx(1000);
        c.inputs.insert("a".to_string(), serde_json::json!(true));
        c.inputs.insert("b".to_string(), serde_json::json!(false));
        let r = OrNode.execute(&mut c).unwrap();
        assert!(r.outputs["result"].as_bool().unwrap());
    }

    // ── Not ───────────────────────────────────────────────────────────

    #[test]
    fn test_not_node_true() {
        let mut c = ctx(1000);
        c.inputs
            .insert("input".to_string(), serde_json::json!(true));
        let r = NotNode.execute(&mut c).unwrap();
        assert!(!r.outputs["result"].as_bool().unwrap());
    }

    #[test]
    fn test_not_node_false() {
        let mut c = ctx(1000);
        c.inputs
            .insert("input".to_string(), serde_json::json!(false));
        let r = NotNode.execute(&mut c).unwrap();
        assert!(r.outputs["result"].as_bool().unwrap());
    }

    #[test]
    fn test_not_node_wrong_type() {
        let mut c = ctx(1000);
        c.inputs
            .insert("input".to_string(), serde_json::json!("not_a_bool"));
        assert!(NotNode.execute(&mut c).is_err());
    }

    // ── Storage ───────────────────────────────────────────────────────

    #[test]
    fn test_write_then_read_storage() {
        let mut c = ctx(10000);
        c.inputs
            .insert("key".to_string(), serde_json::json!("mykey"));
        c.inputs.insert("value".to_string(), serde_json::json!(42));
        WriteStorageNode::new("mykey").execute(&mut c).unwrap();

        // Now read it back
        let mut c2 = crate::nodes::NodeContext::new(c.execution_context);
        c2.inputs
            .insert("key".to_string(), serde_json::json!("mykey"));
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
    fn test_factory_creates_all_39_types() {
        let props = std::collections::HashMap::new();
        let types = [
            // Original 14
            "If",
            "Add",
            "Subtract",
            "Multiply",
            "Divide",
            "And",
            "Or",
            "Not",
            "ReadStorage",
            "WriteStorage",
            "Start",
            "End",
            "VerifySignature",
            "DecodeProof",
            // BaaLS Runtime (10)
            "GetSender",
            "GetContractId",
            "GetBlockTimestamp",
            "GetBlockHeight",
            "EmitEvent",
            "Revert",
            "HashSha256",
            "CallContract",
            "ReadCallResult",
            "TransferValue",
            // ChronoNode (7)
            "FetchChronoBlock",
            "FetchCheckpoint",
            "VerifyChronoProof",
            "ExtractChronoEvent",
            "ExtractTxBySender",
            "ExtractTxByRecipient",
            "VerifyArchiveRange",
            // Resurgence (8)
            "CheckTokenAge",
            "CheckTokenActivityWindow",
            "CheckLiquidityDormancy",
            "CheckGovernanceDormancy",
            "CalculateDormancyScore",
            "NormalizeDeadCoinRisk",
            "GenerateDormancyProof",
            "EmitDormancyOracleResult",
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
