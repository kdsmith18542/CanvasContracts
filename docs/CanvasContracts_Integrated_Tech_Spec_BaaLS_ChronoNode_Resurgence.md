# Canvas Contracts Integrated Technical Specification

**Version:** v0.2-current  
**Date:** 2026-05-23  
**Primary project:** Canvas Contracts  
**Integrated projects:** BaaLS, ChronoNode, Resurgence Protocol  
**Slogan:** Paint Your Logic. Deploy Your Future.

---

## 1. Executive Summary

Canvas Contracts is the visual smart-contract development environment for composing directed graphs of contract logic, validating them, simulating them, compiling them to WASM, and deploying them to a ledger/runtime target. The current primary target is BaaLS, a local-first embeddable Rust blockchain/runtime. ChronoNode provides the archival/proof layer for contract and ledger history. Resurgence Protocol provides the first high-value external protocol integration: proof-of-dormancy logic, DormancyOracle graph templates, and future bridge/oracle workflows between local WASM-ledger logic and EVM staking/governance contracts.

The integrated stack should be treated as four cooperating systems, not one monolith:

```text
Canvas Contracts  -> visual IDE, graph compiler, simulator, debugger
BaaLS             -> first/reference WASM ledger runtime and deployment target
ChronoNode        -> independent archival/proof/query layer
Resurgence        -> EVM protocol integration and DormancyOracle use case
```

The correct long-term rule is:

```text
BaaLS first, not BaaLS only.
ChronoNode independent, adapter-driven.
Resurgence integrated by adapter/template/oracle boundary, not hardwired into the core IDE.
```

---

## 2. Source Repository Status

### 2.1 Canvas Contracts

Current repo notes describe Canvas Contracts as a visual smart-contract development platform. Users compose directed graphs of pre-built nodes, including arithmetic, logic, storage, control, and crypto nodes, which compile to WASM bytecode for deployment on BaaLS.

Current repo status claims:

- Graph validation implemented.
- Execution/simulation implemented.
- Real WASM compilation implemented.
- Wasmtime runtime implemented.
- BaaLS integration implemented through `BaalsClient` with Mock and HTTP clients.
- Tauri/React frontend implemented.
- Debugger wired to `GraphExecutor`.
- DormancyOracle graph validates and simulates end-to-end.
- 75 tests passing, 0 warnings.

### 2.2 BaaLS

BaaLS is the first/reference ledger target. It is a local-first embeddable blockchain runtime written in Rust, designed for tamper-evident state transitions, deterministic execution, optional peer sync, and constrained WASM smart-contract execution.

Current repo status notes:

- Crate: `baals`
- Binary: `baalsd`
- Default consensus: PoA
- Default API port: `8080`
- Health endpoint: `/health` and `/api/v1/health`
- Storage backends: `sled` and `redb`
- CLI, HTTP API, Rust crate, Go SDK path, and Node SDK paths
- WebSocket event bus for blocks, transactions, and mempool events

### 2.3 ChronoNode

ChronoNode is an independent verifiable archival layer for blockchain and app-ledger history. It archives blocks/events into content-addressed storage, stores compact metadata locally, and serves historical queries with Merkle proofs.

Current repo status notes:

- Early design/build phase.
- Not production-ready.
- BaaLS is the first reference adapter.
- Additional networks should be supported through `ChainAdapter`.
- Local filesystem, IPFS, and Pinata storage modes are planned/available through environment selection.
- The intended production topology separates ingest and API processes, even on one VPS.

### 2.4 Resurgence Protocol

Resurgence Protocol is a Proof-of-Dormancy staking protocol for abandoned ERC-20 tokens. Users stake dead coins and earn RESURGE, the protocol’s governance/rewards token.

Current repo status notes:

- Pre-audit.
- 223 tests passing according to README.
- Arbitrum mainnet is the target hub chain.
- UUPS-upgradeable contracts.
- Governance/timelock-controlled upgrades.
- No significant value should be deposited before professional audit completion.

---

## 3. Product Positioning

Canvas Contracts should become the visual studio layer for contract logic across local-first and networked runtimes. Its first production lane should be BaaLS because BaaLS is WASM-native, local-first, and has an embedded developer workflow. ChronoNode then gives Canvas Contracts historical proof and replay capabilities. Resurgence gives Canvas Contracts a real-world protocol template/use case: proof-of-dormancy detection and oracle-style logic that can be visually composed, simulated, audited, and exported.

