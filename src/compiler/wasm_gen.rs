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
            ("baals", "baals_get_sender") => Some(Self {
                params: vec![],
                results: vec![ValType::I64],
            }),
            ("baals", "baals_get_contract_id") => Some(Self {
                params: vec![],
                results: vec![ValType::I64],
            }),
            ("baals", "baals_get_block_timestamp") => Some(Self {
                params: vec![],
                results: vec![ValType::I64],
            }),
            ("baals", "baals_get_block_height") => Some(Self {
                params: vec![],
                results: vec![ValType::I64],
            }),
            ("baals", "baals_emit_event") => Some(Self {
                params: vec![ValType::I64, ValType::I64],
                results: vec![],
            }),
            ("baals", "baals_revert") => Some(Self {
                params: vec![ValType::I64],
                results: vec![],
            }),
            ("baals", "baals_hash_sha256") => Some(Self {
                params: vec![ValType::I64],
                results: vec![ValType::I64],
            }),
            ("baals", "baals_call_contract") => Some(Self {
                params: vec![ValType::I64, ValType::I64, ValType::I64],
                results: vec![],
            }),
            ("baals", "baals_read_call_result") => Some(Self {
                params: vec![ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("baals", "baals_transfer_value") => Some(Self {
                params: vec![ValType::I64, ValType::I64],
                results: vec![],
            }),
            ("crypto", "crypto_verify_signature") => Some(Self {
                params: vec![ValType::I64, ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("crypto", "crypto_decode_proof") => Some(Self {
                params: vec![ValType::I64],
                results: vec![ValType::I64],
            }),
            ("chrononode", "chrononode_fetch_block") => Some(Self {
                params: vec![ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("chrononode", "chrononode_fetch_checkpoint") => Some(Self {
                params: vec![ValType::I64, ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("chrononode", "chrononode_verify_proof") => Some(Self {
                params: vec![ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("chrononode", "chrononode_extract_event") => Some(Self {
                params: vec![ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("chrononode", "chrononode_extract_tx_by_sender") => Some(Self {
                params: vec![ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("chrononode", "chrononode_extract_tx_by_recipient") => Some(Self {
                params: vec![ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("chrononode", "chrononode_verify_archive_range") => Some(Self {
                params: vec![ValType::I64, ValType::I64, ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("resurgence", "resurgence_check_token_age") => Some(Self {
                params: vec![ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("resurgence", "resurgence_check_token_activity_window") => Some(Self {
                params: vec![ValType::I64, ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("resurgence", "resurgence_check_liquidity_dormancy") => Some(Self {
                params: vec![ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("resurgence", "resurgence_check_governance_dormancy") => Some(Self {
                params: vec![ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("resurgence", "resurgence_calculate_dormancy_score") => Some(Self {
                params: vec![ValType::I64, ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("resurgence", "resurgence_normalize_dead_coin_risk") => Some(Self {
                params: vec![ValType::I64],
                results: vec![ValType::I64],
            }),
            ("resurgence", "resurgence_generate_dormancy_proof") => Some(Self {
                params: vec![ValType::I64, ValType::I64, ValType::I64],
                results: vec![ValType::I64],
            }),
            ("resurgence", "resurgence_emit_dormancy_oracle_result") => Some(Self {
                params: vec![ValType::I64, ValType::I64, ValType::I64, ValType::I64],
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

    #[test]
    fn test_baals_runtime_import_wasm_validates() {
        let gen = WasmGenerator::new();
        let ast = AST {
            body: vec![
                ASTNode::Call {
                    import_module: "baals".to_string(),
                    import_name: "baals_get_sender".to_string(),
                    args: vec![],
                },
                ASTNode::Call {
                    import_module: "baals".to_string(),
                    import_name: "baals_get_contract_id".to_string(),
                    args: vec![],
                },
                ASTNode::Call {
                    import_module: "baals".to_string(),
                    import_name: "baals_get_block_timestamp".to_string(),
                    args: vec![],
                },
                ASTNode::Call {
                    import_module: "baals".to_string(),
                    import_name: "baals_get_block_height".to_string(),
                    args: vec![],
                },
                ASTNode::Call {
                    import_module: "baals".to_string(),
                    import_name: "baals_emit_event".to_string(),
                    args: vec![ASTNode::I64Const(1), ASTNode::I64Const(2)],
                },
                ASTNode::Call {
                    import_module: "baals".to_string(),
                    import_name: "baals_hash_sha256".to_string(),
                    args: vec![ASTNode::I64Const(3)],
                },
                ASTNode::Call {
                    import_module: "baals".to_string(),
                    import_name: "baals_call_contract".to_string(),
                    args: vec![
                        ASTNode::I64Const(4),
                        ASTNode::I64Const(5),
                        ASTNode::I64Const(6),
                    ],
                },
                ASTNode::Call {
                    import_module: "baals".to_string(),
                    import_name: "baals_read_call_result".to_string(),
                    args: vec![ASTNode::I64Const(0), ASTNode::I64Const(7)],
                },
                ASTNode::Call {
                    import_module: "baals".to_string(),
                    import_name: "baals_transfer_value".to_string(),
                    args: vec![ASTNode::I64Const(8), ASTNode::I64Const(9)],
                },
                ASTNode::Call {
                    import_module: "baals".to_string(),
                    import_name: "baals_revert".to_string(),
                    args: vec![ASTNode::I64Const(10)],
                },
                ASTNode::Call {
                    import_module: "crypto".to_string(),
                    import_name: "crypto_verify_signature".to_string(),
                    args: vec![
                        ASTNode::I64Const(11),
                        ASTNode::I64Const(12),
                        ASTNode::I64Const(13),
                    ],
                },
                ASTNode::Call {
                    import_module: "crypto".to_string(),
                    import_name: "crypto_decode_proof".to_string(),
                    args: vec![ASTNode::I64Const(14)],
                },
                ASTNode::Call {
                    import_module: "chrononode".to_string(),
                    import_name: "chrononode_fetch_block".to_string(),
                    args: vec![ASTNode::I64Const(15), ASTNode::I64Const(16)],
                },
                ASTNode::Call {
                    import_module: "chrononode".to_string(),
                    import_name: "chrononode_fetch_checkpoint".to_string(),
                    args: vec![
                        ASTNode::I64Const(17),
                        ASTNode::I64Const(18),
                        ASTNode::I64Const(19),
                    ],
                },
                ASTNode::Call {
                    import_module: "chrononode".to_string(),
                    import_name: "chrononode_verify_proof".to_string(),
                    args: vec![ASTNode::I64Const(20), ASTNode::I64Const(21)],
                },
                ASTNode::Call {
                    import_module: "chrononode".to_string(),
                    import_name: "chrononode_extract_event".to_string(),
                    args: vec![ASTNode::I64Const(22), ASTNode::I64Const(23)],
                },
                ASTNode::Call {
                    import_module: "chrononode".to_string(),
                    import_name: "chrononode_extract_tx_by_sender".to_string(),
                    args: vec![ASTNode::I64Const(24), ASTNode::I64Const(25)],
                },
                ASTNode::Call {
                    import_module: "chrononode".to_string(),
                    import_name: "chrononode_extract_tx_by_recipient".to_string(),
                    args: vec![ASTNode::I64Const(26), ASTNode::I64Const(27)],
                },
                ASTNode::Call {
                    import_module: "chrononode".to_string(),
                    import_name: "chrononode_verify_archive_range".to_string(),
                    args: vec![
                        ASTNode::I64Const(28),
                        ASTNode::I64Const(29),
                        ASTNode::I64Const(30),
                        ASTNode::I64Const(31),
                    ],
                },
                ASTNode::Call {
                    import_module: "resurgence".to_string(),
                    import_name: "resurgence_check_token_age".to_string(),
                    args: vec![ASTNode::I64Const(32), ASTNode::I64Const(33)],
                },
                ASTNode::Call {
                    import_module: "resurgence".to_string(),
                    import_name: "resurgence_check_token_activity_window".to_string(),
                    args: vec![
                        ASTNode::I64Const(34),
                        ASTNode::I64Const(35),
                        ASTNode::I64Const(36),
                    ],
                },
                ASTNode::Call {
                    import_module: "resurgence".to_string(),
                    import_name: "resurgence_check_liquidity_dormancy".to_string(),
                    args: vec![ASTNode::I64Const(37), ASTNode::I64Const(38)],
                },
                ASTNode::Call {
                    import_module: "resurgence".to_string(),
                    import_name: "resurgence_check_governance_dormancy".to_string(),
                    args: vec![ASTNode::I64Const(39), ASTNode::I64Const(40)],
                },
                ASTNode::Call {
                    import_module: "resurgence".to_string(),
                    import_name: "resurgence_calculate_dormancy_score".to_string(),
                    args: vec![
                        ASTNode::I64Const(41),
                        ASTNode::I64Const(42),
                        ASTNode::I64Const(43),
                    ],
                },
                ASTNode::Call {
                    import_module: "resurgence".to_string(),
                    import_name: "resurgence_normalize_dead_coin_risk".to_string(),
                    args: vec![ASTNode::I64Const(44)],
                },
                ASTNode::Call {
                    import_module: "resurgence".to_string(),
                    import_name: "resurgence_generate_dormancy_proof".to_string(),
                    args: vec![
                        ASTNode::I64Const(45),
                        ASTNode::I64Const(46),
                        ASTNode::I64Const(47),
                    ],
                },
                ASTNode::Call {
                    import_module: "resurgence".to_string(),
                    import_name: "resurgence_emit_dormancy_oracle_result".to_string(),
                    args: vec![
                        ASTNode::I64Const(48),
                        ASTNode::I64Const(49),
                        ASTNode::I64Const(50),
                        ASTNode::I64Const(51),
                    ],
                },
            ],
            imports: vec![
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
                ("crypto".to_string(), "crypto_verify_signature".to_string()),
                ("crypto".to_string(), "crypto_decode_proof".to_string()),
                (
                    "chrononode".to_string(),
                    "chrononode_fetch_block".to_string(),
                ),
                (
                    "chrononode".to_string(),
                    "chrononode_fetch_checkpoint".to_string(),
                ),
                (
                    "chrononode".to_string(),
                    "chrononode_verify_proof".to_string(),
                ),
                (
                    "chrononode".to_string(),
                    "chrononode_extract_event".to_string(),
                ),
                (
                    "chrononode".to_string(),
                    "chrononode_extract_tx_by_sender".to_string(),
                ),
                (
                    "chrononode".to_string(),
                    "chrononode_extract_tx_by_recipient".to_string(),
                ),
                (
                    "chrononode".to_string(),
                    "chrononode_verify_archive_range".to_string(),
                ),
                (
                    "resurgence".to_string(),
                    "resurgence_check_token_age".to_string(),
                ),
                (
                    "resurgence".to_string(),
                    "resurgence_check_token_activity_window".to_string(),
                ),
                (
                    "resurgence".to_string(),
                    "resurgence_check_liquidity_dormancy".to_string(),
                ),
                (
                    "resurgence".to_string(),
                    "resurgence_check_governance_dormancy".to_string(),
                ),
                (
                    "resurgence".to_string(),
                    "resurgence_calculate_dormancy_score".to_string(),
                ),
                (
                    "resurgence".to_string(),
                    "resurgence_normalize_dead_coin_risk".to_string(),
                ),
                (
                    "resurgence".to_string(),
                    "resurgence_generate_dormancy_proof".to_string(),
                ),
                (
                    "resurgence".to_string(),
                    "resurgence_emit_dormancy_oracle_result".to_string(),
                ),
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
