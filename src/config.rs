//! Configuration management for Canvas Contracts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{CanvasError, CanvasResult};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application settings
    pub app: AppConfig,
    /// Compiler settings
    pub compiler: CompilerConfig,
    /// Runtime settings
    pub runtime: RuntimeConfig,
    /// BaaLS integration settings
    pub baals: BaalsConfig,
    /// Development settings
    pub development: DevelopmentConfig,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application name
    pub name: String,
    /// Application version
    pub version: String,
    /// Data directory
    pub data_dir: PathBuf,
    /// Log level
    pub log_level: String,
    /// Enable debug mode
    pub debug: bool,
}

/// Compiler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    /// Optimization level (0-3)
    pub optimization_level: u8,
    /// Enable debug information
    pub debug_info: bool,
    /// Gas estimation enabled
    pub gas_estimation: bool,
    /// Maximum gas limit
    pub max_gas_limit: u64,
    /// WASM target
    pub wasm_target: String,
    /// Custom compiler flags
    pub flags: Vec<String>,
}

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// WASM runtime type
    pub runtime_type: String,
    /// Memory limit (in pages)
    pub memory_limit: u32,
    /// Stack size limit
    pub stack_size_limit: u32,
    /// Gas metering enabled
    pub gas_metering: bool,
    /// Sandbox mode enabled
    pub sandbox_mode: bool,
    /// Timeout (in seconds)
    pub timeout: u64,
}

/// BaaLS integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaalsConfig {
    /// BaaLS node URL
    pub node_url: String,
    /// Connection timeout
    pub connection_timeout: u64,
    /// Retry attempts
    pub retry_attempts: u32,
    /// Enable local node
    pub enable_local_node: bool,
    /// Local node port
    pub local_node_port: u16,
    /// Authentication token
    pub auth_token: Option<String>,
}

/// Development configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentConfig {
    /// Enable hot reload
    pub hot_reload: bool,
    /// Enable verbose logging
    pub verbose_logging: bool,
    /// Enable performance profiling
    pub profiling: bool,
    /// Test mode enabled
    pub test_mode: bool,
    /// Mock BaaLS enabled
    pub mock_baals: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            compiler: CompilerConfig::default(),
            runtime: RuntimeConfig::default(),
            baals: BaalsConfig::default(),
            development: DevelopmentConfig::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "Canvas Contracts".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            data_dir: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("./data"))
                .join("canvas-contracts"),
            log_level: "info".to_string(),
            debug: false,
        }
    }
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            optimization_level: 2,
            debug_info: false,
            gas_estimation: true,
            max_gas_limit: 10_000_000,
            wasm_target: "wasm32-unknown-unknown".to_string(),
            flags: Vec::new(),
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            runtime_type: "wasmtime".to_string(),
            memory_limit: 1000, // 64MB
            stack_size_limit: 1024 * 1024, // 1MB
            gas_metering: true,
            sandbox_mode: true,
            timeout: 30,
        }
    }
}

impl Default for BaalsConfig {
    fn default() -> Self {
        Self {
            node_url: "http://localhost:8080".to_string(),
            connection_timeout: 30,
            retry_attempts: 3,
            enable_local_node: true,
            local_node_port: 8080,
            auth_token: None,
        }
    }
}

