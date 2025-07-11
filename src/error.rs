//! Error handling for Canvas Contracts

use thiserror::Error;

/// Result type for Canvas Contracts operations
pub type CanvasResult<T> = Result<T, CanvasError>;

/// Main error type for Canvas Contracts
#[derive(Error, Debug)]
pub enum CanvasError {
    #[error("Compilation error: {0}")]
    Compilation(String),

    #[error("WASM error: {0}")]
    Wasm(String),

    #[error("Node error: {0}")]
    Node(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Breakpoint not found: {0}")]
    BreakpointNotFound(String),

    #[error("BaaLS error: {0}")]
    Baals(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Graph error: {0}")]
    Graph(String),

    #[error("Type error: {0}")]
    Type(String),

    #[error("Gas limit exceeded: {0}")]
    GasLimitExceeded(u64),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl CanvasError {
    /// Create a compilation error
    pub fn compilation(msg: impl Into<String>) -> Self {
        Self::Compilation(msg.into())
    }

    /// Create a WASM error
    pub fn wasm(msg: impl Into<String>) -> Self {
        Self::Wasm(msg.into())
    }

    /// Create a node error
    pub fn node(msg: impl Into<String>) -> Self {
        Self::Node(msg.into())
    }

    /// Create a BaaLS error
    pub fn baals(msg: impl Into<String>) -> Self {
        Self::Baals(msg.into())
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a graph error
    pub fn graph(msg: impl Into<String>) -> Self {
        Self::Graph(msg.into())
    }

    /// Create a type error
    pub fn type_error(msg: impl Into<String>) -> Self {
        Self::Type(msg.into())
    }

    /// Check if this is a fatal error
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::Compilation(_) | Self::Wasm(_) | Self::Config(_) | Self::Io(_)
        )
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Validation(_) | Self::Type(_) | Self::GasLimitExceeded(_)
        )
    }
}

/// Error context for better debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub details: String,
    pub location: Option<String>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            details: String::new(),
            location: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = details.into();
        self
    }

    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}

/// Extension trait for adding context to errors
pub trait ErrorContextExt {
    fn with_context(self, context: ErrorContext) -> CanvasError;
}

impl<T> ErrorContextExt for Result<T, CanvasError> {
    fn with_context(self, context: ErrorContext) -> CanvasError {
        match self {
            Ok(_) => CanvasError::Unknown("Unexpected success in error context".to_string()),
            Err(e) => match e {
                CanvasError::Compilation(msg) => {
                    CanvasError::Compilation(format!("{}: {}", context.operation, msg))
                }
                CanvasError::Wasm(msg) => {
                    CanvasError::Wasm(format!("{}: {}", context.operation, msg))
                }
                CanvasError::Node(msg) => {
                    CanvasError::Node(format!("{}: {}", context.operation, msg))
                }
                CanvasError::Baals(msg) => {
                    CanvasError::Baals(format!("{}: {}", context.operation, msg))
                }
                CanvasError::Validation(msg) => {
                    CanvasError::Validation(format!("{}: {}", context.operation, msg))
                }
                CanvasError::Config(msg) => {
                    CanvasError::Config(format!("{}: {}", context.operation, msg))
                }
                CanvasError::Graph(msg) => {
                    CanvasError::Graph(format!("{}: {}", context.operation, msg))
                }
                CanvasError::Type(msg) => {
                    CanvasError::Type(format!("{}: {}", context.operation, msg))
                }
                CanvasError::GasLimitExceeded(limit) => {
                    CanvasError::GasLimitExceeded(limit)
                }
                CanvasError::PermissionDenied(msg) => {
                    CanvasError::PermissionDenied(format!("{}: {}", context.operation, msg))
                }
                CanvasError::NotFound(msg) => {
                    CanvasError::NotFound(format!("{}: {}", context.operation, msg))
                }
                CanvasError::InvalidState(msg) => {
                    CanvasError::InvalidState(format!("{}: {}", context.operation, msg))
                }
                CanvasError::Timeout(msg) => {
                    CanvasError::Timeout(format!("{}: {}", context.operation, msg))
                }
                CanvasError::Network(msg) => {
                    CanvasError::Network(format!("{}: {}", context.operation, msg))
                }
                CanvasError::Unknown(msg) => {
                    CanvasError::Unknown(format!("{}: {}", context.operation, msg))
                }
                CanvasError::Io(e) => CanvasError::Io(e),
                CanvasError::Serialization(e) => CanvasError::Serialization(e),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let comp_error = CanvasError::compilation("test compilation error");
        assert!(matches!(comp_error, CanvasError::Compilation(_)));

        let wasm_error = CanvasError::wasm("test WASM error");
        assert!(matches!(wasm_error, CanvasError::Wasm(_)));
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test operation")
            .with_details("test details")
            .with_location("test location");

        assert_eq!(context.operation, "test operation");
        assert_eq!(context.details, "test details");
        assert_eq!(context.location, Some("test location".to_string()));
    }

    #[test]
    fn test_fatal_error_detection() {
        let fatal_error = CanvasError::compilation("fatal");
        assert!(fatal_error.is_fatal());

        let recoverable_error = CanvasError::validation("recoverable");
        assert!(recoverable_error.is_recoverable());
    }
} 