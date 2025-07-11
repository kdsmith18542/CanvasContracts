// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use canvas_contracts::{
    Compiler, WasmRuntime, BaalsClient, AiAssistant,
    types::{VisualGraph, CompilationResult},
    error::CanvasResult,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

// App state
struct AppState {
    compiler: Mutex<Option<Compiler>>,
    runtime: Mutex<Option<WasmRuntime>>,
    baals_client: Mutex<Option<BaalsClient>>,
    ai_assistant: Mutex<Option<AiAssistant>>,
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
    let compiler = state.compiler.lock().unwrap();
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
    let compiler = state.compiler.lock().unwrap();
    let compiler = compiler.as_ref().ok_or("Compiler not initialized")?;
    
    let validator = compiler.validator()?;
    let result = validator.validate(&graph)?;
    
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn analyze_patterns(
    state: State<'_, AppState>,
    graph: VisualGraph,
) -> Result<serde_json::Value, String> {
    let ai = state.ai_assistant.lock().unwrap();
    let ai = ai.as_ref().ok_or("AI Assistant not initialized")?;
    
    let analysis = ai.analyze_patterns(&graph)?;
    Ok(serde_json::to_value(analysis).map_err(|e| e.to_string())?)
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            compiler: Mutex::new(None),
            runtime: Mutex::new(None),
            baals_client: Mutex::new(None),
            ai_assistant: Mutex::new(None),
        })
        .setup(|app| {
            // Initialize canvas-contracts components
            let config = canvas_contracts::config::Config::default();
            
            if let Ok(compiler) = Compiler::new(&config) {
                *app.state::<AppState>().compiler.lock().unwrap() = Some(compiler);
            }
            
            if let Ok(runtime) = WasmRuntime::new(&config) {
                *app.state::<AppState>().runtime.lock().unwrap() = Some(runtime);
            }
            
            if let Ok(client) = BaalsClient::new(&config) {
                *app.state::<AppState>().baals_client.lock().unwrap() = Some(client);
            }
            
            if let Ok(ai) = AiAssistant::new(&config) {
                *app.state::<AppState>().ai_assistant.lock().unwrap() = Some(ai);
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            compile_contract,
            validate_graph,
            analyze_patterns,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
} 