### 3.1 Target Users

- **Aspiring Decentralizer:** wants smart-contract logic without learning low-level blockchain development.
- **Polyglot Developer:** wants WASM contracts without committing to a chain-specific DSL.
- **Business Logic Designer:** wants visual contract terms, rule flows, and auditable decisions.
- **Web3 Builder:** wants faster contract prototyping, simulation, and deployment.
- **Protocol Engineer:** wants visual oracle/risk/eligibility logic for projects such as Resurgence.

### 3.2 Non-goals

Canvas Contracts should not initially attempt to replace Solidity tooling, Hardhat, or Foundry for Resurgence itself. Resurgence remains an EVM protocol. Canvas Contracts should integrate with it through templates, proof generation, off-chain/oracle logic, and future verified adapter outputs.

Canvas Contracts should not hardcode BaaLS-only assumptions into the core graph IR. BaaLS should remain the first adapter and reference runtime.

ChronoNode should not depend on Canvas Contracts. It should remain an independent archival/proof layer usable by BaaLS, Canvas Contracts, Resurgence workflows, and future networks.

---

## 4. Integrated Architecture

```text
+---------------------------------------------------------------+
|                     Canvas Contracts IDE                      |
|                                                               |
|  React/Tauri UI                                               |
|  - CanvasEditor                                               |
|  - NodePalette                                                |
|  - Toolbar                                                    |
|  - PropertyPanel                                              |
|  - ContractMonitor                                            |
|  - Debugger UI                                                |
|                                                               |
|  Rust Backend                                                 |
|  - Graph parser                                               |
|  - Graph validator                                            |
|  - Graph executor/simulator                                   |
|  - Graph IR -> AST -> WASM compiler                           |
|  - Wasmtime validation/runtime                                |
|  - Adapter clients                                            |
+--------------------+----------------------+-------------------+
                     |                      |
                     | BaaLS Adapter         | Resurgence Adapter/Templates
                     |                      |
                     v                      v
+------------------------------------------------+     +------------------------------+
|                     BaaLS                      |     |      Resurgence Protocol      |
|                                                |     |                              |
| - Local-first ledger runtime                   |     | - RESURGE ERC-20/governance  |
| - PoA consensus                                |     | - DeadCoinStakingPool        |
| - WASM contract engine                         |     | - RewardDistributor          |
| - sled/redb storage                            |     | - StakingPoolManager         |
| - CLI / HTTP API / WebSocket API               |     | - Timelock/Governance        |
+--------------------+---------------------------+  +------------------------------+
                     |
                     | ChainAdapter / block export
                     v
+---------------------------------------------------------------+
|                         ChronoNode                            |
|                                                               |
| - ChainAdapter registry                                       |
| - BaaLS reference adapter                                     |
| - ChronoBlock protobuf canonical model                        |
| - Content-addressed storage: local_fs/IPFS/Pinata              |
| - SQLite metadata index                                       |
| - Merkle checkpoints/proofs                                   |
| - HTTP query/proof API                                        |
+---------------------------------------------------------------+
```

---

## 5. Canvas Contracts Core System

### 5.1 Visual Graph Editor

The graph editor is the primary user interface. It should support:

- Drag-and-drop node creation.
- Directed edge connections.
- Type-safe port connections.
- Graph validation feedback.
- Undo/redo.
- Project save/load.
- Node property editing.
- Breakpoints and step-through debugging.
- Contract monitor panel for deployed contracts.

Current frontend stack:

```text
React + Tauri desktop app
@xyflow/react for visual graph editor
Zustand for frontend state with undo/redo
```

### 5.2 Current Node Types

The current 14 implemented node types are:

| Node | Category | Purpose |
|---|---|---|
| Start | Control | Graph entry point |
| End | Control | Graph exit point |
| Add | Arithmetic | i64 addition |
| Sub | Arithmetic | i64 subtraction |
| Mul | Arithmetic | i64 multiplication |
| Div | Arithmetic | i64 division |
| And | Logic | Boolean AND |
| Or | Logic | Boolean OR |
| Not | Logic | Boolean NOT |
| If | Logic | Conditional branch |
| ReadStorage | State | Read from contract KV state |
| WriteStorage | State | Write to contract KV state |
| VerifySignature | Crypto | Ed25519 signature verification |
| DecodeProof | Crypto | JSON proof decoding/deserialization |

### 5.3 Required Future Node Types

