# Canvas Contracts Documentation

Welcome to the Canvas Contracts documentation!

> **Current release state (2026-05-23):** All core features implemented — graph validation, real WASM compilation, wasmtime runtime, BaaLS integration, 14 node types, frontend visual editor. 75 tests pass. See [plan.md](../plan.md) for full development status.

## Table of Contents

### Getting Started
- [Quick Start Guide](getting-started/quick-start.md) — Install and build your first contract
- [Architecture Spec](canvascontracts.md) — Original technical specification

### User Guide
- [Visual Editor](user-guide/visual-editor.md) — Interface and workflow guide

### CLI Reference
- [CLI Commands](reference/cli.md) — Available `canvas-contracts` commands

### Development
- [API Reference](api/README.md) — Library API documentation *(aspirational, predates implementation)*
- [Plan](../plan.md) — Current development plan and milestone status
- [Agent Context](../agents.md) — Quick reference for contributors

### Deployment
- [Deployment Guide](deployment/README.md) — Production deployment *(aspirational — deployment module is feature-gated, not yet implemented)*

## What is Canvas Contracts?

Canvas Contracts is a visual smart contract development platform. Users compose directed graphs of pre-built nodes that compile to WASM bytecode for deployment on BaaLS.

| Feature | Status |
|---------|--------|
| Visual graph editor | ✅ Drag-and-drop canvas + PropertyPanel |
| Node types | ✅ 14 (arithmetic, logic, storage, control, crypto) |
| Graph validation | ✅ Cycle detection, type checking, reachability |
| Graph simulation | ✅ Toposort data-flow execution |
| WASM compilation | ✅ `wasm-encoder` + wasmtime validation |
| WASM runtime | ✅ Sandboxed execution + fuel metering |
| BaaLS integration | ✅ Trait + Mock + HTTP client + Ed25519 signing |
| Frontend | ✅ Toolbar, undo/redo, save/load, deploy, ContractMonitor |
| Debugger | ✅ Breakpoints, step-through |
| Dormancy Oracle | ✅ Validates and simulates end-to-end |
| AI assistant | ⚠️ Feature-gated, not yet un-gated |
| Deployment (blue-green, scaling) | ⚠️ Feature-gated, not yet implemented |

## Quick Start

```bash
# Build and test
cargo build           # 0 errors, 0 warnings
cargo test            # 75 tests pass

# CLI
canvas-contracts validate --input graph.json
canvas-contracts simulate --graph tests/fixtures/simple_arithmetic.json
canvas-contracts compile --input graph.json --output out.wasm
canvas-contracts deploy --contract out.wasm --key my-key --args '{}'

# Frontend (needs libsoup-2.4 on Linux)
cd frontend && npm install && npm run dev
```
