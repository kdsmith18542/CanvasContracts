use crate::compiler::graph_ir::GraphIR;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub enum ASTNode {
    I64Const(i64),
    I64BinOp {
        op: I64BinOpKind,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    I64UnaryOp {
        op: I64UnaryOpKind,
        operand: Box<ASTNode>,
    },
    I64Condition {
        op: ConditionOpKind,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    I64IfElse {
        condition: Box<ASTNode>,
        when_true: Box<ASTNode>,
        when_false: Box<ASTNode>,
    },
    IfElse {
        condition: Box<ASTNode>,
        true_body: Vec<ASTNode>,
        false_body: Vec<ASTNode>,
    },
    Call {
        import_module: String,
        import_name: String,
        args: Vec<ASTNode>,
    },
    Nop,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum I64BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum I64UnaryOpKind {
    Not,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ConditionOpKind {
    Eq,
    Ne,
    Lt,
    Gt,
    And,
    Or,
}

#[derive(Debug, Clone, Serialize)]
pub struct AST {
    pub body: Vec<ASTNode>,
    pub imports: Vec<(String, String)>,
}

impl AST {
    pub fn new() -> Self {
        Self {
            body: Vec::new(),
            imports: Vec::new(),
        }
    }

    pub fn from_graph_ir(ir: &GraphIR) -> Result<Self, String> {
        let mut ast = AST::new();
        let order = petgraph::algo::toposort(&ir.graph, None)
            .map_err(|_| "Graph contains cycles".to_string())?;

        // Cache: node_id -> (port_id -> AST expression for that output)
        let mut output_exprs: HashMap<crate::types::NodeId, HashMap<String, ASTNode>> =
            HashMap::new();

        for node_idx in order {
            let node_id = ir.graph[node_idx];
            let ir_node = ir
                .nodes
                .get(&node_id)
                .ok_or_else(|| format!("Node not found in IR: {}", node_id))?;

            if !is_compilable_node_type(&ir_node.node_type) {
                return Err(format!(
                    "Node {} of type '{}' is not currently compilable. Use graph simulation instead.",
                    node_id, ir_node.node_type
                ));
            }

            let node_flow_guard = resolve_flow_guard_expr(ir, node_id, &output_exprs);

            let ast_node = match ir_node.node_type.as_str() {
                "Add" | "Subtract" | "Multiply" | "Divide" => {
                    let a =
                        resolve_input_expr("a", ir, node_id, &output_exprs, &ir_node.properties);
                    let b =
                        resolve_input_expr("b", ir, node_id, &output_exprs, &ir_node.properties);
                    let op = match ir_node.node_type.as_str() {
                        "Add" => I64BinOpKind::Add,
                        "Subtract" => I64BinOpKind::Sub,
                        "Multiply" => I64BinOpKind::Mul,
                        "Divide" => I64BinOpKind::Div,
                        _ => unreachable!(),
                    };
                    ASTNode::I64BinOp {
                        op,
                        left: Box::new(a),
                        right: Box::new(b),
                    }
                }
                "And" => {
                    let a =
                        resolve_input_expr("a", ir, node_id, &output_exprs, &ir_node.properties);
                    let b =
                        resolve_input_expr("b", ir, node_id, &output_exprs, &ir_node.properties);
                    ASTNode::I64Condition {
                        op: ConditionOpKind::And,
                        left: Box::new(a),
                        right: Box::new(b),
                    }
                }
                "Or" => {
                    let a =
                        resolve_input_expr("a", ir, node_id, &output_exprs, &ir_node.properties);
                    let b =
                        resolve_input_expr("b", ir, node_id, &output_exprs, &ir_node.properties);
                    ASTNode::I64Condition {
                        op: ConditionOpKind::Or,
                        left: Box::new(a),
                        right: Box::new(b),
                    }
                }
                "Not" => {
                    let operand = resolve_input_expr(
                        "input",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    ASTNode::I64UnaryOp {
                        op: I64UnaryOpKind::Not,
                        operand: Box::new(operand),
                    }
                }
                "ReadStorage" => {
                    ast.register_import("baals", "baals_read_storage");
                    let key_expr = resolve_storage_key_expr(
                        "key",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_read_storage".to_string(),
                        args: vec![key_expr],
                    }
                }
                "WriteStorage" => {
                    ast.register_import("baals", "baals_write_storage");
                    let key_expr = resolve_storage_key_expr(
                        "key",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    let value_expr = resolve_input_expr(
                        "value",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_write_storage".to_string(),
                        args: vec![key_expr, value_expr],
                    }
                }
                "GetSender" => {
                    ast.register_import("baals", "baals_get_sender");
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_get_sender".to_string(),
                        args: vec![],
                    }
                }
                "GetContractId" => {
                    ast.register_import("baals", "baals_get_contract_id");
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_get_contract_id".to_string(),
                        args: vec![],
                    }
                }
                "GetBlockTimestamp" => {
                    ast.register_import("baals", "baals_get_block_timestamp");
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_get_block_timestamp".to_string(),
                        args: vec![],
                    }
                }
                "GetBlockHeight" => {
                    ast.register_import("baals", "baals_get_block_height");
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_get_block_height".to_string(),
                        args: vec![],
                    }
                }
                "EmitEvent" => {
                    ast.register_import("baals", "baals_emit_event");
                    let event_name_expr = resolve_storage_key_expr(
                        "event_name",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    let event_data_expr = resolve_input_expr(
                        "event_data",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_emit_event".to_string(),
                        args: vec![event_name_expr, event_data_expr],
                    }
                }
                "Revert" => {
                    ast.register_import("baals", "baals_revert");
                    let reason_expr = resolve_storage_key_expr(
                        "reason",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_revert".to_string(),
                        args: vec![reason_expr],
                    }
                }
                "HashSha256" => {
                    ast.register_import("baals", "baals_hash_sha256");
                    let input_expr = resolve_input_expr(
                        "input",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_hash_sha256".to_string(),
                        args: vec![input_expr],
                    }
                }
                "CallContract" => {
                    ast.register_import("baals", "baals_call_contract");
                    let contract_expr = resolve_storage_key_expr(
                        "contract_address",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    let method_expr = resolve_storage_key_expr(
                        "method",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    let arguments_expr = resolve_input_expr(
                        "arguments",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_call_contract".to_string(),
                        args: vec![contract_expr, method_expr, arguments_expr],
                    }
                }
                "ReadCallResult" => {
                    ast.register_import("baals", "baals_read_call_result");
                    let result_object_expr = resolve_input_expr(
                        "result_object",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    let field_name_expr = resolve_storage_key_expr(
                        "field_name",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_read_call_result".to_string(),
                        args: vec![result_object_expr, field_name_expr],
                    }
                }
                "TransferValue" => {
                    ast.register_import("baals", "baals_transfer_value");
                    let recipient_expr = resolve_storage_key_expr(
                        "recipient",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    let amount_expr = resolve_input_expr(
                        "amount",
                        ir,
                        node_id,
                        &output_exprs,
                        &ir_node.properties,
                    );
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_transfer_value".to_string(),
                        args: vec![recipient_expr, amount_expr],
                    }
                }
                "If" => ASTNode::Nop,
                "Start" | "End" => ASTNode::Nop,
                _ => unreachable!("unsupported node types are rejected above"),
            };

            let finalized_node = maybe_guard_node_for_flow(
                &ir_node.node_type,
                ast_node.clone(),
                node_flow_guard.clone(),
            );

            let mut outputs = HashMap::new();
            if ir_node.node_type == "If" || !matches!(&finalized_node, ASTNode::Nop) {
                match ir_node.node_type.as_str() {
                    "If" => {
                        let condition_expr = resolve_if_condition_expr(
                            ir,
                            node_id,
                            &output_exprs,
                            &ir_node.properties,
                        );
                        let true_flow = ASTNode::I64Condition {
                            op: ConditionOpKind::And,
                            left: Box::new(node_flow_guard.clone()),
                            right: Box::new(condition_expr.clone()),
                        };
                        let false_flow = ASTNode::I64Condition {
                            op: ConditionOpKind::And,
                            left: Box::new(node_flow_guard.clone()),
                            right: Box::new(ASTNode::I64UnaryOp {
                                op: I64UnaryOpKind::Not,
                                operand: Box::new(condition_expr),
                            }),
                        };
                        outputs.insert("true_flow".to_string(), true_flow);
                        outputs.insert("false_flow".to_string(), false_flow);
                    }
                    "ReadStorage" => {
                        outputs.insert("value".to_string(), finalized_node.clone());
                        outputs.insert("result".to_string(), finalized_node.clone());
                    }
                    "GetSender" => {
                        outputs.insert("sender".to_string(), finalized_node.clone());
                        outputs.insert("result".to_string(), finalized_node.clone());
                    }
                    "GetContractId" => {
                        outputs.insert("contract_id".to_string(), finalized_node.clone());
                        outputs.insert("result".to_string(), finalized_node.clone());
                    }
                    "GetBlockTimestamp" => {
                        outputs.insert("timestamp".to_string(), finalized_node.clone());
                        outputs.insert("result".to_string(), finalized_node.clone());
                    }
                    "GetBlockHeight" => {
                        outputs.insert("height".to_string(), finalized_node.clone());
                        outputs.insert("result".to_string(), finalized_node.clone());
                    }
                    "HashSha256" => {
                        outputs.insert("hash".to_string(), finalized_node.clone());
                        outputs.insert("result".to_string(), finalized_node.clone());
                    }
                    "ReadCallResult" => {
                        outputs.insert("value".to_string(), finalized_node.clone());
                        outputs.insert("result".to_string(), finalized_node.clone());
                    }
                    "CallContract" => {
                        outputs.insert("success".to_string(), node_flow_guard.clone());
                        outputs.insert("output".to_string(), ASTNode::I64Const(0));
                        outputs.insert("result".to_string(), ASTNode::I64Const(0));
                    }
                    "TransferValue" => {
                        outputs.insert("success".to_string(), node_flow_guard.clone());
                        outputs.insert("result".to_string(), node_flow_guard.clone());
                    }
                    "EmitEvent" | "Revert" => {}
                    _ => {
                        outputs.insert("result".to_string(), finalized_node.clone());
                    }
                }
                if ir_node.node_type != "If"
                    && ir_node.outputs.iter().any(|output| output == "flow_out")
                {
                    outputs.insert("flow_out".to_string(), node_flow_guard.clone());
                }
            }
            output_exprs.insert(node_id, outputs);
            ast.body.push(finalized_node);
        }

        Ok(ast)
    }

    fn register_import(&mut self, module: &str, name: &str) {
        let key = (module.to_string(), name.to_string());
        if !self.imports.contains(&key) {
            self.imports.push(key);
        }
    }
}

impl Default for AST {
    fn default() -> Self {
        Self::new()
    }
}

fn is_compilable_node_type(node_type: &str) -> bool {
    matches!(
        node_type,
        "Start"
            | "End"
            | "Add"
            | "Subtract"
            | "Multiply"
            | "Divide"
            | "And"
            | "Or"
            | "Not"
            | "If"
            | "ReadStorage"
            | "WriteStorage"
            | "GetSender"
            | "GetContractId"
            | "GetBlockTimestamp"
            | "GetBlockHeight"
            | "EmitEvent"
            | "Revert"
            | "HashSha256"
            | "CallContract"
            | "ReadCallResult"
            | "TransferValue"
    )
}

fn maybe_guard_node_for_flow(node_type: &str, node: ASTNode, flow_guard: ASTNode) -> ASTNode {
    if is_always_true_expr(&flow_guard) || matches!(node, ASTNode::Nop) {
        return node;
    }

    match node_type {
        // Storage calls are side-effectful and must only run when flow is active.
        "WriteStorage" | "EmitEvent" | "Revert" | "CallContract" | "TransferValue" => {
            ASTNode::IfElse {
                condition: Box::new(flow_guard),
                true_body: vec![node],
                false_body: Vec::new(),
            }
        }
        "ReadStorage" => ASTNode::I64IfElse {
            condition: Box::new(flow_guard),
            when_true: Box::new(node),
            when_false: Box::new(ASTNode::I64Const(0)),
        },
        // Pure expressions default to 0 when the flow gate is false.
        _ => ASTNode::I64IfElse {
            condition: Box::new(flow_guard),
            when_true: Box::new(node),
            when_false: Box::new(ASTNode::I64Const(0)),
        },
    }
}

fn resolve_flow_guard_expr(
    ir: &GraphIR,
    node_id: crate::types::NodeId,
    output_exprs: &HashMap<crate::types::NodeId, HashMap<String, ASTNode>>,
) -> ASTNode {
    let mut guards = Vec::new();
    for conn in &ir.connections {
        if conn.target == node_id
            && conn.target_port == "flow_in"
            && output_exprs.get(&conn.source).is_some()
        {
            if let Some(port_map) = output_exprs.get(&conn.source) {
                if let Some(expr) = port_map.get(&conn.source_port) {
                    guards.push(expr.clone());
                }
            }
        }
    }

    fold_or_exprs(guards).unwrap_or(ASTNode::I64Const(1))
}

fn fold_or_exprs(exprs: Vec<ASTNode>) -> Option<ASTNode> {
    let mut iter = exprs.into_iter();
    let first = iter.next()?;
    Some(iter.fold(first, |acc, expr| ASTNode::I64Condition {
        op: ConditionOpKind::Or,
        left: Box::new(acc),
        right: Box::new(expr),
    }))
}

fn is_always_true_expr(expr: &ASTNode) -> bool {
    matches!(expr, ASTNode::I64Const(v) if *v != 0)
}

fn resolve_input_expr(
    port_id: &str,
    ir: &GraphIR,
    node_id: crate::types::NodeId,
    output_exprs: &HashMap<crate::types::NodeId, HashMap<String, ASTNode>>,
    properties: &HashMap<String, serde_json::Value>,
) -> ASTNode {
    for conn in &ir.connections {
        if conn.target == node_id && conn.target_port == port_id {
            if let Some(port_map) = output_exprs.get(&conn.source) {
                if let Some(expr) = port_map.get(&conn.source_port) {
                    return expr.clone();
                }
            }
        }
    }
    property_to_i64_expr(properties.get(port_id))
}

fn resolve_storage_key_expr(
    port_id: &str,
    ir: &GraphIR,
    node_id: crate::types::NodeId,
    output_exprs: &HashMap<crate::types::NodeId, HashMap<String, ASTNode>>,
    properties: &HashMap<String, serde_json::Value>,
) -> ASTNode {
    if let Some(value) = properties.get(port_id) {
        if let Some(key) = value.as_str() {
            return ASTNode::I64Const(hash_string_to_i64(key));
        }
    }
    resolve_input_expr(port_id, ir, node_id, output_exprs, properties)
}

fn resolve_if_condition_expr(
    ir: &GraphIR,
    node_id: crate::types::NodeId,
    output_exprs: &HashMap<crate::types::NodeId, HashMap<String, ASTNode>>,
    properties: &HashMap<String, serde_json::Value>,
) -> ASTNode {
    for conn in &ir.connections {
        if conn.target == node_id
            && conn.target_port == "condition"
            && output_exprs.get(&conn.source).is_some()
        {
            if let Some(port_map) = output_exprs.get(&conn.source) {
                if let Some(expr) = port_map.get(&conn.source_port) {
                    return expr.clone();
                }
            }
        }
    }

    if let Some(v) = properties.get("condition") {
        if let Some(b) = v.as_bool() {
            return ASTNode::I64Const(if b { 1 } else { 0 });
        }
        if let Some(s) = v.as_str() {
            let normalized = s.trim().to_ascii_lowercase();
            if normalized == "true" {
                return ASTNode::I64Const(1);
            }
            if normalized == "false" {
                return ASTNode::I64Const(0);
            }
        }
    }

    if let Some(v) = properties.get("condition_expression") {
        if let Some(s) = v.as_str() {
            let normalized = s.trim().to_ascii_lowercase();
            if normalized == "true" {
                return ASTNode::I64Const(1);
            }
            if normalized == "false" {
                return ASTNode::I64Const(0);
            }
        }
    }

    ASTNode::I64Const(0)
}

fn property_to_i64_expr(value: Option<&serde_json::Value>) -> ASTNode {
    let Some(value) = value else {
        return ASTNode::I64Const(0);
    };

    if let Some(v) = value.as_i64() {
        return ASTNode::I64Const(v);
    }
    if let Some(v) = value.as_u64() {
        return ASTNode::I64Const(i64::try_from(v).unwrap_or(i64::MAX));
    }
    if let Some(v) = value.as_bool() {
        return ASTNode::I64Const(if v { 1 } else { 0 });
    }
    if let Some(v) = value.as_str() {
        return ASTNode::I64Const(hash_string_to_i64(v));
    }
    ASTNode::I64Const(0)
}

fn hash_string_to_i64(input: &str) -> i64 {
    let digest = Sha256::digest(input.as_bytes());
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    i64::from_be_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Connection, Port, Position, ValueType, VisualGraph, VisualNode};
    use uuid::Uuid;

    #[test]
    fn test_from_graph_ir_rejects_unsupported_node_type() {
        let mut graph = VisualGraph::new("unsupported-node");
        graph.add_node(VisualNode::new(
            Uuid::new_v4(),
            "Start",
            Position::new(0.0, 0.0),
        ));
        graph.add_node(VisualNode::new(
            Uuid::new_v4(),
            "VerifySignature",
            Position::new(150.0, 0.0),
        ));

        let ir = GraphIR::from_visual_graph(&graph);
        let err = AST::from_graph_ir(&ir).unwrap_err();

        assert!(err.contains("VerifySignature"));
        assert!(err.contains("not currently compilable"));
    }

    #[test]
    fn test_baals_runtime_nodes_compile_to_imported_calls() {
        let start_id = Uuid::new_v4();
        let sender_id = Uuid::new_v4();
        let contract_id_id = Uuid::new_v4();
        let timestamp_id = Uuid::new_v4();
        let height_id = Uuid::new_v4();
        let emit_id = Uuid::new_v4();
        let hash_id = Uuid::new_v4();
        let call_id = Uuid::new_v4();
        let read_call_result_id = Uuid::new_v4();
        let transfer_id = Uuid::new_v4();
        let revert_id = Uuid::new_v4();

        let start = VisualNode::new(start_id, "Start", Position::new(0.0, 0.0))
            .with_outputs(vec![Port::new("flow_out", "Flow Out", ValueType::Flow)]);

        let get_sender = VisualNode::new(sender_id, "GetSender", Position::new(150.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required()
            ])
            .with_outputs(vec![
                Port::new("flow_out", "Flow Out", ValueType::Flow),
                Port::new("sender", "Sender", ValueType::String),
            ]);

        let get_contract_id =
            VisualNode::new(contract_id_id, "GetContractId", Position::new(300.0, 0.0))
                .with_inputs(vec![
                    Port::new("flow_in", "Flow In", ValueType::Flow).required()
                ])
                .with_outputs(vec![
                    Port::new("flow_out", "Flow Out", ValueType::Flow),
                    Port::new("contract_id", "Contract ID", ValueType::String),
                ]);

        let get_timestamp =
            VisualNode::new(timestamp_id, "GetBlockTimestamp", Position::new(450.0, 0.0))
                .with_inputs(vec![
                    Port::new("flow_in", "Flow In", ValueType::Flow).required()
                ])
                .with_outputs(vec![
                    Port::new("flow_out", "Flow Out", ValueType::Flow),
                    Port::new("timestamp", "Timestamp", ValueType::Integer),
                ]);

        let get_height = VisualNode::new(height_id, "GetBlockHeight", Position::new(600.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required()
            ])
            .with_outputs(vec![
                Port::new("flow_out", "Flow Out", ValueType::Flow),
                Port::new("height", "Height", ValueType::Integer),
            ]);

        let emit_event = VisualNode::new(emit_id, "EmitEvent", Position::new(750.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required()
            ])
            .with_outputs(vec![Port::new("flow_out", "Flow Out", ValueType::Flow)])
            .with_property("event_name", serde_json::json!("UserUpdated"));

        let hash_sha256 = VisualNode::new(hash_id, "HashSha256", Position::new(900.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required()
            ])
            .with_outputs(vec![
                Port::new("flow_out", "Flow Out", ValueType::Flow),
                Port::new("hash", "Hash", ValueType::Bytes),
            ])
            .with_property("input", serde_json::json!("payload"));

        let call_contract = VisualNode::new(call_id, "CallContract", Position::new(1050.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required()
            ])
            .with_outputs(vec![
                Port::new("flow_out", "Flow Out", ValueType::Flow),
                Port::new("success", "Success", ValueType::Boolean),
                Port::new("output", "Output", ValueType::Any),
            ])
            .with_property("contract_address", serde_json::json!("0xabc"))
            .with_property("method", serde_json::json!("set_value"))
            .with_property("arguments", serde_json::json!([1, 2, 3]));

        let read_call_result = VisualNode::new(
            read_call_result_id,
            "ReadCallResult",
            Position::new(1200.0, 0.0),
        )
        .with_inputs(vec![
            Port::new("flow_in", "Flow In", ValueType::Flow).required()
        ])
        .with_outputs(vec![
            Port::new("flow_out", "Flow Out", ValueType::Flow),
            Port::new("value", "Value", ValueType::Any),
        ])
        .with_property("field_name", serde_json::json!("value"));

        let transfer_value =
            VisualNode::new(transfer_id, "TransferValue", Position::new(1350.0, 0.0))
                .with_inputs(vec![
                    Port::new("flow_in", "Flow In", ValueType::Flow).required()
                ])
                .with_outputs(vec![
                    Port::new("flow_out", "Flow Out", ValueType::Flow),
                    Port::new("success", "Success", ValueType::Boolean),
                ])
                .with_property("recipient", serde_json::json!("0xdef"))
                .with_property("amount", serde_json::json!(100));

        let revert = VisualNode::new(revert_id, "Revert", Position::new(1500.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required()
            ])
            .with_property("reason", serde_json::json!("stop"));

        let mut graph = VisualGraph::new("baals-runtime-compilation");
        graph.add_node(start);
        graph.add_node(get_sender);
        graph.add_node(get_contract_id);
        graph.add_node(get_timestamp);
        graph.add_node(get_height);
        graph.add_node(emit_event);
        graph.add_node(hash_sha256);
        graph.add_node(call_contract);
        graph.add_node(read_call_result);
        graph.add_node(transfer_value);
        graph.add_node(revert);

        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            start_id,
            "flow_out",
            sender_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            sender_id,
            "flow_out",
            contract_id_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            contract_id_id,
            "flow_out",
            timestamp_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            timestamp_id,
            "flow_out",
            height_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            height_id,
            "flow_out",
            emit_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            emit_id,
            "flow_out",
            hash_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            hash_id,
            "flow_out",
            call_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            call_id,
            "flow_out",
            read_call_result_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            call_id,
            "output",
            read_call_result_id,
            "result_object",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            read_call_result_id,
            "flow_out",
            transfer_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            transfer_id,
            "flow_out",
            revert_id,
            "flow_in",
        ));

        let ir = GraphIR::from_visual_graph(&graph);
        let ast = AST::from_graph_ir(&ir).expect("BaaLS runtime nodes should be compilable");

        let expected_imports = [
            ("baals".to_string(), "baals_get_sender".to_string()),
            ("baals".to_string(), "baals_get_contract_id".to_string()),
            ("baals".to_string(), "baals_get_block_timestamp".to_string()),
            ("baals".to_string(), "baals_get_block_height".to_string()),
            ("baals".to_string(), "baals_emit_event".to_string()),
            ("baals".to_string(), "baals_hash_sha256".to_string()),
            ("baals".to_string(), "baals_call_contract".to_string()),
            ("baals".to_string(), "baals_read_call_result".to_string()),
            ("baals".to_string(), "baals_transfer_value".to_string()),
            ("baals".to_string(), "baals_revert".to_string()),
        ];

        for import in expected_imports {
            assert!(
                ast.imports.contains(&import),
                "Expected import {:?} to be registered",
                import
            );
        }
    }

    #[test]
    fn test_read_storage_value_connection_resolves_to_call_expression() {
        let start_id = Uuid::new_v4();
        let read_id = Uuid::new_v4();
        let add_id = Uuid::new_v4();
        let end_id = Uuid::new_v4();

        let start = VisualNode::new(start_id, "Start", Position::new(0.0, 0.0))
            .with_outputs(vec![Port::new("flow_out", "Flow Out", ValueType::Flow)]);

        let read = VisualNode::new(read_id, "ReadStorage", Position::new(200.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required(),
                Port::new("key", "Key", ValueType::String).required(),
            ])
            .with_outputs(vec![
                Port::new("flow_out", "Flow Out", ValueType::Flow),
                Port::new("value", "Value", ValueType::Any),
            ])
            .with_property("key", serde_json::json!("stored_value"));

        let add = VisualNode::new(add_id, "Add", Position::new(400.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required(),
                Port::new("a", "A", ValueType::Integer).required(),
                Port::new("b", "B", ValueType::Integer).required(),
            ])
            .with_outputs(vec![
                Port::new("flow_out", "Flow Out", ValueType::Flow),
                Port::new("result", "Result", ValueType::Integer),
            ])
            .with_property("b", serde_json::json!(5));

        let end =
            VisualNode::new(end_id, "End", Position::new(600.0, 0.0)).with_inputs(vec![Port::new(
                "flow_in",
                "Flow In",
                ValueType::Flow,
            )
            .required()]);

        let mut graph = VisualGraph::new("read-into-add");
        graph.add_node(start);
        graph.add_node(read);
        graph.add_node(add);
        graph.add_node(end);

        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            start_id,
            "flow_out",
            read_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            read_id,
            "flow_out",
            add_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            read_id,
            "value",
            add_id,
            "a",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            add_id,
            "flow_out",
            end_id,
            "flow_in",
        ));

        let ir = GraphIR::from_visual_graph(&graph);
        let ast = AST::from_graph_ir(&ir).expect("AST lowering should succeed");

        let mut found_connected_add = false;
        for node in ast.body {
            if let ASTNode::I64BinOp { op, left, right } = node {
                if op == I64BinOpKind::Add {
                    let left_ok = matches!(
                        left.as_ref(),
                        ASTNode::Call {
                            import_module,
                            import_name,
                            ..
                        } if import_module == "baals" && import_name == "baals_read_storage"
                    );
                    let right_ok = matches!(right.as_ref(), ASTNode::I64Const(5));
                    if left_ok && right_ok {
                        found_connected_add = true;
                        break;
                    }
                }
            }
        }

        assert!(
            found_connected_add,
            "Expected Add node to consume ReadStorage.value expression"
        );
    }

    #[test]
    fn test_if_branch_produces_guarded_write_storage_calls() {
        let start_id = Uuid::new_v4();
        let if_id = Uuid::new_v4();
        let write_true_id = Uuid::new_v4();
        let write_false_id = Uuid::new_v4();
        let end_id = Uuid::new_v4();

        let start = VisualNode::new(start_id, "Start", Position::new(0.0, 0.0))
            .with_outputs(vec![Port::new("flow_out", "Flow Out", ValueType::Flow)]);

        let if_node = VisualNode::new(if_id, "If", Position::new(100.0, 0.0))
            .with_inputs(vec![
                Port::new("flow_in", "Flow In", ValueType::Flow).required(),
                Port::new("condition", "Condition", ValueType::Boolean).required(),
            ])
            .with_outputs(vec![
                Port::new("true_flow", "True Flow", ValueType::Flow),
                Port::new("false_flow", "False Flow", ValueType::Flow),
            ])
            .with_property("condition", serde_json::json!(true));

        let write_true =
            VisualNode::new(write_true_id, "WriteStorage", Position::new(250.0, -60.0))
                .with_inputs(vec![
                    Port::new("flow_in", "Flow In", ValueType::Flow).required(),
                    Port::new("key", "Key", ValueType::String).required(),
                    Port::new("value", "Value", ValueType::Any).required(),
                ])
                .with_outputs(vec![Port::new("flow_out", "Flow Out", ValueType::Flow)])
                .with_property("key", serde_json::json!("branch_result"))
                .with_property("value", serde_json::json!("true_path"));

        let write_false =
            VisualNode::new(write_false_id, "WriteStorage", Position::new(250.0, 60.0))
                .with_inputs(vec![
                    Port::new("flow_in", "Flow In", ValueType::Flow).required(),
                    Port::new("key", "Key", ValueType::String).required(),
                    Port::new("value", "Value", ValueType::Any).required(),
                ])
                .with_outputs(vec![Port::new("flow_out", "Flow Out", ValueType::Flow)])
                .with_property("key", serde_json::json!("branch_result"))
                .with_property("value", serde_json::json!("false_path"));

        let end =
            VisualNode::new(end_id, "End", Position::new(400.0, 0.0)).with_inputs(vec![Port::new(
                "flow_in",
                "Flow In",
                ValueType::Flow,
            )
            .required()]);

        let mut graph = VisualGraph::new("if-branching");
        graph.add_node(start);
        graph.add_node(if_node);
        graph.add_node(write_true);
        graph.add_node(write_false);
        graph.add_node(end);

        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            start_id,
            "flow_out",
            if_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            if_id,
            "true_flow",
            write_true_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            if_id,
            "false_flow",
            write_false_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            write_true_id,
            "flow_out",
            end_id,
            "flow_in",
        ));
        graph.add_connection(Connection::new(
            Uuid::new_v4(),
            write_false_id,
            "flow_out",
            end_id,
            "flow_in",
        ));

        let ir = GraphIR::from_visual_graph(&graph);
        let ast = AST::from_graph_ir(&ir).expect("AST lowering for If branching should succeed");

        let guarded_writes = ast
            .body
            .iter()
            .filter(|n| {
                matches!(
                    n,
                    ASTNode::IfElse {
                        true_body,
                        false_body,
                        ..
                    } if true_body.iter().any(|b| matches!(b, ASTNode::Call { import_name, .. } if import_name == "baals_write_storage"))
                        && false_body.is_empty()
                )
            })
            .count();

        assert_eq!(
            guarded_writes, 2,
            "Expected two guarded WriteStorage calls under If branching"
        );
    }
}
