# Canvas Contracts Documentation

Welcome to the Canvas Contracts documentation!

> **Current release state (2026-05-25):** Core graph/runtime/compiler pipeline is active, plus verifiable artifact generation, WIT package tooling, ChronoNode archive submission, and BaaLS deployment. See [plan.md](../plan.md) for milestone tracking.

## Table of Contents

### Getting Started
- [Quick Start Guide](getting-started/quick-start.md) — Install and build your first contract
- [Architecture Spec](canvascontracts.md) — Original technical specification

### User Guide
- [Visual Editor](user-guide/visual-editor.md) — Interface and workflow guide

### CLI Reference
- [CLI Commands](reference/cli.md) — Available `canvas-contracts` commands

### Development
- [API Reference](api/README.md) — Library API documentation
- [Plan](../plan.md) — Current development plan and milestone status
- [Agent Context](../agents.md) — Quick reference for contributors

### Deployment
- [Deployment Guide](deployment/README.md) — Production deployment

## What is Canvas Contracts?

Canvas Contracts is a visual smart contract development platform. Users compose directed graphs of pre-built nodes that compile to WASM bytecode for deployment on BaaLS.

| Feature | Status |
|---------|--------|
| Visual graph editor | ✅ Drag-and-drop canvas + PropertyPanel |
| Node types | ✅ 39 built-ins (core + BaaLS + ChronoNode + Resurgence) |
| Graph validation | ✅ Cycle detection, type checking, reachability |
| Graph simulation | ✅ Toposort data-flow execution |
| WASM compilation | ✅ `wasm-encoder` + wasmtime validation |
| WASM runtime | ✅ Sandboxed execution + fuel metering |
| BaaLS integration | ✅ Trait + Mock + HTTP client + Ed25519 signing |
| Frontend | ✅ Toolbar, undo/redo, save/load, deploy, ContractMonitor |
| Debugger | ✅ Breakpoints, step-through |
| Dormancy Oracle | ✅ Validates and simulates end-to-end |
| AI assistant | ✅ Compiled module + frontend panel |
| Deployment (blue-green, scaling) | ✅ Compiled module + CLI integration path |

## Artifact Consumers

- **BaaLS** consumes `contract.wasm` plus deployment metadata from `canvas.contract.json` (runtime profile, hashes, receipt fields).
- **ChronoNode** consumes archived bundle payloads and returns a verifiable `sha256:<hex>` content hash plus storage pointer.
- **Resurgence** consumes generated proofs/events and audit material (`safety-report.json`, ABI + WIT package) for DormancyOracle workflows.

## Quick Start

```bash
# Build and test
cargo build           # 0 errors, 0 warnings
cargo test

# CLI
canvas-contracts validate --input graph.json
canvas-contracts simulate --graph tests/fixtures/simple_arithmetic.json
canvas-contracts compile --input graph.json --output out.wasm
canvas-contracts deploy --contract out.wasm --key my-key --args '{}'

# Frontend (needs libsoup-2.4 on Linux)
cd frontend && npm install && npm run dev
```
