//! Canvas Contracts - Main Application Entry Point

use clap::{Parser, Subcommand};
use log::{error, info};

use canvas_contracts::{
    abi::{generate_wit_package, hash_wit_package, validate_wit_package},
    artifact::{
        build_artifact_bundle,
        hash::{canonical_graph_hash, hash_bytes_prefixed, GRAPH_CANONICALIZATION},
        inspect_artifact_manifest, sign_artifact_manifest, verify_artifact_manifest,
    },
    chrononode::{submit_artifact_bundle, validate_content_hash_format},
    compiler::{Compiler, GraphExecutor},
    config::ConfigManager,
    error::{CanvasError, CanvasResult},
    info as lib_info, init,
    nodes::{builtin_node_definitions, NodeRegistry},
    types::{ExecutionContext, VisualGraph},
    validation::{baals_wasm_v1_profile, inspect_wasm, print_wat, validate_wasm_against_profile},
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

    /// Validate and inspect WASM modules
    Wasm {
        #[command(subcommand)]
        action: WasmCommands,
    },

    /// Generate and validate WIT ABI packages
    Wit {
        #[command(subcommand)]
        action: WitCommands,
    },

    /// Submit and verify archived bundles with ChronoNode
    Archive {
        #[command(subcommand)]
        action: ArchiveCommands,
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
    /// Sign artifact manifest with an Ed25519 key
    Sign {
        /// Path to canvas.contract.json
        #[arg(short, long)]
        manifest: String,
        /// Environment variable name containing a hex signing key
        #[arg(long)]
        key_env: Option<String>,
        /// File containing a hex signing key
        #[arg(long)]
        key_file: Option<String>,
    },
    /// Print artifact manifest details
    Inspect {
        /// Path to canvas.contract.json
        #[arg(short, long)]
        manifest: String,
        /// Emit machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum WasmCommands {
    /// Validate a WASM module against a runtime profile
    Validate {
        /// WASM file to validate
        #[arg(short, long)]
        wasm: String,
        /// Runtime profile name
        #[arg(short, long, default_value = "baals-wasm-v1")]
        profile: String,
        /// Optional JSON output report path
        #[arg(short, long)]
        out: Option<String>,
    },
    /// Inspect a WASM module
    Inspect {
        /// WASM file to inspect
        #[arg(short, long)]
        wasm: String,
        /// Emit machine-readable JSON report
        #[arg(long, default_value_t = false)]
        json: bool,
        /// Optional file path to write WAT disassembly
        #[arg(long)]
        wat_out: Option<String>,
    },
}

#[derive(Subcommand)]
enum WitCommands {
    /// Generate WIT package files in an output directory
    Generate {
        /// Optional source graph file (reserved for future graph-driven WIT generation)
        #[arg(short, long)]
        input: Option<String>,
        /// Output directory for WIT files
        #[arg(short, long)]
        out: String,
    },
    /// Validate a WIT package directory
    Validate {
        /// WIT directory to validate
        #[arg(short, long)]
        wit: String,
    },
}

#[derive(Subcommand)]
enum ArchiveCommands {
    /// Submit an artifact bundle file to ChronoNode
    Submit {
        /// Bundle file path (e.g. .canvasbundle.tar.zst)
        #[arg(short, long)]
        bundle: String,
        /// ChronoNode base URL
        #[arg(short = 'u', long)]
        chrononode_url: String,
    },
    /// Verify archive content-hash format
    Verify {
        /// Content hash in sha256:<hex> format
        #[arg(short, long)]
        content_hash: String,
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
            ArtifactCommands::Sign {
                manifest,
                key_env,
                key_file,
            } => {
                sign_artifact(manifest, key_env.as_deref(), key_file.as_deref())?;
            }
            ArtifactCommands::Inspect { manifest, json } => {
                inspect_artifact(manifest, *json)?;
            }
        },

        Some(Commands::Wasm { action }) => match action {
            WasmCommands::Validate { wasm, profile, out } => {
                wasm_validate(wasm, profile, out.as_deref())?;
            }
            WasmCommands::Inspect {
                wasm,
                json,
                wat_out,
            } => {
                wasm_inspect(wasm, *json, wat_out.as_deref())?;
            }
        },

        Some(Commands::Wit { action }) => match action {
            WitCommands::Generate { input, out } => {
                wit_generate(input.as_deref(), out)?;
            }
            WitCommands::Validate { wit } => {
                wit_validate(wit)?;
            }
        },

        Some(Commands::Archive { action }) => match action {
            ArchiveCommands::Submit {
                bundle,
                chrononode_url,
            } => {
                archive_submit(bundle, chrononode_url)?;
            }
            ArchiveCommands::Verify { content_hash } => {
                archive_verify(content_hash)?;
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

fn sign_artifact(
    manifest: &str,
    key_env: Option<&str>,
    key_file: Option<&str>,
) -> CanvasResult<()> {
    let key_hex = resolve_signing_key_hex(key_env, key_file)?;
    let sig = sign_artifact_manifest(std::path::Path::new(manifest), &key_hex)?;
    info!("Artifact manifest signed successfully");
    info!("Algorithm: {}", sig.algorithm);
    info!("Public key: {}", sig.public_key);
    Ok(())
}

fn inspect_artifact(manifest: &str, as_json: bool) -> CanvasResult<()> {
    let manifest = inspect_artifact_manifest(std::path::Path::new(manifest))?;
    if as_json {
        println!("{}", serde_json::to_string_pretty(&manifest)?);
        return Ok(());
    }

    info!("Manifest: {}", manifest.name);
    info!("Version: {}", manifest.version);
    info!("Target: {}", manifest.target);
    info!("Schema: {}", manifest.schema);
    info!("WASM hash: {}", manifest.artifact.wasm_hash);
    info!("Graph hash: {}", manifest.source.graph_hash);
    info!("WIT package: {}", manifest.abi.wit_package);
    info!("WIT hash: {}", manifest.abi.wit_hash);
    info!("Validation status: {}", manifest.validation.status);
    info!("Signatures: {}", manifest.signatures.len());
    Ok(())
}

fn resolve_signing_key_hex(key_env: Option<&str>, key_file: Option<&str>) -> CanvasResult<String> {
    if let Some(env_name) = key_env {
        let value = std::env::var(env_name).map_err(|_| {
            CanvasError::Config(format!("Signing key env var '{}' is not set", env_name))
        })?;
        return Ok(value.trim().to_string());
    }

    if let Some(path) = key_file {
        let value = std::fs::read_to_string(path).map_err(CanvasError::Io)?;
        return Ok(value.trim().to_string());
    }

    Err(CanvasError::Config(
        "Provide --key-env or --key-file to sign manifest".to_string(),
    ))
}

fn wasm_validate(wasm: &str, profile: &str, out: Option<&str>) -> CanvasResult<()> {
    let wasm_bytes = std::fs::read(wasm).map_err(CanvasError::Io)?;
    let runtime_profile = match profile {
        "baals-wasm-v1" => baals_wasm_v1_profile(),
        other => {
            return Err(CanvasError::Config(format!(
                "Unsupported runtime profile '{}'",
                other
            )))
        }
    };

    let report = validate_wasm_against_profile(&wasm_bytes, &runtime_profile)?;
    if let Some(out_path) = out {
        let content = serde_json::to_string_pretty(&report)?;
        std::fs::write(out_path, content).map_err(CanvasError::Io)?;
        info!("WASM validation report written to {}", out_path);
    }

    info!("WASM validation status: {}", report.status);
    info!("Profile: {}", report.target_profile);
    info!("WASM size: {} bytes", report.inspection.size_bytes);
    info!("Imports: {}", report.inspection.imports.len());
    info!("Exports: {}", report.inspection.exports.len());

    if !report.warnings.is_empty() {
        for warning in &report.warnings {
            info!("warning: {}", warning);
        }
    }
    if !report.errors.is_empty() {
        for err in &report.errors {
            error!("error: {}", err);
        }
        return Err(CanvasError::Validation(
            "WASM validation failed".to_string(),
        ));
    }
    Ok(())
}

fn wasm_inspect(wasm: &str, as_json: bool, wat_out: Option<&str>) -> CanvasResult<()> {
    let wasm_bytes = std::fs::read(wasm).map_err(CanvasError::Io)?;
    let inspection = inspect_wasm(&wasm_bytes)?;

    if as_json {
        println!("{}", serde_json::to_string_pretty(&inspection)?);
    } else {
        info!("WASM inspect:");
        info!("  size_bytes={}", inspection.size_bytes);
        info!("  imports={}", inspection.imports.len());
        info!("  exports={}", inspection.exports.len());
        info!("  memory_pages={}", inspection.memory_pages);
        info!("  has_wasi={}", inspection.has_wasi);
        info!("  has_threads={}", inspection.has_threads);
        info!("  has_multi_memory={}", inspection.has_multi_memory);
        info!("  has_memory64={}", inspection.has_memory64);
        info!("  float_operator_count={}", inspection.float_operator_count);
    }

    if let Some(wat_path) = wat_out {
        let wat = print_wat(&wasm_bytes)?;
        std::fs::write(wat_path, wat).map_err(CanvasError::Io)?;
        info!("WAT disassembly written to {}", wat_path);
    }
    Ok(())
}

fn wit_generate(input: Option<&str>, out: &str) -> CanvasResult<()> {
    if let Some(path) = input {
        info!(
            "WIT generation currently uses the canonical package template; graph input '{}' is reserved for graph-driven generation",
            path
        );
    }
    let out_dir = std::path::Path::new(out);
    let files = generate_wit_package(out_dir)?;
    let hash = hash_wit_package(out_dir)?;
    info!(
        "Generated {} WIT files in {}",
        files.len(),
        out_dir.display()
    );
    info!("WIT hash: {}", hash);
    Ok(())
}

fn wit_validate(wit: &str) -> CanvasResult<()> {
    let dir = std::path::Path::new(wit);
    let report = validate_wit_package(dir)?;
    info!("WIT package: {}", report.package);
    if !report.warnings.is_empty() {
        for warning in &report.warnings {
            info!("warning: {}", warning);
        }
    }
    if !report.errors.is_empty() {
        for err in &report.errors {
            error!("error: {}", err);
        }
        return Err(CanvasError::Validation("WIT validation failed".to_string()));
    }
    info!("WIT validation passed");
    Ok(())
}

fn archive_submit(bundle: &str, chrononode_url: &str) -> CanvasResult<()> {
    let result = submit_artifact_bundle(std::path::Path::new(bundle), chrononode_url)?;
    info!("Archive submit successful");
    info!("Storage pointer: {}", result.storage_pointer);
    info!("Content hash: {}", result.content_hash);
    if let Some(checkpoint) = result.checkpoint_id {
        info!("Checkpoint id: {}", checkpoint);
    }
    Ok(())
}

fn archive_verify(content_hash: &str) -> CanvasResult<()> {
    validate_content_hash_format(content_hash)?;
    info!("Archive content hash format is valid: {}", content_hash);
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
    let output_path = std::path::Path::new(output);
    let val_path = derive_compile_sidecar_path(output_path, "validation-report.json");
    let graph_hash = canonical_graph_hash(&graph)?;
    let target_adapter = graph
        .metadata
        .get("target_adapter")
        .cloned()
        .unwrap_or_else(|| "baals".to_string());
    let val_json = serde_json::json!({
        "schema_version": "canvas.validation.v1",
        "graph_schema_version": graph.schema_version,
        "graph_hash": graph_hash,
        "graph_canonicalization": GRAPH_CANONICALIZATION,
        "target_adapter": target_adapter,
        "node_count": graph.nodes.len(),
        "connection_count": graph.connections.len(),
        "is_valid": val_res.is_valid,
        "errors": val_res.errors.clone(),
        "warnings": val_res.warnings.clone()
    });
    std::fs::write(&val_path, serde_json::to_string_pretty(&val_json)?).map_err(CanvasError::Io)?;

    if !val_res.is_valid {
        return Err(CanvasError::Validation(format!(
            "Graph validation failed with {} error(s). See {} for details.",
            val_res.errors.len(),
            val_path.display()
        )));
    }

    // Compile the graph
    let result = compiler.compile(&graph)?;

    // Write WASM output
    std::fs::write(output_path, &result.wasm_bytes).map_err(CanvasError::Io)?;

    // Write ABI
    let abi_path = derive_compile_sidecar_path(output_path, "abi.json");
    let abi_content =
        serde_json::to_string_pretty(&result.abi).map_err(CanvasError::Serialization)?;
    std::fs::write(&abi_path, abi_content).map_err(CanvasError::Io)?;

    // Calculate hashes for lockfile
    let wasm_hash = hash_bytes_prefixed(&result.wasm_bytes);
    let mut node_types: Vec<String> = graph.nodes.iter().map(|n| n.node_type.clone()).collect();
    node_types.sort();
    node_types.dedup();

    // Write graph.lock.json
    let lock_path = derive_compile_sidecar_path(output_path, "graph.lock.json");
    let lock_json = serde_json::json!({
        "schema_version": "canvas.graph.lock.v1",
        "graph_schema_version": graph.schema_version,
        "project_name": graph.name,
        "target_adapter": target_adapter,
        "graph_canonicalization": GRAPH_CANONICALIZATION,
        "node_count": graph.nodes.len(),
        "connection_count": graph.connections.len(),
        "node_types": node_types,
        "gas_estimate": result.gas_estimate,
        "compiler": {
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION"),
            "wasm_target": "wasm32-unknown-unknown",
            "wasm_encoder_version": "0.38",
            "wasmtime_validation_version": "43.0.1"
        },
        "graph_hash": graph_hash,
        "wasm_hash": wasm_hash
    });
    std::fs::write(&lock_path, serde_json::to_string_pretty(&lock_json)?)
        .map_err(CanvasError::Io)?;

    info!("Compilation successful!");
    info!("WASM file: {}", output_path.display());
    info!("ABI file: {}", abi_path.display());
    info!("Lock file: {}", lock_path.display());
    info!("Validation report file: {}", val_path.display());
    info!("Gas estimate: {}", result.gas_estimate);

    if !result.warnings.is_empty() {
        info!("Warnings:");
        for warning in &result.warnings {
            info!("  - {}", warning);
        }
    }

    Ok(())
}

fn derive_compile_sidecar_path(
    output_wasm: &std::path::Path,
    sidecar_suffix: &str,
) -> std::path::PathBuf {
    let parent = output_wasm
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let stem = output_wasm
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("contract");
    parent.join(format!("{}.{}", stem, sidecar_suffix))
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

    // Load private key with basic filesystem permission hardening.
    let private_key = read_private_key_file(key)?;

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
        baals_client.deploy_contract(&wasm_bytes, constructor_args, &private_key)?;

    info!("Deployment successful!");
    info!("Contract address: {}", deployment_result.contract_address);
    info!("Transaction hash: {}", deployment_result.transaction_hash);
    info!("Gas used: {}", deployment_result.gas_used);

    Ok(())
}

fn read_private_key_file(path: &str) -> CanvasResult<String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path).map_err(CanvasError::Io)?;
        let mode = metadata.permissions().mode() & 0o777;
        if (mode & 0o077) != 0 {
            return Err(CanvasError::PermissionDenied(format!(
                "Insecure key file permissions for '{}': {:o}. Expected 600.",
                path, mode
            )));
        }
    }

    let key_content = std::fs::read_to_string(path).map_err(CanvasError::Io)?;
    let private_key = key_content.trim().to_string();
    if private_key.is_empty() {
        return Err(CanvasError::Validation(format!(
            "Private key file '{}' is empty",
            path
        )));
    }
    Ok(private_key)
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

#[cfg(test)]
mod tests {
    use super::read_private_key_file;
    use std::fs;

    #[test]
    fn read_private_key_file_rejects_empty() {
        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join("key.txt");
        fs::write(&key_path, " \n ").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&key_path).unwrap().permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&key_path, perms).unwrap();
        }
        let err = read_private_key_file(key_path.to_str().unwrap()).unwrap_err();
        assert!(err.to_string().contains("is empty"));
    }

    #[cfg(unix)]
    #[test]
    fn read_private_key_file_requires_0600_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join("key.txt");
        fs::write(&key_path, "deadbeef").unwrap();

        let mut perms = fs::metadata(&key_path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&key_path, perms).unwrap();

        let err = read_private_key_file(key_path.to_str().unwrap()).unwrap_err();
        assert!(err.to_string().contains("Expected 600"));
    }

    #[cfg(unix)]
    #[test]
    fn read_private_key_file_accepts_0600_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join("key.txt");
        fs::write(&key_path, "deadbeef").unwrap();

        let mut perms = fs::metadata(&key_path).unwrap().permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&key_path, perms).unwrap();

        let key = read_private_key_file(key_path.to_str().unwrap()).unwrap();
        assert_eq!(key, "deadbeef");
    }
}
