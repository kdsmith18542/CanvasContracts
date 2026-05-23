use crate::compiler::ast::{AST, ASTNode, I64BinOpKind, I64UnaryOpKind, ConditionOpKind};
use wasm_encoder::*;

#[derive(Debug, Clone)]
pub struct WasmGenResult {
    pub wasm_bytes: Vec<u8>,
    pub functions: Vec<String>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
}

pub struct WasmGenerator;

impl Default for WasmGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, ast: &AST) -> Result<WasmGenResult, String> {
        let mut module = Module::new();
        let mut functions = Vec::new();
        let mut import_names = Vec::new();
        let mut export_names: Vec<String> = Vec::new();

        // Collect unique imports from AST
        let mut seen_imports = std::collections::HashSet::new();
        for (module_name, func_name) in &ast.imports {
            let key = format!("{}.{}", module_name, func_name);
            if seen_imports.insert(key) {
                import_names.push(format!("{}.{}", module_name, func_name));
            }
        }

        // Build import section
        if !ast.imports.is_empty() {
            let mut import_section = ImportSection::new();
            let mut import_counts = std::collections::HashMap::<String, u32>::new();
            for (module_name, func_name) in &ast.imports {
                let key = format!("{}.{}", module_name, func_name);
                if !import_counts.contains_key(&key) {
                    let count = import_counts.len() as u32;
                    import_counts.insert(key, count);
                    import_section.import(module_name, func_name, EntityType::Function(0));
                }
            }
            module.section(&import_section);
        }

        // Build type section
        let mut types = TypeSection::new();
        types.function([], [ValType::I64]);
        if !ast.imports.is_empty() {
            types.function([], []);
        }
        module.section(&types);

        // Build function section
        let mut func_section = FunctionSection::new();
        func_section.function(0);
        module.section(&func_section);

        functions.push("main".to_string());

        // Build export section
        let mut exports = ExportSection::new();
        let func_index = ast.imports.len() as u32;
        exports.export("main", ExportKind::Func, func_index);
        export_names.push("main".to_string());
        module.section(&exports);

        // Build code section
        let mut codes = CodeSection::new();
        let mut func_body = Function::new(vec![]);

        let mut has_body = false;
        for node in &ast.body {
            if !matches!(node, ASTNode::Nop) {
                has_body = true;
            }
            self.emit_node(&mut func_body, node)?;
        }

        if !has_body {
            func_body.instruction(&Instruction::I64Const(0));
        }
        func_body.instruction(&Instruction::End);

        codes.function(&func_body);
        module.section(&codes);

        let wasm_bytes = module.finish();

