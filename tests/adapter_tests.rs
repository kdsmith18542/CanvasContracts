#[cfg(test)]
mod adapter_tests {
    use canvas_contracts::{
        adapter::{
            create_chrononode_client, BaaLSAdapter, ChronoNodeClient, LedgerAdapter,
            MockChronoNodeClient,
        },
        config::Config,
    };

    #[test]
    fn test_baals_adapter_creation_and_routing() {
        let mut config = Config::default();
        config.baals.node_url = "mock".to_string();

        let adapter = BaaLSAdapter::new(&config).unwrap();
        let runtime_info = adapter.validate_runtime().unwrap();
        assert_eq!(runtime_info.name, "BaaLS");
        // In local/mock context, since Node URL is "mock", it should be "healthy" or fallback
        assert!(runtime_info.status == "healthy" || runtime_info.status == "unreachable");

        let simulate_res = adapter
            .simulate_contract(&[], serde_json::Value::Null)
            .unwrap();
        assert!(simulate_res["success"].as_bool().unwrap());
    }

    #[test]
    fn test_chrononode_mock_client() {
        let client = MockChronoNodeClient;

        let block = client.get_block("test-chain", 10).unwrap();
        assert_eq!(block["chain_id"], "test-chain");
        assert_eq!(block["height"], 10);

        let range = client.get_block_range("test-chain", 1, 5).unwrap();
        assert!(range.is_array());
        assert_eq!(range[0]["height"], 1);

        let proof = client.get_proof("test-chain", 10).unwrap();
        assert_eq!(proof["height"], 10);
        assert_eq!(proof["root"], "0xroot");

        let is_valid = client.verify_proof(serde_json::json!({})).unwrap();
        assert!(is_valid);

        let txs_sender = client.get_tx_by_sender("test-chain", "0xsender").unwrap();
        assert!(txs_sender.is_array());
        assert_eq!(txs_sender[0]["sender"], "0xsender");

        let txs_recipient = client
            .get_tx_by_recipient("test-chain", "0xrecipient")
            .unwrap();
        assert!(txs_recipient.is_array());
        assert_eq!(txs_recipient[0]["recipient"], "0xrecipient");

        let events = client.get_events("test-chain", "Transfer").unwrap();
        assert!(events.is_array());
        assert_eq!(events[0]["type"], "Transfer");
    }

    #[test]
    fn test_create_chrononode_client_routing() {
        let mut config = Config::default();
        config.baals.node_url = "mock".to_string();

        let client = create_chrononode_client(&config).unwrap();
        let block = client.get_block("local-chain", 42).unwrap();
        assert_eq!(block["height"], 42);

        // When node URL is not mock/empty, it should create HttpChronoNodeClient
        config.baals.node_url = "http://127.0.0.1:8080".to_string();
        let client = create_chrononode_client(&config).unwrap();
        // Since we aren't running a live server in test, we don't execute query but check routing
        assert!(client.get_block("local-chain", 42).is_err());
    }
}
