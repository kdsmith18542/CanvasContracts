# Canvas Contracts

**Paint Your Logic. Deploy Your Future.**

Canvas Contracts is a visual smart contract development platform. Users compose directed graphs of pre-built nodes (arithmetic, conditionals, storage ops, crypto) that compile to WASM bytecode for deployment on BaaLS.

> **Development status**: All core features implemented — graph validation, execution, real WASM compilation, wasmtime runtime, BaaLS integration, 14 node types, full frontend. 75 tests pass, 0 warnings.

## What Works

- **Visual graph editor**: Drag-and-drop canvas (`@xyflow/react` + Tauri desktop app)
- **14 node types**: Arithmetic (Add/Sub/Mul/Div), Logic (And/Or/Not/If), Storage (Read/Write), Control (Start/End), Crypto (VerifySignature/DecodeProof)
- **Graph validation**: Cycle detection, port type checking, reachability analysis — 14 types handled
- **Graph simulation**: Execute graphs via toposort data-flow engine — test logic without deploying
- **Real WASM compilation**: `wasm-encoder` produces valid, input-dependent WASM bytecode; wasmtime-validated
- **Wasmtime runtime**: Sandboxed execution with fuel-based gas metering; host functions for BaaLS storage
- **BaaLS integration**: `BaalsClient` trait with Mock + HTTP client (real BaaLS REST API); Ed25519 signing
- **Frontend**: Toolbar (compile/validate/simulate/deploy) all wired; PropertyPanel; undo/redo; save/load; ContractMonitor
- **Debugger**: Breakpoints, step-through, wired to GraphExecutor
- **DormancyOracle**: Resurgence Protocol oracle graph validates and simulates end-to-end

## Quick Start

### Prerequisites

- **Rust** (latest stable)
- **Node.js** (v18+)

### Installation

```bash
git clone https://github.com/kdsmith18542/CanvasContracts.git
cd CanvasContracts
make install
```

### Development

```bash
# Backend
cargo build           # 0 errors, 0 warnings
cargo test            # 75 tests pass

# CLI
canvas-contracts validate --input graph.json
canvas-contracts simulate --graph tests/fixtures/simple_arithmetic.json
canvas-contracts compile --input graph.json --output out.wasm
canvas-contracts deploy --contract out.wasm --key my-key --args '{}'

# Frontend
cd frontend
npm install
npm run dev           # Vite dev server
npm run tauri dev     # Tauri desktop app (needs libsoup-2.4 on Linux)
```

## Architecture

```
canvascontract/
├── src/                       # Rust backend
│   ├── compiler/              # graph → IR → AST → WASM → wasmtime validation
│   ├── nodes/                 # 14 node definitions + implementations
│   ├── wasm/                  # Real wasmtime runtime (fuel metering)
│   ├── baals/                 # BaalsClient trait, Mock + HTTP client
│   └── debugger/              # Breakpoint debugger
├── frontend/                  # React + Tauri
│   └── src/
│       ├── components/        # CanvasEditor, NodePalette, Toolbar, PropertyPanel, etc.
│       ├── store/             # Zustand with undo/redo
│       └── services/          # TauriService, ProjectService
└── tests/
    ├── fixtures/              # 6 graph JSON fixtures (incl. DormancyOracle)
    ├── executor_tests.rs      # 3 integration tests
    └── graph_tests.rs         # 9 fixture-based tests
```

## Node Types

| Node | Category | Gas | Description |
|------|----------|-----|-------------|
| Start | Control | 0 | Entry point |
| End | Control | 0 | Exit point |
| Add/Sub/Mul/Div | Arithmetic | 3-5 | i64 arithmetic |
| And/Or/Not | Logic | 1-3 | Boolean logic |
| If | Logic | 10 | Conditional branch |
| ReadStorage | State | 100 | Read from contract KV state |
| WriteStorage | State | 200 | Write to contract KV state |
| VerifySignature | Crypto | 100 | Ed25519 signature verification |
| DecodeProof | Crypto | 50 | JSON proof deserialization |

## Documentation

- [Plan](plan.md) — Development plan and milestones
- [Agents](agents.md) — Quick reference for contributors
- [Spec](docs/canvascontracts.md) — Original technical specification

## Testing

```bash
# Rust (75 tests, 0 failures)
cargo test

# Frontend type check
cd frontend && npx tsc --noEmit
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and [agents.md](agents.md) for development context.

## License

MIT — see [LICENSE](LICENSE).
