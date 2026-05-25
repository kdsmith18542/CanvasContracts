# CanvasContracts Verifiable WASM Artifact Pipeline — Technical Specification v0.1

**Project:** CanvasContracts  
**Feature Area:** WIT ABI + WASM validation + BaaLS runtime compatibility + signed manifests + ChronoNode archival  
**Target Repos:** `CanvasContracts`, `BaaLS`, `ChronoNode`, `Resurgence-Protocol`  
**Status:** Proposed implementation spec  
**Primary Goal:** Make every CanvasContracts build produce a verifiable, reproducible, BaaLS-compatible smart-contract artifact bundle.

---

## 1. Executive Summary

CanvasContracts already has the right foundation: a visual graph editor, graph validation, graph simulation, WASM compilation, Wasmtime validation, a BaaLS client, and a DormancyOracle fixture. The next high-impact feature should turn CanvasContracts from a visual builder into a **verifiable contract artifact platform**.

This feature adds a formal artifact pipeline:

```text
Visual Graph
  -> Graph IR
  -> AST
  -> Core WASM
  -> WIT ABI / Component metadata
  -> wasm-tools validation
  -> BaaLS runtime compatibility report
  -> Contract manifest / SBOM
  -> Optional signing
  -> BaaLS deployment
  -> ChronoNode archival
```

The result is a deployable bundle that can be reviewed, archived, reproduced, and verified by BaaLS, ChronoNode, Resurgence, and external developers.

---

## 2. Current Repo Findings

### 2.1 Existing Strengths

CanvasContracts currently exposes core modules for:

```text
compiler
nodes
wasm
baals
debugger
error
types
config
```

The current public API re-exports `Compiler`, `NodeRegistry`, `WasmRuntime`, `BaalsClient`, `MockBaalsClient`, and debugger types.

The README claims the project already supports:

```text
visual graph editor
14 node types
graph validation
graph simulation
real WASM compilation
Wasmtime runtime
BaaLS integration
frontend
debugger
DormancyOracle graph
```

### 2.2 Current Compiler Shape

The compiler pipeline currently follows:

```text
VisualGraph
  -> GraphIR
  -> AST
  -> WasmGenerator
  -> Wasmtime module validation
  -> ABI generation
  -> gas estimation
```

The current `WasmGenerator` uses `wasm-encoder` and produces a core WASM module with a `main` export. The generated code currently focuses on i64 arithmetic, boolean/condition logic, if/else, simple calls, imports, and exports.

### 2.3 Current BaaLS Client Shape

The current BaaLS integration has:

```text
BaalsClient trait
MockBaalsClient
HttpBaalsClient
JWT auth request to /api/v1/auth/token
contract deploy request
contract invoke request
contract state/proof reads
transaction finality lookup
```

The HTTP client is close to what Canvas needs, but production hardening is still needed around key derivation, endpoint compatibility, retry behavior, status parsing, error reporting, and test coverage against a live BaaLS node.

### 2.4 Gaps This Spec Addresses

```text
No formal WIT ABI layer.
No component metadata or interface package.
No signed contract manifest/SBOM.
No strict BaaLS runtime compatibility profile.
No wasm-tools validation/reporting stage.
Gas estimate is still simplistic.
No ChronoNode archival bundle for graph/manifest/WASM.
No reproducible build hash story.
No node-pack registry format.
No public security report artifact.
Canvas public README and local agent notes appear partially out of sync.
```

---

## 3. Feature Goals

### 3.1 Primary Goals

1. Define a canonical BaaLS contract ABI using WIT.
2. Validate generated WASM with `wasm-tools` in addition to Wasmtime.
3. Produce a signed contract manifest for every compile.
4. Enforce a BaaLS runtime compatibility profile.
5. Archive graph + WASM + manifest to ChronoNode.
6. Expose a frontend “Contract Safety Report.”
7. Make the DormancyOracle graph a flagship end-to-end demo.

### 3.2 Non-Goals

The first version should **not** attempt to:

