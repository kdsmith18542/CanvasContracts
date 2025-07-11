//! Developer SDK for Canvas Contracts

use crate::{
    error::{CanvasError, CanvasResult},
    types::{Graph, Node, NodeId, NodeType},
    nodes::custom::{CustomNodeDefinition, CustomNodeBuilder},
    compiler::Compiler,
    wasm::WasmRuntime,
    config::Config,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SDK configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkConfig {
    pub api_version: String,
    pub features: Vec<String>,
    pub debug_mode: bool,
    pub log_level: String,
    pub cache_enabled: bool,
    pub max_cache_size: usize,
}

/// Plugin interface for extending Canvas Contracts
pub trait CanvasPlugin {
    /// Plugin name
    fn name(&self) -> &str;
    
    /// Plugin version
    fn version(&self) -> &str;
    
    /// Plugin description
    fn description(&self) -> &str;
    
    /// Initialize the plugin
    fn initialize(&mut self, config: &SdkConfig) -> CanvasResult<()>;
    
    /// Cleanup the plugin
    fn cleanup(&mut self) -> CanvasResult<()>;
    
    /// Get plugin capabilities
    fn capabilities(&self) -> Vec<PluginCapability>;
}

/// Plugin capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginCapability {
    CustomNodes,
    Templates,
    Validators,
    Optimizers,
    Exporters,
    Importers,
    Visualizers,
}

/// Plugin registry
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn CanvasPlugin>>,
    config: SdkConfig,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new(config: SdkConfig) -> Self {
        Self {
            plugins: HashMap::new(),
            config,
        }
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn CanvasPlugin>) -> CanvasResult<()> {
        let name = plugin.name().to_string();
        
        if self.plugins.contains_key(&name) {
            return Err(CanvasError::Validation(format!("Plugin '{}' already registered", name)));
        }

        // Initialize the plugin
        let mut plugin_mut = plugin;
        plugin_mut.initialize(&self.config)?;
        
        self.plugins.insert(name, plugin_mut);
        Ok(())
    }

    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&Box<dyn CanvasPlugin>> {
        self.plugins.get(name)
    }

    /// Get all plugins
    pub fn get_all_plugins(&self) -> Vec<&Box<dyn CanvasPlugin>> {
        self.plugins.values().collect()
    }

    /// Get plugins by capability
    pub fn get_plugins_by_capability(&self, capability: &PluginCapability) -> Vec<&Box<dyn CanvasPlugin>> {
        self.plugins
            .values()
            .filter(|plugin| {
                plugin.capabilities().contains(capability)
            })
            .collect()
    }

    /// Unregister a plugin
    pub fn unregister_plugin(&mut self, name: &str) -> CanvasResult<()> {
        if let Some(mut plugin) = self.plugins.remove(name) {
            plugin.cleanup()?;
            Ok(())
        } else {
            Err(CanvasError::NotFound(format!("Plugin '{}' not found", name)))
        }
    }
}

/// Graph builder for programmatic graph creation
pub struct GraphBuilder {
    graph: Graph,
    node_counter: u32,
}

impl GraphBuilder {
    /// Create a new graph builder
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_counter: 0,
        }
    }

    /// Add a node to the graph
    pub fn add_node(mut self, node_type: NodeType, position: (f64, f64)) -> Self {
        let node_id = format!("node_{}", self.node_counter);
        self.node_counter += 1;
        
        let node = Node {
            id: node_id,
            node_type,
            position,
            properties: HashMap::new(),
        };
        
        self.graph.add_node(node);
        self
    }

    /// Add a connection between nodes
    pub fn connect(mut self, from: &str, to: &str) -> Self {
        self.graph.add_edge(from.to_string(), to.to_string());
        self
    }

    /// Set node properties
    pub fn set_node_properties(mut self, node_id: &str, properties: HashMap<String, serde_json::Value>) -> Self {
        if let Some(node) = self.graph.get_node_mut(node_id) {
            node.properties = properties;
        }
        self
    }

    /// Build the graph
    pub fn build(self) -> Graph {
        self.graph
    }
}

/// Template builder for creating reusable templates
pub struct TemplateBuilder {
    name: String,
    description: String,
    graph: Graph,
    metadata: HashMap<String, serde_json::Value>,
}

