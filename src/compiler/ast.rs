//! Abstract Syntax Tree (AST) generation from Graph IR

use crate::compiler::graph_ir::GraphIR;
use serde::Serialize;

/// Semantic AST node types for WASM compilation
#[derive(Debug, Clone, Serialize)]
pub enum ASTNode {
    /// I64 binary operation
    I64BinOp {
        op: I64BinOpKind,
        left: i64,
        right: i64,
    },
    /// I64 unary operation
    I64UnaryOp {
        op: I64UnaryOpKind,
        operand: i64,
    },
    /// Condition with I64 operands (evaluated as i64 != 0)
    I64Condition {
        op: ConditionOpKind,
        left: i64,
        right: i64,
    },
    /// If/else branching
    IfElse {
        condition: Box<ASTNode>,
        true_body: Vec<ASTNode>,
        false_body: Vec<ASTNode>,
    },
    /// Host function call (for storage operations)
    Call {
        import_module: String,
        import_name: String,
        args: Vec<ASTNode>,
    },
    /// No-op (for Start/End flow nodes)
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

/// AST representation — the body of the contract's main function
#[derive(Debug, Clone, Serialize)]
pub struct AST {
    pub body: Vec<ASTNode>,
    /// Imports required (module, name)
    pub imports: Vec<(String, String)>,
}

impl AST {
    pub fn new() -> Self {
        Self {
            body: Vec::new(),
            imports: Vec::new(),
        }
    }

    /// Generate AST from Graph IR using topological order
    pub fn from_graph_ir(ir: &GraphIR) -> Self {
        let mut ast = AST::new();

        // Topological sort for execution order
        let order = match petgraph::algo::toposort(&ir.graph, None) {
            Ok(o) => o,
            Err(_) => return ast, // cyclic graph, no AST
        };

        let name_to_index: std::collections::HashMap<_, _> = ir.nodes.keys()
            .enumerate()
            .map(|(i, id)| (*id, i))
            .collect();

        for node_idx in order {
            let node_id = ir.graph[node_idx];
            let ir_node = match ir.nodes.get(&node_id) {
                Some(n) => n,
                None => continue,
            };

            let ast_node = match ir_node.node_type.as_str() {
                "Add" => {
                    let a = get_i64_property(&ir_node.properties, "a").unwrap_or(0);
                    let b = get_i64_property(&ir_node.properties, "b").unwrap_or(0);
                    ASTNode::I64BinOp { op: I64BinOpKind::Add, left: a, right: b }
                }
                "Subtract" => {
                    let a = get_i64_property(&ir_node.properties, "a").unwrap_or(0);
                    let b = get_i64_property(&ir_node.properties, "b").unwrap_or(0);
                    ASTNode::I64BinOp { op: I64BinOpKind::Sub, left: a, right: b }
                }
                "Multiply" => {
                    let a = get_i64_property(&ir_node.properties, "a").unwrap_or(0);
                    let b = get_i64_property(&ir_node.properties, "b").unwrap_or(0);
                    ASTNode::I64BinOp { op: I64BinOpKind::Mul, left: a, right: b }
                }
                "Divide" => {
                    let a = get_i64_property(&ir_node.properties, "a").unwrap_or(0);
                    let b = get_i64_property(&ir_node.properties, "b").unwrap_or(1);
                    ASTNode::I64BinOp { op: I64BinOpKind::Div, left: a, right: b }
                }
                "And" => {
                    let a = get_i64_property(&ir_node.properties, "a").unwrap_or(0);
                    let b = get_i64_property(&ir_node.properties, "b").unwrap_or(0);
                    ASTNode::I64Condition { op: ConditionOpKind::And, left: a, right: b }
                }
                "Or" => {
                    let a = get_i64_property(&ir_node.properties, "a").unwrap_or(0);
                    let b = get_i64_property(&ir_node.properties, "b").unwrap_or(0);
                    ASTNode::I64Condition { op: ConditionOpKind::Or, left: a, right: b }
                }
                "Not" => {
                    let operand = get_i64_property(&ir_node.properties, "input").unwrap_or(0);
                    ASTNode::I64UnaryOp { op: I64UnaryOpKind::Not, operand }
                }
                "ReadStorage" => {
                    let key = get_string_property(&ir_node.properties, "key")
                        .unwrap_or_else(|| "default_key".to_string());
                    ast.imports.push(("baals".to_string(), "baals_read_storage".to_string()));
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_read_storage".to_string(),
                        args: vec![ASTNode::I64BinOp {
                            op: I64BinOpKind::Add,
                            left: 0,
                            right: 0, // placeholder — key is a string, would need string encoding
                        }],
                    }
                }
                "WriteStorage" => {
                    let key = get_string_property(&ir_node.properties, "key")
                        .unwrap_or_else(|| "default_key".to_string());
                    let value = get_i64_property(&ir_node.properties, "value").unwrap_or(0);
                    ast.imports.push(("baals".to_string(), "baals_write_storage".to_string()));
                    ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_write_storage".to_string(),
                        args: vec![ASTNode::I64BinOp {
                            op: I64BinOpKind::Add, left: 0, right: 0 // placeholder
                        }],
                    }
                }
                "If" => {
                    let condition_val = get_bool_property(&ir_node.properties, "condition").unwrap_or(false);
                    let cond = if condition_val {
                        ASTNode::I64BinOp { op: I64BinOpKind::Add, left: 1, right: 0 }
                    } else {
                        ASTNode::I64BinOp { op: I64BinOpKind::Add, left: 0, right: 0 }
                    };
                    // Find true/false flow targets from connections
                    let (true_body, false_body) = find_if_branches(ir, node_id);
                    ASTNode::IfElse {
                        condition: Box::new(cond),
                        true_body,
                        false_body,
                    }
                }
                "Start" | "End" => ASTNode::Nop,
                _ => ASTNode::Nop,
            };

