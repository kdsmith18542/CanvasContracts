//! Contract compilation pipeline

mod graph_ir;
mod ast;
mod wasm_gen;
mod validator;

use crate::{
    config::Config,
    error::{CanvasError, CanvasResult},
    types::{CompilationResult, VisualGraph},
};

pub use validator::Validator;

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
        // TODO: Implement full compilation pipeline
        // 1. Convert visual graph to Graph IR
        // 2. Generate AST from Graph IR
        // 3. Generate WASM from AST
        // 4. Generate ABI
        
        // For now, return a stub implementation
        Err(CanvasError::Compilation("Compilation pipeline not yet implemented".to_string()))
    }

    /// Validate a visual graph
    pub fn validate(&self, graph: &VisualGraph) -> CanvasResult<ValidationResult> {
        let validator = Validator::new(&self.config)?;
        validator.validate(graph)
    }
}

/// Validation result
#[derive(Debug, Clone)]
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