impl Default for DevelopmentConfig {
    fn default() -> Self {
        Self {
            hot_reload: false,
            verbose_logging: false,
            profiling: false,
            test_mode: false,
            mock_baals: false,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn from_file(path: &PathBuf) -> CanvasResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| CanvasError::Config(format!("Failed to read config file: {}", e)))?;
        
        let config: Config = toml::from_str(&content)
            .map_err(|e| CanvasError::Config(format!("Failed to parse config file: {}", e)))?;
        
        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &PathBuf) -> CanvasResult<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| CanvasError::Config(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(path, content)
            .map_err(|e| CanvasError::Config(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }

    /// Load configuration from environment variables
    pub fn from_env() -> CanvasResult<Self> {
        let mut config = Config::default();
        
        // Override with environment variables
        if let Ok(log_level) = std::env::var("CANVAS_LOG_LEVEL") {
            config.app.log_level = log_level;
        }
        
        if let Ok(debug) = std::env::var("CANVAS_DEBUG") {
            config.app.debug = debug.parse().unwrap_or(false);
        }
        
        if let Ok(node_url) = std::env::var("CANVAS_BAALS_NODE_URL") {
            config.baals.node_url = node_url;
        }
        
        if let Ok(auth_token) = std::env::var("CANVAS_BAALS_AUTH_TOKEN") {
            config.baals.auth_token = Some(auth_token);
        }
        
        if let Ok(optimization) = std::env::var("CANVAS_COMPILER_OPTIMIZATION") {
            if let Ok(level) = optimization.parse() {
                config.compiler.optimization_level = level;
            }
        }
        
        if let Ok(gas_limit) = std::env::var("CANVAS_COMPILER_MAX_GAS") {
            if let Ok(limit) = gas_limit.parse() {
                config.compiler.max_gas_limit = limit;
            }
        }
        
        Ok(config)
    }

    /// Get configuration value by key path
    pub fn get_value(&self, key_path: &str) -> Option<serde_json::Value> {
        let keys: Vec<&str> = key_path.split('.').collect();
        
        match keys.as_slice() {
            ["app", key] => match *key {
                "name" => Some(serde_json::Value::String(self.app.name.clone())),
                "version" => Some(serde_json::Value::String(self.app.version.clone())),
                "log_level" => Some(serde_json::Value::String(self.app.log_level.clone())),
                "debug" => Some(serde_json::Value::Bool(self.app.debug)),
                _ => None,
            },
            ["compiler", key] => match *key {
                "optimization_level" => Some(serde_json::Value::Number(self.compiler.optimization_level.into())),
                "debug_info" => Some(serde_json::Value::Bool(self.compiler.debug_info)),
                "gas_estimation" => Some(serde_json::Value::Bool(self.compiler.gas_estimation)),
                "max_gas_limit" => Some(serde_json::Value::Number(self.compiler.max_gas_limit.into())),
                _ => None,
            },
            ["runtime", key] => match *key {
                "runtime_type" => Some(serde_json::Value::String(self.runtime.runtime_type.clone())),
                "memory_limit" => Some(serde_json::Value::Number(self.runtime.memory_limit.into())),
                "gas_metering" => Some(serde_json::Value::Bool(self.runtime.gas_metering)),
                "sandbox_mode" => Some(serde_json::Value::Bool(self.runtime.sandbox_mode)),
                _ => None,
            },
            ["baals", key] => match *key {
                "node_url" => Some(serde_json::Value::String(self.baals.node_url.clone())),
                "connection_timeout" => Some(serde_json::Value::Number(self.baals.connection_timeout.into())),
                "enable_local_node" => Some(serde_json::Value::Bool(self.baals.enable_local_node)),
                _ => None,
            },
            _ => None,
        }
    }

    /// Set configuration value by key path
    pub fn set_value(&mut self, key_path: &str, value: serde_json::Value) -> CanvasResult<()> {
        let keys: Vec<&str> = key_path.split('.').collect();
        
        match keys.as_slice() {
            ["app", key] => match *key {
                "name" => {
                    if let Some(name) = value.as_str() {
                        self.app.name = name.to_string();
                    }
                }
                "log_level" => {
                    if let Some(level) = value.as_str() {
                        self.app.log_level = level.to_string();
                    }
                }
                "debug" => {
                    if let Some(debug) = value.as_bool() {
                        self.app.debug = debug;
                    }
                }
                _ => return Err(CanvasError::Config(format!("Unknown app config key: {}", key))),
            },
            ["compiler", key] => match *key {
                "optimization_level" => {
                    if let Some(level) = value.as_u64() {
                        self.compiler.optimization_level = level as u8;
                    }
                }
                "max_gas_limit" => {
                    if let Some(limit) = value.as_u64() {
                        self.compiler.max_gas_limit = limit;
                    }
                }
                _ => return Err(CanvasError::Config(format!("Unknown compiler config key: {}", key))),
            },
            _ => return Err(CanvasError::Config(format!("Unknown config key path: {}", key_path))),
        }
        
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> CanvasResult<()> {
        // Validate app config
        if self.app.name.is_empty() {
            return Err(CanvasError::Config("App name cannot be empty".to_string()));
        }
        
        if self.app.version.is_empty() {
            return Err(CanvasError::Config("App version cannot be empty".to_string()));
        }
        
        // Validate compiler config
        if self.compiler.optimization_level > 3 {
            return Err(CanvasError::Config("Optimization level must be 0-3".to_string()));
        }
        
        if self.compiler.max_gas_limit == 0 {
            return Err(CanvasError::Config("Max gas limit must be greater than 0".to_string()));
        }
        
        // Validate runtime config
        if self.runtime.memory_limit == 0 {
            return Err(CanvasError::Config("Memory limit must be greater than 0".to_string()));
        }
        
        if self.runtime.timeout == 0 {
            return Err(CanvasError::Config("Timeout must be greater than 0".to_string()));
        }
        
        // Validate BaaLS config
        if self.baals.connection_timeout == 0 {
            return Err(CanvasError::Config("Connection timeout must be greater than 0".to_string()));
        }
        
        if self.baals.retry_attempts == 0 {
            return Err(CanvasError::Config("Retry attempts must be greater than 0".to_string()));
        }
        
        Ok(())
    }
}

/// Configuration manager
pub struct ConfigManager {
    config: Config,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config_path: PathBuf) -> CanvasResult<Self> {
        let config = if config_path.exists() {
            Config::from_file(&config_path)?
        } else {
            let config = Config::from_env()?;
            config.save_to_file(&config_path)?;
            config
        };
        
        config.validate()?;
        
        Ok(Self {
            config,
            config_path,
        })
    }

    /// Get the current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get mutable reference to configuration
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Reload configuration from file
    pub fn reload(&mut self) -> CanvasResult<()> {
        self.config = Config::from_file(&self.config_path)?;
        self.config.validate()?;
        Ok(())
    }

    /// Save current configuration
    pub fn save(&self) -> CanvasResult<()> {
        self.config.save_to_file(&self.config_path)
    }

    /// Get configuration value
    pub fn get_value(&self, key_path: &str) -> Option<serde_json::Value> {
        self.config.get_value(key_path)
    }

    /// Set configuration value
    pub fn set_value(&mut self, key_path: &str, value: serde_json::Value) -> CanvasResult<()> {
        self.config.set_value(key_path, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.app.name, "Canvas Contracts");
        assert_eq!(config.compiler.optimization_level, 2);
        assert!(config.runtime.gas_metering);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());
        
        config.app.name = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_file_io() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = Config::default();
        
        assert!(config.save_to_file(&temp_file.path().to_path_buf()).is_ok());
        
        let loaded_config = Config::from_file(&temp_file.path().to_path_buf()).unwrap();
        assert_eq!(loaded_config.app.name, config.app.name);
    }

    #[test]
    fn test_config_value_access() {
        let config = Config::default();
        
        assert_eq!(
            config.get_value("app.name"),
            Some(serde_json::Value::String("Canvas Contracts".to_string()))
        );
        
        assert_eq!(
            config.get_value("compiler.optimization_level"),
            Some(serde_json::Value::Number(2.into()))
        );
    }
} 