impl TemplateBuilder {
    /// Create a new template builder
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            graph: Graph::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set template metadata
    pub fn metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set the graph for the template
    pub fn graph(mut self, graph: Graph) -> Self {
        self.graph = graph;
        self
    }

    /// Build the template
    pub fn build(self) -> Template {
        Template {
            name: self.name,
            description: self.description,
            graph: self.graph,
            metadata: self.metadata,
        }
    }
}

/// Template structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub graph: Graph,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Yaml,
    Wasm,
    Wat,
    Graphviz,
    Mermaid,
}

/// Exporter trait for different output formats
pub trait Exporter {
    /// Export name
    fn name(&self) -> &str;
    
    /// Export format
    fn format(&self) -> ExportFormat;
    
    /// Export a graph
    fn export_graph(&self, graph: &Graph) -> CanvasResult<Vec<u8>>;
    
    /// Export a template
    fn export_template(&self, template: &Template) -> CanvasResult<Vec<u8>>;
}

/// Importer trait for different input formats
pub trait Importer {
    /// Importer name
    fn name(&self) -> &str;
    
    /// Supported formats
    fn supported_formats(&self) -> Vec<ExportFormat>;
    
    /// Import a graph
    fn import_graph(&self, data: &[u8]) -> CanvasResult<Graph>;
    
    /// Import a template
    fn import_template(&self, data: &[u8]) -> CanvasResult<Template>;
}

/// Validator trait for custom validation rules
pub trait Validator {
    /// Validator name
    fn name(&self) -> &str;
    
    /// Validate a graph
    fn validate_graph(&self, graph: &Graph) -> CanvasResult<ValidationResult>;
    
    /// Validate a node
    fn validate_node(&self, node: &Node) -> CanvasResult<ValidationResult>;
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<String>,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub message: String,
    pub severity: ValidationSeverity,
    pub location: Option<String>,
    pub code: Option<String>,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub message: String,
    pub severity: ValidationSeverity,
    pub location: Option<String>,
    pub suggestion: Option<String>,
}

/// Validation severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Optimizer trait for graph optimization
pub trait Optimizer {
    /// Optimizer name
    fn name(&self) -> &str;
    
    /// Optimize a graph
    fn optimize_graph(&self, graph: &Graph) -> CanvasResult<OptimizationResult>;
}

/// Optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub optimized_graph: Graph,
    pub improvements: Vec<OptimizationImprovement>,
    pub estimated_gas_savings: u64,
    pub estimated_performance_gain: f64,
}

/// Optimization improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationImprovement {
    pub description: String,
    pub impact: OptimizationImpact,
    pub applied: bool,
}

/// Optimization impact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationImpact {
    Gas,
    Performance,
    Security,
    Maintainability,
}

/// Main SDK struct
pub struct CanvasSdk {
    config: SdkConfig,
    plugin_registry: PluginRegistry,
    compiler: Compiler,
    runtime: WasmRuntime,
    exporters: HashMap<String, Box<dyn Exporter>>,
    importers: HashMap<String, Box<dyn Importer>>,
    validators: HashMap<String, Box<dyn Validator>>,
    optimizers: HashMap<String, Box<dyn Optimizer>>,
}

impl CanvasSdk {
    /// Create a new SDK instance
    pub fn new(config: SdkConfig) -> CanvasResult<Self> {
        let plugin_registry = PluginRegistry::new(config.clone());
        let compiler = Compiler::new();
        let runtime = WasmRuntime::new(&Config::default())?;

        Ok(Self {
            config,
            plugin_registry,
            compiler,
            runtime,
            exporters: HashMap::new(),
            importers: HashMap::new(),
            validators: HashMap::new(),
            optimizers: HashMap::new(),
        })
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn CanvasPlugin>) -> CanvasResult<()> {
        self.plugin_registry.register_plugin(plugin)
    }

    /// Register an exporter
    pub fn register_exporter(&mut self, name: String, exporter: Box<dyn Exporter>) {
        self.exporters.insert(name, exporter);
    }

    /// Register an importer
    pub fn register_importer(&mut self, name: String, importer: Box<dyn Importer>) {
        self.importers.insert(name, importer);
    }

    /// Register a validator
    pub fn register_validator(&mut self, name: String, validator: Box<dyn Validator>) {
        self.validators.insert(name, validator);
    }

    /// Register an optimizer
    pub fn register_optimizer(&mut self, name: String, optimizer: Box<dyn Optimizer>) {
        self.optimizers.insert(name, optimizer);
    }

