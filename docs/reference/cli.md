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
| `-d, --input <FILE>` | Input data file (JSON) |
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

Deploy a contract to BaaLS.

```bash
canvas-contracts deploy (--manifest <FILE> | --contract <FILE>) --key <FILE> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--manifest <FILE>` | Path to `canvas.contract.json` (preferred, verified before deploy) |
| `-c, --contract <FILE>` | Contract WASM file (legacy path) |
| `-k, --key <FILE>` | Private key file |
| `-a, --args <JSON>` | Constructor arguments (JSON) |

**Example:**
```bash
canvas-contracts deploy --manifest dist/contracts/DormancyOracle/canvas.contract.json --key my-key
canvas-contracts deploy --contract out.wasm --key my-key --args '{"name": "test"}'
```

### `artifact build`

Build a verifiable contract artifact bundle from a graph.

```bash
canvas-contracts artifact build --input <FILE> --out <DIR>
```

| Option | Description |
|--------|-------------|
| `-i, --input <FILE>` | Input graph JSON file |
| `-o, --out <DIR>` | Output directory for generated bundle files |

**Example:**
```bash
canvas-contracts artifact build --input tests/fixtures/dormancy_oracle.json --out dist/contracts/DormancyOracle
```

Generated files include:
`graph.json`, `graph.canonical.json`, `node-pack.lock`, `contract.wasm`, `abi.json`, `safety-report.json`, `canvas.contract.json`, and `wit/*.wit`.

### `artifact verify`

Verify manifest integrity by recomputing hashes from local artifact files.

```bash
canvas-contracts artifact verify --manifest <FILE>
```

| Option | Description |
|--------|-------------|
| `-m, --manifest <FILE>` | Path to `canvas.contract.json` |

**Example:**
```bash
canvas-contracts artifact verify --manifest dist/contracts/DormancyOracle/canvas.contract.json
```

### `artifact sign`

Sign a manifest using an Ed25519 hex key from an environment variable or file.

```bash
canvas-contracts artifact sign --manifest <FILE> (--key-env <ENV_VAR> | --key-file <FILE>)
```

| Option | Description |
|--------|-------------|
| `-m, --manifest <FILE>` | Path to `canvas.contract.json` |
| `--key-env <ENV_VAR>` | Environment variable containing hex signing key |
| `--key-file <FILE>` | File containing hex signing key |

**Examples:**
```bash
canvas-contracts artifact sign --manifest dist/contracts/DormancyOracle/canvas.contract.json --key-env CANVAS_SIGNING_KEY
canvas-contracts artifact sign --manifest dist/contracts/DormancyOracle/canvas.contract.json --key-file ./signing.key
```

### `artifact inspect`

Inspect a manifest and print summary details (or full JSON).

```bash
canvas-contracts artifact inspect --manifest <FILE> [--json]
```

| Option | Description |
|--------|-------------|
| `-m, --manifest <FILE>` | Path to `canvas.contract.json` |
| `--json` | Emit full manifest JSON |

**Example:**
```bash
canvas-contracts artifact inspect --manifest dist/contracts/DormancyOracle/canvas.contract.json
canvas-contracts artifact inspect --manifest dist/contracts/DormancyOracle/canvas.contract.json --json
```

### `wasm validate`

Validate a WASM module against a runtime profile (`baals-wasm-v1` by default).

```bash
canvas-contracts wasm validate --wasm <FILE> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-w, --wasm <FILE>` | WASM file to validate |
| `-p, --profile <NAME>` | Runtime profile [default: `baals-wasm-v1`] |
| `-o, --out <FILE>` | Optional JSON validation report output path |

**Example:**
```bash
canvas-contracts wasm validate --wasm dist/contracts/DormancyOracle/contract.wasm
canvas-contracts wasm validate --wasm dist/contracts/DormancyOracle/contract.wasm --out report.json
```

### `wasm inspect`

Inspect a WASM module and optionally emit JSON and WAT.

```bash
canvas-contracts wasm inspect --wasm <FILE> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-w, --wasm <FILE>` | WASM file to inspect |
| `--json` | Emit machine-readable JSON inspection output |
| `--wat-out <FILE>` | Write WAT disassembly to file |

**Example:**
```bash
canvas-contracts wasm inspect --wasm dist/contracts/DormancyOracle/contract.wasm --json
canvas-contracts wasm inspect --wasm dist/contracts/DormancyOracle/contract.wasm --wat-out contract.wat
```

### `wit generate`

Generate WIT package files (canonical template or graph-driven).

```bash
canvas-contracts wit generate --out <DIR> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-o, --out <DIR>` | Output directory for generated WIT files |
| `-i, --input <FILE>` | Optional graph file used for graph-driven WIT generation metadata |

**Example:**
```bash
canvas-contracts wit generate --out dist/contracts/DormancyOracle/wit
canvas-contracts wit generate --input tests/fixtures/dormancy_oracle.json --out dist/contracts/DormancyOracle/wit
```

### `wit validate`

Validate a WIT package directory.

```bash
canvas-contracts wit validate --wit <DIR>
```

| Option | Description |
|--------|-------------|
| `-w, --wit <DIR>` | WIT directory to validate |

**Example:**
```bash
canvas-contracts wit validate --wit dist/contracts/DormancyOracle/wit
```

### `archive submit`

Submit an artifact bundle to a ChronoNode endpoint.

```bash
canvas-contracts archive submit --bundle <FILE> --chrononode-url <URL>
```

| Option | Description |
|--------|-------------|
| `-b, --bundle <FILE>` | Bundle file path (for example, `.canvasbundle.tar.zst`) |
| `-u, --chrononode-url <URL>` | ChronoNode base URL |
| `-m, --manifest <FILE>` | Optional manifest path to update with archive pointer/checkpoint fields |

**Example:**
```bash
canvas-contracts archive submit --bundle DormancyOracle.canvasbundle.tar.zst --chrononode-url https://chrono.baals.network --manifest dist/contracts/DormancyOracle/canvas.contract.json
```

### `archive verify`

Verify a ChronoNode content hash string format.

```bash
canvas-contracts archive verify --content-hash <HASH>
```

| Option | Description |
|--------|-------------|
| `-c, --content-hash <HASH>` | Hash in `sha256:<64-hex>` format |

**Example:**
```bash
canvas-contracts archive verify --content-hash sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
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
canvas-contracts artifact build --input tests/fixtures/dormancy_oracle.json --out dist/contracts/DormancyOracle
canvas-contracts artifact verify --manifest dist/contracts/DormancyOracle/canvas.contract.json
canvas-contracts artifact inspect --manifest dist/contracts/DormancyOracle/canvas.contract.json
canvas-contracts wasm validate --wasm dist/contracts/DormancyOracle/contract.wasm --out dist/contracts/DormancyOracle/validation.json
canvas-contracts wit validate --wit dist/contracts/DormancyOracle/wit
canvas-contracts deploy --manifest dist/contracts/DormancyOracle/canvas.contract.json --key my-key
canvas-contracts archive submit --bundle DormancyOracle.canvasbundle.tar.zst --chrononode-url https://chrono.baals.network --manifest dist/contracts/DormancyOracle/canvas.contract.json
canvas-contracts archive verify --content-hash sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
canvas-contracts editor
canvas-contracts info
```