            ast.body.push(ast_node);
        }

        ast
    }
}

impl Default for AST {
    fn default() -> Self {
        Self::new()
    }
}

/// Find the nodes on the true and false branches of an If node
fn find_if_branches(ir: &GraphIR, if_node_id: crate::types::NodeId) -> (Vec<ASTNode>, Vec<ASTNode>) {
    let mut true_body = Vec::new();
    let mut false_body = Vec::new();

    for conn in &ir.connections {
        if conn.source == if_node_id {
            let target_node = ir.nodes.get(&conn.target);
            if let Some(target) = target_node {
                let ast_node = match target.node_type.as_str() {
                    "WriteStorage" => {
                        let key = get_string_property(&target.properties, "key")
                            .unwrap_or_else(|| "default".to_string());
                        let value = get_i64_property(&target.properties, "value").unwrap_or(0);
                        ASTNode::Call {
                            import_module: "baals".to_string(),
                            import_name: "baals_write_storage".to_string(),
                            args: vec![ASTNode::I64BinOp { op: I64BinOpKind::Add, left: 0, right: value }],
                        }
                    }
                    "ReadStorage" => {
                        ASTNode::Call {
                            import_module: "baals".to_string(),
                            import_name: "baals_read_storage".to_string(),
                            args: vec![ASTNode::I64BinOp { op: I64BinOpKind::Add, left: 0, right: 0 }],
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

fn get_i64_property(props: &std::collections::HashMap<String, serde_json::Value>, key: &str) -> Option<i64> {
    props.get(key).and_then(|v| v.as_i64())
}

fn get_string_property(props: &std::collections::HashMap<String, serde_json::Value>, key: &str) -> Option<String> {
    props.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn get_bool_property(props: &std::collections::HashMap<String, serde_json::Value>, key: &str) -> Option<bool> {
    props.get(key).and_then(|v| v.as_bool())
}