    /// Compile a graph to WASM
    pub fn compile_graph(&self, graph: &Graph) -> CanvasResult<Vec<u8>> {
        self.compiler.compile(graph)
    }

    /// Execute a graph
    pub fn execute_graph(&self, graph: &Graph, inputs: HashMap<String, serde_json::Value>) -> CanvasResult<HashMap<String, serde_json::Value>> {
        // Compile the graph first
        let wasm_bytes = self.compile_graph(graph)?;
        
        // Execute the WASM
        self.runtime.execute(&wasm_bytes, inputs)
    }

    /// Validate a graph using all registered validators
    pub fn validate_graph(&self, graph: &Graph) -> Vec<ValidationResult> {
        self.validators
            .values()
            .filter_map(|validator| validator.validate_graph(graph).ok())
            .collect()
    }

    /// Optimize a graph using all registered optimizers
    pub fn optimize_graph(&self, graph: &Graph) -> Vec<OptimizationResult> {
        self.optimizers
            .values()
            .filter_map(|optimizer| optimizer.optimize_graph(graph).ok())
            .collect()
    }

    /// Export a graph in the specified format
    pub fn export_graph(&self, graph: &Graph, format: ExportFormat) -> CanvasResult<Vec<u8>> {
        for exporter in self.exporters.values() {
            if exporter.format() == format {
                return exporter.export_graph(graph);
            }
        }
        
        Err(CanvasError::NotFound(format!("No exporter found for format: {:?}", format)))
    }

    /// Import a graph from the specified format
    pub fn import_graph(&self, data: &[u8], format: ExportFormat) -> CanvasResult<Graph> {
        for importer in self.importers.values() {
            if importer.supported_formats().contains(&format) {
                return importer.import_graph(data);
            }
        }
        
        Err(CanvasError::NotFound(format!("No importer found for format: {:?}", format)))
    }

    /// Create a custom node
    pub fn create_custom_node(&self, builder: CustomNodeBuilder) -> CustomNodeDefinition {
        builder.build()
    }

    /// Get SDK information
    pub fn get_info(&self) -> SdkInfo {
        SdkInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            plugins_count: self.plugin_registry.get_all_plugins().len(),
            exporters_count: self.exporters.len(),
            importers_count: self.importers.len(),
            validators_count: self.validators.len(),
            optimizers_count: self.optimizers.len(),
        }
    }
}

/// SDK information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkInfo {
    pub version: String,
    pub plugins_count: usize,
    pub exporters_count: usize,
    pub importers_count: usize,
    pub validators_count: usize,
    pub optimizers_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_builder() {
        let graph = GraphBuilder::new()
            .add_node(NodeType::Start, (0.0, 0.0))
            .add_node(NodeType::Logic, (100.0, 0.0))
            .add_node(NodeType::End, (200.0, 0.0))
            .connect("node_0", "node_1")
            .connect("node_1", "node_2")
            .build();

        assert_eq!(graph.get_nodes().len(), 3);
        assert_eq!(graph.get_edges().len(), 2);
    }

    #[test]
    fn test_template_builder() {
        let graph = Graph::new();
        let template = TemplateBuilder::new(
            "Test Template".to_string(),
            "A test template".to_string(),
        )
        .metadata("difficulty".to_string(), serde_json::json!("beginner"))
        .graph(graph)
        .build();

        assert_eq!(template.name, "Test Template");
        assert_eq!(template.description, "A test template");
        assert_eq!(template.metadata.get("difficulty"), Some(&serde_json::json!("beginner")));
    }

    #[test]
    fn test_sdk_creation() {
        let config = SdkConfig {
            api_version: "1.0.0".to_string(),
            features: vec!["custom_nodes".to_string()],
            debug_mode: false,
            log_level: "info".to_string(),
            cache_enabled: true,
            max_cache_size: 1000,
        };

        let sdk = CanvasSdk::new(config);
        assert!(sdk.is_ok());
    }

    #[test]
    fn test_plugin_registry() {
        let config = SdkConfig {
            api_version: "1.0.0".to_string(),
            features: vec![],
            debug_mode: false,
            log_level: "info".to_string(),
            cache_enabled: true,
            max_cache_size: 1000,
        };

        let mut registry = PluginRegistry::new(config);
        assert_eq!(registry.get_all_plugins().len(), 0);
    }
} 