        Ok(WasmGenResult {
            wasm_bytes,
            functions,
            imports: import_names,
            exports: export_names,
        })
    }

    fn emit_node(&self, func: &mut Function, node: &ASTNode) -> Result<(), String> {
        match node {
            ASTNode::I64Const(val) => {
                func.instruction(&Instruction::I64Const(*val));
            }
            ASTNode::I64BinOp { op, left, right } => {
                if matches!(op, I64BinOpKind::Div) {
                    // Check for divide-by-zero at codegen time when both sides are constants
                    if let ASTNode::I64Const(r) = right.as_ref() {
                        if *r == 0 {
                            return Err("Division by zero in WASM generation".to_string());
                        }
                    }
                }
                self.emit_node(func, left)?;
                self.emit_node(func, right)?;
                match op {
                    I64BinOpKind::Add => { func.instruction(&Instruction::I64Add); }
                    I64BinOpKind::Sub => { func.instruction(&Instruction::I64Sub); }
                    I64BinOpKind::Mul => { func.instruction(&Instruction::I64Mul); }
                    I64BinOpKind::Div => { func.instruction(&Instruction::I64DivS); }
                }
            }
            ASTNode::I64UnaryOp { op, operand } => {
                self.emit_node(func, operand)?;
                match op {
                    I64UnaryOpKind::Not => { func.instruction(&Instruction::I64Eqz); }
                }
            }
            ASTNode::I64Condition { op, left, right } => {
                self.emit_node(func, left)?;
                self.emit_node(func, right)?;
                match op {
                    ConditionOpKind::And => {
                        // a && b: (a != 0) * (b != 0), result is 0 or 1
                        func.instruction(&Instruction::I64Const(0));
                        func.instruction(&Instruction::I64Ne);
                        func.instruction(&Instruction::I64ExtendI32S);
                        func.instruction(&Instruction::I64Const(0));
                        func.instruction(&Instruction::I64Ne);
                        func.instruction(&Instruction::I64ExtendI32S);
                        func.instruction(&Instruction::I64Mul);
                        return Ok(());
                    }
                    ConditionOpKind::Or => {
                        // a || b: ((a != 0) + (b != 0)) > 0, result is 0 or 1
                        func.instruction(&Instruction::I64Const(0));
                        func.instruction(&Instruction::I64Ne);
                        func.instruction(&Instruction::I64ExtendI32S);
                        func.instruction(&Instruction::I64Const(0));
                        func.instruction(&Instruction::I64Ne);
                        func.instruction(&Instruction::I64ExtendI32S);
                        func.instruction(&Instruction::I64Add);
                        func.instruction(&Instruction::I64Const(0));
                        func.instruction(&Instruction::I64GtS);
                        func.instruction(&Instruction::I64ExtendI32S);
                        return Ok(());
                    }
                    ConditionOpKind::Eq => { func.instruction(&Instruction::I64Eq); }
                    ConditionOpKind::Ne => { func.instruction(&Instruction::I64Ne); }
                    ConditionOpKind::Lt => { func.instruction(&Instruction::I64LtS); }
                    ConditionOpKind::Gt => { func.instruction(&Instruction::I64GtS); }
                }
                func.instruction(&Instruction::I64ExtendI32S);
            }
            ASTNode::IfElse { condition, true_body, false_body } => {
                self.emit_node(func, condition)?;
                func.instruction(&Instruction::I64Eqz);
                func.instruction(&Instruction::If(BlockType::Empty));
                for node in false_body {
                    self.emit_node(func, node)?;
                }
                func.instruction(&Instruction::Else);
                for node in true_body {
                    self.emit_node(func, node)?;
                }
                func.instruction(&Instruction::End);
            }
            ASTNode::Call { args, .. } => {
                for arg in args {
                    self.emit_node(func, arg)?;
                }
                func.instruction(&Instruction::Call(0));
                func.instruction(&Instruction::Drop);
            }
            ASTNode::Nop => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_simple_ast() -> AST {
        AST {
            body: vec![
                ASTNode::I64BinOp {
                    op: I64BinOpKind::Add,
                    left: Box::new(ASTNode::I64Const(10)),
                    right: Box::new(ASTNode::I64Const(20)),
                },
            ],
            imports: vec![],
        }
    }

    fn build_arithmetic_ast() -> AST {
        AST {
            body: vec![
                ASTNode::I64BinOp {
                    op: I64BinOpKind::Add,
                    left: Box::new(ASTNode::I64Const(5)),
                    right: Box::new(ASTNode::I64Const(3)),
                },
                ASTNode::I64BinOp {
                    op: I64BinOpKind::Mul,
                    left: Box::new(ASTNode::I64Const(0)),
                    right: Box::new(ASTNode::I64Const(0)),
                },
                ASTNode::I64BinOp {
                    op: I64BinOpKind::Sub,
                    left: Box::new(ASTNode::I64Const(10)),
                    right: Box::new(ASTNode::I64Const(4)),
                },
            ],
            imports: vec![],
        }
    }

    #[test]
    fn test_generator_creates_valid_wasm() {
        let gen = WasmGenerator::new();
        let ast = build_simple_ast();
        let result = gen.generate(&ast).unwrap();

        assert_eq!(&result.wasm_bytes[0..4], &[0x00, 0x61, 0x73, 0x6d]);
        assert_eq!(&result.wasm_bytes[4..8], &[0x01, 0x00, 0x00, 0x00]);
        assert!(!result.wasm_bytes.is_empty());
        assert!(result.exports.contains(&"main".to_string()));
    }

    #[test]
    fn test_different_asts_produce_different_wasm() {
        let gen = WasmGenerator::new();

        let ast1 = build_simple_ast();
        let result1 = gen.generate(&ast1).unwrap();

        let ast2 = build_arithmetic_ast();
        let result2 = gen.generate(&ast2).unwrap();

        assert_ne!(result1.wasm_bytes, result2.wasm_bytes,
            "Different ASTs must produce different WASM bytecode");
    }

    #[test]
    fn test_generated_wasm_validates_with_wasmtime() {
        let gen = WasmGenerator::new();
        let ast = build_simple_ast();
        let result = gen.generate(&ast).unwrap();

        let engine = wasmtime::Engine::default();
        let validation = wasmtime::Module::validate(&engine, &result.wasm_bytes);
        assert!(validation.is_ok(), "WASM validation failed: {:?}", validation.err());
    }

    #[test]
    fn test_arithmetic_wasm_executes_correctly() {
        let gen = WasmGenerator::new();
        let ast = AST {
            body: vec![
                ASTNode::I64BinOp {
                    op: I64BinOpKind::Add,
                    left: Box::new(ASTNode::I64Const(15)),
                    right: Box::new(ASTNode::I64Const(25)),
                },
            ],
            imports: vec![],
        };
        let result = gen.generate(&ast).unwrap();

        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::new(&engine, &result.wasm_bytes).unwrap();
        let mut store = wasmtime::Store::new(&engine, ());
        let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
        let main = instance.get_typed_func::<(), i64>(&mut store, "main").unwrap();
        let val = main.call(&mut store, ()).unwrap();
        assert_eq!(val, 40, "15 + 25 should equal 40");
    }

    #[test]
    fn test_divide_by_zero_returns_error() {
        let gen = WasmGenerator::new();
        let ast = AST {
            body: vec![
                ASTNode::I64BinOp {
                    op: I64BinOpKind::Div,
                    left: Box::new(ASTNode::I64Const(10)),
                    right: Box::new(ASTNode::I64Const(0)),
                },
            ],
            imports: vec![],
        };
        assert!(gen.generate(&ast).is_err());
    }

    #[test]
    fn test_nested_expression_chain() {
        let gen = WasmGenerator::new();
        // (10 + 20) * 2
        let ast = AST {
            body: vec![
                ASTNode::I64BinOp {
                    op: I64BinOpKind::Mul,
                    left: Box::new(ASTNode::I64BinOp {
                        op: I64BinOpKind::Add,
                        left: Box::new(ASTNode::I64Const(10)),
                        right: Box::new(ASTNode::I64Const(20)),
                    }),
                    right: Box::new(ASTNode::I64Const(2)),
                },
            ],
            imports: vec![],
        };
        let result = gen.generate(&ast).unwrap();

        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::new(&engine, &result.wasm_bytes).unwrap();
        let mut store = wasmtime::Store::new(&engine, ());
        let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
        let main = instance.get_typed_func::<(), i64>(&mut store, "main").unwrap();
        let val = main.call(&mut store, ()).unwrap();
        assert_eq!(val, 60, "(10 + 20) * 2 should equal 60");
    }
}
