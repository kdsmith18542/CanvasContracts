use std::path::{Path, PathBuf};

use sha2::Digest;

use crate::error::{CanvasError, CanvasResult};

pub const WIT_PACKAGE_NAME: &str = "baals:contract@1.0.0";
pub const DEFAULT_WIT_SOURCE_DIR: &str = "wit/baals-contract-v1";

pub const REQUIRED_WIT_FILES: &[&str] = &[
    "package.wit",
    "types.wit",
    "storage.wit",
    "crypto.wit",
    "proof.wit",
    "contract.wit",
];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WitValidationReport {
    pub valid: bool,
    pub package: String,
    pub missing_files: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub fn generate_wit_package(out_dir: &Path) -> CanvasResult<Vec<PathBuf>> {
    std::fs::create_dir_all(out_dir).map_err(CanvasError::Io)?;

    let source_dir = Path::new(DEFAULT_WIT_SOURCE_DIR);
    if !source_dir.exists() {
        return Err(CanvasError::Validation(format!(
            "WIT source directory '{}' not found",
            DEFAULT_WIT_SOURCE_DIR
        )));
    }

    let mut written = Vec::new();
    for file in REQUIRED_WIT_FILES {
        let src = source_dir.join(file);
        if !src.exists() {
            return Err(CanvasError::Validation(format!(
                "Required source WIT file missing: {}",
                src.display()
            )));
        }
        let dest = out_dir.join(file);
        std::fs::copy(&src, &dest).map_err(CanvasError::Io)?;
        written.push(dest);
    }

    Ok(written)
}

pub fn validate_wit_package(dir: &Path) -> CanvasResult<WitValidationReport> {
    let mut missing_files = Vec::new();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for file in REQUIRED_WIT_FILES {
        let path = dir.join(file);
        if !path.exists() {
            missing_files.push(file.to_string());
        }
    }

    if !missing_files.is_empty() {
        errors.push(format!(
            "Missing {} required WIT file(s)",
            missing_files.len()
        ));
        return Ok(WitValidationReport {
            valid: false,
            package: WIT_PACKAGE_NAME.to_string(),
            missing_files,
            warnings,
            errors,
        });
    }

    let package_wit = std::fs::read_to_string(dir.join("package.wit")).map_err(CanvasError::Io)?;
    if !package_wit.contains("package baals:contract@1.0.0") {
        errors.push("package.wit does not declare baals:contract@1.0.0".to_string());
    }

    let contract_wit =
        std::fs::read_to_string(dir.join("contract.wit")).map_err(CanvasError::Io)?;
    if !contract_wit.contains("world baals-contract") {
        errors.push("contract.wit is missing world baals-contract".to_string());
    }
    if !contract_wit.contains("export call") {
        warnings.push("contract.wit does not export call".to_string());
    }
    if !contract_wit.contains("export query") {
        warnings.push("contract.wit does not export query".to_string());
    }

    Ok(WitValidationReport {
        valid: errors.is_empty(),
        package: WIT_PACKAGE_NAME.to_string(),
        missing_files,
        warnings,
        errors,
    })
}

pub fn hash_wit_package(dir: &Path) -> CanvasResult<String> {
    let mut files = Vec::new();
    for file in REQUIRED_WIT_FILES {
        let path = dir.join(file);
        let bytes = std::fs::read(&path).map_err(CanvasError::Io)?;
        files.push((file.to_string(), bytes));
    }

    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = sha2::Sha256::new();
    for (name, bytes) in files {
        hasher.update(name.as_bytes());
        hasher.update([0u8]);
        hasher.update((bytes.len() as u64).to_be_bytes());
        hasher.update([0u8]);
        hasher.update(bytes);
    }

    Ok(format!("sha256:{}", hex::encode(hasher.finalize())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_repo_wit_package() {
        let report = validate_wit_package(Path::new(DEFAULT_WIT_SOURCE_DIR)).unwrap();
        assert!(report.valid);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn generate_and_hash_wit_package() {
        let temp_dir = tempfile::tempdir().unwrap();
        generate_wit_package(temp_dir.path()).unwrap();
        let hash = hash_wit_package(temp_dir.path()).unwrap();
        assert!(hash.starts_with("sha256:"));
    }
}
