use crate::{
    config::Config,
    error::{CanvasError, CanvasResult},
    types::{Event, Gas},
};
use std::collections::HashMap;

pub struct WasmRuntime {
    engine: wasmtime::Engine,
}

#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub output: serde_json::Value,
    pub gas_used: Gas,
    pub events: Vec<Event>,
    pub execution_time: std::time::Duration,
}

struct HostState {
    events: Vec<Event>,
}

impl HostState {
    fn new() -> Self {
        Self { events: Vec::new() }
    }
}

impl WasmRuntime {
    pub fn new(_config: &Config) -> CanvasResult<Self> {
        let mut config_wasm = wasmtime::Config::new();
        config_wasm.consume_fuel(true);
        let engine = wasmtime::Engine::new(&config_wasm)
            .map_err(|e| CanvasError::Wasm(format!("Failed to create WASM engine: {}", e)))?;
        Ok(Self { engine })
    }

    pub fn simulate(
        &self,
        wasm_bytes: &[u8],
        _input_data: serde_json::Value,
        gas_limit: Gas,
    ) -> CanvasResult<SimulationResult> {
        let start_time = std::time::Instant::now();

        let module = wasmtime::Module::new(&self.engine, wasm_bytes)
            .map_err(|e| CanvasError::Wasm(format!("Failed to compile WASM module: {}", e)))?;

        let mut store = wasmtime::Store::new(&self.engine, HostState::new());
        store
            .set_fuel(gas_limit)
            .map_err(|e| CanvasError::Wasm(format!("Failed to set fuel: {}", e)))?;

        let mut linker = wasmtime::Linker::new(&self.engine);

        let storage = std::sync::Arc::new(std::sync::Mutex::new(HashMap::<i64, i64>::new()));
        let storage_clone = storage.clone();
        linker
            .func_wrap("baals", "baals_read_storage", move |key: i64| -> i64 {
                let map = storage_clone.lock().unwrap();
                map.get(&key).copied().unwrap_or(0)
            })
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "baals",
                "baals_write_storage",
                move |key: i64, value: i64| {
                    let mut map = storage.lock().unwrap();
                    map.insert(key, value);
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| CanvasError::Wasm(format!("Failed to instantiate WASM module: {}", e)))?;

        let fuel_before = store
            .get_fuel()
            .map_err(|e| CanvasError::Wasm(format!("Failed to get fuel: {}", e)))?;

        let main = instance
            .get_typed_func::<(), i64>(&mut store, "main")
            .map_err(|e| CanvasError::Wasm(format!("Failed to get main function: {}", e)))?;
        let result = main
            .call(&mut store, ())
            .map_err(|e| CanvasError::Wasm(format!("Execution failed: {}", e)))?;

        let fuel_after = store.get_fuel().unwrap_or(0);
        let gas_used = fuel_before.saturating_sub(fuel_after);
        let execution_time = start_time.elapsed();

        let host_state = store.into_data();
        let events = host_state.events;

        Ok(SimulationResult {
            output: serde_json::json!({"result": result}),
            gas_used,
            events,
            execution_time,
        })
    }

    pub fn execute_function(
        &self,
        wasm_bytes: &[u8],
        function_name: &str,
        _arguments: Vec<serde_json::Value>,
        gas_limit: Gas,
    ) -> CanvasResult<SimulationResult> {
        let start_time = std::time::Instant::now();

        let module = wasmtime::Module::new(&self.engine, wasm_bytes)
            .map_err(|e| CanvasError::Wasm(format!("Failed to compile WASM module: {}", e)))?;

        let mut store = wasmtime::Store::new(&self.engine, HostState::new());
        store
            .set_fuel(gas_limit)
            .map_err(|e| CanvasError::Wasm(format!("Failed to set fuel: {}", e)))?;

        let linker = wasmtime::Linker::new(&self.engine);

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| CanvasError::Wasm(format!("Failed to instantiate WASM module: {}", e)))?;

        let fuel_before = store
            .get_fuel()
            .map_err(|e| CanvasError::Wasm(format!("Failed to get fuel: {}", e)))?;

