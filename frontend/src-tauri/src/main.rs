#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use canvas_contracts::{
    Compiler, WasmRuntime, create_client, BaalsClient,
    nodes::builtin_node_definitions,
    types::{VisualGraph, CompilationResult},
    error::CanvasResult,
};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, PoisonError};
use tauri::State;

struct AppState {
    compiler: Mutex<Option<Compiler>>,
    runtime: Mutex<Option<WasmRuntime>>,
    baals_client: Mutex<Option<Box<dyn BaalsClient>>>,
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
        Ok(result) => Ok(CompileResponse {
            success: true,
            wasm_size: result.wasm_bytes.len(),
            gas_estimate: result.gas_estimate,
            error: None,
        }),
        Err(e) => Ok(CompileResponse {
            success: false,
            wasm_size: 0,
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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            compile_contract,
            validate_graph,
            get_node_definitions,
            deploy_contract,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
