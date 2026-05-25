//! Canvas Contracts - Main Application Entry Point

use clap::{Parser, Subcommand};
use log::{error, info};

use canvas_contracts::{
    artifact::{build_artifact_bundle, verify_artifact_manifest},
    compiler::{Compiler, GraphExecutor},
    config::ConfigManager,
    error::{CanvasError, CanvasResult},
    info as lib_info, init,
    nodes::{builtin_node_definitions, NodeRegistry},
    types::{ExecutionContext, VisualGraph},
};

#[derive(Parser)]
#[command(name = "canvas-contracts")]
#[command(about = "Visual Smart Contract Development Platform")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a visual contract to WASM
    Compile {
        /// Input graph file
        #[arg(short, long)]
        input: String,

        /// Output WASM file
        #[arg(short, long)]
        output: String,

        /// Enable optimization
        #[arg(short = 'O', long)]
        optimize: bool,
    },

    /// Run a contract simulation
    Simulate {
        /// Contract WASM file
        #[arg(short, long)]
        contract: Option<String>,

        /// Graph JSON file (compile + simulate in one step)
        #[arg(long)]
        graph: Option<String>,

        /// Input data file (JSON)
        #[arg(short = 'd', long)]
        input: Option<String>,

        /// Gas limit
        #[arg(short, long, default_value = "1000000")]
        gas_limit: u64,
    },

    /// Deploy a contract to BaaLS
    Deploy {
        /// Contract WASM file
        #[arg(short, long)]
        contract: String,

        /// Constructor arguments (JSON)
        #[arg(short, long)]
        args: Option<String>,

        /// Private key file
        #[arg(short, long)]
        key: String,
    },

    /// Start the visual editor
    Editor {
        /// Port for the editor server
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Host address
        #[arg(long, default_value = "localhost")]
        host: String,
    },

    /// Show application information
    Info,

    /// Validate a visual graph
    Validate {
        /// Input graph file
        #[arg(short, long)]
        input: String,
    },

    /// Build and verify contract artifacts
    Artifact {
        #[command(subcommand)]
        action: ArtifactCommands,
    },
}

#[derive(Subcommand)]
enum ArtifactCommands {
    /// Build a verifiable artifact bundle from a graph
    Build {
        /// Input graph file
        #[arg(short, long)]
        input: String,

        /// Output directory for bundle files
        #[arg(short, long)]
        out: String,
    },
    /// Verify artifact manifest hashes against local files
    Verify {
        /// Path to canvas.contract.json
        #[arg(short, long)]
        manifest: String,
    },
}

