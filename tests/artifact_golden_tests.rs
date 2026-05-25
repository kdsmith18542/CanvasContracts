use canvas_contracts::{
    build_artifact_bundle, config::Config, inspect_artifact_manifest, types::VisualGraph,
    verify_artifact_manifest,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ExpectedGolden {
    status: String,
    graph_hash: String,
    node_pack_lock_hash: String,
    wasm_hash: String,
    wit_hash: String,
    json_abi_hash: String,
    safety_report_hash: String,
    wasm_size_bytes: usize,
    imports: Vec<String>,
    exports: Vec<String>,
}

fn load_fixture(name: &str) -> VisualGraph {
    let path = format!("tests/fixtures/{}.json", name);
    let data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path, e));
    serde_json::from_str(&data)
        .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", path, e))
}

#[test]
fn dormancy_oracle_golden_artifact_hashes_match() {
    let expected: ExpectedGolden =
        serde_json::from_str(include_str!("golden/dormancy_oracle/expected_hashes.json")).unwrap();

    let graph = load_fixture("dormancy_oracle");
    let config = Config::default();
    let tmp = tempfile::tempdir().unwrap();

    let output = build_artifact_bundle(&graph, &config, tmp.path()).unwrap();
    let verify = verify_artifact_manifest(&output.manifest_path).unwrap();
    let manifest = inspect_artifact_manifest(&output.manifest_path).unwrap();

    assert_eq!(verify.status, expected.status);
    assert_eq!(verify.graph_hash, expected.graph_hash);
    assert_eq!(verify.node_pack_lock_hash, expected.node_pack_lock_hash);
    assert_eq!(verify.wasm_hash, expected.wasm_hash);
    assert_eq!(verify.wit_hash, expected.wit_hash);
    assert_eq!(verify.json_abi_hash, expected.json_abi_hash);
    assert_eq!(verify.safety_report_hash, expected.safety_report_hash);

    assert_eq!(manifest.artifact.wasm_size_bytes, expected.wasm_size_bytes);
    assert_eq!(manifest.artifact.imports, expected.imports);
    assert_eq!(manifest.artifact.exports, expected.exports);
}
