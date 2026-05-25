use serde::{Deserialize, Serialize};

use crate::{
    error::{CanvasError, CanvasResult},
    VERSION,
};

pub const MANIFEST_SCHEMA: &str = "canvas.contract.manifest.v1";
pub const TARGET_PROFILE_BAALS_WASM_V1: &str = "baals-wasm-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractManifest {
    pub schema: String,
    pub name: String,
    pub version: String,
    pub target: String,
    pub created_at: String,
    pub compiler: CompilerSection,
    pub source: SourceSection,
    pub abi: AbiSection,
    pub artifact: ArtifactSection,
    pub runtime: RuntimeSection,
    pub validation: ValidationSection,
    pub deployment: DeploymentSection,
    pub archive: ArchiveSection,
    pub signatures: Vec<SignatureEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerSection {
    pub name: String,
    pub version: String,
    pub git_commit: String,
    pub wasm_encoder_version: String,
    pub wasmtime_validation_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSection {
    pub graph_hash: String,
    pub graph_canonicalization: String,
    pub node_pack_lock_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiSection {
    pub wit_package: String,
    pub wit_hash: String,
    pub json_abi_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactSection {
    pub wasm_hash: String,
    pub wasm_size_bytes: usize,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSection {
    pub profile: String,
    pub max_memory_pages: u32,
    pub default_fuel: u64,
    pub deterministic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSection {
    pub safety_report_hash: String,
    pub status: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSection {
    pub network: String,
    pub contract_id: Option<String>,
    pub transaction_hash: Option<String>,
    pub block_height: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveSection {
    pub chrononode_pointer: Option<String>,
    pub checkpoint_id: Option<String>,
    pub checkpoint_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureEntry {
    pub algorithm: String,
    pub public_key: String,
    pub signature: String,
}

impl ContractManifest {
    pub fn read_from_path(path: &std::path::Path) -> CanvasResult<Self> {
        let content = std::fs::read_to_string(path).map_err(CanvasError::Io)?;
        serde_json::from_str(&content).map_err(CanvasError::Serialization)
    }

    pub fn write_to_path(&self, path: &std::path::Path) -> CanvasResult<()> {
        let content = serde_json::to_string_pretty(self).map_err(CanvasError::Serialization)?;
        std::fs::write(path, content).map_err(CanvasError::Io)
    }

    pub fn validate_required_fields(&self) -> CanvasResult<()> {
        if self.schema != MANIFEST_SCHEMA {
            return Err(CanvasError::Validation(format!(
                "Unsupported manifest schema '{}', expected '{}'",
                self.schema, MANIFEST_SCHEMA
            )));
        }

        if self.source.graph_hash.is_empty()
            || self.artifact.wasm_hash.is_empty()
            || self.source.node_pack_lock_hash.is_empty()
            || self.abi.wit_hash.is_empty()
            || self.abi.json_abi_hash.is_empty()
            || self.validation.safety_report_hash.is_empty()
        {
            return Err(CanvasError::Validation(
                "Manifest is missing one or more required hashes".to_string(),
            ));
        }

        Ok(())
    }
}

pub fn default_compiler_section() -> CompilerSection {
    CompilerSection {
        name: "canvas-contracts".to_string(),
        version: VERSION.to_string(),
        git_commit: option_env!("GIT_COMMIT").unwrap_or("unknown").to_string(),
        wasm_encoder_version: "0.38".to_string(),
        wasmtime_validation_version: "43.0.1".to_string(),
    }
}