fn main() -> CanvasResult<()> {
    let cli = Cli::parse();

    // Set up logging first, before anything that might log
    let log_level = if cli.debug { "debug" } else { &cli.log_level };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    // Initialize the library (no longer calls env_logger::init)
    init()?;

    info!("Starting Canvas Contracts v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config_path = std::path::PathBuf::from(&cli.config);
    let config_manager = ConfigManager::new(config_path)?;

    match &cli.command {
        Some(Commands::Compile {
            input,
            output,
            optimize,
        }) => compile_contract(input, output, *optimize, &config_manager)?,

        Some(Commands::Simulate {
            contract,
            graph,
            input,
            gas_limit,
        }) => {
            if let Some(graph_path) = graph {
                simulate_graph(graph_path, input.as_deref(), *gas_limit, &config_manager)?
            } else if let Some(contract_path) = contract {
                simulate_contract(contract_path, input.as_deref(), *gas_limit, &config_manager)?
            } else {
                return Err(CanvasError::Config(
                    "Either --contract or --graph must be provided".to_string(),
                ));
            }
        }

        Some(Commands::Deploy {
            contract,
            args,
            key,
        }) => deploy_contract(contract, args.as_deref(), key, &config_manager)?,

        Some(Commands::Editor { port, host }) => start_editor(*port, host, &config_manager)?,

        Some(Commands::Info) => show_info()?,

        Some(Commands::Validate { input }) => validate_graph(input, &config_manager)?,

        Some(Commands::Artifact { action }) => match action {
            ArtifactCommands::Build { input, out } => {
                build_artifact(input, out, &config_manager)?;
            }
            ArtifactCommands::Verify { manifest } => {
                verify_artifact(manifest, &config_manager)?;
            }
        },

        None => {
            // Default: start the visual editor
            start_editor(3000, "localhost", &config_manager)?
        }
    }

    Ok(())
}

fn build_artifact(input: &str, out: &str, config_manager: &ConfigManager) -> CanvasResult<()> {
    info!("Building artifact bundle from {} into {}", input, out);

    let graph_content = std::fs::read_to_string(input).map_err(CanvasError::Io)?;
    let graph: VisualGraph =
        serde_json::from_str(&graph_content).map_err(CanvasError::Serialization)?;

    let output = build_artifact_bundle(&graph, config_manager.config(), std::path::Path::new(out))?;

    info!("Artifact bundle built successfully");
    info!("Graph file: {}", output.graph_path.display());
    info!(
        "Canonical graph file: {}",
        output.canonical_graph_path.display()
    );
    info!(
        "Node-pack lock file: {}",
        output.node_pack_lock_path.display()
    );
    info!("WASM file: {}", output.wasm_path.display());
    info!("ABI file: {}", output.abi_path.display());
    info!("WIT directory: {}", output.wit_dir.display());
    info!(
        "Safety report file: {}",
        output.safety_report_path.display()
    );
    info!("Manifest file: {}", output.manifest_path.display());
    Ok(())
}

fn verify_artifact(manifest: &str, _config_manager: &ConfigManager) -> CanvasResult<()> {
    info!("Verifying artifact manifest {}", manifest);
    let result = verify_artifact_manifest(std::path::Path::new(manifest))?;
    info!("Artifact verification status: {}", result.status);
    info!("Graph hash: {}", result.graph_hash);
    info!("Node-pack lock hash: {}", result.node_pack_lock_hash);
    info!("WASM hash: {}", result.wasm_hash);
    info!("WIT hash: {}", result.wit_hash);
    info!("ABI hash: {}", result.json_abi_hash);
    info!("Safety report hash: {}", result.safety_report_hash);
    Ok(())
}

fn compile_contract(
    input: &str,
    output: &str,
    _optimize: bool,
    config_manager: &ConfigManager,
) -> CanvasResult<()> {
    info!("Compiling contract from {} to {}", input, output);

    // Load the visual graph
    let graph_content = std::fs::read_to_string(input).map_err(CanvasError::Io)?;

    let graph: canvas_contracts::types::VisualGraph =
        serde_json::from_str(&graph_content).map_err(CanvasError::Serialization)?;

    // Create compiler
    let compiler = Compiler::new(config_manager.config())?;

    // Validate before generating artifacts
    let validator = compiler.validator()?;
    let val_res = validator.validate(&graph)?;
    let val_path = output.replace(".wasm", ".validation-report.json");
    let val_json = serde_json::json!({
        "schema_version": "canvas.validation.v1",
        "is_valid": val_res.is_valid,
        "errors": val_res.errors.clone(),
        "warnings": val_res.warnings.clone()
    });
    std::fs::write(&val_path, serde_json::to_string_pretty(&val_json)?).map_err(CanvasError::Io)?;

    if !val_res.is_valid {
        return Err(CanvasError::Validation(format!(
            "Graph validation failed with {} error(s). See {} for details.",
            val_res.errors.len(),
            val_path
        )));
    }

    // Compile the graph
    let result = compiler.compile(&graph)?;

    // Write WASM output
    std::fs::write(output, &result.wasm_bytes).map_err(CanvasError::Io)?;

    // Write ABI
    let abi_path = output.replace(".wasm", ".abi.json");
    let abi_content =
        serde_json::to_string_pretty(&result.abi).map_err(CanvasError::Serialization)?;
    std::fs::write(&abi_path, abi_content).map_err(CanvasError::Io)?;

    // Calculate hashes for lockfile
    use sha2::{Digest, Sha256};
    let graph_hash = hex::encode(Sha256::digest(graph_content.as_bytes()));
    let wasm_hash = hex::encode(Sha256::digest(&result.wasm_bytes));

    // Write graph.lock.json
    let lock_path = output.replace(".wasm", ".lock.json");
    let target_adapter = graph
        .metadata
        .get("target_adapter")
        .cloned()
        .unwrap_or_else(|| "baals".to_string());
    let lock_json = serde_json::json!({
        "schema_version": "canvas.graph.v1",
        "project_name": graph.name,
        "target_adapter": target_adapter,
        "compiler": {
            "version": env!("CARGO_PKG_VERSION"),
            "wasm_target": "wasm32-unknown-unknown"
        },
        "graph_hash": format!("sha256:{}", graph_hash),
        "wasm_hash": format!("sha256:{}", wasm_hash)
    });
    std::fs::write(&lock_path, serde_json::to_string_pretty(&lock_json)?)
        .map_err(CanvasError::Io)?;

    info!("Compilation successful!");
    info!("WASM file: {}", output);
    info!("ABI file: {}", abi_path);
    info!("Lock file: {}", lock_path);
    info!("Validation report file: {}", val_path);
    info!("Gas estimate: {}", result.gas_estimate);

    if !result.warnings.is_empty() {
        info!("Warnings:");
        for warning in &result.warnings {
            info!("  - {}", warning);
        }
    }

    Ok(())
}

fn simulate_graph(
    graph_path: &str,
    _input: Option<&str>,
    gas_limit: u64,
    _config_manager: &ConfigManager,
) -> CanvasResult<()> {
    info!("Simulating graph: {}", graph_path);

    // Load the visual graph
    let graph_content = std::fs::read_to_string(graph_path).map_err(CanvasError::Io)?;
    let graph: VisualGraph =
        serde_json::from_str(&graph_content).map_err(CanvasError::Serialization)?;

    // Build a fully populated node registry
    let mut registry = NodeRegistry::new();
    for def in builtin_node_definitions() {
        registry.register_node(def);
    }

    // Create executor and execution context
    let executor = GraphExecutor::new(registry);
    let context = ExecutionContext::new(gas_limit);

    // Execute the graph
    let (trace, _final_context) = executor.execute(&graph, context)?;

    // Print execution trace
    info!("Execution completed!");
    info!("Success: {}", trace.success);
    info!("Total gas used: {}", trace.total_gas);
    info!("Steps: {}", trace.steps.len());

    for step in &trace.steps {
        info!(
            "  Step {}: {} ({})",
            step.step_number, step.node_type, step.node_id
        );
        info!(
            "    Gas: {}, Duration: {}ms",
            step.gas_consumed, step.duration_ms
        );
        if let Some(err) = &step.error {
            info!("    Error: {}", err);
        }
    }

    if let Some(err) = &trace.error {
        error!("Execution failed: {}", err);
        return Err(CanvasError::ExecutionError(err.clone()));
    }

    Ok(())
}

fn simulate_contract(
    contract: &str,
    input: Option<&str>,
    gas_limit: u64,
    config_manager: &ConfigManager,
) -> CanvasResult<()> {
    info!("Simulating contract: {}", contract);

    // Load WASM bytes
    let wasm_bytes = std::fs::read(contract).map_err(CanvasError::Io)?;

    // Load input data if provided
    let input_data = if let Some(input_file) = input {
        let content = std::fs::read_to_string(input_file).map_err(CanvasError::Io)?;
        serde_json::from_str(&content).map_err(CanvasError::Serialization)?
    } else {
        serde_json::Value::Null
    };

    // Create runtime
    let runtime = canvas_contracts::wasm::WasmRuntime::new(config_manager.config())?;

    // Simulate execution
    let result = runtime.simulate(&wasm_bytes, input_data, gas_limit)?;

    info!("Simulation completed!");
    info!("Gas used: {}", result.gas_used);
    info!("Output: {}", serde_json::to_string_pretty(&result.output)?);

    if !result.events.is_empty() {
        info!("Events emitted:");
        for event in &result.events {
            info!(
                "  - {}: {}",
                event.name,
                serde_json::to_string_pretty(&event.data)?
            );
        }
    }

    Ok(())
}

fn deploy_contract(
    contract: &str,
    args: Option<&str>,
    key: &str,
    config_manager: &ConfigManager,
) -> CanvasResult<()> {
    info!("Deploying contract: {}", contract);

    // Load WASM bytes
    let wasm_bytes = std::fs::read(contract).map_err(CanvasError::Io)?;

    // Load private key
    let key_content = std::fs::read_to_string(key).map_err(CanvasError::Io)?;
    let private_key = key_content.trim();

    // Parse constructor arguments
    let constructor_args = if let Some(args_str) = args {
        serde_json::from_str(args_str).map_err(CanvasError::Serialization)?
    } else {
        serde_json::Value::Null
    };

    // Create BaaLS client
    let baals_client = canvas_contracts::baals::create_client(config_manager.config())?;

    // Deploy contract
    let deployment_result =
        baals_client.deploy_contract(&wasm_bytes, constructor_args, private_key)?;

    info!("Deployment successful!");
    info!("Contract address: {}", deployment_result.contract_address);
    info!("Transaction hash: {}", deployment_result.transaction_hash);
    info!("Gas used: {}", deployment_result.gas_used);

    Ok(())
}

fn start_editor(port: u16, host: &str, config_manager: &ConfigManager) -> CanvasResult<()> {
    info!("Starting visual editor on {}:{}", host, port);

    // This would start the web-based editor
    // For now, we'll just print a message
    info!("Visual editor would start here");
    info!("Please implement the editor frontend");
    info!("Config: {:?}", config_manager.config().app);

    // In a real implementation, this would:
    // 1. Start a web server
    // 2. Serve the React frontend
    // 3. Handle WebSocket connections for real-time updates
    // 4. Provide API endpoints for compilation and simulation

    Ok(())
}

fn show_info() -> CanvasResult<()> {
    let info = lib_info();
    println!("Canvas Contracts");
    println!("===============");
    println!("Name: {}", info.name);
    println!("Version: {}", info.version);
    println!("Description: {}", info.description);
    println!();
    println!("Features:");
    println!("  - Visual smart contract development");
    println!("  - WASM compilation pipeline");
    println!("  - BaaLS integration");
    println!("  - Real-time simulation");
    println!("  - Cross-platform support");

    Ok(())
}

fn validate_graph(input: &str, config_manager: &ConfigManager) -> CanvasResult<()> {
    info!("Validating graph: {}", input);

    // Load the visual graph
    let graph_content = std::fs::read_to_string(input).map_err(CanvasError::Io)?;

    let graph: canvas_contracts::types::VisualGraph =
        serde_json::from_str(&graph_content).map_err(CanvasError::Serialization)?;

    // Create validator
    let validator = canvas_contracts::compiler::Validator::new(config_manager.config())?;

    // Validate the graph
    let validation_result = validator.validate(&graph)?;

    if validation_result.is_valid {
        info!("Graph validation successful!");
        if !validation_result.warnings.is_empty() {
            info!("Warnings:");
            for warning in &validation_result.warnings {
                info!("  - {}", warning);
            }
        }
    } else {
        error!("Graph validation failed!");
        for error in &validation_result.errors {
            error!("  - {}", error);
        }
        return Err(CanvasError::Validation(
            "Graph validation failed".to_string(),
        ));
    }

    Ok(())
}
