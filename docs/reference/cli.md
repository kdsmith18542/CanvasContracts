# CLI Command Reference

The Canvas Contracts CLI provides command-line tools for building, testing, deploying, and managing smart contracts.

## Overview

```bash
canvas-contracts [OPTIONS] <COMMAND>
```

### Global Options

| Option | Description |
|--------|-------------|
| `-c, --config <FILE>` | Configuration file path |
| `-d, --debug` | Enable debug logging |
| `-l, --log-level <LEVEL>` | Log level (error, warn, info, debug, trace) |
| `-h, --help` | Print help information |
| `-V, --version` | Print version information |

## Commands

### `compile`

Compile a visual graph to WASM bytecode.

```bash
canvas-contracts compile [OPTIONS] --input <FILE> --output <FILE>
```

**Options:**
- `-i, --input <FILE>` - Input graph file (JSON/YAML)
- `-o, --output <FILE>` - Output WASM file
- `-O, --optimize <LEVEL>` - Optimization level (0-3) [default: 1]
- `--gas-limit <LIMIT>` - Maximum gas limit
- `--debug-info` - Include debug information

**Examples:**
```bash
# Basic compilation
canvas-contracts compile -i contract.json -o contract.wasm

# Optimized compilation
canvas-contracts compile -i contract.json -o contract.wasm -O 3

# With gas limit
canvas-contracts compile -i contract.json -o contract.wasm --gas-limit 1000000
```

### `validate`

Validate a visual graph for errors and warnings.

```bash
canvas-contracts validate [OPTIONS] --input <FILE>
```

**Options:**
- `-i, --input <FILE>` - Input graph file
- `--strict` - Enable strict validation
- `--format <FORMAT>` - Output format (text, json, yaml)

**Examples:**
```bash
# Basic validation
canvas-contracts validate -i contract.json

# Strict validation with JSON output
canvas-contracts validate -i contract.json --strict --format json
```

### `test`

Test a contract with simulation.

```bash
canvas-contracts test [OPTIONS] --contract <FILE>
```

**Options:**
- `-c, --contract <FILE>` - Contract WASM file
- `-i, --input <FILE>` - Test input file (JSON)
- `--gas-limit <LIMIT>` - Gas limit for testing
- `--trace` - Enable execution tracing
- `--profile` - Enable performance profiling

**Examples:**
```bash
# Basic testing
canvas-contracts test -c contract.wasm

# With test inputs
canvas-contracts test -c contract.wasm -i test_inputs.json

# With tracing
canvas-contracts test -c contract.wasm --trace --profile
```

### `deploy`

Deploy a contract to a blockchain.

```bash
canvas-contracts deploy [OPTIONS] --contract <FILE>
```

**Options:**
- `-c, --contract <FILE>` - Contract WASM file
- `-n, --name <NAME>` - Deployment name
- `-r, --replicas <COUNT>` - Number of replicas
- `--config <FILE>` - Deployment configuration file
- `--auto-scale` - Enable auto-scaling
- `--min-replicas <COUNT>` - Minimum replicas for auto-scaling
- `--max-replicas <COUNT>` - Maximum replicas for auto-scaling
- `--local` - Deploy to local BaaLS node

**Examples:**
```bash
# Local deployment
canvas-contracts deploy -c contract.wasm -n my-contract --local

# Production deployment
canvas-contracts deploy -c contract.wasm -n my-contract -r 3 --config prod.yaml

# Auto-scaling deployment
canvas-contracts deploy -c contract.wasm -n my-contract --auto-scale --min-replicas 2 --max-replicas 10
```

### `scale`

Scale a deployment.

```bash
canvas-contracts scale [OPTIONS] --name <NAME> --replicas <COUNT>
```

**Options:**
- `-n, --name <NAME>` - Deployment name
- `-r, --replicas <COUNT>` - Number of replicas
- `--wait` - Wait for scaling to complete

