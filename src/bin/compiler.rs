//! Canvas Contracts Compiler Binary

use canvas_contracts::{
    compiler::Compiler,
    config::Config,
    error::CanvasResult,
    types::VisualGraph,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "canvas-compiler")]
#[command(about = "Canvas Contracts Compiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a visual graph to WASM
    Compile {
        /// Input graph file
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output WASM file
        #[arg(short, long)]
        output: PathBuf,
        
        /// Optimization level (0-3)
        #[arg(short, long, default_value = "1")]
        optimize: u8,
    },
    
    /// Validate a visual graph
    Validate {
        /// Input graph file
        #[arg(short, long)]
        input: PathBuf,
    },
}

fn main() -> CanvasResult<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    let config = Config::default();
    
    match cli.command {
        Commands::Compile { input, output, optimize } => {
            compile_graph(&input, &output, optimize, &config)?;
        }
        Commands::Validate { input } => {
            validate_graph(&input, &config)?;
        }
    }
    
    Ok(())
}

fn compile_graph(input: &PathBuf, output: &PathBuf, optimize: u8, config: &Config) -> CanvasResult<()> {
    println!("Compiling graph from {} to {}", input.display(), output.display());
    
    // TODO: Load graph from file
    let graph = VisualGraph::new("test");
    
    let compiler = Compiler::new(config)?;
    let result = compiler.compile(&graph)?;
    
    // TODO: Write WASM to output file
    println!("Compilation successful!");
    println!("WASM size: {} bytes", result.wasm_bytes.len());
    println!("Gas estimate: {}", result.gas_estimate);
    
    Ok(())
}

fn validate_graph(input: &PathBuf, config: &Config) -> CanvasResult<()> {
    println!("Validating graph from {}", input.display());
    
    // TODO: Load graph from file
    let graph = VisualGraph::new("test");
    
    let compiler = Compiler::new(config)?;
    let validator = compiler.validator()?;
    let result = validator.validate(&graph)?;
    
    if result.is_valid {
        println!("Validation successful!");
    } else {
        println!("Validation failed!");
        for error in &result.errors {
            println!("Error: {}", error);
        }
    }
    
    for warning in &result.warnings {
        println!("Warning: {}", warning);
    }
    
    Ok(())
} 