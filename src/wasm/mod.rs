use crate::{
    config::Config,
    error::{CanvasError, CanvasResult},
    types::{Event, Gas},
};
use sha2::{Digest, Sha256};
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

fn hash_bytes_to_i64(bytes: &[u8]) -> i64 {
    let digest = Sha256::digest(bytes);
    let mut out = [0u8; 8];
    out.copy_from_slice(&digest[..8]);
    i64::from_be_bytes(out)
}

fn mix_tagged_i64(tag: &str, values: &[i64]) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(tag.as_bytes());
    for value in values {
        hasher.update(value.to_be_bytes());
    }
    let digest = hasher.finalize();
    let mut out = [0u8; 8];
    out.copy_from_slice(&digest[..8]);
    i64::from_be_bytes(out)
}

fn json_to_wasm_value(
    value: &serde_json::Value,
    expected_type: wasmtime::ValType,
) -> CanvasResult<wasmtime::Val> {
    match expected_type {
        wasmtime::ValType::I32 => {
            let val = value.as_i64().ok_or_else(|| {
                CanvasError::Type("Expected integer argument for i32".to_string())
            })?;
            let casted = i32::try_from(val).map_err(|_| {
                CanvasError::Type(format!("Value {} does not fit into i32 argument", val))
            })?;
            Ok(wasmtime::Val::I32(casted))
        }
        wasmtime::ValType::I64 => {
            let val = value.as_i64().ok_or_else(|| {
                CanvasError::Type("Expected integer argument for i64".to_string())
            })?;
            Ok(wasmtime::Val::I64(val))
        }
        wasmtime::ValType::F32 => {
            let val = value.as_f64().ok_or_else(|| {
                CanvasError::Type("Expected numeric argument for f32".to_string())
            })?;
            Ok(wasmtime::Val::F32((val as f32).to_bits()))
        }
        wasmtime::ValType::F64 => {
            let val = value.as_f64().ok_or_else(|| {
                CanvasError::Type("Expected numeric argument for f64".to_string())
            })?;
            Ok(wasmtime::Val::F64(val.to_bits()))
        }
        other => Err(CanvasError::Type(format!(
            "Unsupported wasm argument type: {:?}",
            other
        ))),
    }
}

fn default_wasm_result_value(result_type: wasmtime::ValType) -> CanvasResult<wasmtime::Val> {
    match result_type {
        wasmtime::ValType::I32 => Ok(wasmtime::Val::I32(0)),
        wasmtime::ValType::I64 => Ok(wasmtime::Val::I64(0)),
        wasmtime::ValType::F32 => Ok(wasmtime::Val::F32(0f32.to_bits())),
        wasmtime::ValType::F64 => Ok(wasmtime::Val::F64(0f64.to_bits())),
        other => Err(CanvasError::Type(format!(
            "Unsupported wasm result type: {:?}",
            other
        ))),
    }
}

fn wasm_value_to_json(value: &wasmtime::Val) -> CanvasResult<serde_json::Value> {
    match value {
        wasmtime::Val::I32(v) => Ok(serde_json::json!(v)),
        wasmtime::Val::I64(v) => Ok(serde_json::json!(v)),
        wasmtime::Val::F32(bits) => Ok(serde_json::json!(f32::from_bits(*bits))),
        wasmtime::Val::F64(bits) => Ok(serde_json::json!(f64::from_bits(*bits))),
        other => Err(CanvasError::Type(format!(
            "Unsupported wasm output value: {:?}",
            other
        ))),
    }
}