To fully support BaaLS + ChronoNode + Resurgence workflows, add these node groups:

#### BaaLS runtime nodes

```text
GetSender
GetContractId
GetBlockTimestamp
GetBlockHeight
EmitEvent
Revert
CallContract
ReadCallResult
HashSha256
TransferValue
```

#### ChronoNode proof nodes

```text
FetchChronoBlock
FetchCheckpoint
VerifyChronoProof
ExtractChronoEvent
ExtractTxBySender
ExtractTxByRecipient
VerifyArchiveRange
```

#### Resurgence/DormancyOracle nodes

```text
CheckTokenAge
CheckTokenActivityWindow
CheckLiquidityDormancy
CheckGovernanceDormancy
CalculateDormancyScore
NormalizeDeadCoinRisk
GenerateDormancyProof
EmitDormancyOracleResult
```

### 5.4 Graph Validation Rules

Canvas Contracts must validate graphs before simulation or compilation.

Required validation:

- Type-compatible ports.
- Required inputs connected.
- Single valid entry point unless multi-entry graph mode is explicitly enabled.
- No unintended cycles.
- No unreachable nodes.
- No dangling required outputs.
- No state writes in read-only simulation mode.
- Storage writes must have deterministic key/value types.
- Crypto operations must specify algorithm and input length constraints.
- Resurgence template graphs must include audit labels and oracle-output disclaimers.

### 5.5 Graph Compilation Pipeline

```text
Graph JSON
  -> Graph parser
  -> Graph validator
  -> Graph IR
  -> AST / control-flow representation
  -> WASM generation through wasm-encoder
  -> Wasmtime validation
  -> ABI/manifest generation
  -> BaaLS deployable WASM artifact
```

Required outputs:

```text
contract.wasm
contract.abi.json
contract.manifest.json
graph.lock.json
validation-report.json
```

`graph.lock.json` should include graph hash, compiler version, node versions, adapter target, and deterministic build metadata.

---

## 6. BaaLS Integration Specification

### 6.1 Integration Role

BaaLS is Canvas Contracts’ first deployment and simulation target. It provides:

- Local/offline node execution.
- WASM contract runtime.
- Deterministic ledger/state transitions.
- Storage-backed contract state.
- HTTP API and SDK clients.
- WebSocket event feed for deployed-contract monitoring.

### 6.2 Adapter Rule

Canvas Contracts must define a generic ledger/runtime adapter interface and implement BaaLS as the first adapter.

```rust
pub trait LedgerAdapter {
    fn validate_runtime(&self) -> Result<RuntimeInfo>;
    fn simulate_contract(&self, wasm: &[u8], input: SimulationInput) -> Result<SimulationResult>;
    fn deploy_contract(&self, wasm: &[u8], args: DeployArgs, signer: SignerRef) -> Result<DeployResult>;
    fn call_contract(&self, contract_id: String, method: String, args: Vec<Vec<u8>>, signer: SignerRef) -> Result<CallResult>;
    fn query_contract(&self, contract_id: String, method: String, args: Vec<Vec<u8>>) -> Result<QueryResult>;
    fn subscribe_events(&self, channels: Vec<EventChannel>) -> Result<EventStream>;
}
```

### 6.3 BaaLS API Mapping

| Canvas action | BaaLS operation |
|---|---|
| Validate runtime | `GET /health` or `/api/v1/health` |
| Deploy graph contract | `tx deploy-contract` / deploy API |
| Call deployed contract | contract call tx / call API |
| Query read-only contract | read-only contract call endpoint |
| Monitor contract | WebSocket channels: blocks, transactions, mempool |
| Query account/state | `query account`, `query contract-state` |
| Generate proof | BaaLS proof endpoints or `baalsd proof` |

### 6.4 Gas and Fee Policy

Canvas Contracts should expose gas estimation visually. BaaLS should support three fee modes:

```text
none      -> local embedded mode; gas prevents runaway execution only
metered   -> credits/quotas for SaaS or enterprise deployments
economic  -> real tx fee split for networked mode
```

Recommended economic split policy:

```text
70% operator / validator
20% protocol treasury
10% burn / reserve
```

Also allow simple mode:

```text
70% operator / validator
30% treasury
```

The split must be configurable and chain-configured, not hardcoded into Canvas Contracts.

### 6.5 BaaLS Hardening Requirements Before Production

Before Canvas Contracts targets BaaLS for untrusted users or valuable workflows, the BaaLS integration must require:

