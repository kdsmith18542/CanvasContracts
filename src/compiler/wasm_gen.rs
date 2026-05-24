use crate::compiler::ast::{ASTNode, ConditionOpKind, I64BinOpKind, I64UnaryOpKind, AST};
use std::collections::{HashMap, HashSet};
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

#[derive(Debug, Clone)]
struct ImportSignature {
    params: Vec<ValType>,
    results: Vec<ValType>,
}

impl ImportSignature {
    fn for_import(module: &str, name: &str) -> Option<Self> {
        match (module, name) {
            ("baals", "baals_read_storage") => Some(Self {
                params: vec![ValType::I64],
                results: vec![ValType::I64],
            }),
            ("baals", "baals_write_storage") => Some(Self {
                params: vec![ValType::I64, ValType::I64],
                results: vec![],
            }),
            _ => None,
        }
    }

    fn returns_value(&self) -> bool {
        !self.results.is_empty()
    }
}

impl WasmGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, ast: &AST) -> Result<WasmGenResult, String> {
        let mut module = Module::new();

        let ordered_imports = self.collect_ordered_imports(ast);
        let (types, import_type_indices) = self.build_type_section(&ordered_imports)?;
        module.section(&types);

        let mut import_names = Vec::new();
        let (imports, import_func_indices) =
            self.build_import_section(&ordered_imports, &import_type_indices, &mut import_names)?;
        if !ordered_imports.is_empty() {
            module.section(&imports);
        }

        let mut functions = FunctionSection::new();
        // Type index 0 is always main: () -> i64.
        functions.function(0);
        module.section(&functions);

        let mut exports = ExportSection::new();
        let main_func_index = ordered_imports.len() as u32;
        exports.export("main", ExportKind::Func, main_func_index);
        module.section(&exports);

        let mut codes = CodeSection::new();
        // One local that stores the latest produced i64 expression.
        let mut func_body = Function::new(vec![(1, ValType::I64)]);
        let result_local = 0u32;
        let mut has_result = false;

        for node in &ast.body {
            let produced_value = self.emit_node(&mut func_body, node, &import_func_indices)?;
            if produced_value {
                func_body.instruction(&Instruction::LocalSet(result_local));
                has_result = true;
            }
        }

        if has_result {
            func_body.instruction(&Instruction::LocalGet(result_local));
        } else {
            func_body.instruction(&Instruction::I64Const(0));
        }
        func_body.instruction(&Instruction::End);

        codes.function(&func_body);
        module.section(&codes);

        let wasm_bytes = module.finish();