fn map_wasm_execution_error(function_name: &str, error: wasmtime::Error) -> CanvasError {
    let message = error.to_string();
    if message.contains("Execution reverted with reason hash") {
        return CanvasError::ExecutionError(message);
    }
    CanvasError::Wasm(format!(
        "Execution of '{}' failed: {}",
        function_name, message
    ))
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

        let linker = self.build_default_linker()?;

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
        arguments: Vec<serde_json::Value>,
        gas_limit: Gas,
    ) -> CanvasResult<SimulationResult> {
        let start_time = std::time::Instant::now();

        let module = wasmtime::Module::new(&self.engine, wasm_bytes)
            .map_err(|e| CanvasError::Wasm(format!("Failed to compile WASM module: {}", e)))?;

        let mut store = wasmtime::Store::new(&self.engine, HostState::new());
        store
            .set_fuel(gas_limit)
            .map_err(|e| CanvasError::Wasm(format!("Failed to set fuel: {}", e)))?;

        let linker = self.build_default_linker()?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| CanvasError::Wasm(format!("Failed to instantiate WASM module: {}", e)))?;

        let fuel_before = store
            .get_fuel()
            .map_err(|e| CanvasError::Wasm(format!("Failed to get fuel: {}", e)))?;

        let func = instance
            .get_func(&mut store, function_name)
            .ok_or_else(|| CanvasError::Wasm(format!("Function '{}' not found", function_name)))?;
        let function_type = func.ty(&store);
        let expected_params = function_type.params().collect::<Vec<_>>();
        if expected_params.len() != arguments.len() {
            return Err(CanvasError::Validation(format!(
                "Function '{}' expects {} arguments but received {}",
                function_name,
                expected_params.len(),
                arguments.len()
            )));
        }
        let params = arguments
            .iter()
            .zip(expected_params.iter())
            .map(|(arg, expected)| json_to_wasm_value(arg, expected.clone()))
            .collect::<CanvasResult<Vec<_>>>()?;

        let expected_results = function_type.results().collect::<Vec<_>>();
        let mut results = expected_results
            .iter()
            .map(|result_type| default_wasm_result_value(result_type.clone()))
            .collect::<CanvasResult<Vec<_>>>()?;

        func.call(&mut store, &params, &mut results)
            .map_err(|e| map_wasm_execution_error(function_name, e))?;

        let output_value = if results.is_empty() {
            serde_json::Value::Null
        } else if results.len() == 1 {
            wasm_value_to_json(&results[0])?
        } else {
            let mut output_items = Vec::with_capacity(results.len());
            for result in &results {
                output_items.push(wasm_value_to_json(result)?);
            }
            serde_json::Value::Array(output_items)
        };

        let fuel_after = store.get_fuel().unwrap_or(0);
        let gas_used = fuel_before.saturating_sub(fuel_after);
        let execution_time = start_time.elapsed();

        let host_state = store.into_data();
        let events = host_state.events;

        Ok(SimulationResult {
            output: serde_json::json!({"result": output_value}),
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

    fn build_default_linker(&self) -> CanvasResult<wasmtime::Linker<HostState>> {
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

        linker
            .func_wrap("baals", "baals_get_sender", || -> i64 {
                hash_bytes_to_i64(b"baals.default_sender")
            })
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap("baals", "baals_get_contract_id", || -> i64 {
                hash_bytes_to_i64(b"baals.default_contract")
            })
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap("baals", "baals_get_block_timestamp", || -> i64 {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
            })
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap("baals", "baals_get_block_height", || -> i64 { 1 })
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "baals",
                "baals_emit_event",
                |mut caller: wasmtime::Caller<'_, HostState>,
                 event_name_hash: i64,
                 event_data_hash: i64| {
                    let mut data = HashMap::new();
                    data.insert(
                        "event_name_hash".to_string(),
                        serde_json::json!(event_name_hash),
                    );
                    data.insert(
                        "event_data_hash".to_string(),
                        serde_json::json!(event_data_hash),
                    );
                    caller.data_mut().events.push(Event {
                        name: format!("event_{}", event_name_hash),
                        data,
                        indexed_data: vec![],
                    });
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap::<_, Result<(), wasmtime::Error>>(
                "baals",
                "baals_revert",
                |reason_hash: i64| {
                    Err(wasmtime::Error::msg(format!(
                        "Execution reverted with reason hash {}",
                        reason_hash
                    )))
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "baals",
                "baals_revert_with_reason",
                |reason_hash: i64| -> i64 {
                    // Auxiliary non-trapping API for modules that prefer explicit status checks.
                    reason_hash
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap("baals", "baals_hash_sha256", |input_hash: i64| -> i64 {
                hash_bytes_to_i64(&input_hash.to_be_bytes())
            })
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap::<_, Result<(), wasmtime::Error>>(
                "baals",
                "baals_call_contract",
                |_contract_hash: i64, _method_hash: i64, _args_hash: i64| {
                    Err(wasmtime::Error::msg(
                        "baals_call_contract is disabled in local simulation runtime",
                    ))
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "baals",
                "baals_read_call_result",
                |result_handle: i64, field_hash: i64| -> i64 {
                    result_handle
                        .wrapping_mul(31)
                        .wrapping_add(field_hash.wrapping_mul(17))
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap::<_, Result<(), wasmtime::Error>>(
                "baals",
                "baals_transfer_value",
                |_recipient_hash: i64, _amount: i64| {
                    Err(wasmtime::Error::msg(
                        "baals_transfer_value is disabled in local simulation runtime",
                    ))
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "crypto",
                "crypto_verify_signature",
                |_message_hash: i64, _signature_hash: i64, _public_key_hash: i64| -> i64 { 0 },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap("crypto", "crypto_decode_proof", |proof_hash: i64| -> i64 {
                mix_tagged_i64("crypto_decode_proof", &[proof_hash])
            })
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "chrononode",
                "chrononode_fetch_block",
                |chain_hash: i64, height: i64| -> i64 {
                    mix_tagged_i64("chrononode_fetch_block", &[chain_hash, height])
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "chrononode",
                "chrononode_fetch_checkpoint",
                |chain_hash: i64, from_height: i64, to_height: i64| -> i64 {
                    mix_tagged_i64(
                        "chrononode_fetch_checkpoint",
                        &[chain_hash, from_height, to_height],
                    )
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "chrononode",
                "chrononode_verify_proof",
                |_proof_hash: i64, _data_hash: i64| -> i64 { 0 },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "chrononode",
                "chrononode_extract_event",
                |block_hash: i64, event_type_hash: i64| -> i64 {
                    mix_tagged_i64("chrononode_extract_event", &[block_hash, event_type_hash])
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "chrononode",
                "chrononode_extract_tx_by_sender",
                |block_hash: i64, sender_hash: i64| -> i64 {
                    mix_tagged_i64(
                        "chrononode_extract_tx_by_sender",
                        &[block_hash, sender_hash],
                    )
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "chrononode",
                "chrononode_extract_tx_by_recipient",
                |block_hash: i64, recipient_hash: i64| -> i64 {
                    mix_tagged_i64(
                        "chrononode_extract_tx_by_recipient",
                        &[block_hash, recipient_hash],
                    )
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "chrononode",
                "chrononode_verify_archive_range",
                |_chain_hash: i64, _from_height: i64, _to_height: i64, _proof_hash: i64| -> i64 {
                    0
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "resurgence",
                "resurgence_check_token_age",
                |token_hash: i64, current_block: i64| -> i64 {
                    mix_tagged_i64("resurgence_check_token_age", &[token_hash, current_block])
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "resurgence",
                "resurgence_check_token_activity_window",
                |token_hash: i64, window_start: i64, window_end: i64| -> i64 {
                    if token_hash != 0 && window_start <= window_end {
                        1
                    } else {
                        0
                    }
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "resurgence",
                "resurgence_check_liquidity_dormancy",
                |pool_hash: i64, threshold: i64| -> i64 {
                    mix_tagged_i64(
                        "resurgence_check_liquidity_dormancy",
                        &[pool_hash, threshold],
                    )
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "resurgence",
                "resurgence_check_governance_dormancy",
                |token_hash: i64, window: i64| -> i64 {
                    mix_tagged_i64(
                        "resurgence_check_governance_dormancy",
                        &[token_hash, window],
                    )
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "resurgence",
                "resurgence_calculate_dormancy_score",
                |age_score: i64, liquidity_score: i64, governance_score: i64| -> i64 {
                    ((age_score.max(0) * 4)
                        + (liquidity_score.max(0) * 3)
                        + (governance_score.max(0) * 3))
                        / 10
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "resurgence",
                "resurgence_normalize_dead_coin_risk",
                |raw_score: i64| -> i64 { raw_score.clamp(0, 100) },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "resurgence",
                "resurgence_generate_dormancy_proof",
                |token_hash: i64, dormancy_score: i64, evidence_hash: i64| -> i64 {
                    mix_tagged_i64(
                        "resurgence_generate_dormancy_proof",
                        &[token_hash, dormancy_score, evidence_hash],
                    )
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        linker
            .func_wrap(
                "resurgence",
                "resurgence_emit_dormancy_oracle_result",
                |mut caller: wasmtime::Caller<'_, HostState>,
                 token_hash: i64,
                 score: i64,
                 label_hash: i64,
                 proof_hash: i64| {
                    let mut data = HashMap::new();
                    data.insert("token_hash".to_string(), serde_json::json!(token_hash));
                    data.insert("score".to_string(), serde_json::json!(score));
                    data.insert("label_hash".to_string(), serde_json::json!(label_hash));
                    data.insert("proof_hash".to_string(), serde_json::json!(proof_hash));
                    caller.data_mut().events.push(Event {
                        name: "DormancyOracleResult".to_string(),
                        data,
                        indexed_data: vec![],
                    });
                },
            )
            .map_err(|e| CanvasError::Wasm(format!("Failed to link host function: {}", e)))?;

        Ok(linker)
    }
}

pub struct WasmAnalyzer;

impl WasmAnalyzer {
    pub fn new(_config: &Config) -> CanvasResult<Self> {
        Ok(Self)
    }

    pub fn analyze_security(&self, wasm_bytes: &[u8]) -> CanvasResult<SecurityAnalysis> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        if wasm_bytes.len() > 1_000_000 {
            warnings.push("Module size is very large (>1MB)".to_string());
        }

        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::new(&engine, wasm_bytes)
            .map_err(|e| CanvasError::Wasm(format!("Security analysis failed: {}", e)))?;

        let has_main = module.exports().any(|export| export.name() == "main");
        if !has_main {
            warnings.push("Module does not export a main function".to_string());
        }

        for import in module.imports() {
            let namespace = import.module();
            let name = import.name();
            let allowed = match namespace {
                "baals" => matches!(
                    name,
                    "baals_get_sender"
                        | "baals_get_contract_id"
                        | "baals_get_block_timestamp"
                        | "baals_get_block_height"
                        | "baals_emit_event"
                        | "baals_revert"
                        | "baals_revert_with_reason"
                        | "baals_hash_sha256"
                        | "baals_call_contract"
                        | "baals_read_call_result"
                        | "baals_transfer_value"
                ),
                "crypto" => matches!(name, "crypto_verify_signature" | "crypto_decode_proof"),
                "chrononode" => matches!(
                    name,
                    "chrononode_fetch_block"
                        | "chrononode_fetch_checkpoint"
                        | "chrononode_verify_proof"
                        | "chrononode_extract_event"
                        | "chrononode_extract_tx_by_sender"
                        | "chrononode_extract_tx_by_recipient"
                        | "chrononode_verify_archive_range"
                ),
                "resurgence" => matches!(
                    name,
                    "resurgence_check_token_age"
                        | "resurgence_check_token_activity_window"
                        | "resurgence_check_liquidity_dormancy"
                        | "resurgence_check_governance_dormancy"
                        | "resurgence_calculate_dormancy_score"
                        | "resurgence_normalize_dead_coin_risk"
                        | "resurgence_generate_dormancy_proof"
                        | "resurgence_emit_dormancy_oracle_result"
                ),
                _ => false,
            };

            if !allowed {
                issues.push(format!("Unknown host import '{}::{}'", namespace, name));
            }
        }

        let risk_level = if !issues.is_empty() {
            RiskLevel::High
        } else if !warnings.is_empty() {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        Ok(SecurityAnalysis {
            issues,
            warnings,
            risk_level,
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

/// WASM module wrapper for custom node execution.
pub struct WasmModule {
    path: String,
    bytes: Vec<u8>,
}

impl WasmModule {
    pub fn new(path: &str) -> CanvasResult<Self> {
        let bytes = std::fs::read(path).map_err(|e| {
            CanvasError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read WASM module '{}': {}", path, e),
            ))
        })?;
        let engine = wasmtime::Engine::default();
        wasmtime::Module::validate(&engine, &bytes)
            .map_err(|e| CanvasError::Wasm(format!("Invalid WASM module '{}': {}", path, e)))?;
        Ok(Self {
            path: path.to_string(),
            bytes,
        })
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
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
    fn test_wasm_security_analysis_flags_unknown_imports() {
        let analyzer = WasmAnalyzer::new(&Config::default()).unwrap();

        use wasm_encoder::*;
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.function([], [ValType::I64]);
        types.function([], []);
        module.section(&types);

        let mut imports = ImportSection::new();
        imports.import("evil", "syscall", EntityType::Function(1));
        module.section(&imports);

        let mut funcs = FunctionSection::new();
        funcs.function(0);
        module.section(&funcs);

        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 1);
        module.section(&exports);

        let mut codes = CodeSection::new();
        let mut func = Function::new(vec![]);
        func.instruction(&Instruction::I64Const(7));
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);

        let analysis = analyzer.analyze_security(&module.finish()).unwrap();
        assert!(!analysis.issues.is_empty());
        assert!(matches!(analysis.risk_level, RiskLevel::High));
    }

    #[test]
    fn test_wasm_security_analysis_warns_when_main_is_missing() {
        let analyzer = WasmAnalyzer::new(&Config::default()).unwrap();

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

        let analysis = analyzer.analyze_security(&module.finish()).unwrap();
        assert!(!analysis.warnings.is_empty());
        assert!(matches!(analysis.risk_level, RiskLevel::Medium));
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
    fn test_execute_function_supports_imported_modules() {
        let config = Config::default();
        let runtime = WasmRuntime::new(&config).unwrap();

        use wasm_encoder::*;
        let mut module = Module::new();

        let mut types = TypeSection::new();
        // type 0: main () -> i64
        types.function([], [ValType::I64]);
        // type 1: read (i64) -> i64
        types.function([ValType::I64], [ValType::I64]);
        // type 2: write (i64, i64) -> ()
        types.function([ValType::I64, ValType::I64], []);
        module.section(&types);

        let mut imports = ImportSection::new();
        imports.import("baals", "baals_read_storage", EntityType::Function(1));
        imports.import("baals", "baals_write_storage", EntityType::Function(2));
        module.section(&imports);

        let mut funcs = FunctionSection::new();
        funcs.function(0);
        module.section(&funcs);

        let mut exports = ExportSection::new();
        // main local function index = imported funcs (2) + local func offset (0)
        exports.export("main", ExportKind::Func, 2);
        module.section(&exports);

        let mut codes = CodeSection::new();
        let mut func = Function::new(vec![]);
        func.instruction(&Instruction::I64Const(7));
        func.instruction(&Instruction::I64Const(99));
        func.instruction(&Instruction::Call(1)); // write
        func.instruction(&Instruction::I64Const(7));
        func.instruction(&Instruction::Call(0)); // read
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);

        let wasm_bytes = module.finish();
        let result = runtime
            .execute_function(&wasm_bytes, "main", Vec::new(), 1000)
            .unwrap();

        assert_eq!(result.output, serde_json::json!({"result": Some(99i64)}));
    }

    #[test]
    fn test_execute_function_supports_arguments() {
        let config = Config::default();
        let runtime = WasmRuntime::new(&config).unwrap();

        use wasm_encoder::*;
        let mut module = Module::new();
        let mut types = TypeSection::new();
        // type 0: add(i64, i64) -> i64
        types.function([ValType::I64, ValType::I64], [ValType::I64]);
        module.section(&types);
        let mut funcs = FunctionSection::new();
        funcs.function(0);
        module.section(&funcs);
        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 0);
        module.section(&exports);
        let mut codes = CodeSection::new();
        let mut func = Function::new(vec![]);
        func.instruction(&Instruction::LocalGet(0));
        func.instruction(&Instruction::LocalGet(1));
        func.instruction(&Instruction::I64Add);
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);
        let wasm_bytes = module.finish();

        let result = runtime
            .execute_function(
                &wasm_bytes,
                "main",
                vec![serde_json::json!(2), serde_json::json!(40)],
                1000,
            )
            .unwrap();
        assert_eq!(result.output, serde_json::json!({"result": 42}));

        let err = runtime
            .execute_function(&wasm_bytes, "main", vec![serde_json::json!(1)], 1000)
            .unwrap_err();
        assert!(err.to_string().contains("expects 2 arguments"));
    }

    #[test]
    fn test_simulation_supports_baals_runtime_import_family() {
        let config = Config::default();
        let runtime = WasmRuntime::new(&config).unwrap();

        use wasm_encoder::*;
        let mut module = Module::new();

        let mut types = TypeSection::new();
        // type 0: main () -> i64
        types.function([], [ValType::I64]);
        // type 1: () -> i64
        types.function([], [ValType::I64]);
        // type 2: (i64, i64) -> ()
        types.function([ValType::I64, ValType::I64], []);
        // type 3: (i64) -> i64
        types.function([ValType::I64], [ValType::I64]);
        // type 4: (i64, i64, i64) -> ()
        types.function([ValType::I64, ValType::I64, ValType::I64], []);
        // type 5: (i64, i64) -> i64
        types.function([ValType::I64, ValType::I64], [ValType::I64]);
        module.section(&types);

        let mut imports = ImportSection::new();
        imports.import("baals", "baals_get_sender", EntityType::Function(1));
        imports.import("baals", "baals_get_contract_id", EntityType::Function(1));
        imports.import(
            "baals",
            "baals_get_block_timestamp",
            EntityType::Function(1),
        );
        imports.import("baals", "baals_get_block_height", EntityType::Function(1));
        imports.import("baals", "baals_emit_event", EntityType::Function(2));
        imports.import("baals", "baals_hash_sha256", EntityType::Function(3));
        imports.import("baals", "baals_call_contract", EntityType::Function(4));
        imports.import("baals", "baals_read_call_result", EntityType::Function(5));
        imports.import("baals", "baals_transfer_value", EntityType::Function(2));
        module.section(&imports);

        let mut funcs = FunctionSection::new();
        funcs.function(0);
        module.section(&funcs);

        let mut exports = ExportSection::new();
        // main local function index = 9 imports + local offset 0
        exports.export("main", ExportKind::Func, 9);
        module.section(&exports);

        let mut codes = CodeSection::new();
        let mut func = Function::new(vec![(1, ValType::I64)]);
        func.instruction(&Instruction::Call(0)); // get_sender
        func.instruction(&Instruction::LocalSet(0));
        func.instruction(&Instruction::Call(1)); // get_contract_id
        func.instruction(&Instruction::Drop);
        func.instruction(&Instruction::Call(2)); // get_block_timestamp
        func.instruction(&Instruction::Drop);
        func.instruction(&Instruction::Call(3)); // get_block_height
        func.instruction(&Instruction::Drop);
        func.instruction(&Instruction::I64Const(123)); // event_name_hash
        func.instruction(&Instruction::I64Const(456)); // event_data_hash
        func.instruction(&Instruction::Call(4)); // emit_event
        func.instruction(&Instruction::LocalGet(0));
        func.instruction(&Instruction::Call(5)); // hash_sha256
        func.instruction(&Instruction::LocalSet(0));
        func.instruction(&Instruction::I64Const(0));
        func.instruction(&Instruction::I64Const(44));
        func.instruction(&Instruction::Call(7)); // read_call_result
        func.instruction(&Instruction::Drop);
        func.instruction(&Instruction::LocalGet(0));
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);

        let wasm_bytes = module.finish();
        let result = runtime
            .simulate(&wasm_bytes, serde_json::json!({}), 10_000)
            .unwrap();

        assert!(result.output["result"].as_i64().unwrap_or(0) != 0);
        assert_eq!(result.events.len(), 1);
    }

    #[test]
    fn test_simulation_supports_crypto_chrono_resurgence_import_families() {
        let config = Config::default();
        let runtime = WasmRuntime::new(&config).unwrap();

        use wasm_encoder::*;
        let mut module = Module::new();

        let mut types = TypeSection::new();
        // type 0: main () -> i64
        types.function([], [ValType::I64]);
        // type 1: (i64, i64, i64) -> i64
        types.function([ValType::I64, ValType::I64, ValType::I64], [ValType::I64]);
        // type 2: (i64) -> i64
        types.function([ValType::I64], [ValType::I64]);
        // type 3: (i64, i64) -> i64
        types.function([ValType::I64, ValType::I64], [ValType::I64]);
        // type 4: (i64, i64, i64) -> i64
        types.function([ValType::I64, ValType::I64, ValType::I64], [ValType::I64]);
        // type 5: (i64, i64, i64, i64) -> i64
        types.function(
            [ValType::I64, ValType::I64, ValType::I64, ValType::I64],
            [ValType::I64],
        );
        // type 6: (i64, i64, i64, i64) -> ()
        types.function([ValType::I64, ValType::I64, ValType::I64, ValType::I64], []);
        module.section(&types);

        let mut imports = ImportSection::new();
        imports.import("crypto", "crypto_verify_signature", EntityType::Function(1));
        imports.import("crypto", "crypto_decode_proof", EntityType::Function(2));
        imports.import(
            "chrononode",
            "chrononode_fetch_block",
            EntityType::Function(3),
        );
        imports.import(
            "chrononode",
            "chrononode_fetch_checkpoint",
            EntityType::Function(4),
        );
        imports.import(
            "chrononode",
            "chrononode_verify_proof",
            EntityType::Function(3),
        );
        imports.import(
            "chrononode",
            "chrononode_extract_event",
            EntityType::Function(3),
        );
        imports.import(
            "chrononode",
            "chrononode_extract_tx_by_sender",
            EntityType::Function(3),
        );
        imports.import(
            "chrononode",
            "chrononode_extract_tx_by_recipient",
            EntityType::Function(3),
        );
        imports.import(
            "chrononode",
            "chrononode_verify_archive_range",
            EntityType::Function(5),
        );
        imports.import(
            "resurgence",
            "resurgence_check_token_age",
            EntityType::Function(3),
        );
        imports.import(
            "resurgence",
            "resurgence_check_token_activity_window",
            EntityType::Function(4),
        );
        imports.import(
            "resurgence",
            "resurgence_check_liquidity_dormancy",
            EntityType::Function(3),
        );
        imports.import(
            "resurgence",
            "resurgence_check_governance_dormancy",
            EntityType::Function(3),
        );
        imports.import(
            "resurgence",
            "resurgence_calculate_dormancy_score",
            EntityType::Function(4),
        );
        imports.import(
            "resurgence",
            "resurgence_normalize_dead_coin_risk",
            EntityType::Function(2),
        );
        imports.import(
            "resurgence",
            "resurgence_generate_dormancy_proof",
            EntityType::Function(4),
        );
        imports.import(
            "resurgence",
            "resurgence_emit_dormancy_oracle_result",
            EntityType::Function(6),
        );
        module.section(&imports);

        let mut funcs = FunctionSection::new();
        funcs.function(0);
        module.section(&funcs);

        let mut exports = ExportSection::new();
        // 17 imports + local offset 0
        exports.export("main", ExportKind::Func, 17);
        module.section(&exports);

        let mut codes = CodeSection::new();
        let mut func = Function::new(vec![(1, ValType::I64)]);

        func.instruction(&Instruction::I64Const(1));
        func.instruction(&Instruction::I64Const(2));
        func.instruction(&Instruction::I64Const(3));
        func.instruction(&Instruction::Call(0)); // crypto_verify_signature
        func.instruction(&Instruction::LocalSet(0));

        func.instruction(&Instruction::I64Const(4));
        func.instruction(&Instruction::Call(1)); // crypto_decode_proof
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(5));
        func.instruction(&Instruction::I64Const(6));
        func.instruction(&Instruction::Call(2)); // chrononode_fetch_block
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(7));
        func.instruction(&Instruction::I64Const(8));
        func.instruction(&Instruction::I64Const(9));
        func.instruction(&Instruction::Call(3)); // chrononode_fetch_checkpoint
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(10));
        func.instruction(&Instruction::I64Const(11));
        func.instruction(&Instruction::Call(4)); // chrononode_verify_proof
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(12));
        func.instruction(&Instruction::I64Const(13));
        func.instruction(&Instruction::Call(5)); // chrononode_extract_event
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(14));
        func.instruction(&Instruction::I64Const(15));
        func.instruction(&Instruction::Call(6)); // chrononode_extract_tx_by_sender
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(16));
        func.instruction(&Instruction::I64Const(17));
        func.instruction(&Instruction::Call(7)); // chrononode_extract_tx_by_recipient
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(18));
        func.instruction(&Instruction::I64Const(19));
        func.instruction(&Instruction::I64Const(20));
        func.instruction(&Instruction::I64Const(21));
        func.instruction(&Instruction::Call(8)); // chrononode_verify_archive_range
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(22));
        func.instruction(&Instruction::I64Const(23));
        func.instruction(&Instruction::Call(9)); // resurgence_check_token_age
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(24));
        func.instruction(&Instruction::I64Const(25));
        func.instruction(&Instruction::I64Const(26));
        func.instruction(&Instruction::Call(10)); // resurgence_check_token_activity_window
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(27));
        func.instruction(&Instruction::I64Const(28));
        func.instruction(&Instruction::Call(11)); // resurgence_check_liquidity_dormancy
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(29));
        func.instruction(&Instruction::I64Const(30));
        func.instruction(&Instruction::Call(12)); // resurgence_check_governance_dormancy
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(31));
        func.instruction(&Instruction::I64Const(32));
        func.instruction(&Instruction::I64Const(33));
        func.instruction(&Instruction::Call(13)); // resurgence_calculate_dormancy_score
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(34));
        func.instruction(&Instruction::Call(14)); // resurgence_normalize_dead_coin_risk
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(35));
        func.instruction(&Instruction::I64Const(36));
        func.instruction(&Instruction::I64Const(37));
        func.instruction(&Instruction::Call(15)); // resurgence_generate_dormancy_proof
        func.instruction(&Instruction::Drop);

        func.instruction(&Instruction::I64Const(38));
        func.instruction(&Instruction::I64Const(39));
        func.instruction(&Instruction::I64Const(40));
        func.instruction(&Instruction::I64Const(41));
        func.instruction(&Instruction::Call(16)); // resurgence_emit_dormancy_oracle_result

        func.instruction(&Instruction::LocalGet(0));
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);

        let wasm_bytes = module.finish();
        let result = runtime
            .simulate(&wasm_bytes, serde_json::json!({}), 20_000)
            .unwrap();

        assert_eq!(result.output["result"].as_i64().unwrap_or(0), 0);
        assert_eq!(result.events.len(), 1);
    }

    #[test]
    fn test_simulation_traps_on_disabled_side_effect_host_calls() {
        let config = Config::default();
        let runtime = WasmRuntime::new(&config).unwrap();

        use wasm_encoder::*;
        let mut module = Module::new();
        let mut types = TypeSection::new();
        // type 0: main () -> i64
        types.function([], [ValType::I64]);
        // type 1: (i64, i64, i64) -> ()
        types.function([ValType::I64, ValType::I64, ValType::I64], []);
        module.section(&types);

        let mut imports = ImportSection::new();
        imports.import("baals", "baals_call_contract", EntityType::Function(1));
        module.section(&imports);

        let mut funcs = FunctionSection::new();
        funcs.function(0);
        module.section(&funcs);

        let mut exports = ExportSection::new();
        // one import + local function
        exports.export("main", ExportKind::Func, 1);
        module.section(&exports);

        let mut codes = CodeSection::new();
        let mut func = Function::new(vec![]);
        func.instruction(&Instruction::I64Const(1));
        func.instruction(&Instruction::I64Const(2));
        func.instruction(&Instruction::I64Const(3));
        func.instruction(&Instruction::Call(0));
        func.instruction(&Instruction::I64Const(42));
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);

        let wasm_bytes = module.finish();
        let result = runtime.simulate(&wasm_bytes, serde_json::json!({}), 10_000);
        assert!(result.is_err());
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