**Examples:**
```bash
# Scale up
canvas-contracts scale -n my-contract -r 5

# Scale down
canvas-contracts scale -n my-contract -r 2

# Scale to zero
canvas-contracts scale -n my-contract -r 0
```

### `update`

Update a deployment with a new contract.

```bash
canvas-contracts update [OPTIONS] --name <NAME> --contract <FILE>
```

**Options:**
- `-n, --name <NAME>` - Deployment name
- `-c, --contract <FILE>` - New contract WASM file
- `--strategy <STRATEGY>` - Update strategy (rolling, blue-green, canary)
- `--wait` - Wait for update to complete

**Examples:**
```bash
# Rolling update
canvas-contracts update -n my-contract -c contract-v2.wasm

# Blue-green update
canvas-contracts update -n my-contract -c contract-v2.wasm --strategy blue-green
```

### `status`

Get deployment status.

```bash
canvas-contracts status [OPTIONS] --name <NAME>
```

**Options:**
- `-n, --name <NAME>` - Deployment name
- `--format <FORMAT>` - Output format (text, json, yaml)
- `--watch` - Watch for status changes

**Examples:**
```bash
# Get status
canvas-contracts status -n my-contract

# Watch status
canvas-contracts status -n my-contract --watch

# JSON output
canvas-contracts status -n my-contract --format json
```

### `logs`

View deployment logs.

```bash
canvas-contracts logs [OPTIONS] --name <NAME>
```

**Options:**
- `-n, --name <NAME>` - Deployment name
- `-f, --follow` - Follow log output
- `--tail <LINES>` - Number of lines to show
- `--since <TIME>` - Show logs since time (e.g., "2h", "1d")
- `--level <LEVEL>` - Log level filter

**Examples:**
```bash
# View logs
canvas-contracts logs -n my-contract

# Follow logs
canvas-contracts logs -n my-contract -f

# Recent logs
canvas-contracts logs -n my-contract --tail 100

# Filter by level
canvas-contracts logs -n my-contract --level error
```

### `metrics`

View deployment metrics.

```bash
canvas-contracts metrics [OPTIONS] --name <NAME>
```

**Options:**
- `-n, --name <NAME>` - Deployment name
- `--format <FORMAT>` - Output format (text, json, yaml)
- `--watch` - Watch for metric changes
- `--duration <DURATION>` - Time range (e.g., "1h", "24h")

**Examples:**
```bash
# View metrics
canvas-contracts metrics -n my-contract

# Watch metrics
canvas-contracts metrics -n my-contract --watch

# Historical metrics
canvas-contracts metrics -n my-contract --duration 24h
```

### `health`

Check deployment health.

```bash
canvas-contracts health [OPTIONS] --name <NAME>
```

**Options:**
- `-n, --name <NAME>` - Deployment name
- `--format <FORMAT>` - Output format (text, json, yaml)
- `--watch` - Watch health status

**Examples:**
```bash
# Check health
canvas-contracts health -n my-contract

# Watch health
canvas-contracts health -n my-contract --watch
```

### `blue-green`

Manage blue-green deployments.

```bash
canvas-contracts blue-green <SUBCOMMAND>
```

**Subcommands:**

#### `create`
```bash
canvas-contracts blue-green create --name <NAME> --config <FILE>
```

#### `deploy-blue`
```bash
canvas-contracts blue-green deploy-blue --name <NAME> --contract <FILE>
```

#### `deploy-green`
```bash
canvas-contracts blue-green deploy-green --name <NAME> --contract <FILE>
```

#### `switch-to-green`
```bash
canvas-contracts blue-green switch-to-green --name <NAME>
```

#### `switch-to-blue`
```bash
canvas-contracts blue-green switch-to-blue --name <NAME>
```

#### `rollback`
```bash
canvas-contracts blue-green rollback --name <NAME>
```