```text
replace BaaLS runtime validation
support every WASI interface
deploy to random EVM chains
create a marketplace immediately
prove compilation with zkVM immediately
make CanvasContracts a rollup framework
```

---

## 4. High-Level Architecture

```text
┌─────────────────────┐
│ Visual Graph Editor │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Graph Validator     │
│ - type checks       │
│ - reachability      │
│ - auth linting      │
│ - storage linting   │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Graph IR / AST      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ WASM Generator      │
│ - core wasm v1      │
│ - WIT metadata      │
│ - host imports      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Validation Pipeline │
│ - wasmtime validate │
│ - wasm-tools        │
│ - import whitelist  │
│ - memory/fuel check │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Artifact Manifest   │
│ - graph hash        │
│ - wasm hash         │
│ - ABI hash          │
│ - compiler version  │
│ - runtime profile   │
│ - safety report     │
└──────────┬──────────┘
           │
           ├──────────────► BaaLS Deploy
           │
           └──────────────► ChronoNode Archive
```

---

## 5. New Modules

### 5.1 `src/abi/`

Purpose: Manage WIT interfaces and Canvas ABI generation.

```text
src/abi/
  mod.rs
  wit.rs
  schema.rs
  canonical.rs
  compatibility.rs
```

Responsibilities:

```text
Generate WIT from graph ABI.
Parse WIT packages.
Validate contract exports against target world.
Map Canvas ValueType to WIT types.
Generate canonical ABI hash.
Emit JSON ABI for frontend and BaaLS SDK tooling.
```

### 5.2 `src/artifact/`

Purpose: Generate the deployable artifact bundle.

```text
src/artifact/
  mod.rs
  manifest.rs
  bundle.rs
  hash.rs
  signer.rs
  sbom.rs
```

Responsibilities:

```text
Build ContractManifest.
Hash graph JSON deterministically.
Hash WASM bytes.
Hash WIT package.
Package graph + wasm + manifest + validation report.
Optionally sign manifest with Ed25519.
Verify manifest signatures.
```

### 5.3 `src/validation/`

Purpose: Strict WASM and runtime compatibility checks.

```text
src/validation/
  mod.rs
  wasm_tools.rs
  import_policy.rs
  runtime_profile.rs
  safety_report.rs
```

Responsibilities:

```text
Run wasmparser validation.
Run import/export policy checks.
Check memory limits.
Check forbidden sections.
Check disallowed WASM features.
Check required exports.
Check host function ABI.
Generate safety report.
```

### 5.4 `src/chrononode/`

Purpose: Archive Canvas artifact bundles.

```text
src/chrononode/
  mod.rs
  client.rs
  archive.rs
  proof.rs
```

Responsibilities:

```text
Submit artifact bundle to ChronoNode.
Store returned content pointer.
Fetch archived artifact by hash.
Verify archived artifact hash.
Link deployment hash to archived bundle.
```

### 5.5 `src/node_packs/`

Purpose: Standardize reusable node packs.

```text
src/node_packs/
  mod.rs
  manifest.rs
  registry.rs
  resolver.rs
```

Responsibilities:

```text
Load node-pack manifests.
Validate node-pack WIT dependencies.
Map node-pack visual behavior to compiler backend.
Support versioned node packs.
Generate node-pack security metadata.
```

---

## 6. WIT ABI Design

### 6.1 Directory Layout

```text
wit/
  baals-contract-v1/
    package.wit
    types.wit
    storage.wit
    crypto.wit
    proof.wit
    contract.wit
```

### 6.2 Example WIT Package

```wit
package baals:contract@1.0.0;

interface types {
  type address = list<u8>;
  type bytes32 = list<u8>;

  record call-context {
    caller: address,
    contract: address,
    value: u64,
    gas-limit: u64,
  }

  variant storage-error {
    not-found,
    permission-denied,
    invalid-key,
    host-error(string),
  }
}

interface storage {
  use types.{storage-error};

  read: func(key: string) -> result<option<list<u8>>, storage-error>;
  write: func(key: string, value: list<u8>) -> result<_, storage-error>;
  delete: func(key: string) -> result<_, storage-error>;
}

interface crypto {
  verify-ed25519: func(pubkey: list<u8>, message: list<u8>, signature: list<u8>) -> bool;
  sha256: func(input: list<u8>) -> list<u8>;
}

interface proof {
  decode-json-proof: func(input: list<u8>) -> result<list<u8>, string>;
}

world baals-contract {
  import storage;
  import crypto;
  import proof;

  export init: func(args: list<u8>) -> result<_, string>;
  export call: func(method: string, args: list<list<u8>>) -> result<list<u8>, string>;
  export query: func(method: string, args: list<list<u8>>) -> result<list<u8>, string>;
}
```

