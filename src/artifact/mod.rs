pub mod hash;
pub mod manifest;

use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::{
    abi::validate_wit_package,
    compiler::Compiler,
    config::Config,
    error::{CanvasError, CanvasResult},
    types::VisualGraph,
    validation::{baals_wasm_v1_profile, validate_wasm_against_profile},
    wasm::WasmRuntime,
};

use self::{
    hash::{
        canonical_graph_hash, canonical_graph_json, canonicalize_json_value, hash_bytes_prefixed,
        hash_value_prefixed, GRAPH_CANONICALIZATION,
    },
    manifest::{
        default_compiler_section, AbiSection, ArchiveSection, ArtifactSection, ContractManifest,
        DeploymentSection, RuntimeSection, SourceSection, ValidationSection, MANIFEST_SCHEMA,
        TARGET_PROFILE_BAALS_WASM_V1,
    },
};

const DEFAULT_WIT_PACKAGE: &str = "baals:contract@1.0.0";
const MAX_MEMORY_PAGES: u32 = 16;
const DEFAULT_FUEL: u64 = 1_000_000;
const DEFAULT_NODE_PACK_LOCK_SCHEMA: &str = "canvas.nodepack.lock.v1";
const REPO_WIT_DIR: &str = "wit/baals-contract-v1";

const DEFAULT_WIT_FILES: &[(&str, &str)] = &[
    ("package.wit", "package baals:contract@1.0.0;\n"),
    (
        "types.wit",
        "interface types {\n  type address = list<u8>;\n  type bytes32 = list<u8>;\n\n  record call-context {\n    caller: address,\n    contract: address,\n    value: u64,\n    gas-limit: u64,\n  }\n\n  variant storage-error {\n    not-found,\n    permission-denied,\n    invalid-key,\n    host-error(string),\n  }\n}\n",
    ),
    (
        "storage.wit",
        "interface storage {\n  use types.{storage-error};\n\n  read: func(key: string) -> result<option<list<u8>>, storage-error>;\n  write: func(key: string, value: list<u8>) -> result<_, storage-error>;\n  delete: func(key: string) -> result<_, storage-error>;\n}\n",
    ),
    (
        "crypto.wit",
        "interface crypto {\n  verify-ed25519: func(pubkey: list<u8>, message: list<u8>, signature: list<u8>) -> bool;\n  sha256: func(input: list<u8>) -> list<u8>;\n}\n",
    ),
    (
        "proof.wit",
        "interface proof {\n  decode-json-proof: func(input: list<u8>) -> result<list<u8>, string>;\n}\n",
    ),
    (
        "contract.wit",
        "world baals-contract {\n  import storage;\n  import crypto;\n  import proof;\n\n  export init: func(args: list<u8>) -> result<_, string>;\n  export call: func(method: string, args: list<list<u8>>) -> result<list<u8>, string>;\n  export query: func(method: string, args: list<list<u8>>) -> result<list<u8>, string>;\n}\n",
    ),
];