        let func = instance
            .get_typed_func::<(), i64>(&mut store, function_name)
            .map_err(|e| {
                CanvasError::Wasm(format!("Failed to get function '{}': {}", function_name, e))
            })?;
        let result = func.call(&mut store, ()).map_err(|e| {
            CanvasError::Wasm(format!("Execution of '{}' failed: {}", function_name, e))
        })?;

        let fuel_after = store.get_fuel().unwrap_or(0);
        let gas_used = fuel_before.saturating_sub(fuel_after);
        let execution_time = start_time.elapsed();

        let host_state = store.into_data();
        let events = host_state.events;

        Ok(SimulationResult {
            output: serde_json::json!({"result": result}),
            gas_used,
            events,
            execution_time,
        })
    }

    pub fn validate_module(&self, wasm_bytes: &[u8]) -> CanvasResult<()> {
        wasmtime::Module::validate(&self.engine, wasm_bytes)
            .map_err(|e| CanvasError::Wasm(format!("WASM validation failed: {}", e)))
    }

    pub fn get_exports(&self, wasm_bytes: &[u8]) -> CanvasResult<Vec<String>> {
        let module = wasmtime::Module::new(&self.engine, wasm_bytes)
            .map_err(|e| CanvasError::Wasm(format!("Failed to parse WASM module: {}", e)))?;

        let exports: Vec<String> = module.exports().map(|e| e.name().to_string()).collect();
        Ok(exports)
    }

    pub fn get_imports(&self, wasm_bytes: &[u8]) -> CanvasResult<Vec<String>> {
        let module = wasmtime::Module::new(&self.engine, wasm_bytes)
            .map_err(|e| CanvasError::Wasm(format!("Failed to parse WASM module: {}", e)))?;

        let imports: Vec<String> = module
            .imports()
            .map(|i| format!("{}.{}", i.module(), i.name()))
            .collect();
        Ok(imports)
    }
}

pub struct WasmAnalyzer;

impl WasmAnalyzer {
    pub fn new(_config: &Config) -> CanvasResult<Self> {
        Ok(Self)
    }

    pub fn analyze_security(&self, wasm_bytes: &[u8]) -> CanvasResult<SecurityAnalysis> {
        let issues = Vec::new();
        let mut warnings = Vec::new();

        if wasm_bytes.len() > 1_000_000 {
            warnings.push("Module size is very large (>1MB)".to_string());
        }

        Ok(SecurityAnalysis {
            issues,
            warnings,
            risk_level: RiskLevel::Low,
        })
    }

    pub fn analyze_performance(&self, wasm_bytes: &[u8]) -> CanvasResult<PerformanceAnalysis> {
        Ok(PerformanceAnalysis {
            estimated_gas_cost: wasm_bytes.len() as u64 * 10,
            complexity_score: wasm_bytes.len() as f64 / 1000.0,
            optimization_suggestions: vec![
                "Consider reducing module size".to_string(),
                "Optimize function calls".to_string(),
            ],
        })
    }
}

#[derive(Debug, Clone)]
pub struct SecurityAnalysis {
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    pub estimated_gas_cost: Gas,
    pub complexity_score: f64,
    pub optimization_suggestions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_runtime_creation() {
        let config = Config::default();
        let runtime = WasmRuntime::new(&config);
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_wasm_validation() {
        let config = Config::default();
        let runtime = WasmRuntime::new(&config).unwrap();

        // Minimal valid WASM module
        use wasm_encoder::*;
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.function([], []);
        module.section(&types);
        let mut funcs = FunctionSection::new();
        funcs.function(0);
        module.section(&funcs);
        let mut exports = ExportSection::new();
        exports.export("dummy", ExportKind::Func, 0);
        module.section(&exports);
        let mut codes = CodeSection::new();
        let mut func = Function::new(vec![]);
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);
        let valid_wasm = module.finish();

        assert!(runtime.validate_module(&valid_wasm).is_ok());
        assert!(runtime.validate_module(b"invalid").is_err());
    }