        Ok(WasmGenResult {
            wasm_bytes,
            functions: vec!["main".to_string()],
            imports: import_names,
            exports: vec!["main".to_string()],
        })
    }

    fn collect_ordered_imports(&self, ast: &AST) -> Vec<(String, String)> {
        let mut seen = HashSet::new();
        let mut ordered = Vec::new();

        for (module_name, func_name) in &ast.imports {
            let key = import_key(module_name, func_name);
            if seen.insert(key) {
                ordered.push((module_name.clone(), func_name.clone()));
            }
        }

        ordered
    }

    fn build_type_section(
        &self,
        ordered_imports: &[(String, String)],
    ) -> Result<(TypeSection, HashMap<String, u32>), String> {
        let mut types = TypeSection::new();
        let mut import_type_indices = HashMap::new();

        // Type index 0: main () -> i64
        types.function([], [ValType::I64]);

        let mut next_type_index = 1u32;
        for (module_name, func_name) in ordered_imports {
            let key = import_key(module_name, func_name);
            if import_type_indices.contains_key(&key) {
                continue;
            }

            let sig = ImportSignature::for_import(module_name, func_name).ok_or_else(|| {
                format!(
                    "Unsupported import function '{}.{}' during WASM generation",
                    module_name, func_name
                )
            })?;

            types.function(sig.params, sig.results);
            import_type_indices.insert(key, next_type_index);
            next_type_index += 1;
        }

        Ok((types, import_type_indices))
    }

    fn build_import_section(
        &self,
        ordered_imports: &[(String, String)],
        import_type_indices: &HashMap<String, u32>,
        import_names: &mut Vec<String>,
    ) -> Result<(ImportSection, HashMap<String, u32>), String> {
        let mut import_section = ImportSection::new();
        let mut import_func_indices = HashMap::new();

        for (func_index, (module_name, func_name)) in ordered_imports.iter().enumerate() {
            let key = import_key(module_name, func_name);
            let type_index = import_type_indices.get(&key).ok_or_else(|| {
                format!(
                    "Missing type index for import '{}.{}' during WASM generation",
                    module_name, func_name
                )
            })?;

            import_section.import(module_name, func_name, EntityType::Function(*type_index));
            import_func_indices.insert(key.clone(), func_index as u32);
            import_names.push(key);
        }

        Ok((import_section, import_func_indices))
    }

    fn emit_node(
        &self,
        func: &mut Function,
        node: &ASTNode,
        import_func_indices: &HashMap<String, u32>,
    ) -> Result<bool, String> {
        match node {
            ASTNode::I64Const(value) => {
                func.instruction(&Instruction::I64Const(*value));
                Ok(true)
            }
            ASTNode::I64BinOp { op, left, right } => {
                let left_produces = self.emit_node(func, left, import_func_indices)?;
                let right_produces = self.emit_node(func, right, import_func_indices)?;
                if !left_produces || !right_produces {
                    return Err("Binary operation requires value-producing operands".to_string());
                }

                if matches!(op, I64BinOpKind::Div) {
                    if let ASTNode::I64Const(0) = right.as_ref() {
                        return Err("Division by zero in WASM generation".to_string());
                    }
                }

                match op {
                    I64BinOpKind::Add => {
                        func.instruction(&Instruction::I64Add);
                    }
                    I64BinOpKind::Sub => {
                        func.instruction(&Instruction::I64Sub);
                    }
                    I64BinOpKind::Mul => {
                        func.instruction(&Instruction::I64Mul);
                    }
                    I64BinOpKind::Div => {
                        func.instruction(&Instruction::I64DivS);
                    }
                }

                Ok(true)
            }
            ASTNode::I64UnaryOp { op, operand } => {
                let produces = self.emit_node(func, operand, import_func_indices)?;
                if !produces {
                    return Err("Unary operation requires a value-producing operand".to_string());
                }

                match op {
                    I64UnaryOpKind::Not => {
                        func.instruction(&Instruction::I64Eqz);
                        func.instruction(&Instruction::I64ExtendI32U);
                    }
                }

                Ok(true)
            }
            ASTNode::I64Condition { op, left, right } => match op {
                ConditionOpKind::And => {
                    let left_produces = self.emit_node(func, left, import_func_indices)?;
                    if !left_produces {
                        return Err("AND condition left operand must produce a value".to_string());
                    }
                    func.instruction(&Instruction::I64Const(0));
                    func.instruction(&Instruction::I64Ne);

                    let right_produces = self.emit_node(func, right, import_func_indices)?;
                    if !right_produces {
                        return Err("AND condition right operand must produce a value".to_string());
                    }
                    func.instruction(&Instruction::I64Const(0));
                    func.instruction(&Instruction::I64Ne);

                    func.instruction(&Instruction::I32And);
                    func.instruction(&Instruction::I64ExtendI32U);
                    Ok(true)
                }
                ConditionOpKind::Or => {
                    let left_produces = self.emit_node(func, left, import_func_indices)?;
                    if !left_produces {
                        return Err("OR condition left operand must produce a value".to_string());
                    }
                    func.instruction(&Instruction::I64Const(0));
                    func.instruction(&Instruction::I64Ne);

                    let right_produces = self.emit_node(func, right, import_func_indices)?;
                    if !right_produces {
                        return Err("OR condition right operand must produce a value".to_string());
                    }
                    func.instruction(&Instruction::I64Const(0));
                    func.instruction(&Instruction::I64Ne);

                    func.instruction(&Instruction::I32Or);
                    func.instruction(&Instruction::I64ExtendI32U);
                    Ok(true)
                }
                ConditionOpKind::Eq
                | ConditionOpKind::Ne
                | ConditionOpKind::Lt
                | ConditionOpKind::Gt => {
                    let left_produces = self.emit_node(func, left, import_func_indices)?;
                    let right_produces = self.emit_node(func, right, import_func_indices)?;
                    if !left_produces || !right_produces {
                        return Err(
                            "Comparison condition requires value-producing operands".to_string()
                        );
                    }

                    match op {
                        ConditionOpKind::Eq => {
                            func.instruction(&Instruction::I64Eq);
                        }
                        ConditionOpKind::Ne => {
                            func.instruction(&Instruction::I64Ne);
                        }
                        ConditionOpKind::Lt => {
                            func.instruction(&Instruction::I64LtS);
                        }
                        ConditionOpKind::Gt => {
                            func.instruction(&Instruction::I64GtS);
                        }
                        ConditionOpKind::And | ConditionOpKind::Or => unreachable!(),
                    }

                    func.instruction(&Instruction::I64ExtendI32U);
                    Ok(true)
                }
            },
            ASTNode::I64IfElse {
                condition,
                when_true,
                when_false,
            } => {
                let cond_produces = self.emit_node(func, condition, import_func_indices)?;
                if !cond_produces {
                    return Err("I64IfElse condition must produce a value".to_string());
                }

                func.instruction(&Instruction::I64Eqz);
                func.instruction(&Instruction::If(BlockType::Result(ValType::I64)));

                let false_produces = self.emit_node(func, when_false, import_func_indices)?;
                if !false_produces {
                    return Err("I64IfElse false branch must produce a value".to_string());
                }

                func.instruction(&Instruction::Else);

                let true_produces = self.emit_node(func, when_true, import_func_indices)?;
                if !true_produces {
                    return Err("I64IfElse true branch must produce a value".to_string());
                }

                func.instruction(&Instruction::End);
                Ok(true)
            }
            ASTNode::IfElse {
                condition,
                true_body,
                false_body,
            } => {
                let cond_produces = self.emit_node(func, condition, import_func_indices)?;
                if !cond_produces {
                    return Err("IfElse condition must produce a value".to_string());
                }

                func.instruction(&Instruction::I64Eqz);
                func.instruction(&Instruction::If(BlockType::Empty));

                for stmt in false_body {
                    if self.emit_node(func, stmt, import_func_indices)? {
                        func.instruction(&Instruction::Drop);
                    }
                }

                func.instruction(&Instruction::Else);

                for stmt in true_body {
                    if self.emit_node(func, stmt, import_func_indices)? {
                        func.instruction(&Instruction::Drop);
                    }
                }

                func.instruction(&Instruction::End);
                Ok(false)
            }
            ASTNode::Call {
                import_module,
                import_name,
                args,
            } => {
                let signature = ImportSignature::for_import(import_module, import_name)
                    .ok_or_else(|| {
                        format!(
                            "Unsupported import call '{}.{}' during WASM generation",
                            import_module, import_name
                        )
                    })?;

                if args.len() != signature.params.len() {
                    return Err(format!(
                        "Import '{}.{}' expects {} argument(s), found {}",
                        import_module,
                        import_name,
                        signature.params.len(),
                        args.len()
                    ));
                }

                for arg in args {
                    let produced = self.emit_node(func, arg, import_func_indices)?;
                    if !produced {
                        return Err(format!(
                            "Import argument for '{}.{}' does not produce a value",
                            import_module, import_name
                        ));
                    }
                }

                let key = import_key(import_module, import_name);
                let func_index = import_func_indices
                    .get(&key)
                    .ok_or_else(|| format!("Missing function index for import '{}'", key))?;

                func.instruction(&Instruction::Call(*func_index));
                Ok(signature.returns_value())
            }
            ASTNode::Nop => Ok(false),
        }
    }
}