#[derive(Debug, Clone)]
pub struct ArtifactBuildOutput {
    pub output_dir: PathBuf,
    pub graph_path: PathBuf,
    pub canonical_graph_path: PathBuf,
    pub node_pack_lock_path: PathBuf,
    pub wasm_path: PathBuf,
    pub abi_path: PathBuf,
    pub wit_dir: PathBuf,
    pub safety_report_path: PathBuf,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ArtifactVerifyOutput {
    pub manifest_path: PathBuf,
    pub status: String,
    pub graph_hash: String,
    pub node_pack_lock_hash: String,
    pub wasm_hash: String,
    pub wit_hash: String,
    pub json_abi_hash: String,
    pub safety_report_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyReport {
    pub status: String,
    pub target_profile: String,
    pub wasm: WasmReport,
    pub graph: GraphReport,
    pub gas: GasReport,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmReport {
    pub valid: bool,
    pub size_bytes: usize,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub memory_pages: u32,
    pub forbidden_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphReport {
    pub nodes: usize,
    pub connections: usize,
    pub cycles: usize,
    pub unreachable_nodes: usize,
    pub storage_writes: usize,
    pub auth_guards: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasReport {
    pub estimate: u64,
    pub max_configured: u64,
}

pub fn build_artifact_bundle(
    graph: &VisualGraph,
    config: &Config,
    output_dir: &Path,
) -> CanvasResult<ArtifactBuildOutput> {
    std::fs::create_dir_all(output_dir).map_err(CanvasError::Io)?;

    let compiler = Compiler::new(config)?;
    let validator = compiler.validator()?;
    let validation_result = validator.validate(graph)?;
    if !validation_result.is_valid {
        return Err(CanvasError::Validation(format!(
            "Graph validation failed with {} error(s)",
            validation_result.errors.len()
        )));
    }

    let compilation = compiler.compile(graph)?;
    let runtime = WasmRuntime::new(config)?;
    runtime.validate_module(&compilation.wasm_bytes)?;
    let wasm_validation =
        validate_wasm_against_profile(&compilation.wasm_bytes, &baals_wasm_v1_profile())?;

    let mut warnings = validation_result.warnings.clone();
    warnings.extend(wasm_validation.warnings.clone());
    let errors = wasm_validation.errors.clone();
    let imports = wasm_validation.inspection.imports.clone();
    let exports = wasm_validation.inspection.exports.clone();

    let mut forbidden_features = Vec::new();
    if wasm_validation.inspection.has_wasi {
        forbidden_features.push("wasi".to_string());
    }
    if wasm_validation.inspection.has_threads {
        forbidden_features.push("threads".to_string());
    }
    if wasm_validation.inspection.has_multi_memory {
        forbidden_features.push("multi-memory".to_string());
    }
    if wasm_validation.inspection.has_memory64 {
        forbidden_features.push("memory64".to_string());
    }
    if wasm_validation.inspection.float_operator_count > 0 {
        forbidden_features.push("floating-point".to_string());
    }

    let graph_json = serde_json::to_string_pretty(graph).map_err(CanvasError::Serialization)?;
    let canonical_graph = canonical_graph_json(graph)?;
    let node_pack_lock_json = default_node_pack_lock_json();
    let abi_value = serde_json::to_value(&compilation.abi).map_err(CanvasError::Serialization)?;
    let canonical_abi_value = canonicalize_json_value(&abi_value);

    let graph_hash = canonical_graph_hash(graph)?;
    let node_pack_lock_hash = hash_value_prefixed(&node_pack_lock_json)?;
    let wasm_hash = hash_bytes_prefixed(&compilation.wasm_bytes);
    let wit_hash = hash_default_wit_files()?;
    let json_abi_hash = hash_value_prefixed(&canonical_abi_value)?;

    let safety_report = SafetyReport {
        status: if errors.is_empty() {
            "pass".to_string()
        } else {
            "fail".to_string()
        },
        target_profile: TARGET_PROFILE_BAALS_WASM_V1.to_string(),
        wasm: WasmReport {
            valid: errors.is_empty(),
            size_bytes: compilation.wasm_bytes.len(),
            imports: imports.clone(),
            exports: exports.clone(),
            memory_pages: wasm_validation.inspection.memory_pages,
            forbidden_features,
        },
        graph: GraphReport {
            nodes: graph.nodes.len(),
            connections: graph.connections.len(),
            cycles: 0,
            unreachable_nodes: 0,
            storage_writes: count_graph_nodes(graph, "WriteStorage"),
            auth_guards: count_graph_nodes(graph, "VerifySignature"),
        },
        gas: GasReport {
            estimate: compilation.gas_estimate,
            max_configured: config.compiler.max_gas_limit,
        },
        warnings,
        errors,
    };

    let safety_report_value =
        serde_json::to_value(&safety_report).map_err(CanvasError::Serialization)?;
    let safety_report_hash = hash_value_prefixed(&canonicalize_json_value(&safety_report_value))?;

    let output = ArtifactBuildOutput {
        output_dir: output_dir.to_path_buf(),
        graph_path: output_dir.join("graph.json"),
        canonical_graph_path: output_dir.join("graph.canonical.json"),
        node_pack_lock_path: output_dir.join("node-pack.lock"),
        wasm_path: output_dir.join("contract.wasm"),
        abi_path: output_dir.join("abi.json"),
        wit_dir: output_dir.join("wit"),
        safety_report_path: output_dir.join("safety-report.json"),
        manifest_path: output_dir.join("canvas.contract.json"),
    };

    std::fs::write(&output.graph_path, graph_json).map_err(CanvasError::Io)?;
    std::fs::write(&output.canonical_graph_path, canonical_graph).map_err(CanvasError::Io)?;
    std::fs::write(
        &output.node_pack_lock_path,
        serde_json::to_string_pretty(&node_pack_lock_json).map_err(CanvasError::Serialization)?,
    )
    .map_err(CanvasError::Io)?;
    std::fs::write(&output.wasm_path, &compilation.wasm_bytes).map_err(CanvasError::Io)?;
    std::fs::write(
        &output.abi_path,
        serde_json::to_string_pretty(&canonical_abi_value).map_err(CanvasError::Serialization)?,
    )
    .map_err(CanvasError::Io)?;
    std::fs::write(
        &output.safety_report_path,
        serde_json::to_string_pretty(&safety_report).map_err(CanvasError::Serialization)?,
    )
    .map_err(CanvasError::Io)?;
    write_default_wit_files(&output.wit_dir)?;

    let manifest = ContractManifest {
        schema: MANIFEST_SCHEMA.to_string(),
        name: graph.name.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        target: TARGET_PROFILE_BAALS_WASM_V1.to_string(),
        created_at: Utc::now().to_rfc3339(),
        compiler: default_compiler_section(),
        source: SourceSection {
            graph_hash,
            graph_canonicalization: GRAPH_CANONICALIZATION.to_string(),
            node_pack_lock_hash,
        },
        abi: AbiSection {
            wit_package: DEFAULT_WIT_PACKAGE.to_string(),
            wit_hash,
            json_abi_hash,
        },
        artifact: ArtifactSection {
            wasm_hash,
            wasm_size_bytes: compilation.wasm_bytes.len(),
            exports,
            imports,
        },
        runtime: RuntimeSection {
            profile: TARGET_PROFILE_BAALS_WASM_V1.to_string(),
            max_memory_pages: MAX_MEMORY_PAGES,
            default_fuel: DEFAULT_FUEL,
            deterministic: true,
        },
        validation: ValidationSection {
            safety_report_hash,
            status: safety_report.status.clone(),
            warnings: safety_report.warnings.clone(),
            errors: safety_report.errors.clone(),
        },
        deployment: DeploymentSection {
            network: graph
                .metadata
                .get("target_adapter")
                .cloned()
                .unwrap_or_else(|| "baals-local".to_string()),
            contract_id: None,
            transaction_hash: None,
            block_height: None,
        },
        archive: ArchiveSection {
            chrononode_pointer: None,
            checkpoint_id: None,
            checkpoint_root: None,
        },
        signatures: Vec::new(),
    };

    manifest.write_to_path(&output.manifest_path)?;

    if safety_report.status == "fail" {
        return Err(CanvasError::Validation(
            "Artifact build failed runtime compatibility checks".to_string(),
        ));
    }

    Ok(output)
}

pub fn verify_artifact_manifest(manifest_path: &Path) -> CanvasResult<ArtifactVerifyOutput> {
    let manifest = ContractManifest::read_from_path(manifest_path)?;
    manifest.validate_required_fields()?;

    let base_dir = manifest_path.parent().ok_or_else(|| {
        CanvasError::Validation("Manifest path must have a parent directory".to_string())
    })?;

    let graph_canonical_path = base_dir.join("graph.canonical.json");
    let wasm_path = base_dir.join("contract.wasm");
    let abi_path = base_dir.join("abi.json");
    let safety_report_path = base_dir.join("safety-report.json");
    let node_pack_lock_path = base_dir.join("node-pack.lock");
    let wit_dir = base_dir.join("wit");

    let graph_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&graph_canonical_path).map_err(CanvasError::Io)?,
    )
    .map_err(CanvasError::Serialization)?;
    let graph_hash = hash_value_prefixed(&canonicalize_json_value(&graph_value))?;

    let wasm_bytes = std::fs::read(&wasm_path).map_err(CanvasError::Io)?;
    let wasm_hash = hash_bytes_prefixed(&wasm_bytes);

    let node_pack_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&node_pack_lock_path).map_err(CanvasError::Io)?,
    )
    .map_err(CanvasError::Serialization)?;
    let node_pack_lock_hash = hash_value_prefixed(&canonicalize_json_value(&node_pack_value))?;

    let wit_hash = hash_wit_directory(&wit_dir)?;

    let abi_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&abi_path).map_err(CanvasError::Io)?)
            .map_err(CanvasError::Serialization)?;
    let json_abi_hash = hash_value_prefixed(&canonicalize_json_value(&abi_value))?;

    let safety_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&safety_report_path).map_err(CanvasError::Io)?,
    )
    .map_err(CanvasError::Serialization)?;
    let safety_report_hash = hash_value_prefixed(&canonicalize_json_value(&safety_value))?;

    if graph_hash != manifest.source.graph_hash {
        return Err(CanvasError::Validation(format!(
            "Graph hash mismatch: manifest={}, actual={}",
            manifest.source.graph_hash, graph_hash
        )));
    }

    if wasm_hash != manifest.artifact.wasm_hash {
        return Err(CanvasError::Validation(format!(
            "WASM hash mismatch: manifest={}, actual={}",
            manifest.artifact.wasm_hash, wasm_hash
        )));
    }

    if node_pack_lock_hash != manifest.source.node_pack_lock_hash {
        return Err(CanvasError::Validation(format!(
            "Node-pack lock hash mismatch: manifest={}, actual={}",
            manifest.source.node_pack_lock_hash, node_pack_lock_hash
        )));
    }

    if wit_hash != manifest.abi.wit_hash {
        return Err(CanvasError::Validation(format!(
            "WIT hash mismatch: manifest={}, actual={}",
            manifest.abi.wit_hash, wit_hash
        )));
    }

    if json_abi_hash != manifest.abi.json_abi_hash {
        return Err(CanvasError::Validation(format!(
            "ABI hash mismatch: manifest={}, actual={}",
            manifest.abi.json_abi_hash, json_abi_hash
        )));
    }

    if safety_report_hash != manifest.validation.safety_report_hash {
        return Err(CanvasError::Validation(format!(
            "Safety report hash mismatch: manifest={}, actual={}",
            manifest.validation.safety_report_hash, safety_report_hash
        )));
    }

    let wit_validation = validate_wit_package(&wit_dir)?;
    if !wit_validation.valid {
        return Err(CanvasError::Validation(format!(
            "WIT package validation failed: {}",
            wit_validation.errors.join("; ")
        )));
    }

    let wasm_validation = validate_wasm_against_profile(&wasm_bytes, &baals_wasm_v1_profile())?;
    if wasm_validation.status != "pass" {
        return Err(CanvasError::Validation(format!(
            "WASM runtime profile validation failed: {}",
            wasm_validation.errors.join("; ")
        )));
    }

    Ok(ArtifactVerifyOutput {
        manifest_path: manifest_path.to_path_buf(),
        status: manifest.validation.status,
        graph_hash,
        node_pack_lock_hash,
        wasm_hash,
        wit_hash,
        json_abi_hash,
        safety_report_hash,
    })
}