    #[test]
    fn test_wasm_runtime_get_exports() {
        let config = Config::default();
        let runtime = WasmRuntime::new(&config).unwrap();

        use wasm_encoder::*;
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.function([], [ValType::I64]);
        module.section(&types);
        let mut funcs = FunctionSection::new();
        funcs.function(0);
        module.section(&funcs);
        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 0);
        module.section(&exports);
        let mut codes = CodeSection::new();
        let mut func = Function::new(vec![]);
        func.instruction(&Instruction::I64Const(42));
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);
        let wasm_bytes = module.finish();

        let exports = runtime.get_exports(&wasm_bytes).unwrap();
        assert!(exports.contains(&"main".to_string()));
    }

    #[test]
    fn test_simulation_executes_real_wasm() {
        let config = Config::default();
        let runtime = WasmRuntime::new(&config).unwrap();

        use wasm_encoder::*;
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.function([], [ValType::I64]);
        module.section(&types);
        let mut funcs = FunctionSection::new();
        funcs.function(0);
        module.section(&funcs);
        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 0);
        module.section(&exports);
        let mut codes = CodeSection::new();
        let mut func = Function::new(vec![]);
        func.instruction(&Instruction::I64Const(42));
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);
        let wasm_bytes = module.finish();

        let result = runtime
            .simulate(&wasm_bytes, serde_json::json!({}), 1000)
            .unwrap();
        assert_eq!(result.output, serde_json::json!({"result": Some(42i64)}));
        assert!(result.gas_used > 0);
    }

    #[test]
    fn test_fuel_metering_consumes_gas() {
        let config = Config::default();
        let runtime = WasmRuntime::new(&config).unwrap();

        use wasm_encoder::*;
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.function([], [ValType::I64]);
        module.section(&types);
        let mut funcs = FunctionSection::new();
        funcs.function(0);
        module.section(&funcs);
        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 0);
        module.section(&exports);
        let mut codes = CodeSection::new();
        let mut func = Function::new(vec![]);
        // Complex arithmetic chain to consume fuel
        for _ in 0..100 {
            func.instruction(&Instruction::I64Const(1));
            func.instruction(&Instruction::I64Const(2));
            func.instruction(&Instruction::I64Add);
            func.instruction(&Instruction::Drop);
        }
        func.instruction(&Instruction::I64Const(42));
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);
        let wasm_bytes = module.finish();

        let result = runtime
            .simulate(&wasm_bytes, serde_json::json!({}), 1000)
            .unwrap();
        assert!(
            result.gas_used > 0,
            "Fuel should be consumed during execution"
        );
        assert!(result.gas_used < 1000, "Should not use all fuel");
        assert_eq!(result.output, serde_json::json!({"result": Some(42i64)}));
    }

    #[test]
    fn test_fuel_exhaustion() {
        let config = Config::default();
        let runtime = WasmRuntime::new(&config).unwrap();

        use wasm_encoder::*;
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.function([], [ValType::I64]);
        module.section(&types);
        let mut funcs = FunctionSection::new();
        funcs.function(0);
        module.section(&funcs);
        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 0);
        module.section(&exports);
        let mut codes = CodeSection::new();
        let mut func = Function::new(vec![]);
        // Many operations that should exceed 1 unit of fuel
        for _ in 0..10 {
            func.instruction(&Instruction::I64Const(1));
            func.instruction(&Instruction::I64Const(2));
            func.instruction(&Instruction::I64Add);
            func.instruction(&Instruction::Drop);
        }
        func.instruction(&Instruction::I64Const(0));
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);
        let wasm_bytes = module.finish();

        let result = runtime.simulate(&wasm_bytes, serde_json::json!({}), 1);
        assert!(
            result.is_err(),
            "Should fail with fuel exhaustion when using only 1 fuel unit"
        );
    }
}

/// Stub WasmModule type for custom nodes compatibility
pub struct WasmModule;

impl WasmModule {
    pub fn new(_path: &str) -> CanvasResult<Self> {
        Ok(Self)
    }
}