### 6.3 Versioning Rules

```text
baals:contract@1.0.0 = stable initial target
baals:contract@1.1.0 = additive host functions only
baals:contract@2.0.0 = breaking ABI change
```

Canvas must store the target WIT package version in every artifact manifest.

---

## 7. BaaLS Runtime Compatibility Profile

### 7.1 Profile Name

```text
baals-wasm-v1
```

### 7.2 Allowed Exports

```text
main          legacy compatibility
init          optional component-compatible lifecycle
call          required for method calls
query         required for read-only calls
```

### 7.3 Allowed Imports

```text
baals.storage.read
baals.storage.write
baals.storage.delete
baals.crypto.sha256
baals.crypto.verify-ed25519
baals.proof.decode-json-proof
baals.context.caller
baals.context.contract
baals.events.emit
```

### 7.4 Forbidden Capabilities

```text
WASI filesystem
WASI sockets/network
wall-clock time
randomness without deterministic host seed
floating point in consensus-critical paths
threads
shared memory
multi-memory unless explicitly supported by BaaLS
memory64 unless explicitly supported by BaaLS
unbounded recursion
unbounded dynamic allocation
unapproved host imports
```

### 7.5 Resource Limits

Initial defaults:

```toml
[runtime_profile.baals_wasm_v1]
max_wasm_size_bytes = 1048576
max_memory_pages = 16
max_call_depth = 16
max_host_calls = 1000
default_fuel = 1000000
max_fuel = 10000000
allow_float = false
allow_wasi = false
allow_threads = false
```

---

## 8. Validation Pipeline

### 8.1 Validation Stages

```text
Stage 1: Graph validation
Stage 2: Graph security lint
Stage 3: WASM generation
Stage 4: Wasmtime validation
Stage 5: wasm-tools validation
Stage 6: import/export policy
Stage 7: runtime profile check
Stage 8: manifest generation
Stage 9: optional signing
Stage 10: optional deploy/archive
```

### 8.2 wasm-tools Checks

Canvas should add `wasmparser`, `wasmprinter`, `wasm-metadata`, and eventually `wit-component`.

Checks:

```text
validate binary
print WAT for debug view
extract/import section
extract/export section
detect forbidden sections/features
show metadata/producers
strip debug sections for release builds
embed component-type metadata when available
```

### 8.3 Safety Report Schema

```json
{
  "status": "pass",
  "target_profile": "baals-wasm-v1",
  "wasm": {
    "valid": true,
    "size_bytes": 4182,
    "imports": ["baals.storage.read", "baals.storage.write"],
    "exports": ["main"],
    "memory_pages": 1,
    "forbidden_features": []
  },
  "graph": {
    "nodes": 12,
    "connections": 15,
    "cycles": 0,
    "unreachable_nodes": 0,
    "storage_writes": 2,
    "auth_guards": 2
  },
  "gas": {
    "estimate": 84100,
    "max_configured": 1000000
  },
  "warnings": [],
  "errors": []
}
```

---

## 9. Contract Manifest / SBOM

### 9.1 Manifest File Name

```text
canvas.contract.json
```

### 9.2 Manifest Schema

