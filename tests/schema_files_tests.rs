#[cfg(test)]
mod schema_files_tests {
    use std::fs;

    fn load_json(path: &str) -> serde_json::Value {
        let raw =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {}: {}", path, e));
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("failed to parse {}: {}", path, e))
    }

    #[test]
    fn graph_schema_file_is_present_and_valid_json() {
        let value = load_json("schemas/canvas.graph.v1.json");
        assert_eq!(
            value["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(value["title"], "Canvas Graph v1");
    }

    #[test]
    fn graph_lock_schema_file_is_present_and_valid_json() {
        let value = load_json("schemas/canvas.graph.lock.v1.json");
        assert_eq!(
            value["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(value["title"], "Canvas Graph Lock v1");
    }

    #[test]
    fn validation_schema_file_is_present_and_valid_json() {
        let value = load_json("schemas/canvas.validation.v1.json");
        assert_eq!(
            value["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(value["title"], "Canvas Validation Report v1");
    }
}
