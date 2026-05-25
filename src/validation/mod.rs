use crate::{
    error::{CanvasError, CanvasResult},
    wasm::WasmRuntime,
};

#[derive(Debug, Clone)]
pub struct RuntimeProfile {
    pub name: String,
    pub max_wasm_size_bytes: usize,
    pub max_memory_pages: u32,
    pub allow_float: bool,
    pub allow_wasi: bool,
    pub allow_threads: bool,
    pub allow_multi_memory: bool,
    pub required_exports: Vec<String>,
    pub allowed_imports: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WasmInspection {
    pub size_bytes: usize,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub memory_pages: u32,
    pub has_start: bool,
    pub has_wasi: bool,
    pub has_threads: bool,
    pub has_multi_memory: bool,
    pub has_memory64: bool,
    pub float_operator_count: usize,
    pub custom_sections: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WasmValidationReport {
    pub status: String,
    pub target_profile: String,
    pub inspection: WasmInspection,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

const BAALS_ALLOWED_IMPORTS: &[&str] = &[
    "baals.baals_read_storage",
    "baals.baals_write_storage",
    "baals.baals_get_sender",
    "baals.baals_get_contract_id",
    "baals.baals_get_block_timestamp",
    "baals.baals_get_block_height",
    "baals.baals_emit_event",
    "baals.baals_revert",
    "baals.baals_revert_with_reason",
    "baals.baals_hash_sha256",
    "baals.baals_call_contract",
    "baals.baals_read_call_result",
    "baals.baals_transfer_value",
    "crypto.crypto_verify_signature",
    "crypto.crypto_decode_proof",
    "chrononode.chrononode_fetch_block",
    "chrononode.chrononode_fetch_checkpoint",
    "chrononode.chrononode_verify_proof",
    "chrononode.chrononode_extract_event",
    "chrononode.chrononode_extract_tx_by_sender",
    "chrononode.chrononode_extract_tx_by_recipient",
    "chrononode.chrononode_verify_archive_range",
    "resurgence.resurgence_check_token_age",
    "resurgence.resurgence_check_token_activity_window",
    "resurgence.resurgence_check_liquidity_dormancy",
    "resurgence.resurgence_check_governance_dormancy",
    "resurgence.resurgence_calculate_dormancy_score",
    "resurgence.resurgence_normalize_dead_coin_risk",
    "resurgence.resurgence_generate_dormancy_proof",
    "resurgence.resurgence_emit_dormancy_oracle_result",
];

pub fn baals_wasm_v1_profile() -> RuntimeProfile {
    RuntimeProfile {
        name: "baals-wasm-v1".to_string(),
        max_wasm_size_bytes: 1_048_576,
        max_memory_pages: 16,
        allow_float: false,
        allow_wasi: false,
        allow_threads: false,
        allow_multi_memory: false,
        required_exports: vec!["main".to_string()],
        allowed_imports: BAALS_ALLOWED_IMPORTS
            .iter()
            .map(|s| s.to_string())
            .collect(),
    }
}

pub fn validate_wasm_against_profile(
    wasm_bytes: &[u8],
    profile: &RuntimeProfile,
) -> CanvasResult<WasmValidationReport> {
    let inspect = inspect_wasm(wasm_bytes)?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if inspect.size_bytes > profile.max_wasm_size_bytes {
        errors.push(format!(
            "WASM size {} exceeds maximum {}",
            inspect.size_bytes, profile.max_wasm_size_bytes
        ));
    }

    if inspect.memory_pages > profile.max_memory_pages {
        errors.push(format!(
            "WASM memory pages {} exceeds maximum {}",
            inspect.memory_pages, profile.max_memory_pages
        ));
    }

    if !profile.allow_wasi && inspect.has_wasi {
        errors.push("WASI imports are not allowed in this profile".to_string());
    }

    if !profile.allow_threads && inspect.has_threads {
        errors.push("Thread-related features/imports are not allowed in this profile".to_string());
    }

    if !profile.allow_multi_memory && inspect.has_multi_memory {
        errors.push("Multiple memories are not allowed in this profile".to_string());
    }

    if inspect.has_memory64 {
        errors.push("memory64 is not supported in this profile".to_string());
    }

    if !profile.allow_float && inspect.float_operator_count > 0 {
        errors.push(format!(
            "Found {} floating-point operators but profile disallows float",
            inspect.float_operator_count
        ));
    }

    for required in &profile.required_exports {
        if !inspect.exports.iter().any(|e| e == required) {
            errors.push(format!("Missing required export '{}'", required));
        }
    }

    for import in &inspect.imports {
        if !profile
            .allowed_imports
            .iter()
            .any(|allowed| allowed == import)
        {
            errors.push(format!("Non-whitelisted import detected: {}", import));
        }
    }

    if !inspect.exports.iter().any(|e| e == "call") {
        warnings.push("Export 'call' not found; ABI remains legacy-main oriented".to_string());
    }
    if !inspect.exports.iter().any(|e| e == "query") {
        warnings.push("Export 'query' not found; read-only ABI route unavailable".to_string());
    }

    Ok(WasmValidationReport {
        status: if errors.is_empty() {
            "pass".to_string()
        } else {
            "fail".to_string()
        },
        target_profile: profile.name.clone(),
        inspection: inspect,
        warnings,
        errors,
    })
}

pub fn inspect_wasm(wasm_bytes: &[u8]) -> CanvasResult<WasmInspection> {
    let runtime = WasmRuntime::new(&crate::config::Config::default())?;
    runtime.validate_module(wasm_bytes)?;

    let mut imports = runtime.get_imports(wasm_bytes)?;
    imports.sort();
    imports.dedup();

    let mut exports = runtime.get_exports(wasm_bytes)?;
    exports.sort();
    exports.dedup();

    let mut memory_pages: u32 = 0;
    let mut memory_count: u32 = 0;
    let mut has_memory64 = false;
    let mut has_threads = false;
    let mut custom_sections = Vec::new();
    let mut has_start = false;
    let mut float_operator_count = 0usize;

    let parser = wasmparser::Parser::new(0);
    for payload in parser.parse_all(wasm_bytes) {
        let payload =
            payload.map_err(|e| CanvasError::Wasm(format!("WASM parse failed: {}", e)))?;
        match payload {
            wasmparser::Payload::MemorySection(reader) => {
                memory_count += reader.count();
                for mem in reader {
                    let mem =
                        mem.map_err(|e| CanvasError::Wasm(format!("Memory parse failed: {}", e)))?;
                    memory_pages = memory_pages.max(mem.initial.try_into().unwrap_or(u32::MAX));
                    if mem.memory64 {
                        has_memory64 = true;
                    }
                    if mem.shared {
                        has_threads = true;
                    }
                }
            }
            wasmparser::Payload::CustomSection(section) => {
                custom_sections.push(section.name().to_string());
            }
            wasmparser::Payload::StartSection { .. } => {
                has_start = true;
            }
            wasmparser::Payload::CodeSectionEntry(body) => {
                let mut reader = body.get_operators_reader().map_err(|e| {
                    CanvasError::Wasm(format!("Operator section parse failed: {}", e))
                })?;
                while !reader.eof() {
                    let op = reader
                        .read()
                        .map_err(|e| CanvasError::Wasm(format!("Operator parse failed: {}", e)))?;
                    let repr = format!("{:?}", op);
                    if repr.starts_with("F32") || repr.starts_with("F64") {
                        float_operator_count += 1;
                    }
                }
            }
            _ => {}
        }
    }

    let has_wasi = imports.iter().any(|i| i.starts_with("wasi."));

    Ok(WasmInspection {
        size_bytes: wasm_bytes.len(),
        imports,
        exports,
        memory_pages,
        has_start,
        has_wasi,
        has_threads,
        has_multi_memory: memory_count > 1,
        has_memory64,
        float_operator_count,
        custom_sections,
    })
}

pub fn print_wat(wasm_bytes: &[u8]) -> CanvasResult<String> {
    wasmprinter::print_bytes(wasm_bytes)
        .map_err(|e| CanvasError::Wasm(format!("WAT conversion failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_wasm() -> Vec<u8> {
        use wasm_encoder::{
            CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module,
            TypeSection, ValType,
        };

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
        func.instruction(&Instruction::I64Const(1));
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);

        module.finish()
    }

    #[test]
    fn validate_minimal_module_passes() {
        let wasm = minimal_wasm();
        let report = validate_wasm_against_profile(&wasm, &baals_wasm_v1_profile()).unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn wat_print_works() {
        let wasm = minimal_wasm();
        let wat = print_wat(&wasm).unwrap();
        assert!(wat.contains("(module"));
    }
}