```json
{
  "schema": "canvas.contract.manifest.v1",
  "name": "DormancyOracle",
  "version": "0.1.0",
  "target": "baals-wasm-v1",
  "created_at": "2026-05-24T00:00:00Z",
  "compiler": {
    "name": "canvas-contracts",
    "version": "0.1.0",
    "git_commit": "unknown",
    "wasm_encoder_version": "0.38",
    "wasmtime_validation_version": "15.0"
  },
  "source": {
    "graph_hash": "sha256:...",
    "graph_canonicalization": "canvas-json-c14n-v1",
    "node_pack_lock_hash": "sha256:..."
  },
  "abi": {
    "wit_package": "baals:contract@1.0.0",
    "wit_hash": "sha256:...",
    "json_abi_hash": "sha256:..."
  },
  "artifact": {
    "wasm_hash": "sha256:...",
    "wasm_size_bytes": 4182,
    "exports": ["main"],
    "imports": ["baals.storage.read", "baals.storage.write"]
  },
  "runtime": {
    "profile": "baals-wasm-v1",
    "max_memory_pages": 16,
    "default_fuel": 1000000,
    "deterministic": true
  },
  "validation": {
    "safety_report_hash": "sha256:...",
    "status": "pass",
    "warnings": [],
    "errors": []
  },
  "deployment": {
    "network": "baals-local",
    "contract_id": null,
    "transaction_hash": null,
    "block_height": null
  },
  "archive": {
    "chrononode_pointer": null,
    "checkpoint_id": null,
    "checkpoint_root": null
  },
  "signatures": [
    {
      "algorithm": "ed25519",
      "public_key": "hex...",
      "signature": "hex..."
    }
  ]
}
```

### 9.3 Required Hashes

```text
graph_hash
wasm_hash
wit_hash
json_abi_hash
node_pack_lock_hash
safety_report_hash
bundle_hash
```

---

## 10. Artifact Bundle Format

### 10.1 Bundle Directory

```text
dist/contracts/<contract-name>/
  graph.json
  graph.canonical.json
  contract.wasm
  contract.wat
  canvas.contract.json
  safety-report.json
  abi.json
  wit/
    package.wit
    types.wit
    storage.wit
    crypto.wit
    proof.wit
    contract.wit
  node-pack.lock
```

### 10.2 Bundle Archive

```text
<contract-name>-<bundle-hash>.canvasbundle.tar.zst
```

### 10.3 Bundle Verification

Command:

```bash
canvas-contracts artifact verify dist/contracts/DormancyOracle/canvas.contract.json
```

Checks:

```text
Recalculate hashes.
Validate signature.
Validate WASM.
Validate WIT hash.
Validate safety report hash.
Optionally fetch ChronoNode artifact and compare bundle hash.
```

---

## 11. ChronoNode Integration

### 11.1 Purpose

ChronoNode should archive Canvas artifacts so that contract logic, graph source, manifest, and deployed WASM can be proven later.

### 11.2 Archive Flow

```text
canvas-contracts compile
  -> produce bundle
canvas-contracts archive --bundle dist/...tar.zst
  -> POST to ChronoNode
  -> get storage pointer
  -> update manifest archive section
  -> optionally checkpoint manifest hash
```

### 11.3 API Assumption

Initial simple API:

```http
POST /v1/artifacts
Content-Type: application/octet-stream

Response:
{
  "storage_pointer": "local_fs:...",
  "content_hash": "sha256:...",
  "checkpoint_id": null
}
```

If ChronoNode does not yet expose this exact route, implement a small compatibility endpoint or use its existing storage/checkpoint API as a backend.

---

## 12. BaaLS Deployment Integration

### 12.1 Deploy Flow

```text
canvas-contracts deploy \
  --bundle dist/contracts/DormancyOracle/DormancyOracle-<hash>.canvasbundle.tar.zst \
  --baals-url https://127.0.0.1:18080 \
  --key-env CANVAS_BAALS_DEPLOYER_KEY
```

Steps:

```text
Load manifest.
Verify bundle.
Request BaaLS JWT token.
Deploy WASM.
Poll finality.
Update manifest deployment section.
Archive updated manifest.
Display deployment receipt.
```

### 12.2 Key Handling Fix

The current HTTP client needs to replace any “use private key bytes as public key” shortcuts with deterministic Ed25519 key derivation:

```text
private key / seed
  -> SigningKey
  -> VerifyingKey
  -> public_key_hex
```

