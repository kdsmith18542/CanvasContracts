//! Custom node system for user-defined nodes

use crate::{
    config::Config,
    error::{CanvasError, CanvasResult},
    wasm::{WasmModule, WasmRuntime},
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Custom node definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomNodeDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub inputs: Vec<CustomNodePort>,
    pub outputs: Vec<CustomNodePort>,
    pub properties: Vec<CustomNodeProperty>,
    pub wasm_module: Option<WasmModuleInfo>,
    pub implementation: CustomNodeImplementation,
}

/// Custom node port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomNodePort {
    pub name: String,
    pub port_type: String,
    pub required: bool,
    pub description: String,
}

/// Custom node property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomNodeProperty {
    pub name: String,
    pub property_type: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub description: String,
}

/// WASM module information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmModuleInfo {
    pub module_path: String,
    pub exported_functions: Vec<String>,
    pub abi: String,
}

/// Custom node implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomNodeImplementation {
    /// Composite node (sub-graph)
    Composite {
        sub_graph: String, // JSON serialized graph
    },
    /// WASM-backed node
    Wasm {
        function_name: String,
        module_info: WasmModuleInfo,
    },
    /// Script-based node
    Script {
        language: String, // "rust", "go", "assemblyscript"
        code: String,
    },
}

/// Custom node registry
pub struct CustomNodeRegistry {
    nodes: HashMap<String, CustomNodeDefinition>,
    wasm_modules: HashMap<String, WasmModule>,
}