fn default_node_pack_lock_json() -> serde_json::Value {
    serde_json::json!({
        "schema": DEFAULT_NODE_PACK_LOCK_SCHEMA,
        "packs": []
    })
}

fn write_default_wit_files(wit_dir: &Path) -> CanvasResult<()> {
    std::fs::create_dir_all(wit_dir).map_err(CanvasError::Io)?;
    for (name, bytes) in load_wit_files()? {
        let path = wit_dir.join(name);
        std::fs::write(path, bytes).map_err(CanvasError::Io)?;
    }
    Ok(())
}

fn hash_default_wit_files() -> CanvasResult<String> {
    hash_named_files(&load_wit_files()?)
}

fn hash_wit_directory(wit_dir: &Path) -> CanvasResult<String> {
    let mut files = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(wit_dir)
        .map_err(CanvasError::Io)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(CanvasError::Io)?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if !entry.file_type().map_err(CanvasError::Io)?.is_file() {
            continue;
        }
        let name = entry
            .file_name()
            .to_str()
            .ok_or_else(|| {
                CanvasError::Validation("Invalid UTF-8 file name in wit dir".to_string())
            })?
            .to_string();
        let bytes = std::fs::read(entry.path()).map_err(CanvasError::Io)?;
        files.push((name, bytes));
    }
    hash_named_files(&files)
}