No private key should be passed around as a plain string in frontend state. CLI should support:

```text
--key-env
--key-file
--wallet-name
```

### 12.3 BaaLS Endpoint Compatibility Test

Add an integration test that proves Canvas can call the current BaaLS API:

```text
POST /api/v1/auth/token
POST /api/v1/contracts/deploy
POST /api/v1/contracts/invoke
GET  /api/v1/transactions/{hash}/finality
GET  /api/v1/proofs/contract/{id}/storage/{key}
```

---

## 13. Frontend UX

### 13.1 New Panels

```text
Artifact Panel
- graph hash
- wasm hash
- WIT version
- BaaLS runtime profile
- validation status
- signature status
- archive pointer
- deployment receipt

Safety Report Panel
- graph risks
- forbidden imports
- storage writes
- auth guards
- gas estimate
- warnings/errors

WIT ABI Panel
- generated interfaces
- host imports
- contract exports
- method schemas

ChronoNode Archive Panel
- archive status
- content hash
- checkpoint root
- proof verify button
```

### 13.2 Toolbar Additions

```text
Validate
Simulate
Compile
Inspect WASM
Generate Manifest
Archive
Deploy
Verify Deployed Artifact
```

### 13.3 User-Facing Build Status

Use clear labels:

```text
Graph Valid
WASM Valid
BaaLS-Compatible
Manifest Signed
Archived
Deployed
Proof Available
```

---

## 14. CLI Additions

### 14.1 Artifact Commands

```bash
canvas-contracts artifact build \
  --input graph.json \
  --out dist/contracts/DormancyOracle

canvas-contracts artifact verify \
  --manifest dist/contracts/DormancyOracle/canvas.contract.json

canvas-contracts artifact sign \
  --manifest dist/contracts/DormancyOracle/canvas.contract.json \
  --key-env CANVAS_SIGNING_KEY

canvas-contracts artifact inspect \
  --manifest dist/contracts/DormancyOracle/canvas.contract.json
```

### 14.2 WIT Commands

```bash
canvas-contracts wit generate \
  --input graph.json \
  --out wit/

canvas-contracts wit validate \
  --wit wit/
```

### 14.3 WASM Commands

```bash
canvas-contracts wasm validate \
  --wasm contract.wasm \
  --profile baals-wasm-v1

canvas-contracts wasm inspect \
  --wasm contract.wasm \
  --json
```

### 14.4 ChronoNode Commands

```bash
canvas-contracts archive submit \
  --bundle dist/contracts/DormancyOracle.canvasbundle.tar.zst \
  --chrononode-url https://chrono.baals.network

canvas-contracts archive verify \
  --content-hash sha256:...
```

### 14.5 BaaLS Commands

```bash
canvas-contracts deploy \
  --manifest canvas.contract.json \
  --baals-url https://127.0.0.1:18080 \
  --key-env CANVAS_BAALS_DEPLOYER_KEY

canvas-contracts call \
  --contract-id <id> \
  --method call \
  --args args.json
```

---

## 15. Node-Pack Registry

### 15.1 Node-Pack Manifest

```json
{
  "schema": "canvas.nodepack.v1",
  "name": "@canvas/baals-storage",
  "version": "1.0.0",
  "description": "BaaLS storage read/write nodes",
  "wit_dependencies": ["baals:contract@1.0.0"],
  "nodes": [
    {
      "type": "ReadStorage",
      "category": "State",
      "gas": 100,
      "imports": ["baals.storage.read"],
      "security": {
        "read_only": true,
        "requires_auth_guard": false
      }
    },
    {
      "type": "WriteStorage",
      "category": "State",
      "gas": 200,
      "imports": ["baals.storage.write"],
      "security": {
        "read_only": false,
        "requires_auth_guard": true
      }
    }
  ]
}
```

### 15.2 Initial Node Packs

```text
@canvas/core
@canvas/baals-storage
@canvas/crypto-ed25519
@canvas/proof-json
@canvas/resurgence-dormancy
@canvas/governance
```

---

