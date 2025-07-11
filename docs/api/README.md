# API Reference

This document provides comprehensive API documentation for the Canvas Contracts platform. The API is designed to be modular, extensible, and easy to integrate into your applications.

## Table of Contents

- [Core Types](types.md)
- [Compiler API](compiler.md)
- [Runtime API](runtime.md)
- [AI Assistant API](ai.md)
- [Debugger API](debugger.md)
- [Deployment API](deployment.md)
- [Monitoring API](monitoring.md)
- [SDK API](sdk.md)

## Quick Start

```rust
use canvas_contracts::{
    Compiler, WasmRuntime, AiAssistant, DeploymentManager,
    types::{Graph, Node, NodeType},
    error::CanvasResult,
};

#[tokio::main]
async fn main() -> CanvasResult<()> {
    // Initialize the library
    canvas_contracts::init()?;
    
    // Create a compiler
    let compiler = Compiler::new()?;
    
    // Create a graph
    let mut graph = Graph::new("my_contract");
    
    // Add nodes to the graph
    let start_node = Node::new("start", NodeType::Start);
    let end_node = Node::new("end", NodeType::End);
    
    graph.add_node(start_node)?;
    graph.add_node(end_node)?;
    graph.add_edge("start", "end", "output", "input")?;
    
    // Compile the graph
    let result = compiler.compile(&graph)?;
    println!("Compiled WASM size: {} bytes", result.wasm_bytes.len());
    
    // Deploy the contract
    let deployment_manager = DeploymentManager::new(&config)?;
    let deployment_id = deployment_manager.deploy("my_contract", &graph, config).await?;
    println!("Deployed with ID: {}", deployment_id);
    
    Ok(())
}
```

## Core Concepts

### Graph

A `Graph` represents a visual smart contract as a collection of nodes and edges:

```rust
use canvas_contracts::types::{Graph, Node, NodeType};

let mut graph = Graph::new("contract_name");

// Add nodes
let start = Node::new("start", NodeType::Start);
let logic = Node::new("logic", NodeType::Logic);
let end = Node::new("end", NodeType::End);

graph.add_node(start)?;
graph.add_node(logic)?;
graph.add_node(end)?;

// Add edges
graph.add_edge("start", "logic", "output", "input")?;
graph.add_edge("logic", "end", "output", "input")?;
```

### Node

A `Node` represents a single operation in the contract:

```rust
use canvas_contracts::types::{Node, NodeType, NodeProperties};

let node = Node {
    id: "my_node".to_string(),
    node_type: NodeType::Logic,
    properties: NodeProperties::new(),
    position: (100, 200),
    inputs: vec!["input1".to_string()],
    outputs: vec!["output1".to_string()],
};
```

### Compilation Result

The compiler produces a `CompilationResult`:

```rust
use canvas_contracts::types::CompilationResult;

let result = CompilationResult {
    wasm_bytes: vec![/* compiled WASM */],
    gas_estimate: 1000,
    abi: serde_json::Value::Object(/* ABI */),
    warnings: vec![],
    errors: vec![],
};
```

## Error Handling

All API functions return `CanvasResult<T>`, which is an alias for `Result<T, CanvasError>`:

```rust
use canvas_contracts::error::{CanvasError, CanvasResult};

fn process_graph(graph: &Graph) -> CanvasResult<()> {
    match graph.validate() {
        Ok(_) => println!("Graph is valid"),
        Err(CanvasError::ValidationError(msg)) => {
            eprintln!("Validation failed: {}", msg);
        }
        Err(e) => return Err(e),
    }
    Ok(())
}
```

## Configuration

Most components accept a `Config` object for customization:

```rust
use canvas_contracts::config::Config;

let config = Config {
    app: AppConfig {
        name: "My App".to_string(),
        version: "1.0.0".to_string(),
        data_dir: PathBuf::from("./data"),
        log_level: "info".to_string(),
        debug: false,
    },
    compiler: CompilerConfig {
        optimization_level: 2,
        debug_info: false,
        gas_estimation: true,
        max_gas_limit: 10_000_000,
        wasm_target: "wasm32-unknown-unknown".to_string(),
        flags: vec![],
    },
    // ... other config sections
};
```

## Async Operations

Many operations are asynchronous, especially those involving I/O or network calls:

```rust
use canvas_contracts::{DeploymentManager, BaalsClient};

#[tokio::main]
async fn main() -> CanvasResult<()> {
    let deployment_manager = DeploymentManager::new(&config)?;
    
    // Deploy contract
    let deployment_id = deployment_manager.deploy("contract", &graph, config).await?;
    
    // Scale deployment
    deployment_manager.scale(&deployment_id, 3).await?;
    
    // Update deployment
    deployment_manager.update(&deployment_id, &new_graph).await?;
    
    Ok(())
}
```

## Examples

### Basic Contract Compilation

```rust
use canvas_contracts::{Compiler, types::Graph};

fn compile_simple_contract() -> CanvasResult<()> {
    let compiler = Compiler::new()?;
    let graph = create_simple_graph()?;
    
    let result = compiler.compile(&graph)?;
    println!("Compilation successful!");
    println!("WASM size: {} bytes", result.wasm_bytes.len());
    println!("Gas estimate: {}", result.gas_estimate);
    
    Ok(())
}
```

### AI-Assisted Development