impl Default for CustomNodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomNodeRegistry {
    /// Create a new custom node registry
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            wasm_modules: HashMap::new(),
        }
    }

    /// Register a custom node
    pub fn register_node(&mut self, definition: CustomNodeDefinition) -> CanvasResult<()> {
        // Validate the node definition
        self.validate_node_definition(&definition)?;

        // Load WASM module if specified
        if let Some(wasm_info) = &definition.wasm_module {
            let wasm_module = self.load_wasm_module(wasm_info)?;
            self.wasm_modules.insert(definition.id.clone(), wasm_module);
        }

        self.nodes.insert(definition.id.clone(), definition);
        Ok(())
    }

    /// Get a custom node definition
    pub fn get_node(&self, node_id: &str) -> Option<&CustomNodeDefinition> {
        self.nodes.get(node_id)
    }

    /// List all custom nodes
    pub fn list_nodes(&self) -> Vec<&CustomNodeDefinition> {
        self.nodes.values().collect()
    }

    /// Remove a custom node
    pub fn remove_node(&mut self, node_id: &str) -> CanvasResult<()> {
        if self.nodes.remove(node_id).is_some() {
            self.wasm_modules.remove(node_id);
            Ok(())
        } else {
            Err(CanvasError::NodeNotFound(node_id.to_string()))
        }
    }

    /// Execute a custom node
    pub fn execute_node(
        &self,
        node_id: &str,
        inputs: HashMap<String, serde_json::Value>,
        properties: HashMap<String, serde_json::Value>,
    ) -> CanvasResult<HashMap<String, serde_json::Value>> {
        let definition = self
            .nodes
            .get(node_id)
            .ok_or_else(|| CanvasError::NodeNotFound(node_id.to_string()))?;

        match &definition.implementation {
            CustomNodeImplementation::Composite { sub_graph } => {
                self.execute_composite_node(definition, inputs, properties, sub_graph)
            }
            CustomNodeImplementation::Wasm {
                function_name,
                module_info,
            } => self.execute_wasm_node(definition, inputs, properties, function_name, module_info),
            CustomNodeImplementation::Script { language, code } => {
                self.execute_script_node(definition, inputs, properties, language, code)
            }
        }
    }

    /// Validate node definition
    fn validate_node_definition(&self, definition: &CustomNodeDefinition) -> CanvasResult<()> {
        // Check for duplicate IDs
        if self.nodes.contains_key(&definition.id) {
            return Err(CanvasError::Validation(format!(
                "Node with ID '{}' already exists",
                definition.id
            )));
        }

        // Validate inputs
        for input in &definition.inputs {
            if input.name.is_empty() {
                return Err(CanvasError::Validation(
                    "Input name cannot be empty".to_string(),
                ));
            }
        }

        // Validate outputs
        for output in &definition.outputs {
            if output.name.is_empty() {
                return Err(CanvasError::Validation(
                    "Output name cannot be empty".to_string(),
                ));
            }
        }

        // Validate properties
        for property in &definition.properties {
            if property.name.is_empty() {
                return Err(CanvasError::Validation(
                    "Property name cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Load WASM module
    fn load_wasm_module(&self, wasm_info: &WasmModuleInfo) -> CanvasResult<WasmModule> {
        if wasm_info.module_path.trim().is_empty() {
            return Err(CanvasError::Validation(
                "WASM module_path cannot be empty".to_string(),
            ));
        }
        let module_path = Path::new(&wasm_info.module_path);
        if !module_path.exists() {
            return Err(CanvasError::NotFound(format!(
                "WASM module file not found: {}",
                wasm_info.module_path
            )));
        }
        if module_path.extension().and_then(|ext| ext.to_str()) != Some("wasm") {
            return Err(CanvasError::Validation(format!(
                "WASM module must have .wasm extension: {}",
                wasm_info.module_path
            )));
        }
        WasmModule::new(&wasm_info.module_path)
    }

    /// Execute composite node
    fn execute_composite_node(
        &self,
        definition: &CustomNodeDefinition,
        inputs: HashMap<String, serde_json::Value>,
        properties: HashMap<String, serde_json::Value>,
        sub_graph_json: &str,
    ) -> CanvasResult<HashMap<String, serde_json::Value>> {
        log::info!("Executing composite node: {}", definition.name);
        self.validate_required_ports(definition, &inputs)?;
        self.validate_required_properties(definition, &properties)?;

        let parsed_graph = if sub_graph_json.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str::<serde_json::Value>(sub_graph_json).map_err(|e| {
                CanvasError::Validation(format!(
                    "Composite node sub-graph JSON is invalid for '{}': {}",
                    definition.id, e
                ))
            })?
        };

        let output_map = parsed_graph
            .get("output_map")
            .and_then(|value| value.as_object())
            .cloned()
            .unwrap_or_default();

        let mut outputs = HashMap::with_capacity(definition.outputs.len());
        for output in &definition.outputs {
            let mapped = output_map.get(&output.name).and_then(|v| v.as_str());
            let value = self.resolve_output_value(mapped, &output.name, &inputs, &properties)?;
            outputs.insert(output.name.clone(), value);
        }

        Ok(outputs)
    }

    /// Execute WASM-backed node
    fn execute_wasm_node(
        &self,
        definition: &CustomNodeDefinition,
        inputs: HashMap<String, serde_json::Value>,
        properties: HashMap<String, serde_json::Value>,
        function_name: &str,
        module_info: &WasmModuleInfo,
    ) -> CanvasResult<HashMap<String, serde_json::Value>> {
        let wasm_module = self
            .wasm_modules
            .get(&definition.id)
            .ok_or_else(|| CanvasError::Wasm("WASM module not loaded".to_string()))?;

        self.validate_required_ports(definition, &inputs)?;
        self.validate_required_properties(definition, &properties)?;
        if !module_info.exported_functions.is_empty()
            && !module_info
                .exported_functions
                .iter()
                .any(|name| name == function_name)
        {
            return Err(CanvasError::Validation(format!(
                "WASM function '{}' is not listed in module exports metadata",
                function_name
            )));
        }

        log::info!(
            "Executing WASM node: {} with function: {}",
            definition.name,
            function_name
        );

        let runtime = WasmRuntime::new(&Config::default())?;
        let arguments = definition
            .inputs
            .iter()
            .map(|input| {
                inputs
                    .get(&input.name)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null)
            })
            .collect::<Vec<_>>();
        let result =
            runtime.execute_function(wasm_module.bytes(), function_name, arguments, 100_000)?;

        let mut outputs = HashMap::with_capacity(definition.outputs.len());
        if definition.outputs.len() == 1 {
            let output_name = &definition.outputs[0].name;
            outputs.insert(
                output_name.clone(),
                result
                    .output
                    .get("result")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            );
            return Ok(outputs);
        }

        let raw_result = result
            .output
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        match raw_result {
            serde_json::Value::Object(obj) => {
                for output in &definition.outputs {
                    outputs.insert(
                        output.name.clone(),
                        obj.get(&output.name)
                            .cloned()
                            .unwrap_or(serde_json::Value::Null),
                    );
                }
            }
            serde_json::Value::Array(values) => {
                for (index, output) in definition.outputs.iter().enumerate() {
                    outputs.insert(
                        output.name.clone(),
                        values
                            .get(index)
                            .cloned()
                            .unwrap_or(serde_json::Value::Null),
                    );
                }
            }
            scalar => {
                for output in &definition.outputs {
                    outputs.insert(output.name.clone(), scalar.clone());
                }
            }
        }

        Ok(outputs)
    }

    /// Execute script-based node
    fn execute_script_node(
        &self,
        definition: &CustomNodeDefinition,
        inputs: HashMap<String, serde_json::Value>,
        properties: HashMap<String, serde_json::Value>,
        language: &str,
        code: &str,
    ) -> CanvasResult<HashMap<String, serde_json::Value>> {
        self.validate_required_ports(definition, &inputs)?;
        self.validate_required_properties(definition, &properties)?;

        log::info!(
            "Executing script node: {} with language: {}",
            definition.name,
            language
        );

        let mut outputs = HashMap::with_capacity(definition.outputs.len());
        let lower_language = language.to_ascii_lowercase();
        let script_mapping = if lower_language == "json" || lower_language == "inline" {
            serde_json::from_str::<serde_json::Value>(code).ok()
        } else {
            None
        };

        for output in &definition.outputs {
            let value = if let Some(mapping) = &script_mapping {
                mapping
                    .get(&output.name)
                    .cloned()
                    .or_else(|| mapping.get("result").cloned())
                    .or_else(|| inputs.get(&output.name).cloned())
                    .or_else(|| properties.get(&output.name).cloned())
                    .unwrap_or(serde_json::Value::Null)
            } else {
                inputs
                    .get(&output.name)
                    .cloned()
                    .or_else(|| properties.get(&output.name).cloned())
                    .unwrap_or_else(|| {
                        serde_json::json!({
                            "language": lower_language,
                            "code_len": code.len(),
                            "input_count": inputs.len(),
                            "property_count": properties.len()
                        })
                    })
            };
            outputs.insert(output.name.clone(), value);
        }

        Ok(outputs)
    }

    fn validate_required_ports(
        &self,
        definition: &CustomNodeDefinition,
        inputs: &HashMap<String, serde_json::Value>,
    ) -> CanvasResult<()> {
        for port in definition.inputs.iter().filter(|port| port.required) {
            if !inputs.contains_key(&port.name) {
                return Err(CanvasError::Validation(format!(
                    "Missing required input '{}' for custom node '{}'",
                    port.name, definition.id
                )));
            }
        }
        Ok(())
    }

    fn validate_required_properties(
        &self,
        definition: &CustomNodeDefinition,
        properties: &HashMap<String, serde_json::Value>,
    ) -> CanvasResult<()> {
        for property in definition
            .properties
            .iter()
            .filter(|property| property.required)
        {
            if !properties.contains_key(&property.name) {
                return Err(CanvasError::Validation(format!(
                    "Missing required property '{}' for custom node '{}'",
                    property.name, definition.id
                )));
            }
        }
        Ok(())
    }

    fn resolve_output_value(
        &self,
        mapping: Option<&str>,
        output_name: &str,
        inputs: &HashMap<String, serde_json::Value>,
        properties: &HashMap<String, serde_json::Value>,
    ) -> CanvasResult<serde_json::Value> {
        if let Some(mapping_value) = mapping {
            if let Some(key) = mapping_value.strip_prefix("input:") {
                return Ok(inputs.get(key).cloned().unwrap_or(serde_json::Value::Null));
            }
            if let Some(key) = mapping_value.strip_prefix("property:") {
                return Ok(properties
                    .get(key)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null));
            }
            if let Some(raw_json) = mapping_value.strip_prefix("const:") {
                return serde_json::from_str::<serde_json::Value>(raw_json).map_err(|e| {
                    CanvasError::Validation(format!(
                        "Invalid const mapping JSON '{}': {}",
                        raw_json, e
                    ))
                });
            }
            return Ok(inputs
                .get(mapping_value)
                .cloned()
                .or_else(|| properties.get(mapping_value).cloned())
                .unwrap_or(serde_json::Value::Null));
        }

        Ok(inputs
            .get(output_name)
            .cloned()
            .or_else(|| properties.get(output_name).cloned())
            .unwrap_or(serde_json::Value::Null))
    }
}

/// Custom node builder for creating nodes programmatically
pub struct CustomNodeBuilder {
    definition: CustomNodeDefinition,
}

impl CustomNodeBuilder {
    /// Create a new custom node builder
    pub fn new(id: String, name: String) -> Self {
        Self {
            definition: CustomNodeDefinition {
                id,
                name,
                description: String::new(),
                category: "Custom".to_string(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                properties: Vec::new(),
                wasm_module: None,
                implementation: CustomNodeImplementation::Composite {
                    sub_graph: String::new(),
                },
            },
        }
    }

    /// Set the node description
    pub fn description(mut self, description: String) -> Self {
        self.definition.description = description;
        self
    }

    /// Set the node category
    pub fn category(mut self, category: String) -> Self {
        self.definition.category = category;
        self
    }

    /// Add an input port
    pub fn input(
        mut self,
        name: String,
        port_type: String,
        required: bool,
        description: String,
    ) -> Self {
        self.definition.inputs.push(CustomNodePort {
            name,
            port_type,
            required,
            description,
        });
        self
    }

    /// Add an output port
    pub fn output(mut self, name: String, port_type: String, description: String) -> Self {
        self.definition.outputs.push(CustomNodePort {
            name,
            port_type,
            required: false,
            description,
        });
        self
    }

    /// Add a property
    pub fn property(
        mut self,
        name: String,
        property_type: String,
        required: bool,
        default_value: Option<String>,
        description: String,
    ) -> Self {
        self.definition.properties.push(CustomNodeProperty {
            name,
            property_type,
            required,
            default_value,
            description,
        });
        self
    }

    /// Set as composite node
    pub fn composite(mut self, sub_graph: String) -> Self {
        self.definition.implementation = CustomNodeImplementation::Composite { sub_graph };
        self
    }

    /// Set as WASM-backed node
    pub fn wasm(mut self, function_name: String, module_info: WasmModuleInfo) -> Self {
        self.definition.wasm_module = Some(module_info.clone());
        self.definition.implementation = CustomNodeImplementation::Wasm {
            function_name,
            module_info,
        };
        self
    }

    /// Set as script-based node
    pub fn script(mut self, language: String, code: String) -> Self {
        self.definition.implementation = CustomNodeImplementation::Script { language, code };
        self
    }

    /// Build the custom node definition
    pub fn build(self) -> CustomNodeDefinition {
        self.definition
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_node_registry() {
        let mut registry = CustomNodeRegistry::new();

        let definition = CustomNodeBuilder::new("test-node".to_string(), "Test Node".to_string())
            .description("A test custom node".to_string())
            .category("Test".to_string())
            .input(
                "input1".to_string(),
                "number".to_string(),
                true,
                "First input".to_string(),
            )
            .output(
                "output1".to_string(),
                "number".to_string(),
                "First output".to_string(),
            )
            .composite("{}".to_string())
            .build();

        assert!(registry.register_node(definition).is_ok());
        assert!(registry.get_node("test-node").is_some());
    }

    #[test]
    fn test_duplicate_node_registration() {
        let mut registry = CustomNodeRegistry::new();

        let definition1 = CustomNodeBuilder::new("test-node".to_string(), "Test Node".to_string())
            .composite("{}".to_string())
            .build();

        let definition2 =
            CustomNodeBuilder::new("test-node".to_string(), "Another Test Node".to_string())
                .composite("{}".to_string())
                .build();

        assert!(registry.register_node(definition1).is_ok());
        assert!(registry.register_node(definition2).is_err());
    }
}
