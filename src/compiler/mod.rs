//! Contract compilation pipeline

mod graph_ir;
mod ast;
mod wasm_gen;
mod validator;
mod executor;

use crate::{
    config::Config,
    error::{CanvasError, CanvasResult},
    types::{CompilationResult, VisualGraph, ContractABI, FunctionABI, ParameterABI, StateMutability, ValueType},
};

pub use validator::Validator;
pub use graph_ir::GraphIR;
pub use ast::AST;
pub use wasm_gen::WasmGenerator;
pub use executor::GraphExecutor;

/// Main compiler for converting visual graphs to WASM
pub struct Compiler {
    config: Config,
}

impl Compiler {
    /// Create a new compiler instance
    pub fn new(config: &Config) -> CanvasResult<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    /// Compile a visual graph to WASM
    pub fn compile(&self, graph: &VisualGraph) -> CanvasResult<CompilationResult> {
        // Step 1: Convert visual graph to Graph IR
        let graph_ir = GraphIR::from_visual_graph(graph);

        // Step 2: Generate AST from Graph IR
        let ast = AST::from_graph_ir(&graph_ir);

        // Step 3: Generate WASM from AST
        let wasm_gen = WasmGenerator::new();
        let wasm_result = wasm_gen.generate(&ast)
            .map_err(CanvasError::Compilation)?;

        // Step 3.5: Validate generated WASM with wasmtime
        let engine = wasmtime::Engine::default();
        wasmtime::Module::validate(&engine, &wasm_result.wasm_bytes)
            .map_err(|e| CanvasError::Compilation(format!("WASM validation failed: {}", e)))?;

        // Step 4: Generate ABI
        let abi = self.generate_abi(graph, &wasm_result);

        // Calculate gas estimate
        let gas_estimate = self.estimate_gas(graph, &wasm_result.wasm_bytes);

        // Collect warnings
        let mut warnings = Vec::new();
        if wasm_result.wasm_bytes.len() > 1_000_000 {
            warnings.push("WASM module size exceeds 1MB".to_string());
        }
        if graph.nodes.len() > 100 {
            warnings.push("Large graph detected, consider splitting into modules".to_string());
        }

        Ok(CompilationResult {
            wasm_bytes: wasm_result.wasm_bytes,
            abi,
            gas_estimate,
            warnings,
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Generate ABI from visual graph
    fn generate_abi(&self, graph: &VisualGraph, wasm_result: &wasm_gen::WasmGenResult) -> ContractABI {
        let mut functions = Vec::new();

        // Generate function ABI from graph nodes
        for node in &graph.nodes {
            if node.node_type == "Start" || node.node_type == "End" {
                continue;
            }

            let mut inputs = Vec::new();
            for input in &node.inputs {
                inputs.push(ParameterABI {
                    name: input.name.clone(),
                    value_type: input.value_type.clone(),
                    indexed: false,
                });
            }

            let mut outputs = Vec::new();
            for output in &node.outputs {
                outputs.push(ParameterABI {
                    name: output.name.clone(),
                    value_type: output.value_type.clone(),
                    indexed: false,
                });
            }

            functions.push(FunctionABI {
                name: node.node_type.clone(),
                inputs,
                outputs,
                state_mutability: StateMutability::NonPayable,
                gas_estimate: None,
            });
        }

        // Add main function
        functions.push(FunctionABI {
            name: "main".to_string(),
            inputs: Vec::new(),
            outputs: vec![ParameterABI {
                name: "result".to_string(),
                value_type: ValueType::Any,
                indexed: false,
            }],
            state_mutability: StateMutability::NonPayable,
            gas_estimate: Some(self.estimate_gas(graph, &wasm_result.wasm_bytes)),
        });

        ContractABI {
            functions,
            events: Vec::new(),
            errors: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Estimate gas usage
    fn estimate_gas(&self, graph: &VisualGraph, wasm_bytes: &[u8]) -> u64 {
        let base_gas = 10_000;
        let node_gas = graph.nodes.len() as u64 * 100;
        let edge_gas = graph.connections.len() as u64 * 50;
        let wasm_size_gas = (wasm_bytes.len() as u64 / 100) * 10;

        base_gas + node_gas + edge_gas + wasm_size_gas
    }

    /// Create a validator for graph validation
    pub fn validator(&self) -> CanvasResult<Validator> {
        Validator::new(&self.config)
    }

    /// Validate a visual graph
    pub fn validate(&self, graph: &VisualGraph) -> CanvasResult<ValidationResult> {
        self.validator()?.validate(graph)
    }
}

/// Validation result
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self.is_valid = false;
        self
    }
}