**Examples:**
```bash
# Create blue-green deployment
canvas-contracts blue-green create -n my-contract --config bg-config.yaml

# Deploy to blue environment
canvas-contracts blue-green deploy-blue -n my-contract -c contract.wasm

# Deploy to green environment
canvas-contracts blue-green deploy-green -n my-contract -c contract-v2.wasm

# Switch to green
canvas-contracts blue-green switch-to-green -n my-contract

# Rollback to blue
canvas-contracts blue-green rollback -n my-contract
```

### `canary`

Manage canary deployments.

```bash
canvas-contracts canary <SUBCOMMAND>
```

**Subcommands:**

#### `create`
```bash
canvas-contracts canary create --name <NAME> --stable-contract <FILE> --canary-contract <FILE>
```

#### `update-traffic`
```bash
canvas-contracts canary update-traffic --name <NAME> --stable-percentage <PERCENT> --canary-percentage <PERCENT>
```

#### `promote`
```bash
canvas-contracts canary promote --name <NAME>
```

#### `rollback`
```bash
canvas-contracts canary rollback --name <NAME>
```

**Examples:**
```bash
# Create canary deployment
canvas-contracts canary create -n my-contract --stable-contract stable.wasm --canary-contract canary.wasm

# Update traffic split
canvas-contracts canary update-traffic -n my-contract --stable-percentage 90 --canary-percentage 10

# Promote canary
canvas-contracts canary promote -n my-contract

# Rollback canary
canvas-contracts canary rollback -n my-contract
```

### `baals`

Manage BaaLS blockchain integration.

```bash
canvas-contracts baals <SUBCOMMAND>
```

**Subcommands:**

#### `start`
```bash
canvas-contracts baals start [OPTIONS]
```

**Options:**
- `--port <PORT>` - Port number [default: 8080]
- `--data-dir <DIR>` - Data directory
- `--config <FILE>` - BaaLS configuration file

#### `stop`
```bash
canvas-contracts baals stop
```

#### `status`
```bash
canvas-contracts baals status
```

#### `test`
```bash
canvas-contracts baals test --url <URL>
```

**Examples:**
```bash
# Start local BaaLS node
canvas-contracts baals start --port 8080

# Check BaaLS status
canvas-contracts baals status

# Test BaaLS connection
canvas-contracts baals test --url https://baals.example.com

# Stop BaaLS node
canvas-contracts baals stop
```

### `ai`

AI Assistant commands.

```bash
canvas-contracts ai <SUBCOMMAND>
```

**Subcommands:**

#### `analyze`
```bash
canvas-contracts ai analyze --input <FILE> [OPTIONS]
```

**Options:**
- `-i, --input <FILE>` - Input graph file
- `--patterns` - Analyze patterns
- `--security` - Security analysis
- `--optimization` - Optimization analysis
- `--format <FORMAT>` - Output format

#### `suggest`
```bash
canvas-contracts ai suggest --input <FILE> --node <NODE>
```

**Examples:**
```bash
# Analyze contract
canvas-contracts ai analyze -i contract.json --patterns --security

# Get suggestions
canvas-contracts ai suggest -i contract.json --node logic_node
```

### `debug`

Debugging commands.

```bash
canvas-contracts debug <SUBCOMMAND>
```

**Subcommands:**

#### `start`
```bash
canvas-contracts debug start --contract <FILE> [OPTIONS]
```

**Options:**
- `-c, --contract <FILE>` - Contract WASM file
- `-i, --input <FILE>` - Input data file
- `--breakpoints <FILE>` - Breakpoints file
- `--step` - Step through execution

#### `step`
```bash
canvas-contracts debug step
```

#### `continue`
```bash
canvas-contracts debug continue
```

#### `variables`
```bash
canvas-contracts debug variables
```

**Examples:**
```bash
# Start debugging
canvas-contracts debug start -c contract.wasm -i input.json --step

# Step through execution
canvas-contracts debug step

# Continue execution
canvas-contracts debug continue

# View variables
canvas-contracts debug variables
```

### `marketplace`

Marketplace commands.

```bash
canvas-contracts marketplace <SUBCOMMAND>
```

**Subcommands:**

#### `search`
```bash
canvas-contracts marketplace search <QUERY> [OPTIONS]
```

