//! Canvas Contracts - Main Application Entry Point

use clap::{Parser, Subcommand};
use log::{error, info};

use canvas_contracts::{
    compiler::Compiler,
    config::ConfigManager,
    error::{CanvasError, CanvasResult},
    init, info as lib_info,
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
        #[arg(short, long)]
        optimize: bool,
    },

    /// Run a contract simulation
    Simulate {
        /// Contract WASM file
        #[arg(short, long)]
        contract: String,

        /// Input data file (JSON)
        #[arg(short, long)]
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
}

fn main() -> CanvasResult<()> {
    let cli = Cli::parse();

    // Initialize the library
    init()?;

    // Set up logging
    let log_level = if cli.debug {
        "debug"
    } else {
        &cli.log_level
    };
    std::env::set_var("RUST_LOG", log_level);
    env_logger::init();

    info!("Starting Canvas Contracts v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config_path = std::path::PathBuf::from(&cli.config);
    let mut config_manager = ConfigManager::new(config_path)?;

    match &cli.command {
        Some(Commands::Compile { input, output, optimize }) => {
            compile_contract(input, output, *optimize, &config_manager)?
        }

        Some(Commands::Simulate { contract, input, gas_limit }) => {
            simulate_contract(contract, input.as_deref(), *gas_limit, &config_manager)?
        }

        Some(Commands::Deploy { contract, args, key }) => {
            deploy_contract(contract, args.as_deref(), key, &config_manager)?
        }

        Some(Commands::Editor { port, host }) => {
            start_editor(*port, host, &config_manager)?
        }

        Some(Commands::Info) => {
            show_info()?
        }

        Some(Commands::Validate { input }) => {
            validate_graph(input, &config_manager)?
        }

        None => {
            // Default: start the visual editor
            start_editor(3000, "localhost", &config_manager)?
        }
    }

    Ok(())
}

fn compile_contract(
    input: &str,
    output: &str,
    optimize: bool,
    config_manager: &ConfigManager,
) -> CanvasResult<()> {
    info!("Compiling contract from {} to {}", input, output);

    // Load the visual graph
    let graph_content = std::fs::read_to_string(input)
        .map_err(|e| CanvasError::Io(e))?;

    let graph: canvas_contracts::types::VisualGraph = serde_json::from_str(&graph_content)
        .map_err(|e| CanvasError::Serialization(e))?;

    // Create compiler
    let compiler = Compiler::new(config_manager.config())?;

    // Compile the graph
    let result = compiler.compile(&graph)?;

    // Write WASM output
    std::fs::write(output, &result.wasm_bytes)
        .map_err(|e| CanvasError::Io(e))?;

    // Write ABI
    let abi_path = output.replace(".wasm", ".abi.json");
    let abi_content = serde_json::to_string_pretty(&result.abi)
        .map_err(|e| CanvasError::Serialization(e))?;
    std::fs::write(&abi_path, abi_content)
        .map_err(|e| CanvasError::Io(e))?;

    info!("Compilation successful!");
    info!("WASM file: {}", output);
    info!("ABI file: {}", abi_path);
    info!("Gas estimate: {}", result.gas_estimate);

    if !result.warnings.is_empty() {
        info!("Warnings:");
        for warning in &result.warnings {
            info!("  - {}", warning);
        }
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
    let wasm_bytes = std::fs::read(contract)
        .map_err(|e| CanvasError::Io(e))?;

    // Load input data if provided
    let input_data = if let Some(input_file) = input {
        let content = std::fs::read_to_string(input_file)
            .map_err(|e| CanvasError::Io(e))?;
        serde_json::from_str(&content)
            .map_err(|e| CanvasError::Serialization(e))?
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
            info!("  - {}: {}", event.name, serde_json::to_string_pretty(&event.data)?);
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
    let wasm_bytes = std::fs::read(contract)
        .map_err(|e| CanvasError::Io(e))?;

    // Load private key
    let key_content = std::fs::read_to_string(key)
        .map_err(|e| CanvasError::Io(e))?;
    let private_key = key_content.trim();

    // Parse constructor arguments
    let constructor_args = if let Some(args_str) = args {
        serde_json::from_str(args_str)
            .map_err(|e| CanvasError::Serialization(e))?
    } else {
        serde_json::Value::Null
    };

    // Create BaaLS client
    let baals_client = canvas_contracts::baals::BaalsClient::new(config_manager.config())?;

    // Deploy contract
    let deployment_result = baals_client.deploy_contract(
        &wasm_bytes,
        constructor_args,
        private_key,
    )?;

    info!("Deployment successful!");
    info!("Contract address: {}", deployment_result.contract_address);
    info!("Transaction hash: {}", deployment_result.transaction_hash);
    info!("Gas used: {}", deployment_result.gas_used);

    Ok(())
}

fn start_editor(
    port: u16,
    host: &str,
    config_manager: &ConfigManager,
) -> CanvasResult<()> {
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

fn validate_graph(
    input: &str,
    config_manager: &ConfigManager,
) -> CanvasResult<()> {
    info!("Validating graph: {}", input);

    // Load the visual graph
    let graph_content = std::fs::read_to_string(input)
        .map_err(|e| CanvasError::Io(e))?;

    let graph: canvas_contracts::types::VisualGraph = serde_json::from_str(&graph_content)
        .map_err(|e| CanvasError::Serialization(e))?;

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
        return Err(CanvasError::Validation("Graph validation failed".to_string()));
    }

    Ok(())
} 