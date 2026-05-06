//! Canvas Contracts - Visual Smart Contract Development Platform
//! 
//! This library provides the core functionality for building, compiling, and executing
//! visual smart contracts using WebAssembly.

// ── Core modules (Phase 1) ──────────────────────────────────────────
pub mod compiler;
pub mod nodes;
pub mod wasm;
pub mod baals;
pub mod error;
pub mod types;
pub mod config;

// ── Phase 3+ modules ────────────────────────────────────────────────
// These are gated behind feature flags until their prerequisites are
// implemented. They currently reference types and modules that don't
// exist yet, causing ~100 compilation errors.
#[cfg(feature = "ai")]
pub mod ai;
#[cfg(feature = "debugger")]
pub mod debugger;
#[cfg(feature = "marketplace")]
pub mod marketplace;
#[cfg(feature = "sdk")]
pub mod sdk;
#[cfg(feature = "community")]
pub mod community;
#[cfg(feature = "deployment")]
pub mod deployment;
#[cfg(feature = "monitoring")]
pub mod monitoring;
#[cfg(feature = "optimization")]
pub mod optimization;

// NOTE: src/validator.rs is a duplicate of compiler::Validator with a
// divergent ValidationResult type. Removed from the public API to
// consolidate on compiler::Validator. The file is preserved on disk
// for reference.
// pub mod validator;

pub use error::{CanvasError, CanvasResult};
pub use types::*;
pub use serde::{Deserialize, Serialize};

/// Re-export commonly used types
pub use compiler::Compiler;
pub use nodes::{Node, NodeContext, NodeDefinition};
pub use wasm::WasmRuntime;
pub use baals::BaalsClient;

// Re-exports for feature-gated modules
#[cfg(feature = "ai")]
pub use ai::AiAssistant;
#[cfg(feature = "debugger")]
pub use debugger::{DebugSession, DebuggerUtils, DebugConfig};
#[cfg(feature = "monitoring")]
pub use monitoring::{MetricsCollector, HealthChecker, CircuitBreaker};
#[cfg(feature = "optimization")]
pub use optimization::{PerformanceOptimizer, ResourceUsageAnalyzer};
#[cfg(feature = "deployment")]
pub use deployment::{DeploymentManager, BlueGreenDeploymentManager, CanaryDeploymentManager};

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