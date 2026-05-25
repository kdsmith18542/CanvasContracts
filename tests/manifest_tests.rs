#[cfg(test)]
mod manifest_tests {
    use canvas_contracts::{
        artifact::hash::{canonical_graph_hash, hash_bytes_prefixed, GRAPH_CANONICALIZATION},
        compiler::Compiler,
        config::Config,
        types::VisualGraph,
    };
    use std::fs;
    use tempfile::tempdir;

    fn load_fixture(name: &str) -> VisualGraph {
        let path = format!("tests/fixtures/{}.json", name);
        let data = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path, e));
        serde_json::from_str(&data)
            .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", path, e))
    }

    #[test]
    fn test_lockfile_and_validation_report_generation() {
        let temp_dir = tempdir().unwrap();
        let wasm_output_path = temp_dir.path().join("test_contract.wasm");
        let abi_output_path = temp_dir.path().join("test_contract.abi.json");
        let lock_output_path = temp_dir.path().join("test_contract.graph.lock.json");
        let report_output_path = temp_dir.path().join("test_contract.validation-report.json");

        // Load the simple arithmetic graph
        let graph = load_fixture("simple_arithmetic");

        // Create compiler
        let config = Config::default();
        let compiler = Compiler::new(&config).unwrap();

        // 1. Run compilation manually via the library Compiler struct
        let result = compiler.compile(&graph).unwrap();
        assert!(!result.wasm_bytes.is_empty());
        assert!(!result.abi.functions.is_empty());

        // 2. Validate using the CLI-like steps (similar to compile_contract in main.rs)
        fs::write(&wasm_output_path, &result.wasm_bytes).unwrap();

        let abi_content = serde_json::to_string_pretty(&result.abi).unwrap();
        fs::write(&abi_output_path, abi_content).unwrap();

        let graph_hash = canonical_graph_hash(&graph).unwrap();
        let wasm_hash = hash_bytes_prefixed(&result.wasm_bytes);
        let target_adapter = graph
            .metadata
            .get("target_adapter")
            .cloned()
            .unwrap_or_else(|| "baals".to_string());
        let mut node_types: Vec<String> = graph.nodes.iter().map(|n| n.node_type.clone()).collect();
        node_types.sort();
        node_types.dedup();
        let lock_json = serde_json::json!({
            "schema_version": "canvas.graph.lock.v1",
            "project_name": graph.name,
            "target_adapter": target_adapter,
            "graph_canonicalization": GRAPH_CANONICALIZATION,
            "node_count": graph.nodes.len(),
            "connection_count": graph.connections.len(),
            "node_types": node_types,
            "gas_estimate": result.gas_estimate,
            "compiler": {
                "name": env!("CARGO_PKG_NAME"),
                "version": env!("CARGO_PKG_VERSION"),
                "wasm_target": "wasm32-unknown-unknown",
                "wasm_encoder_version": "0.38",
                "wasmtime_validation_version": "43.0.1"
            },
            "graph_hash": graph_hash,
            "wasm_hash": wasm_hash
        });
        fs::write(
            &lock_output_path,
            serde_json::to_string_pretty(&lock_json).unwrap(),
        )
        .unwrap();

        let validator = compiler.validator().unwrap();
        let val_res = validator.validate(&graph).unwrap();
        let val_json = serde_json::json!({
            "schema_version": "canvas.validation.v1",
            "graph_hash": canonical_graph_hash(&graph).unwrap(),
            "graph_canonicalization": GRAPH_CANONICALIZATION,
            "target_adapter": graph.metadata.get("target_adapter").cloned().unwrap_or_else(|| "baals".to_string()),
            "node_count": graph.nodes.len(),
            "connection_count": graph.connections.len(),
            "is_valid": val_res.is_valid,
            "errors": val_res.errors,
            "warnings": val_res.warnings
        });
        fs::write(
            &report_output_path,
            serde_json::to_string_pretty(&val_json).unwrap(),
        )
        .unwrap();

        // Check if all compiler outputs exist and are valid
        assert!(wasm_output_path.exists());
        assert!(abi_output_path.exists());
        assert!(lock_output_path.exists());
        assert!(report_output_path.exists());

        // Read lockfile back and verify metadata
        let lock_content = fs::read_to_string(&lock_output_path).unwrap();
        let parsed_lock: serde_json::Value = serde_json::from_str(&lock_content).unwrap();
        assert_eq!(parsed_lock["schema_version"], "canvas.graph.lock.v1");
        assert_eq!(parsed_lock["project_name"], "Simple Arithmetic");
        assert_eq!(
            parsed_lock["graph_canonicalization"],
            GRAPH_CANONICALIZATION
        );

        // Read validation report back and verify status
        let report_content = fs::read_to_string(&report_output_path).unwrap();
        let parsed_report: serde_json::Value = serde_json::from_str(&report_content).unwrap();
        assert_eq!(parsed_report["schema_version"], "canvas.validation.v1");
        assert!(parsed_report["is_valid"].is_boolean());
        assert!(parsed_report["errors"].is_array());
        assert!(parsed_report["warnings"].is_array());
    }
}
