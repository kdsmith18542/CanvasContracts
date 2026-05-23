use crate::compiler::graph_ir::GraphIR;
use serde::Serialize;

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
pub enum I64BinOpKind { Add, Sub, Mul, Div }

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum I64UnaryOpKind { Not }

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ConditionOpKind { Eq, Ne, Lt, Gt, And, Or }

#[derive(Debug, Clone, Serialize)]
pub struct AST {
    pub body: Vec<ASTNode>,
    pub imports: Vec<(String, String)>,
}

impl AST {
    pub fn new() -> Self {
        Self { body: Vec::new(), imports: Vec::new() }
    }

    pub fn from_graph_ir(ir: &GraphIR) -> Self {
        let mut ast = AST::new();

        let order = match petgraph::algo::toposort(&ir.graph, None) {
            Ok(o) => o,
            Err(_) => return ast,
        };

        // Cache: node_id → (port_id → ASTNode expression for that output)
        use crate::types::NodeId;
        use std::collections::HashMap;
        let mut output_exprs: HashMap<NodeId, HashMap<String, ASTNode>> = HashMap::new();

        for node_idx in order {
            let node_id = ir.graph[node_idx];
            let ir_node = match ir.nodes.get(&node_id) {
                Some(n) => n,
                None => continue,
            };

            // Resolve an input port's value: check connections, then properties, then default
            let resolve_input = |port_id: &str, ir: &GraphIR, node_id: &NodeId,
                                 output_exprs: &HashMap<NodeId, HashMap<String, ASTNode>>,
                                 properties: &HashMap<String, serde_json::Value>| -> ASTNode {
                // Check if this port has an incoming connection
                for conn in &ir.connections {
                    if conn.target == *node_id && conn.target_port == port_id {
                        // Found connection — use predecessor's cached output expression
                        if let Some(port_map) = output_exprs.get(&conn.source) {
                            if let Some(expr) = port_map.get(&conn.source_port) {
                                return expr.clone();
                            }
                        }
                    }
                }
                // Fall back to property value
                if let Some(val) = properties.get(port_id).and_then(|v| v.as_i64()) {
                    return ASTNode::I64Const(val);
                }
                ASTNode::I64Const(0)
            };

            let ast_node = match ir_node.node_type.as_str() {
                "Add" | "Subtract" | "Multiply" | "Divide" => {
                    let a = resolve_input("a", ir, &node_id, &output_exprs, &ir_node.properties);
                    let b = resolve_input("b", ir, &node_id, &output_exprs, &ir_node.properties);
                    let op = match ir_node.node_type.as_str() {
                        "Add" => I64BinOpKind::Add,
                        "Subtract" => I64BinOpKind::Sub,
                        "Multiply" => I64BinOpKind::Mul,
                        "Divide" => I64BinOpKind::Div,
                        _ => unreachable!(),
                    };
                    // Wrap in a struct-like node, output is the result
                    ASTNode::I64BinOp { op, left: Box::new(a), right: Box::new(b) }
                }
                "And" => {
                    let a = resolve_input("a", ir, &node_id, &output_exprs, &ir_node.properties);
                    let b = resolve_input("b", ir, &node_id, &output_exprs, &ir_node.properties);
                    ASTNode::I64Condition { op: ConditionOpKind::And, left: Box::new(a), right: Box::new(b) }
                }
                "Or" => {
                    let a = resolve_input("a", ir, &node_id, &output_exprs, &ir_node.properties);
                    let b = resolve_input("b", ir, &node_id, &output_exprs, &ir_node.properties);
                    ASTNode::I64Condition { op: ConditionOpKind::Or, left: Box::new(a), right: Box::new(b) }
                }
                "Not" => {
                    let operand = resolve_input("input", ir, &node_id, &output_exprs, &ir_node.properties);
                    ASTNode::I64UnaryOp { op: I64UnaryOpKind::Not, operand: Box::new(operand) }
                }
                "ReadStorage" => {
                    ast.imports.push(("baals".to_string(), "baals_read_storage".to_string()));
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_read_storage".to_string(),
                        args: vec![ASTNode::I64Const(0)],
                    }
                }
                "WriteStorage" => {
                    let value = resolve_input("value", ir, &node_id, &output_exprs, &ir_node.properties);
                    ast.imports.push(("baals".to_string(), "baals_write_storage".to_string()));
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_write_storage".to_string(),
                        args: vec![value],
                    }
                }
                "If" => {
                    let condition_val = ir_node.properties.get("condition").and_then(|v| v.as_bool()).unwrap_or(false);
                    let cond = Box::new(ASTNode::I64Const(if condition_val { 1 } else { 0 }));
                    let (true_body, false_body) = find_if_branches(ir, node_id);
                    ASTNode::IfElse { condition: cond, true_body, false_body }
                }
                "Start" | "End" | "VerifySignature" | "DecodeProof" => ASTNode::Nop,
                _ => ASTNode::Nop,
            };

            // Cache this node's output expressions for downstream consumers
            let mut outputs = HashMap::new();
            if matches!(&ast_node, ASTNode::Nop) {
                // Nop nodes produce no value output
            } else if matches!(&ast_node, ASTNode::Call { .. }) {
                // Call nodes may produce side effects but no meaningful output to chain
                outputs.insert("result".to_string(), ast_node.clone());
            } else {
                outputs.insert("result".to_string(), ast_node.clone());
            }
            // For non-If nodes, also cache a flow output
            if ir_node.node_type != "If" && !matches!(&ast_node, ASTNode::Nop) {
                outputs.insert("flow_out".to_string(), ast_node.clone());
            }
            output_exprs.insert(node_id, outputs);

            // Add to body
            ast.body.push(ast_node);
        }

        ast
    }
}

impl Default for AST {
    fn default() -> Self { Self::new() }
}

fn find_if_branches(ir: &GraphIR, if_node_id: crate::types::NodeId) -> (Vec<ASTNode>, Vec<ASTNode>) {
    let mut true_body = Vec::new();
    let mut false_body = Vec::new();

    for conn in &ir.connections {
        if conn.source == if_node_id {
            let target_node = ir.nodes.get(&conn.target);
            if let Some(target) = target_node {
                let ast_node = match target.node_type.as_str() {
                    "WriteStorage" => {
                        let value = target.properties.get("value").and_then(|v| v.as_i64()).unwrap_or(0);
                        ASTNode::Call {
                            import_module: "baals".to_string(),
                            import_name: "baals_write_storage".to_string(),
                            args: vec![ASTNode::I64Const(value)],
                        }
                    }
                    "ReadStorage" => {
                        ASTNode::Call {
                            import_module: "baals".to_string(),
                            import_name: "baals_read_storage".to_string(),
                            args: vec![ASTNode::I64Const(0)],
                        }
                    }
                    _ => ASTNode::Nop,
                };
                if conn.source_port == "true_flow" {
                    true_body.push(ast_node);
                } else if conn.source_port == "false_flow" {
                    false_body.push(ast_node);
                }
            }
        }
    }

    (true_body, false_body)
}
