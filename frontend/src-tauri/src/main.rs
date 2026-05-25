#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use canvas_contracts::{
    Compiler, WasmRuntime, create_client, BaalsClient,
    artifact::hash::{canonical_graph_hash, hash_bytes_prefixed, GRAPH_CANONICALIZATION},
    nodes::builtin_node_definitions,
    types::VisualGraph,
    error::CanvasResult,
    debugger::DebugSession,
    nodes::NodeRegistry,
    adapter::ChronoNodeClient,
};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, PoisonError};
use tauri::State;

struct AppState {
    compiler: Mutex<Option<Compiler>>,
    runtime: Mutex<Option<WasmRuntime>>,
    baals_client: Mutex<Option<Box<dyn BaalsClient>>>,
    chrononode_client: Mutex<Option<Box<dyn ChronoNodeClient>>>,
}

fn lock_compiler(state: &AppState) -> Result<std::sync::MutexGuard<'_, Option<Compiler>>, String> {
    state.compiler.lock().map_err(|_| "Internal error: compiler mutex poisoned".to_string())
}

fn lock_client(state: &AppState) -> Result<std::sync::MutexGuard<'_, Option<Box<dyn BaalsClient>>>, String> {
    state.baals_client.lock().map_err(|_| "Internal error: BaaLS client mutex poisoned".to_string())
}