```rust
use canvas_contracts::{AiAssistant, types::Graph};

fn analyze_contract(graph: &Graph) -> CanvasResult<()> {
    let ai = AiAssistant::new(&config)?;
    
    // Analyze patterns
    let patterns = ai.analyze_patterns(graph)?;
    println!("Found patterns: {:?}", patterns.patterns_found);
    
    // Validate contract
    let validation = ai.validate_contract(graph)?;
    println!("Validation result: {:?}", validation);
    
    // Optimize contract
    let optimization = ai.optimize_contract(graph)?;
    println!("Gas savings: {}", optimization.gas_savings);
    
    Ok(())
}
```

### Debugging

```rust
use canvas_contracts::{debugger::DebugSession, types::Graph};

fn debug_contract(graph: &Graph) -> CanvasResult<()> {
    let mut debugger = DebugSession::new(graph.clone())?;
    
    // Set breakpoints
    debugger.set_breakpoint("logic_node", None)?;
    
    // Start debugging
    let state = debugger.start_debugging()?;
    
    // Step through execution
    while let Some(step) = debugger.step()? {
        println!("Executing node: {}", step.node_id);
        println!("Gas consumed: {}", step.gas_consumed);
    }
    
    Ok(())
}
```

### Deployment

```rust
use canvas_contracts::{DeploymentManager, types::Graph};

async fn deploy_contract(graph: &Graph) -> CanvasResult<String> {
    let deployment_manager = DeploymentManager::new(&config)?;
    
    let deployment_config = DeploymentConfig {
        replicas: 3,
        resources: ResourceRequirements {
            cpu_requests: "100m".to_string(),
            cpu_limits: "500m".to_string(),
            memory_requests: "128Mi".to_string(),
            memory_limits: "512Mi".to_string(),
            storage_requests: "1Gi".to_string(),
        },
        // ... other config
    };
    
    let deployment_id = deployment_manager.deploy("my_contract", graph, deployment_config).await?;
    println!("Deployed with ID: {}", deployment_id);
    
    Ok(deployment_id)
}
```

### Monitoring

```rust
use canvas_contracts::{monitoring::MetricsCollector, deployment::DeploymentManager};

async fn monitor_deployment(deployment_id: &str) -> CanvasResult<()> {
    let metrics = MetricsCollector::new(&config)?;
    let deployment_manager = DeploymentManager::new(&config)?;
    
    // Record metrics
    metrics.increment_counter("contract_calls", 1)?;
    metrics.set_gauge("cpu_usage", 0.75)?;
    
    // Get deployment metrics
    if let Some(deployment_metrics) = deployment_manager.get_metrics(deployment_id) {
        println!("CPU usage: {}%", deployment_metrics.cpu_usage * 100.0);
        println!("Memory usage: {}%", deployment_metrics.memory_usage * 100.0);
        println!("Request count: {}", deployment_metrics.request_count);
    }
    
    Ok(())
}
```

## SDK Integration

The SDK provides high-level abstractions for common use cases:

```rust
use canvas_contracts::sdk::{CanvasSdk, SdkConfig};

fn use_sdk() -> CanvasResult<()> {
    let config = SdkConfig {
        api_key: "your_api_key".to_string(),
        endpoint: "https://api.canvascontracts.com".to_string(),
        timeout: Duration::from_secs(30),
    };
    
    let sdk = CanvasSdk::new(config)?;
    
    // Compile and execute
    let outputs = sdk.execute_graph(&graph, inputs)?;
    
    // Validate
    let validations = sdk.validate_graph(&graph);
    
    // Optimize
    let optimizations = sdk.optimize_graph(&graph);
    
    Ok(())
}
```

## Best Practices

### Error Handling

1. **Always check return values**
   ```rust
   let result = compiler.compile(&graph)?;
   ```

2. **Use specific error types**
   ```rust
   match error {
       CanvasError::ValidationError(msg) => handle_validation_error(msg),
       CanvasError::CompilationError(msg) => handle_compilation_error(msg),
       _ => handle_other_error(error),
   }
   ```

3. **Provide meaningful error context**
   ```rust
   .map_err(|e| CanvasError::Internal(format!("Failed to compile graph: {}", e)))
   ```

### Performance

1. **Reuse components when possible**
   ```rust
   let compiler = Compiler::new()?;
   for graph in graphs {
       let result = compiler.compile(&graph)?;
       // Process result
   }
   ```

2. **Use async operations for I/O**
   ```rust
   let deployment_id = deployment_manager.deploy("contract", &graph, config).await?;
   ```

3. **Batch operations when possible**
   ```rust
   for metric in metrics {
       metrics_collector.increment_counter(&metric.name, metric.value)?;
   }
   ```

### Security

1. **Validate all inputs**
   ```rust
   if !graph.validate()?.is_valid {
       return Err(CanvasError::ValidationError("Invalid graph".to_string()));
   }
   ```

2. **Use secure configuration**
   ```rust
   let config = Config {
       security: SecurityConfig {
           enable_tls: true,
           allowed_origins: vec!["https://trusted-domain.com".to_string()],
           // ...
       },
       // ...
   };
   ```

## Versioning

The API follows semantic versioning. Breaking changes are introduced in major versions:

- **v1.x.x**: Stable API
- **v2.x.x**: Breaking changes introduced
- **v3.x.x**: Future major version

## Migration Guide

When upgrading between major versions, check the migration guide for breaking changes and new features.

## Support

- **Documentation**: This API reference
- **Examples**: [GitHub Examples](https://github.com/kdsmith18542/CanvasContracts/examples)
- **Issues**: [GitHub Issues](https://github.com/kdsmith18542/CanvasContracts/issues)
- **Community**: [Discord](https://discord.gg/canvascontracts) 