- Current patched Wasmtime line.
- `cargo audit` and `cargo deny` clean.
- Host-function gas failures must abort execution.
- Contract side effects must be atomic with block application.
- Synced/imported blocks must pass consensus + ledger validation.
- P2P must use authenticated peer identity for production.
- Keystore file permissions and key zeroization must be hardened.
- Malicious-WASM tests must pass.

---

## 7. ChronoNode Integration Specification

### 7.1 Integration Role

ChronoNode gives Canvas Contracts a historical proof and replay layer. It should be used for:

- Contract execution history.
- Historical block lookups.
- Contract event timelines.
- Merkle proof generation.
- Checkpoint export.
- Audit evidence bundles.
- DormancyOracle evidence collection for Resurgence-style workflows.

### 7.2 ChronoNode Flow

```text
BaaLS block/event
  -> BaaLS ChainAdapter
  -> ChronoBlock canonical model
  -> protobuf serialization
  -> content-addressed storage
  -> SQLite metadata index
  -> Merkle checkpoint
  -> HTTP API query/proof
  -> Canvas ContractMonitor / ProofPanel
```

### 7.3 Canvas UI Features Backed by ChronoNode

Add a `Proof & History` panel to Canvas Contracts:

```text
Contract History
- blocks involving this contract
- tx timeline
- emitted events
- storage changes if available

Proof Explorer
- block proof
- event proof
- checkpoint proof
- verify proof JSON

Audit Export
- graph source hash
- WASM hash
- deployment tx
- relevant blocks/events
- ChronoNode checkpoint proof
```

### 7.4 ChronoNode API Mapping

| Canvas feature | ChronoNode endpoint |
|---|---|
| Health/status | `GET /health` |
| List chains | `GET /v1/chains` |
| Block by height | `GET /v1/chains/{chain_id}/blocks/{height}` |
| Block range | `GET /v1/chains/{chain_id}/blocks?from=0&to=100` |
| Block proof | `GET /v1/chains/{chain_id}/proofs/block/{height}` |
| Verify proof | `POST /v1/proofs/verify` |
| Tx by sender | `GET /v1/chains/{chain_id}/txs/sender/{sender}` |
| Tx by recipient | `GET /v1/chains/{chain_id}/txs/recipient/{recipient}` |
| Events by type | `GET /v1/chains/{chain_id}/events/{event_type}` |
| Metrics | `GET /metrics` |
| API docs | `GET /api-docs` |

### 7.5 ChronoNode Adapter Boundary

Canvas Contracts should not assume ChronoNode only archives BaaLS. ChronoNode’s core contract is `ChainAdapter` and canonical `ChronoBlock` conversion. BaaLS is the reference adapter.

Required adapter outputs:

```rust
pub struct CanonicalBlockRef {
    pub chain_id: String,
    pub height: u64,
    pub hash: String,
    pub timestamp: u64,
    pub storage_cid: String,
    pub tx_count: u64,
    pub event_count: u64,
}
```

---

## 8. Resurgence Protocol Integration Specification

### 8.1 Integration Role

Resurgence Protocol is the first external protocol/use-case integration. It should not be embedded directly into Canvas Contracts core. Instead, it should appear as:

- A template pack.
- A visual DormancyOracle graph.
- A contract-analysis ruleset.
- A ChronoNode-backed proof/evidence workflow.
- Optional future adapter for EVM/Hardhat deployment metadata.

### 8.2 Resurgence Protocol Overview

Resurgence Protocol includes:

- `ResurgeToken`: ERC-20 + votes/permit/capped token.
- `DeadCoinStakingPool`: per-dead-coin staking pool.
- `ResurgeStakingPool`: native RESURGE staking with boosts and early-unstake penalty.
- `StakingPoolManager`: deploys/configures dead coin pools.
- `RewardDistributor`: mints rewards and enforces cap.
- `ResurgenceGovernance`: governance voting.
- `ResurgenceTimelockController`: delayed execution of privileged actions.

### 8.3 DormancyOracle Template

Canvas Contracts should ship a `DormancyOracle` project template with graph modules that evaluate whether a token appears abandoned/dormant.

Inputs:

```text
token_address
chain_id
lookback_window_days
min_inactivity_score
liquidity_threshold
holder_distribution_threshold
governance_activity_window
optional_chrononode_proof
optional_subgraph_snapshot
```

Outputs:

```text
dormancy_score: u64
risk_label: Active | Dormant | Abandoned | Unknown
evidence_hash: bytes32
proof_manifest: JSON
oracle_result_event: JSON/Event payload
```

Recommended graph stages:

```text
Start
  -> Fetch token metadata
  -> Fetch activity evidence
  -> Verify ChronoNode proof if present
  -> Calculate inactivity score
  -> Calculate liquidity dormancy score
  -> Calculate governance dormancy score
  -> Normalize final score
  -> Emit DormancyOracleResult
  -> End
```

### 8.4 Resurgence Boundary Conditions

Resurgence is pre-audit, so Canvas Contracts must display strong warnings when using Resurgence templates:

```text
This template is for simulation, analysis, and oracle design.
It is not a financial recommendation.
It must not be used to deposit significant value before Resurgence audit completion.
EVM contract deployment remains owned by the Resurgence Hardhat pipeline unless an EVM adapter is explicitly implemented.
```

### 8.5 Future Resurgence/EVM Adapter

A future adapter may read deployment manifests from Resurgence and generate Canvas visual overlays for:

- Pool lifecycle.
- Reward distribution rules.
- Governance proposal execution paths.
- Timelock-controlled upgrades.
- Dead coin eligibility evidence.

This adapter should be read/analysis-first. Write/deploy operations to EVM should stay outside MVP.

---

## 9. Cross-System Data Contracts

### 9.1 Canvas Graph Manifest

```json
{
  "schema_version": "canvas.graph.v1",
  "project_name": "DormancyOracle",
  "target_adapter": "baals",
  "nodes": [],
  "edges": [],
  "compiler": {
    "version": "0.2-current",
    "wasm_target": "wasm32-unknown-unknown"
  },
  "integrations": {
    "baals": {
      "enabled": true,
      "api_url": "http://127.0.0.1:8080"
    },
    "chrononode": {
      "enabled": true,
      "api_url": "http://127.0.0.1:8080",
      "chain_id": "baals-local"
    },
    "resurgence": {
      "enabled": true,
      "mode": "template-only"
    }
  }
}
```

### 9.2 Deployment Manifest

```json
{
  "schema_version": "canvas.deploy.v1",
  "graph_hash": "sha256:...",
  "wasm_hash": "sha256:...",
  "target": "baals",
  "contract_id": "...",
  "deployer": "...",
  "deploy_tx": "...",
  "block_height": 123,
  "chrononode_checkpoint": "optional-checkpoint-id"
}
```

### 9.3 DormancyOracle Result Manifest

```json
{
  "schema_version": "resurgence.dormancy_oracle.v1",
  "token_address": "0x...",
  "chain_id": "arbitrum",
  "dormancy_score": 87,
  "risk_label": "Dormant",
  "evidence_hash": "0x...",
  "evidence_sources": [
    "chrononode-proof",
    "subgraph-snapshot",
    "manual-review"
  ],
  "generated_by": {
    "canvas_graph_hash": "sha256:...",
    "wasm_hash": "sha256:..."
  }
}
```

---

## 10. Command Surface

### 10.1 Canvas Contracts

```bash
make install
cargo build
cargo test

canvas-contracts validate --input graph.json
canvas-contracts simulate --graph tests/fixtures/simple_arithmetic.json
canvas-contracts compile --input graph.json --output out.wasm
canvas-contracts deploy --contract out.wasm --key my-key --args '{}'

cd frontend
npm install
npm run dev
npm run tauri dev
```

### 10.2 BaaLS

```bash
cargo build --release --locked
./target/release/baalsd node config init --output config.toml
./target/release/baalsd node start --data-dir ./data --port 8080
curl http://127.0.0.1:8080/health
./target/release/baalsd wallet create --name alice
./target/release/baalsd query head --data-dir ./data
```

### 10.3 ChronoNode

```bash
docker compose up

docker compose --profile ipfs up

cargo build --release --workspace
./target/release/chrononode-cli init
./target/release/chrononode-cli ingest --chain mock --from 0
./target/release/chrononode-cli query block --chain mock --height 0
./target/release/chrononode-cli prove --chain mock --height 0
```

### 10.4 Resurgence Protocol

```bash
npm install
npx hardhat compile
npx hardhat test
REPORT_GAS=true npx hardhat test
npx hardhat run scripts/deployResurgenceProtocol.js --network localhost
npx hardhat run scripts/deployResurgenceProtocol.js --network arbitrumSepolia
```

---

## 11. Testing Strategy

