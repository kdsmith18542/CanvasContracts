use std::path::Path;

use crate::error::{CanvasError, CanvasResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveSubmitResult {
    pub storage_pointer: String,
    pub content_hash: String,
    pub checkpoint_id: Option<String>,
}

pub fn submit_artifact_bundle(
    bundle_path: &Path,
    chrononode_url: &str,
) -> CanvasResult<ArchiveSubmitResult> {
    let bytes = std::fs::read(bundle_path).map_err(CanvasError::Io)?;
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/v1/artifacts", chrononode_url.trim_end_matches('/'));

    let response = client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
        .body(bytes)
        .send()
        .map_err(|e| CanvasError::Network(format!("ChronoNode archive submit failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(CanvasError::Network(format!(
            "ChronoNode archive submit returned status {}",
            response.status()
        )));
    }

    let body: serde_json::Value = response
        .json()
        .map_err(|e| CanvasError::Network(format!("ChronoNode response parse failed: {}", e)))?;

    let storage_pointer = body
        .get("storage_pointer")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            CanvasError::Validation("ChronoNode response missing storage_pointer".to_string())
        })?
        .to_string();

    let content_hash = body
        .get("content_hash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            CanvasError::Validation("ChronoNode response missing content_hash".to_string())
        })?
        .to_string();

    validate_content_hash_format(&content_hash)?;

    Ok(ArchiveSubmitResult {
        storage_pointer,
        content_hash,
        checkpoint_id: body
            .get("checkpoint_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    })
}

pub fn validate_content_hash_format(content_hash: &str) -> CanvasResult<()> {
    if !content_hash.starts_with("sha256:") {
        return Err(CanvasError::Validation(format!(
            "Invalid content hash format '{}': expected sha256:<hex>",
            content_hash
        )));
    }
    let hex_part = &content_hash[7..];
    if hex_part.len() != 64 || !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(CanvasError::Validation(format!(
            "Invalid content hash hex '{}': expected 64 hex chars",
            hex_part
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_hash_format_passes() {
        validate_content_hash_format(
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap();
    }

    #[test]
    fn invalid_hash_format_fails() {
        assert!(validate_content_hash_format("abc").is_err());
        assert!(validate_content_hash_format("sha256:xyz").is_err());
    }
}