fn hash_named_files(files: &[(String, Vec<u8>)]) -> CanvasResult<String> {
    let mut sorted = files.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = sha2::Sha256::new();
    for (name, bytes) in sorted {
        hasher.update(name.as_bytes());
        hasher.update([0u8]);
        hasher.update((bytes.len() as u64).to_be_bytes());
        hasher.update([0u8]);
        hasher.update(bytes);
    }
    Ok(format!("sha256:{}", hex::encode(hasher.finalize())))
}

fn load_wit_files() -> CanvasResult<Vec<(String, Vec<u8>)>> {
    let repo_wit_dir = Path::new(REPO_WIT_DIR);
    if repo_wit_dir.exists() && repo_wit_dir.is_dir() {
        let mut files = Vec::new();
        let mut entries: Vec<_> = std::fs::read_dir(repo_wit_dir)
            .map_err(CanvasError::Io)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(CanvasError::Io)?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            if !entry.file_type().map_err(CanvasError::Io)?.is_file() {
                continue;
            }
            let name = entry
                .file_name()
                .to_str()
                .ok_or_else(|| {
                    CanvasError::Validation(
                        "Invalid UTF-8 filename in repository wit directory".to_string(),
                    )
                })?
                .to_string();
            files.push((name, std::fs::read(entry.path()).map_err(CanvasError::Io)?));
        }
        if !files.is_empty() {
            return Ok(files);
        }
    }

    let mut fallback = Vec::with_capacity(DEFAULT_WIT_FILES.len());
    for (name, content) in DEFAULT_WIT_FILES {
        fallback.push((name.to_string(), content.as_bytes().to_vec()));
    }
    Ok(fallback)
}