fn lock_mut<T>(m: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, String> {
    m.lock().map_err(|_| "Internal error: mutex poisoned".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
struct CompileRequest {
    graph: VisualGraph,
    optimization_level: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct CompileResponse {
    success: bool,
    wasm_size: usize,
    wasm_bytes: String, // base64-encoded
    gas_estimate: u64,
    error: Option<String>,
}

#[tauri::command]
async fn compile_contract(
    state: State<'_, AppState>,
    request: CompileRequest,
) -> Result<CompileResponse, String> {
    let compiler = lock_compiler(&state)?;
    let compiler = compiler.as_ref().ok_or("Compiler not initialized")?;

    match compiler.compile(&request.graph) {
        Ok(result) => {
            let wasm_bytes_hex = hex::encode(&result.wasm_bytes);
            Ok(CompileResponse {
                success: true,
                wasm_size: result.wasm_bytes.len(),
                wasm_bytes: wasm_bytes_hex,
                gas_estimate: result.gas_estimate,
                error: None,
            })
        }
        Err(e) => Ok(CompileResponse {
            success: false,
            wasm_size: 0,
            wasm_bytes: String::new(),
            gas_estimate: 0,
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
async fn validate_graph(
    state: State<'_, AppState>,
    graph: VisualGraph,
) -> Result<serde_json::Value, String> {
    let compiler = lock_compiler(&state)?;
    let compiler = compiler.as_ref().ok_or("Compiler not initialized")?;

    let validator = compiler.validator()?;
    let result = validator.validate(&graph)?;

    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[derive(Debug, Serialize, Deserialize)]
struct DeployRequest {
    wasm_bytes: Vec<u8>,
    private_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeployResponse {
    success: bool,
    contract_address: String,
    transaction_hash: String,
    gas_used: u64,
    error: Option<String>,
}

#[tauri::command]
async fn deploy_contract(
    state: State<'_, AppState>,
    request: DeployRequest,
) -> Result<DeployResponse, String> {
    let client = lock_client(&state)?;
    let client = client.as_ref().ok_or("BaaLS client not initialized")?;

    match client.deploy_contract(&request.wasm_bytes, serde_json::json!({}), &request.private_key) {
        Ok(result) => Ok(DeployResponse {
            success: true,
            contract_address: result.contract_address,
            transaction_hash: result.transaction_hash,
            gas_used: result.gas_used,
            error: None,
        }),
        Err(e) => Ok(DeployResponse {
            success: false,
            contract_address: String::new(),
            transaction_hash: String::new(),
            gas_used: 0,
            error: Some(e.to_string()),
        }),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DebugStartResponse {
    success: bool,
    state: String,
    total_steps: usize,
    current_step: usize,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DebugStepResponse {
    success: bool,
    state: String,
    current_step: usize,
    total_steps: usize,
    variables: std::collections::HashMap<String, serde_json::Value>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DebugTraceResponse {
    success: bool,
    trace: Vec<ExecutionStep>,
    total_gas: u64,
    success_at_end: bool,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NodeDefinitionResponse {
    id: String,
    name: String,
    description: String,
    category: String,
    inputs: Vec<PortInfo>,
    outputs: Vec<PortInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PortInfo {
    name: String,
    value_type: String,
    required: bool,
}

#[tauri::command]
async fn get_node_definitions() -> Result<Vec<NodeDefinitionResponse>, String> {
    let definitions = builtin_node_definitions();
    let response: Vec<NodeDefinitionResponse> = definitions.iter().map(|d| {
        NodeDefinitionResponse {
            id: d.id.clone(),
            name: d.name.clone(),
            description: d.description.clone(),
            category: d.category.clone(),
            inputs: d.inputs.iter().map(|p| PortInfo {
                name: p.name.clone(),
                value_type: format!("{:?}", p.value_type),
                required: p.required,
            }).collect(),
            outputs: d.outputs.iter().map(|p| PortInfo {
                name: p.name.clone(),
                value_type: format!("{:?}", p.value_type),
                required: p.required,
            }).collect(),
        }
    }).collect();
    Ok(response)
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            compiler: Mutex::new(None),
            runtime: Mutex::new(None),
            baals_client: Mutex::new(None),
            chrononode_client: Mutex::new(None),
        })
        .setup(|app| {
            let config = canvas_contracts::config::Config::default();

            if let Ok(compiler) = Compiler::new(&config) {
                *lock_mut(&app.state::<AppState>().compiler).unwrap_or_else(|_| panic!("lock")) = Some(compiler);
            }
            if let Ok(runtime) = WasmRuntime::new(&config) {
                *lock_mut(&app.state::<AppState>().runtime).unwrap_or_else(|_| panic!("lock")) = Some(runtime);
            }
            if let Ok(client) = create_client(&config) {
                *lock_mut(&app.state::<AppState>().baals_client).unwrap_or_else(|_| panic!("lock")) = Some(client);
            }
            if let Ok(client) = canvas_contracts::adapter::create_chrononode_client(&config) {
                *lock_mut(&app.state::<AppState>().chrononode_client).unwrap_or_else(|_| panic!("lock")) = Some(client);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            compile_contract,
            validate_graph,
            get_node_definitions,
            deploy_contract,
            debug_start,
            debug_step,
            debug_continue,
            debug_get_trace,
            query_history,
            verify_proof,
            export_audit_bundle,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Debug commands - using a separate mutex-guarded session
static DEBUG_SESSION: std::sync::OnceLock<Mutex<Option<DebugSession>>> = std::sync::OnceLock::new();

fn get_debug_session() -> &'static Mutex<Option<DebugSession>> {
    DEBUG_SESSION.get_or_init(|| Mutex::new(None))
}

#[tauri::command]
fn debug_start(graph: VisualGraph) -> Result<DebugStartResponse, String> {
    let mut registry = NodeRegistry::new();
    for def in builtin_node_definitions() {
        registry.register_node(def);
    }
    
    let session = DebugSession::new(graph, registry);
    let mut guard = get_debug_session().lock().map_err(|_| "Debug mutex poisoned")?;
    *guard = Some(session);
    
    Ok(DebugStartResponse {
        success: true,
        state: "paused".to_string(),
        total_steps: 0,
        current_step: 0,
        error: None,
    })
}

#[tauri::command]
fn debug_step() -> Result<DebugStepResponse, String> {
    let guard = get_debug_session().lock().map_err(|_| "Debug mutex poisoned")?;
    let session = guard.as_ref().ok_or("Debug session not started")?;
    
    let trace = session.get_trace();
    let current = session.get_variables();
    
    Ok(DebugStepResponse {
        success: true,
        state: "stepping".to_string(),
        current_step: trace.len(),
        total_steps: trace.len(),
        variables: current.clone(),
        error: None,
    })
}

#[tauri::command]
fn debug_continue() -> Result<DebugStepResponse, String> {
    let guard = get_debug_session().lock().map_err(|_| "Debug mutex poisoned")?;
    let session = guard.as_ref().ok_or("Debug session not started")?;
    
    let trace = session.get_trace();
    let current = session.get_variables();
    
    Ok(DebugStepResponse {
        success: true,
        state: "finished".to_string(),
        current_step: trace.len(),
        total_steps: trace.len(),
        variables: current.clone(),
        error: None,
    })
}

#[tauri::command]
fn debug_get_trace() -> Result<DebugTraceResponse, String> {
    let guard = get_debug_session().lock().map_err(|_| "Debug mutex poisoned")?;
    let session = guard.as_ref().ok_or("Debug session not started")?;
    
    let trace = session.get_trace();
    let total_gas: u64 = trace.iter().map(|s| s.gas_consumed).sum();
    
    Ok(DebugTraceResponse {
        success: true,
        trace: trace.to_vec(),
        total_gas,
        success_at_end: trace.iter().all(|s| s.error.is_none()),
        error: None,
    })
}

#[tauri::command]
async fn query_history(
    state: State<'_, AppState>,
    contract_address: String,
    query_type: String,
) -> Result<serde_json::Value, String> {
    let client_guard = lock_mut(&state.chrononode_client)?;
    let client = client_guard.as_ref().ok_or("ChronoNode client not initialized")?;

    match query_type.as_str() {
        "blocks" => {
            client.get_block_range("baals-local", 0, 100).map_err(|e| e.to_string())
        }
        "transactions" => {
            client.get_tx_by_sender("baals-local", &contract_address).map_err(|e| e.to_string())
        }
        "events" => {
            client.get_events("baals-local", "DormancyOracleResult").map_err(|e| e.to_string())
        }
        _ => Err("Invalid query type".to_string()),
    }
}

#[tauri::command]
async fn verify_proof(
    state: State<'_, AppState>,
    proof: serde_json::Value,
) -> Result<bool, String> {
    let client_guard = lock_mut(&state.chrononode_client)?;
    let client = client_guard.as_ref().ok_or("ChronoNode client not initialized")?;

    client.verify_proof(proof).map_err(|e| e.to_string())
}

#[tauri::command]
async fn export_audit_bundle(
    state: State<'_, AppState>,
    graph: VisualGraph,
) -> Result<serde_json::Value, String> {
    let compiler_guard = lock_mut(&state.compiler)?;
    let compiler = compiler_guard.as_ref().ok_or("Compiler not initialized")?;

    let result = compiler.compile(&graph).map_err(|e| e.to_string())?;

    // Create a mock validation report
    let validator = compiler.validator().map_err(|e| e.to_string())?;
    let val_res = validator.validate(&graph).map_err(|e| e.to_string())?;

    let graph_hash = canonical_graph_hash(&graph).map_err(|e| e.to_string())?;
    let wasm_hash = hash_bytes_prefixed(&result.wasm_bytes);
    let mut node_types: Vec<String> = graph.nodes.iter().map(|n| n.node_type.clone()).collect();
    node_types.sort();
    node_types.dedup();

    let lock_json = serde_json::json!({
        "schema_version": "canvas.graph.lock.v1",
        "graph_schema_version": graph.schema_version,
        "project_name": graph.name,
        "target_adapter": graph.metadata.get("target_adapter").cloned().unwrap_or_else(|| "baals".to_string()),
        "graph_canonicalization": GRAPH_CANONICALIZATION,
        "node_count": graph.nodes.len(),
        "connection_count": graph.connections.len(),
        "node_types": node_types,
        "gas_estimate": result.gas_estimate,
        "compiler": {
            "name": env!("CARGO_PKG_NAME"),
            "version": canvas_contracts::VERSION,
            "wasm_target": "wasm32-unknown-unknown",
            "wasm_encoder_version": "0.38",
            "wasmtime_validation_version": "43.0.1"
        },
        "graph_hash": graph_hash,
        "wasm_hash": wasm_hash
    });

    let val_json = serde_json::json!({
        "schema_version": "canvas.validation.v1",
        "graph_schema_version": graph.schema_version,
        "graph_hash": canonical_graph_hash(&graph).map_err(|e| e.to_string())?,
        "graph_canonicalization": GRAPH_CANONICALIZATION,
        "target_adapter": graph.metadata.get("target_adapter").cloned().unwrap_or_else(|| "baals".to_string()),
        "node_count": graph.nodes.len(),
        "connection_count": graph.connections.len(),
        "is_valid": val_res.is_valid,
        "errors": val_res.errors,
        "warnings": val_res.warnings
    });

    Ok(serde_json::json!({
        "wasm_bytes": hex::encode(result.wasm_bytes),
        "abi": result.abi,
        "lock": lock_json,
        "validation_report": val_json
    }))
}
