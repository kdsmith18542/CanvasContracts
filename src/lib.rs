//! Canvas Contracts - Visual Smart Contract Development Platform
//!
//! This library provides the core functionality for building, compiling, and executing
//! visual smart contracts using WebAssembly.

// ── Core modules ─────────────────────────────────────────────────────
pub mod adapter;
pub mod artifact;
pub mod baals;
pub mod compiler;
pub mod config;
pub mod debugger;
pub mod error;
pub mod nodes;
pub mod types;
pub mod wasm;

// ── Ungated modules ──────────────────────────────────────────────────
pub mod ai;
pub mod community;
pub mod deployment;
pub mod marketplace;
pub mod monitoring;
pub mod optimization;
pub mod sdk;

// NOTE: Removed the separate src/validator.rs — consolidated on compiler::Validator.

pub use error::{CanvasError, CanvasResult};
pub use serde::{Deserialize, Serialize};
pub use types::*;

pub use adapter::{
    create_chrononode_client, BaaLSAdapter, ChronoNodeClient, HttpChronoNodeClient, LedgerAdapter,
    MockChronoNodeClient, RuntimeInfo,
};
pub use artifact::{build_artifact_bundle, verify_artifact_manifest};
pub use baals::{create_client, sign_payload, BaalsClient, MockBaalsClient};
/// Re-export commonly used types
pub use compiler::Compiler;
pub use debugger::{DebugConfig, DebugSession, DebuggerUtils};
pub use nodes::{Node, NodeContext, NodeDefinition, NodeRegistry};
pub use wasm::{WasmModule, WasmRuntime};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Initialize the Canvas Contracts library
pub fn init() -> CanvasResult<()> {
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