### 11.1 Canvas Contracts Tests

Required test categories:

```text
Graph parser tests
Graph validation tests
Graph simulation tests
Graph-to-WASM compile tests
Wasmtime validation tests
BaaLS mock client tests
BaaLS HTTP integration tests
Debugger step-through tests
DormancyOracle fixture tests
Frontend type-check tests
```

Commands:

```bash
cargo test
cd frontend && npx tsc --noEmit
```

### 11.2 BaaLS Integration Tests

Required test categories:

```text
Deploy Canvas-generated WASM to BaaLS
Call Canvas-generated contract
Query Canvas-generated contract state
Subscribe to WebSocket block/tx events
Recover deployment manifest after restart
Verify gas/fee estimation matches BaaLS execution
Reject malicious or invalid WASM
Rollback failed contract side effects
```

### 11.3 ChronoNode Integration Tests

Required test categories:

```text
Ingest BaaLS blocks
Convert BaaLS block to ChronoBlock
Store content-addressed block payload
Index block/tx/event metadata
Query block by height/hash
Generate block proof
Verify proof from Canvas UI
Export audit bundle
```

### 11.4 Resurgence Integration Tests

Required test categories:

```text
DormancyOracle graph validates
DormancyOracle graph simulates deterministic input
DormancyOracle output manifest matches schema
Invalid proof is rejected
Unknown token returns Unknown risk label
Pre-audit warning appears in UI
No EVM write/deploy action occurs from Canvas MVP
```

---

## 12. Production Readiness Requirements

### 12.1 Shared CI Gate

Every repo should run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --release --all-features
cargo audit
cargo deny check
```

For Node/Hardhat/Frontend repos:

```bash
npm ci
npm run lint
npm test
npm run build
npx hardhat test
REPORT_GAS=true npx hardhat test
```

### 12.2 Security Rules

- No stubs in production paths.
- No silent gas-limit failures.
- No unsigned/untrusted block imports.
- No unauthenticated production P2P.
- No unreviewed upgrade paths.
- No private key leakage in logs.
- No Resurgence deposit/value movement before audit readiness.
- No Canvas-generated oracle result should be treated as authoritative unless its evidence manifest verifies.

---

## 13. Roadmap

### Phase 0 — Interface Freeze

- Freeze `LedgerAdapter` trait.
- Freeze `ChronoNodeClient` interface.
- Freeze graph manifest schema.
- Freeze deployment manifest schema.
- Freeze DormancyOracle result schema.

### Phase 1 — BaaLS Reference Integration

- Deploy Canvas-generated WASM to local BaaLS.
- Query contract state through BaaLS.
- Subscribe to BaaLS WebSocket events.
- Show deployment details in ContractMonitor.
- Add gas/fee visualizer.

### Phase 2 — ChronoNode Proof Integration

- Ingest BaaLS block/event history.
- Expose Contract History panel in Canvas UI.
- Generate block/event proof from Canvas UI.
- Export audit bundle containing graph hash, WASM hash, deployment tx, and ChronoNode proof.

### Phase 3 — Resurgence DormancyOracle Pack

- Ship DormancyOracle template.
- Add Resurgence warning and audit status metadata.
- Add dormancy scoring node pack.
- Add proof/evidence manifest export.
- Add read-only EVM/Resurgence manifest viewer.

### Phase 4 — Hardening and Release

- Malicious WASM tests.
- Fuzz graph JSON parser.
- Fuzz ChronoNode proof parser.
- Full CI across all repos.
- Security documentation.
- Versioned schemas.
- Signed release artifacts.

---

## 14. Final Design Decision

Canvas Contracts should be the developer-facing visual layer. BaaLS should be the first supported execution/runtime target. ChronoNode should provide independent archival and proof capabilities. Resurgence should be treated as a high-value integration and demonstration protocol, especially through DormancyOracle, but not hardwired into the core deployment path until a formal EVM adapter exists.

The most valuable combined product is:

```text
Visual contract logic -> WASM runtime deployment -> verifiable archive -> proof-backed external protocol integration
```

That gives Canvas Contracts a unique identity beyond a visual editor: it becomes a full verifiable contract design, deployment, monitoring, and proof workflow.

---

## 15. Reviewed Source Repositories

- `kdsmith18542/CanvasContracts`
- `kdsmith18542/BaaLS`
- `kdsmith18542/chrononode`
- `kdsmith18542/Resurgence-Protocol`

