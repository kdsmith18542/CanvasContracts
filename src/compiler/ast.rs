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
                "Start" | "End" => ASTNode::Nop,
                _ => unreachable!("unsupported node types are rejected above"),
            };

            let mut outputs = HashMap::new();
            if !matches!(&ast_node, ASTNode::Nop) {
                outputs.insert("result".to_string(), ast_node.clone());
                outputs.insert("flow_out".to_string(), ASTNode::I64Const(1));
            }
            output_exprs.insert(node_id, outputs);
            ast.body.push(ast_node);
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
            | "ReadStorage"
            | "WriteStorage"
    )
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
    use crate::types::{Position, VisualGraph, VisualNode};
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
            "GetSender",
            Position::new(150.0, 0.0),
        ));

        let ir = GraphIR::from_visual_graph(&graph);
        let err = AST::from_graph_ir(&ir).unwrap_err();

        assert!(err.contains("GetSender"));
        assert!(err.contains("not currently compilable"));
    }
}
