//! Canvas Contracts - Visual Smart Contract Development Platform
//! 
//! This library provides the core functionality for building, compiling, and executing
//! visual smart contracts using WebAssembly.

pub mod compiler;
pub mod nodes;
pub mod validator;
pub mod wasm;
pub mod baals;
pub mod ai;
pub mod debugger;
pub mod marketplace;
pub mod sdk;
pub mod community;
pub mod error;
pub mod types;
pub mod config;

pub use error::{CanvasError, CanvasResult};
pub use types::*;
pub use serde::{Deserialize, Serialize};

/// Re-export commonly used types
pub use compiler::Compiler;
pub use nodes::{Node, NodeContext, NodeDefinition};
pub use wasm::WasmRuntime;
pub use baals::BaalsClient;
pub use ai::AiAssistant;
pub use debugger::{DebugSession, DebuggerUtils, DebugConfig};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Initialize the Canvas Contracts library
pub fn init() -> CanvasResult<()> {
    env_logger::init();
    log::info!("Initializing Canvas Contracts v{}", VERSION);
    Ok(())
}

/// Get library information
pub fn info() -> LibraryInfo {
    LibraryInfo {
        name: NAME.to_string(),
        version: VERSION.to_string(),
        description: "Visual smart contract development platform".to_string(),
    }
}

/// Library information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }

    #[test]
    fn test_info() {
        let info = info();
        assert_eq!(info.name, "canvas-contracts");
        assert!(!info.version.is_empty());
        assert!(!info.description.is_empty());
    }
} 