//! Custom node system for user-defined nodes

use crate::{
    error::{CanvasError, CanvasResult},
    types::{Node, NodeId, NodeType},
    wasm::WasmModule,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        let definition = self.nodes.get(node_id)
            .ok_or_else(|| CanvasError::NodeNotFound(node_id.to_string()))?;

        match &definition.implementation {
            CustomNodeImplementation::Composite { sub_graph } => {
                self.execute_composite_node(definition, inputs, properties, sub_graph)
            }
            CustomNodeImplementation::Wasm { function_name, module_info } => {
                self.execute_wasm_node(definition, inputs, properties, function_name, module_info)
            }
            CustomNodeImplementation::Script { language, code } => {
                self.execute_script_node(definition, inputs, properties, language, code)
            }
        }
    }

    /// Validate node definition
    fn validate_node_definition(&self, definition: &CustomNodeDefinition) -> CanvasResult<()> {
        // Check for duplicate IDs
        if self.nodes.contains_key(&definition.id) {
            return Err(CanvasError::ValidationError(
                format!("Node with ID '{}' already exists", definition.id)
            ));
        }

        // Validate inputs
        for input in &definition.inputs {
            if input.name.is_empty() {
                return Err(CanvasError::ValidationError(
                    "Input name cannot be empty".to_string()
                ));
            }
        }

        // Validate outputs
        for output in &definition.outputs {
            if output.name.is_empty() {
                return Err(CanvasError::ValidationError(
                    "Output name cannot be empty".to_string()
                ));
            }
        }

        // Validate properties
        for property in &definition.properties {
            if property.name.is_empty() {
                return Err(CanvasError::ValidationError(
                    "Property name cannot be empty".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Load WASM module
    fn load_wasm_module(&self, wasm_info: &WasmModuleInfo) -> CanvasResult<WasmModule> {
        // TODO: Implement WASM module loading
        // For now, return a placeholder
        Ok(WasmModule::new(&wasm_info.module_path)?)
    }

    /// Execute composite node
    fn execute_composite_node(
        &self,
        definition: &CustomNodeDefinition,
        inputs: HashMap<String, serde_json::Value>,
        properties: HashMap<String, serde_json::Value>,
        sub_graph_json: &str,
    ) -> CanvasResult<HashMap<String, serde_json::Value>> {
        // TODO: Implement composite node execution
        // This would involve:
        // 1. Deserializing the sub-graph
        // 2. Setting up input values
        // 3. Executing the sub-graph
        // 4. Collecting output values
        
        log::info!("Executing composite node: {}", definition.name);
        
        // Placeholder implementation
        let mut outputs = HashMap::new();
        for output in &definition.outputs {
            outputs.insert(output.name.clone(), serde_json::Value::Null);
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
        let wasm_module = self.wasm_modules.get(&definition.id)
            .ok_or_else(|| CanvasError::WasmError("WASM module not loaded".to_string()))?;

        // TODO: Implement WASM function execution
        // This would involve:
        // 1. Converting inputs to WASM-compatible format
        // 2. Calling the WASM function
        // 3. Converting outputs back to JSON format
        
        log::info!("Executing WASM node: {} with function: {}", definition.name, function_name);
        
        // Placeholder implementation
        let mut outputs = HashMap::new();
        for output in &definition.outputs {
            outputs.insert(output.name.clone(), serde_json::Value::Null);
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
        // TODO: Implement script execution
        // This would involve:
        // 1. Compiling the script to WASM (if needed)
        // 2. Setting up the execution environment
        // 3. Running the script with inputs
        // 4. Collecting outputs
        
        log::info!("Executing script node: {} with language: {}", definition.name, language);
        
        // Placeholder implementation
        let mut outputs = HashMap::new();
        for output in &definition.outputs {
            outputs.insert(output.name.clone(), serde_json::Value::Null);
        }
        
        Ok(outputs)
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
    pub fn input(mut self, name: String, port_type: String, required: bool, description: String) -> Self {
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
        
        let definition = CustomNodeBuilder::new(
            "test-node".to_string(),
            "Test Node".to_string(),
        )
        .description("A test custom node".to_string())
        .category("Test".to_string())
        .input("input1".to_string(), "number".to_string(), true, "First input".to_string())
        .output("output1".to_string(), "number".to_string(), "First output".to_string())
        .composite("{}".to_string())
        .build();

        assert!(registry.register_node(definition).is_ok());
        assert!(registry.get_node("test-node").is_some());
    }

    #[test]
    fn test_duplicate_node_registration() {
        let mut registry = CustomNodeRegistry::new();
        
        let definition1 = CustomNodeBuilder::new(
            "test-node".to_string(),
            "Test Node".to_string(),
        )
        .composite("{}".to_string())
        .build();

        let definition2 = CustomNodeBuilder::new(
            "test-node".to_string(),
            "Another Test Node".to_string(),
        )
        .composite("{}".to_string())
        .build();

        assert!(registry.register_node(definition1).is_ok());
        assert!(registry.register_node(definition2).is_err());
    }
} 