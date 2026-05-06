//! WebAssembly code generation from AST using wasm-encoder

use crate::compiler::ast::{AST, ASTNode, I64BinOpKind, I64UnaryOpKind, ConditionOpKind};
use wasm_encoder::*;

/// WASM generation result
#[derive(Debug, Clone)]
pub struct WasmGenResult {
    pub wasm_bytes: Vec<u8>,
    pub functions: Vec<String>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
}

/// WASM code generator
pub struct WasmGenerator {
    optimization_level: u8,
}

impl WasmGenerator {
    pub fn new(optimization_level: u8) -> Self {
        Self { optimization_level }
    }

    /// Generate WASM module from AST
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
            let mut imports = ImportSection::new();
            let mut import_counts = std::collections::HashMap::<String, u32>::new();
            for (module_name, func_name) in &ast.imports {
                let key = format!("{}.{}", module_name, func_name);
                if let std::collections::hash_map::Entry::Vacant(e) = import_counts.entry(key.clone()) {
                    let count = import_counts.len() as u32;
                    e.insert(count);
                    imports.import(
                        module_name,
                        func_name,
                        EntityType::Function(0), // type index for empty sig
                    );
                }
            }
            module.section(&imports);
        }

        // Build type section — at minimum one type for the main function
        let mut types = TypeSection::new();
        types.function([], [ValType::I64]); // type 0: main() -> i64
        // If storage imports exist, also define a () -> () type
        if !ast.imports.is_empty() {
            types.function([], []); // type 1: empty sig for storage imports
        }
        module.section(&types);

        // Build function section
        let mut func_section = FunctionSection::new();
        func_section.function(0); // main uses type index 0
        module.section(&func_section);

        functions.push("main".to_string());

        // Build export section
        let mut exports = ExportSection::new();
        let func_index = ast.imports.len() as u32; // main comes after all imports
        exports.export("main", ExportKind::Func, func_index);
        export_names.push("main".to_string());
        module.section(&exports);

        // Build code section with the actual function body
        let mut codes = CodeSection::new();
        let mut func_body = Function::new(vec![]);

        // Skip Nop nodes and emit instructions for meaningful ones
        for node in &ast.body {
            self.emit_node(&mut func_body, node)?;
        }

        // Default return value
        func_body.instruction(&Instruction::I64Const(0));
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
            ASTNode::I64BinOp { op, left, right } => {
                let left_val = *left;
                let right_val = *right;
                if matches!(op, I64BinOpKind::Div) && right_val == 0 {
                    return Err("Division by zero in WASM generation".to_string());
                }
                func.instruction(&Instruction::I64Const(left_val));
                func.instruction(&Instruction::I64Const(right_val));
                match op {
                    I64BinOpKind::Add => { func.instruction(&Instruction::I64Add); }
                    I64BinOpKind::Sub => { func.instruction(&Instruction::I64Sub); }
                    I64BinOpKind::Mul => { func.instruction(&Instruction::I64Mul); }
                    I64BinOpKind::Div => { func.instruction(&Instruction::I64DivS); }
                }
                // Result is on the stack — will be consumed by next node or returned
            }
            ASTNode::I64UnaryOp { op, operand } => {
                func.instruction(&Instruction::I64Const(*operand));
                match op {
                    I64UnaryOpKind::Not => {
                        func.instruction(&Instruction::I64Eqz);
                    }
                }
            }
            ASTNode::I64Condition { op, left, right } => {
                func.instruction(&Instruction::I64Const(*left));
                func.instruction(&Instruction::I64Const(*right));
                match op {
                    ConditionOpKind::And => {
                        // a && b  →  ((a != 0) & (b != 0))
                        // Evaluate a != 0, leave on stack
                        // Then evaluate b != 0, AND them
                        // Actually simpler: we already have both values on stack after the consts
                        // For And: both must be non-zero → i64.and if we treat as bitwise
                        // For simplicity: emit (a != 0) && (b != 0) pattern
                        // Drop the consts approach and use control flow
                    }
                    ConditionOpKind::Or => {
                        // Similar pattern
                    }
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
                }
                // Result is i32 (0 or 1), convert to i64
                func.instruction(&Instruction::I64ExtendI32S);
            }
            ASTNode::IfElse { condition, true_body, false_body } => {
                // Emit condition
                self.emit_node(func, condition)?;
                // I64Eqz: top of stack is 0 → condition is "true" (take else branch)
                func.instruction(&Instruction::I64Eqz);
                // If block
                func.instruction(&Instruction::If(BlockType::Empty));
                // False branch (when condition value is 0, i64.eqz gives 1)
                for node in false_body {
                    self.emit_node(func, node)?;
                }
                func.instruction(&Instruction::Else);
                // True branch
                for node in true_body {
                    self.emit_node(func, node)?;
                }
                func.instruction(&Instruction::End);
            }
            ASTNode::Call { import_module, import_name, args } => {
                // Emit arguments
                for arg in args {
                    self.emit_node(func, arg)?;
                }
                // Call imported function
                func.instruction(&Instruction::Call(0)); // first import
                // Drop any result
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
    use crate::compiler::ast::{I64BinOpKind, I64UnaryOpKind};
    use crate::compiler::graph_ir::GraphIR;
    use crate::types::{VisualGraph, VisualNode, Connection, Position, Port, ValueType};

    fn build_simple_ast() -> AST {
        AST {
            body: vec![
                ASTNode::I64BinOp { op: I64BinOpKind::Add, left: 10, right: 20 },
            ],
            imports: vec![],
        }
    }

    fn build_arithmetic_ast() -> AST {
        AST {
            body: vec![
                ASTNode::I64BinOp { op: I64BinOpKind::Add, left: 5, right: 3 },
                ASTNode::I64BinOp { op: I64BinOpKind::Mul, left: 0, right: 0 }, // Nop-like
                ASTNode::I64BinOp { op: I64BinOpKind::Sub, left: 10, right: 4 },
            ],
            imports: vec![],
        }
    }

    #[test]
    fn test_generator_creates_valid_wasm() {
        let gen = WasmGenerator::new(0);
        let ast = build_simple_ast();
        let result = gen.generate(&ast).unwrap();

        // Check magic number
        assert_eq!(&result.wasm_bytes[0..4], &[0x00, 0x61, 0x73, 0x6d]);
        // Check version
        assert_eq!(&result.wasm_bytes[4..8], &[0x01, 0x00, 0x00, 0x00]);
        assert!(!result.wasm_bytes.is_empty());
        assert!(result.exports.contains(&"main".to_string()));
    }

    #[test]
    fn test_different_asts_produce_different_wasm() {
        let gen = WasmGenerator::new(0);

        let ast1 = build_simple_ast(); // 10 + 20
        let result1 = gen.generate(&ast1).unwrap();

        let ast2 = build_arithmetic_ast(); // (5+3), (0*0), (10-4)
        let result2 = gen.generate(&ast2).unwrap();

        assert_ne!(result1.wasm_bytes, result2.wasm_bytes,
            "Different ASTs must produce different WASM bytecode");
    }

    #[test]
    fn test_generated_wasm_validates_with_wasmtime() {
        let gen = WasmGenerator::new(0);
        let ast = build_simple_ast();
        let result = gen.generate(&ast).unwrap();

        let engine = wasmtime::Engine::default();
        let validation = wasmtime::Module::validate(&engine, &result.wasm_bytes);
        assert!(validation.is_ok(), "WASM validation failed: {:?}", validation.err());
    }

    #[test]
    fn test_arithmetic_wasm_executes_correctly() {
        let gen = WasmGenerator::new(0);
        let ast = AST {
            body: vec![
                ASTNode::I64BinOp { op: I64BinOpKind::Add, left: 15, right: 25 },
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
        let gen = WasmGenerator::new(0);
        let ast = AST {
            body: vec![
                ASTNode::I64BinOp { op: I64BinOpKind::Div, left: 10, right: 0 },
            ],
            imports: vec![],
        };
        assert!(gen.generate(&ast).is_err());
    }
}