## 16. DormancyOracle Flagship Demo

### 16.1 Purpose

This should be the flagship proof that CanvasContracts matters to the ecosystem.

### 16.2 End-to-End Flow

```text
Open DormancyOracle template
Validate graph
Compile to WASM
Generate WIT ABI
Generate manifest
Validate with wasm-tools
Deploy to BaaLS
Submit sample ChronoNode DormancyProof
Store attestation in BaaLS
ChronoNode archives artifact + proof
Resurgence references attestation hash
Frontend displays proof trail
```

### 16.3 Acceptance Criteria

```text
Graph compiles deterministically.
WASM validates under baals-wasm-v1.
Manifest bundle verifies.
BaaLS deploy succeeds.
BaaLS call writes expected attestation.
ChronoNode archive returns content pointer.
Safety report shows pass.
No mock-only data required for final demo path.
```

---

## 17. Testing Strategy

### 17.1 Unit Tests

```text
abi::wit generation
manifest hashing
canonical graph serialization
runtime profile validation
import whitelist
forbidden feature detection
bundle verification
signature verification
```

### 17.2 Integration Tests

```text
compile fixture -> manifest -> verify
compile fixture -> wasm-tools validate
compile fixture -> BaaLS profile pass
compile DormancyOracle -> BaaLS deploy using mock
compile DormancyOracle -> live BaaLS if env set
archive bundle -> ChronoNode if env set
```

### 17.3 E2E Tests

```bash
CANVAS_E2E_BAALS_URL=https://127.0.0.1:18080 \
CANVAS_E2E_CHRONONODE_URL=https://chrono.baals.network \
CANVAS_E2E_KEY_ENV=CANVAS_BAALS_DEPLOYER_KEY \
cargo test --test e2e_artifact_pipeline -- --ignored
```

### 17.4 Golden Tests

Store known outputs:

```text
tests/golden/simple_arithmetic/
  graph.canonical.json
  contract.wasm.sha256
  abi.json
  manifest.json

tests/golden/dormancy_oracle/
  graph.canonical.json
  contract.wasm.sha256
  abi.json
  manifest.json
```

Any compiler change that changes hashes must be intentional.

---

## 18. Implementation Phases

### Phase 1 — Manifest + Runtime Profile

```text
Add artifact module.
Add deterministic graph canonicalization.
Add manifest schema.
Add BaaLS runtime profile.
Add import/export whitelist.
Add CLI artifact build/verify.
```

Estimated complexity: low/medium.

### Phase 2 — wasm-tools Validation

```text
Add wasmparser/wasmprinter/wasm-metadata.
Generate WAT output.
Generate section/import/export report.
Block forbidden features.
Add safety-report.json.
```

Estimated complexity: medium.

### Phase 3 — WIT ABI

```text
Add wit/ package.
Generate WIT from graph ABI.
Add WIT hash to manifest.
Add WIT panel in frontend.
Begin component metadata embedding.
```

Estimated complexity: medium.

### Phase 4 — ChronoNode Archive

```text
Add ChronoNode client.
Submit artifact bundle.
Fetch/verify archived artifact.
Update manifest with archive pointer.
Add frontend archive status panel.
```

Estimated complexity: medium.

### Phase 5 — BaaLS Live Deploy Hardening

```text
Fix key derivation.
Add JWT refresh handling.
Add endpoint compatibility tests.
Add finality polling.
Add deployment receipt.
Add frontend deployment logs.
```

Estimated complexity: medium/high.

### Phase 6 — Node-Pack Registry

```text
Add node-pack manifest.
Add node-pack lock file.
Add node-pack dependency validation.
Add first official node packs.
```

Estimated complexity: medium.

### Phase 7 — zk Proof Prototype

```text
Prototype graph -> WASM compiler proof.
Store proof receipt pointer in manifest.
Do off-chain verification first.
Do not block core launch on this.
```

Estimated complexity: high.

---

## 19. Suggested Cargo Updates

Current dependencies include `wasmtime = "15.0"` and `wasm-encoder = "0.38"`. Add:

```toml
[dependencies]
wasmparser = "0.220"
wasmprinter = "0.220"
wasm-metadata = "0.220"
wit-parser = "0.220"
wit-component = "0.220"
zstd = "0.13"
tar = "0.4"
chrono = { version = "0.4", features = ["serde"] }
blake3 = "1.5"
```

Version numbers should be aligned with the actual `wasm-tools` release family selected during implementation.

Also align the Wasmtime version with the BaaLS runtime target to avoid validating artifacts under one runtime profile and deploying to another.

---

## 20. Configuration

### 20.1 Canvas Config

```toml
[artifact]
enabled = true
output_dir = "dist/contracts"
sign_manifests = true
signing_key_env = "CANVAS_ARTIFACT_SIGNING_KEY"

[runtime_profiles.baals_wasm_v1]
max_wasm_size_bytes = 1048576
max_memory_pages = 16
default_fuel = 1000000
allow_wasi = false
allow_float = false
allow_threads = false

[baals]
node_url = "https://127.0.0.1:18080"
auth_key_env = "CANVAS_BAALS_DEPLOYER_KEY"

[chrononode]
url = "https://chrono.baals.network"
archive_artifacts = true
```

---

## 21. Security Considerations

### 21.1 Must-Have Controls

```text
Never store private keys in graph/project files.
Never expose BaaLS deployer keys to frontend browser state.
Require explicit user confirmation before live deploy.
Validate every host import.
Disallow WASI by default.
Use deterministic fuel/gas policy.
Sign manifests only after validation passes.
Archive immutable artifacts, not mutable project workspace folders.
```

### 21.2 Known Risks

```text
Component Model support may require staged adoption because BaaLS currently runs core WASM.
wasm-tools and Wasmtime feature sets must be aligned.
Graph canonicalization must be stable or artifact hashes will drift.
Node packs can become supply-chain risks.
Frontend signing/deploy flows can become phishing or key-leak risks if not carefully designed.
```

---

## 22. Acceptance Criteria

The feature is complete when:

```text
1. `canvas-contracts artifact build` creates a full bundle.
2. `canvas-contracts artifact verify` passes on the bundle.
3. `wasm-tools validate` or embedded wasmparser validation passes.
4. Runtime profile rejects forbidden imports/features.
5. Manifest includes graph_hash, wasm_hash, WIT hash, validation hash, and compiler version.
6. Frontend shows Contract Safety Report.
7. DormancyOracle bundle deploys to BaaLS.
8. ChronoNode archives the bundle and returns a verifiable pointer.
9. CI runs golden artifact tests.
10. Documentation explains how BaaLS, ChronoNode, and Resurgence consume the artifact.
```

---

## 23. Recommended First PR Breakdown

### PR 1: Artifact manifest foundation

```text
src/artifact/*
canonical graph hash
manifest schema
artifact build/verify CLI
tests
```

### PR 2: Runtime profile validation

```text
src/validation/*
baals-wasm-v1 profile
import/export whitelist
safety-report.json
tests
```

### PR 3: wasm-tools integration

```text
wasmparser validation
WAT output generation
section/import/export report
CLI inspect
tests
```

### PR 4: WIT ABI package

```text
wit/baals-contract-v1/*
src/abi/*
WIT generation
WIT hash in manifest
tests
```

### PR 5: BaaLS deploy hardening

```text
key derivation fix
JWT refresh
finality polling
endpoint compatibility tests
deployment receipt
```

### PR 6: ChronoNode archival

```text
src/chrononode/*
archive submit/verify commands
manifest archive update
tests
```

---

## 24. Final Recommendation

Build this as the next serious CanvasContracts milestone:

```text
CanvasContracts Milestone: Verifiable BaaLS WASM Artifacts
```

This will make CanvasContracts feel legitimate because it will no longer just “draw and compile.” It will produce an auditable artifact trail:

```text
graph source
compiler output
WASM module
WIT ABI
runtime safety report
deployment receipt
ChronoNode archive proof
```

That is exactly the kind of feature that strengthens the whole ecosystem without adding unnecessary blockchain/token complexity.