**Options:**
- `--category <CATEGORY>` - Filter by category
- `--author <AUTHOR>` - Filter by author
- `--rating <RATING>` - Minimum rating
- `--format <FORMAT>` - Output format

#### `install`
```bash
canvas-contracts marketplace install <PACKAGE> [OPTIONS]
```

**Options:**
- `--version <VERSION>` - Package version
- `--force` - Force installation

#### `publish`
```bash
canvas-contracts marketplace publish <PACKAGE> [OPTIONS]
```

**Options:**
- `--description <DESC>` - Package description
- `--category <CATEGORY>` - Package category
- `--tags <TAGS>` - Package tags

#### `list`
```bash
canvas-contracts marketplace list [OPTIONS]
```

**Options:**
- `--installed` - Show installed packages only
- `--format <FORMAT>` - Output format

**Examples:**
```bash
# Search packages
canvas-contracts marketplace search "voting contract"

# Install package
canvas-contracts marketplace install voting-contract

# Publish package
canvas-contracts marketplace publish my-package --description "My custom package"

# List installed packages
canvas-contracts marketplace list --installed
```

### `sdk`

SDK commands.

```bash
canvas-contracts sdk <SUBCOMMAND>
```

**Subcommands:**

#### `init`
```bash
canvas-contracts sdk init [OPTIONS]
```

**Options:**
- `--name <NAME>` - Project name
- `--template <TEMPLATE>` - Template to use
- `--language <LANG>` - Programming language

#### `build`
```bash
canvas-contracts sdk build [OPTIONS]
```

**Options:**
- `--target <TARGET>` - Build target
- `--release` - Release build

#### `test`
```bash
canvas-contracts sdk test [OPTIONS]
```

**Options:**
- `--coverage` - Generate coverage report

**Examples:**
```bash
# Initialize SDK project
canvas-contracts sdk init --name my-sdk-project --template rust

# Build SDK
canvas-contracts sdk build --release

# Test SDK
canvas-contracts sdk test --coverage
```

### `config`

Configuration commands.

```bash
canvas-contracts config <SUBCOMMAND>
```

**Subcommands:**

#### `show`
```bash
canvas-contracts config show [OPTIONS]
```

**Options:**
- `--format <FORMAT>` - Output format

#### `set`
```bash
canvas-contracts config set <KEY> <VALUE>
```

#### `get`
```bash
canvas-contracts config get <KEY>
```

#### `validate`
```bash
canvas-contracts config validate --file <FILE>
```

**Examples:**
```bash
# Show configuration
canvas-contracts config show

# Set configuration value
canvas-contracts config set baals.node_url "https://baals.example.com"

# Get configuration value
canvas-contracts config get baals.node_url

# Validate configuration file
canvas-contracts config validate --file config.yaml
```

### `completion`

Generate shell completion scripts.

```bash
canvas-contracts completion <SHELL>
```

**Supported shells:**
- `bash`
- `zsh`
- `fish`
- `powershell`

**Examples:**
```bash
# Generate bash completion
canvas-contracts completion bash > ~/.bash_completion

# Generate zsh completion
canvas-contracts completion zsh > ~/.zsh_completion
```

## Configuration Files

### Global Configuration

Located at `~/.config/canvas-contracts/config.yaml`:

```yaml
app:
  name: "Canvas Contracts"
  version: "1.0.0"
  data_dir: "~/.local/share/canvas-contracts"
  log_level: "info"
  debug: false

compiler:
  optimization_level: 2
  debug_info: false
  gas_estimation: true
  max_gas_limit: 10000000
  wasm_target: "wasm32-unknown-unknown"

baals:
  node_url: "http://localhost:8080"
  connection_timeout: 30
  retry_attempts: 3
  enable_local_node: true
  local_node_port: 8080
```

### Project Configuration

Located at `./canvas-contracts.yaml`:

```yaml
project:
  name: "my-contract"
  version: "1.0.0"
  description: "My smart contract"

deployment:
  environment: "development"
  replicas: 1
  resources:
    cpu_requests: "100m"
    cpu_limits: "500m"
    memory_requests: "128Mi"
    memory_limits: "512Mi"

monitoring:
  enabled: true
  metrics_endpoint: "/metrics"
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CANVAS_CONFIG_FILE` | Configuration file path | `~/.config/canvas-contracts/config.yaml` |
| `CANVAS_LOG_LEVEL` | Log level | `info` |
| `CANVAS_DEBUG` | Enable debug mode | `false` |
| `BAALS_NODE_URL` | BaaLS node URL | `http://localhost:8080` |
| `BAALS_AUTH_TOKEN` | BaaLS authentication token | None |

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Validation error |
| 4 | Compilation error |
| 5 | Deployment error |
| 6 | Network error |
| 7 | Permission error |

## Examples

### Complete Workflow

```bash
# 1. Create and validate a contract
canvas-contracts validate -i contract.json

# 2. Compile the contract
canvas-contracts compile -i contract.json -o contract.wasm -O 2

# 3. Test the contract
canvas-contracts test -c contract.wasm -i test_inputs.json --trace

# 4. Start local BaaLS node
canvas-contracts baals start --port 8080

# 5. Deploy the contract
canvas-contracts deploy -c contract.wasm -n my-contract --local

# 6. Check deployment status
canvas-contracts status -n my-contract

# 7. View logs
canvas-contracts logs -n my-contract -f

# 8. Scale deployment
canvas-contracts scale -n my-contract -r 3

# 9. Update contract
canvas-contracts update -n my-contract -c contract-v2.wasm

# 10. Monitor metrics
canvas-contracts metrics -n my-contract --watch
```

### Blue-Green Deployment

```bash
# 1. Create blue-green deployment
canvas-contracts blue-green create -n my-contract --config bg-config.yaml

# 2. Deploy current version to blue
canvas-contracts blue-green deploy-blue -n my-contract -c contract.wasm

# 3. Deploy new version to green
canvas-contracts blue-green deploy-green -n my-contract -c contract-v2.wasm

# 4. Switch traffic to green
canvas-contracts blue-green switch-to-green -n my-contract

# 5. Monitor and rollback if needed
canvas-contracts blue-green rollback -n my-contract
```

### Canary Deployment

```bash
# 1. Create canary deployment
canvas-contracts canary create -n my-contract --stable-contract stable.wasm --canary-contract canary.wasm

# 2. Start with 10% traffic to canary
canvas-contracts canary update-traffic -n my-contract --stable-percentage 90 --canary-percentage 10

# 3. Gradually increase canary traffic
canvas-contracts canary update-traffic -n my-contract --stable-percentage 50 --canary-percentage 50

# 4. Promote canary to stable
canvas-contracts canary promote -n my-contract

# 5. Rollback if issues arise
canvas-contracts canary rollback -n my-contract
```

## Troubleshooting

### Common Issues

**Command not found**
```bash
# Install Canvas Contracts
cargo install canvas-contracts

# Or build from source
git clone https://github.com/kdsmith18542/CanvasContracts
cd CanvasContracts
cargo build --release
```

**Permission denied**
```bash
# Check file permissions
ls -la contract.wasm

# Fix permissions
chmod 644 contract.wasm
```

**Connection refused**
```bash
# Check if BaaLS node is running
canvas-contracts baals status

# Start BaaLS node
canvas-contracts baals start
```

**Configuration error**
```bash
# Validate configuration
canvas-contracts config validate --file config.yaml

# Show current configuration
canvas-contracts config show
```

### Debug Mode

Enable debug mode for detailed logging:

```bash
# Set debug environment variable
export CANVAS_DEBUG=true

# Or use debug flag
canvas-contracts --debug deploy -c contract.wasm -n my-contract
```

### Getting Help

```bash
# General help
canvas-contracts --help

# Command-specific help
canvas-contracts deploy --help

# Version information
canvas-contracts --version
``` 