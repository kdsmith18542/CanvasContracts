# CLI Command Reference

Canvas Contracts CLI for building, validating, simulating, and deploying visual smart contracts.

## Global Options

```
canvas-contracts [OPTIONS] <COMMAND>
```

| Option | Description |
|--------|-------------|
| `-c, --config <FILE>` | Configuration file path [default: config.toml] |
| `-d, --debug` | Enable debug logging |
| `-l, --log-level <LEVEL>` | Log level [default: info] |

## Commands

### `validate`

Validate a visual graph JSON file.

```bash
canvas-contracts validate --input <FILE>
```

| Option | Description |
|--------|-------------|
| `-i, --input <FILE>` | Input graph JSON file |

**Example:**
```bash
canvas-contracts validate --input contract.json
canvas-contracts validate --input tests/fixtures/dormancy_oracle.json
```

### `compile`

Compile a visual graph JSON to WASM bytecode.

```bash
canvas-contracts compile --input <FILE> --output <FILE>
```

| Option | Description |
|--------|-------------|
| `-i, --input <FILE>` | Input graph JSON file |
| `-o, --output <FILE>` | Output WASM file |
| `-O, --optimize` | Enable optimization |

**Example:**
```bash
canvas-contracts compile --input contract.json --output out.wasm
```

### `simulate`

Simulate a contract — either from a compiled WASM file or directly from a graph JSON.

```bash
canvas-contracts simulate (--contract <FILE> | --graph <FILE>) [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-c, --contract <FILE>` | Contract WASM file |
| `--graph <FILE>` | Graph JSON file (compile + simulate in one step) |
| `-i, --input <FILE>` | Input data file (JSON) |
| `-g, --gas-limit <LIMIT>` | Gas limit [default: 1000000] |

**Examples:**
```bash
# Simulate using wasmtime
canvas-contracts simulate --contract contract.wasm

# Simulate directly from graph JSON
canvas-contracts simulate --graph tests/fixtures/simple_arithmetic.json

# With input data and gas limit
canvas-contracts simulate --contract contract.wasm -i input.json -g 500000
```

### `deploy`

Deploy a compiled WASM contract to BaaLS.

```bash
canvas-contracts deploy --contract <FILE> --key <FILE> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-c, --contract <FILE>` | Contract WASM file |
| `-k, --key <FILE>` | Private key file |
| `-l, --args <JSON>` | Constructor arguments (JSON) |

**Example:**
```bash
canvas-contracts deploy --contract out.wasm --key my-key
canvas-contracts deploy --contract out.wasm --key my-key --args '{"name": "test"}'
```

### `editor`

Start the visual editor server.

```bash
canvas-contracts editor [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-p, --port <PORT>` | Editor port [default: 3000] |
| `--host <HOST>` | Host address [default: localhost] |

### `info`

Show application information.

```bash
canvas-contracts info
```

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | Error (validation, compilation, deployment, etc.) |

## Examples

```bash
# Full workflow
canvas-contracts validate --input contract.json
canvas-contracts compile --input contract.json --output out.wasm
canvas-contracts simulate --graph tests/fixtures/simple_arithmetic.json
canvas-contracts deploy --contract out.wasm --key my-key
canvas-contracts editor
canvas-contracts info
```