fn import_key(module: &str, name: &str) -> String {
    format!("{}.{}", module, name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_simple_ast() -> AST {
        AST {
            body: vec![ASTNode::I64BinOp {
                op: I64BinOpKind::Add,
                left: Box::new(ASTNode::I64Const(10)),
                right: Box::new(ASTNode::I64Const(20)),
            }],
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
                    left: Box::new(ASTNode::I64Const(2)),
                    right: Box::new(ASTNode::I64Const(7)),
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

        assert_ne!(
            result1.wasm_bytes, result2.wasm_bytes,
            "Different ASTs must produce different WASM bytecode"
        );
    }

    #[test]
    fn test_generated_wasm_validates_with_wasmtime() {
        let gen = WasmGenerator::new();
        let ast = build_simple_ast();
        let result = gen.generate(&ast).unwrap();

        let engine = wasmtime::Engine::default();
        let validation = wasmtime::Module::validate(&engine, &result.wasm_bytes);
        assert!(
            validation.is_ok(),
            "WASM validation failed: {:?}",
            validation.err()
        );
    }

    #[test]
    fn test_arithmetic_wasm_executes_correctly() {
        let gen = WasmGenerator::new();
        let ast = AST {
            body: vec![ASTNode::I64BinOp {
                op: I64BinOpKind::Add,
                left: Box::new(ASTNode::I64Const(15)),
                right: Box::new(ASTNode::I64Const(25)),
            }],
            imports: vec![],
        };
        let result = gen.generate(&ast).unwrap();

        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::new(&engine, &result.wasm_bytes).unwrap();
        let mut store = wasmtime::Store::new(&engine, ());
        let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
        let main = instance
            .get_typed_func::<(), i64>(&mut store, "main")
            .unwrap();
        let val = main.call(&mut store, ()).unwrap();
        assert_eq!(val, 40, "15 + 25 should equal 40");
    }

    #[test]
    fn test_divide_by_zero_returns_error() {
        let gen = WasmGenerator::new();
        let ast = AST {
            body: vec![ASTNode::I64BinOp {
                op: I64BinOpKind::Div,
                left: Box::new(ASTNode::I64Const(10)),
                right: Box::new(ASTNode::I64Const(0)),
            }],
            imports: vec![],
        };
        assert!(gen.generate(&ast).is_err());
    }

    #[test]
    fn test_nested_expression_chain() {
        let gen = WasmGenerator::new();
        // (10 + 20) * 2
        let ast = AST {
            body: vec![ASTNode::I64BinOp {
                op: I64BinOpKind::Mul,
                left: Box::new(ASTNode::I64BinOp {
                    op: I64BinOpKind::Add,
                    left: Box::new(ASTNode::I64Const(10)),
                    right: Box::new(ASTNode::I64Const(20)),
                }),
                right: Box::new(ASTNode::I64Const(2)),
            }],
            imports: vec![],
        };
        let result = gen.generate(&ast).unwrap();

        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::new(&engine, &result.wasm_bytes).unwrap();
        let mut store = wasmtime::Store::new(&engine, ());
        let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
        let main = instance
            .get_typed_func::<(), i64>(&mut store, "main")
            .unwrap();
        let val = main.call(&mut store, ()).unwrap();
        assert_eq!(val, 60, "(10 + 20) * 2 should equal 60");
    }

    #[test]
    fn test_storage_import_wasm_validates() {
        let gen = WasmGenerator::new();
        let ast = AST {
            body: vec![
                ASTNode::Call {
                    import_module: "baals".to_string(),
                    import_name: "baals_write_storage".to_string(),
                    args: vec![ASTNode::I64Const(7), ASTNode::I64Const(99)],
                },
                ASTNode::Call {
                    import_module: "baals".to_string(),
                    import_name: "baals_read_storage".to_string(),
                    args: vec![ASTNode::I64Const(7)],
                },
            ],
            imports: vec![
                ("baals".to_string(), "baals_write_storage".to_string()),
                ("baals".to_string(), "baals_read_storage".to_string()),
            ],
        };

        let result = gen.generate(&ast).unwrap();
        let engine = wasmtime::Engine::default();
        let validation = wasmtime::Module::validate(&engine, &result.wasm_bytes);
        assert!(
            validation.is_ok(),
            "WASM validation failed: {:?}",
            validation.err()
        );
        assert!(result
            .imports
            .contains(&"baals.baals_read_storage".to_string()));
        assert!(result
            .imports
            .contains(&"baals.baals_write_storage".to_string()));
    }

    #[test]
    fn test_if_else_branching_wasm_validates() {
        let gen = WasmGenerator::new();
        let ast = AST {
            body: vec![
                ASTNode::IfElse {
                    condition: Box::new(ASTNode::I64Const(1)),
                    true_body: vec![ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_write_storage".to_string(),
                        args: vec![ASTNode::I64Const(7), ASTNode::I64Const(42)],
                    }],
                    false_body: vec![ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_write_storage".to_string(),
                        args: vec![ASTNode::I64Const(7), ASTNode::I64Const(0)],
                    }],
                },
                ASTNode::I64IfElse {
                    condition: Box::new(ASTNode::I64Const(1)),
                    when_true: Box::new(ASTNode::Call {
                        import_module: "baals".to_string(),
                        import_name: "baals_read_storage".to_string(),
                        args: vec![ASTNode::I64Const(7)],
                    }),
                    when_false: Box::new(ASTNode::I64Const(0)),
                },
            ],
            imports: vec![
                ("baals".to_string(), "baals_write_storage".to_string()),
                ("baals".to_string(), "baals_read_storage".to_string()),
            ],
        };

        let result = gen.generate(&ast).unwrap();
        let engine = wasmtime::Engine::default();
        let validation = wasmtime::Module::validate(&engine, &result.wasm_bytes);
        assert!(
            validation.is_ok(),
            "WASM validation failed: {:?}",
            validation.err()
        );
    }
}