fn count_graph_nodes(graph: &VisualGraph, node_type: &str) -> usize {
    graph
        .nodes
        .iter()
        .filter(|node| node.node_type == node_type)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_encoder::{
        CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module,
        TypeSection, ValType,
    };

    fn load_fixture(name: &str) -> VisualGraph {
        let path = format!("tests/fixtures/{}.json", name);
        let data =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read fixture failed: {}", e));
        serde_json::from_str(&data).unwrap_or_else(|e| panic!("parse fixture failed: {}", e))
    }

    fn float_wasm_module() -> Vec<u8> {
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.function([], [ValType::I64]);
        module.section(&types);

        let mut funcs = FunctionSection::new();
        funcs.function(0);
        module.section(&funcs);

        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 0);
        module.section(&exports);

        let mut codes = CodeSection::new();
        let mut func = Function::new(vec![]);
        func.instruction(&Instruction::F32Const(1.0f32.into()));
        func.instruction(&Instruction::Drop);
        func.instruction(&Instruction::I64Const(7));
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);

        module.finish()
    }

    #[test]
    fn build_and_verify_roundtrip() {
        let graph = load_fixture("simple_arithmetic");
        let config = Config::default();
        let tmp = tempfile::tempdir().unwrap();

        let out = build_artifact_bundle(&graph, &config, tmp.path()).unwrap();
        assert!(out.manifest_path.exists());
        assert!(out.wasm_path.exists());
        assert!(out.canonical_graph_path.exists());
        assert!(out.node_pack_lock_path.exists());
        assert!(out.wit_dir.join("package.wit").exists());

        let verified = verify_artifact_manifest(&out.manifest_path).unwrap();
        assert_eq!(verified.status, "pass");
        assert!(verified.wasm_hash.starts_with("sha256:"));
        assert!(verified.wit_hash.starts_with("sha256:"));
    }

    #[test]
    fn verify_fails_when_wasm_is_tampered() {
        let graph = load_fixture("simple_arithmetic");
        let config = Config::default();
        let tmp = tempfile::tempdir().unwrap();

        let out = build_artifact_bundle(&graph, &config, tmp.path()).unwrap();
        std::fs::write(&out.wasm_path, [0, 1, 2, 3]).unwrap();

        let err = verify_artifact_manifest(&out.manifest_path).unwrap_err();
        assert!(err.to_string().contains("WASM hash mismatch"));
    }

    #[test]
    fn verify_fails_when_wit_is_tampered() {
        let graph = load_fixture("simple_arithmetic");
        let config = Config::default();
        let tmp = tempfile::tempdir().unwrap();

        let out = build_artifact_bundle(&graph, &config, tmp.path()).unwrap();
        std::fs::write(out.wit_dir.join("contract.wit"), "world broken {}\n").unwrap();

        let err = verify_artifact_manifest(&out.manifest_path).unwrap_err();
        assert!(err.to_string().contains("WIT hash mismatch"));
    }

    #[test]
    fn verify_fails_when_wit_is_invalid_even_with_matching_hash() {
        let graph = load_fixture("simple_arithmetic");
        let config = Config::default();
        let tmp = tempfile::tempdir().unwrap();

        let out = build_artifact_bundle(&graph, &config, tmp.path()).unwrap();
        std::fs::write(out.wit_dir.join("contract.wit"), "world broken {}\n").unwrap();

        let mut manifest = ContractManifest::read_from_path(&out.manifest_path).unwrap();
        manifest.abi.wit_hash = hash_wit_directory(&out.wit_dir).unwrap();
        manifest.write_to_path(&out.manifest_path).unwrap();

        let err = verify_artifact_manifest(&out.manifest_path).unwrap_err();
        assert!(err.to_string().contains("WIT package validation failed"));
    }

    #[test]
    fn verify_fails_when_wasm_profile_is_invalid_even_with_matching_hash() {
        let graph = load_fixture("simple_arithmetic");
        let config = Config::default();
        let tmp = tempfile::tempdir().unwrap();

        let out = build_artifact_bundle(&graph, &config, tmp.path()).unwrap();
        let wasm_bytes = float_wasm_module();
        std::fs::write(&out.wasm_path, &wasm_bytes).unwrap();

        let mut manifest = ContractManifest::read_from_path(&out.manifest_path).unwrap();
        manifest.artifact.wasm_hash = hash_bytes_prefixed(&wasm_bytes);
        manifest.write_to_path(&out.manifest_path).unwrap();

        let err = verify_artifact_manifest(&out.manifest_path).unwrap_err();
        assert!(err
            .to_string()
            .contains("WASM runtime profile validation failed"));
    }
}
