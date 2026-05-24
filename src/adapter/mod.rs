//! Traits and adapters for decoupling the Canvas Contracts engine from specific ledger and history implementations.

use crate::baals::{create_client, BaalsClient, DeploymentResult, TransactionResult};
use crate::config::Config;
use crate::error::{CanvasError, CanvasResult};

/// Information about the connected ledger runtime
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RuntimeInfo {
    pub name: String,
    pub version: String,
    pub status: String,
}

/// Generic interface to interact with a smart contract ledger target
pub trait LedgerAdapter: Send + Sync {
    /// Verify health and get information about the connected runtime
    fn validate_runtime(&self) -> CanvasResult<RuntimeInfo>;
    /// Simulate running a contract without recording it on the ledger
    fn simulate_contract(
        &self,
        wasm: &[u8],
        input: serde_json::Value,
    ) -> CanvasResult<serde_json::Value>;
    /// Deploy a contract WASM to the ledger
    fn deploy_contract(
        &self,
        wasm: &[u8],
        args: serde_json::Value,
        signer_key: &str,
    ) -> CanvasResult<DeploymentResult>;
    /// Invoke a method on a deployed contract
    fn call_contract(
        &self,
        contract_address: &str,
        function_name: &str,
        arguments: Vec<serde_json::Value>,
        signer_key: &str,
    ) -> CanvasResult<TransactionResult>;
    /// Query a read-only view of a deployed contract state
    fn query_contract(
        &self,
        contract_address: &str,
        function_name: &str,
        arguments: Vec<serde_json::Value>,
    ) -> CanvasResult<serde_json::Value>;
    /// Subscribe to real-time execution events
    fn subscribe_events(&self, channels: Vec<String>) -> CanvasResult<()>;
}

/// BaaLS blockchain adapter
pub struct BaaLSAdapter {
    client: Box<dyn BaalsClient>,
}

impl BaaLSAdapter {
    /// Create a new BaaLS blockchain adapter
    pub fn new(config: &Config) -> CanvasResult<Self> {
        let client = create_client(config)?;
        Ok(Self { client })
    }
}

impl LedgerAdapter for BaaLSAdapter {
    fn validate_runtime(&self) -> CanvasResult<RuntimeInfo> {
        match self.client.get_block_info(0) {
            Ok(_) => Ok(RuntimeInfo {
                name: "BaaLS".to_string(),
                version: "0.1.0".to_string(),
                status: "healthy".to_string(),
            }),
            Err(_) => Ok(RuntimeInfo {
                name: "BaaLS".to_string(),
                version: "0.1.0".to_string(),
                status: "unreachable".to_string(),
            }),
        }
    }

    fn simulate_contract(
        &self,
        _wasm: &[u8],
        _input: serde_json::Value,
    ) -> CanvasResult<serde_json::Value> {
        Ok(serde_json::json!({
            "success": true,
            "gas_used": 1000,
            "output": {}
        }))
    }

    fn deploy_contract(
        &self,
        wasm: &[u8],
        args: serde_json::Value,
        signer_key: &str,
    ) -> CanvasResult<DeploymentResult> {
        self.client.deploy_contract(wasm, args, signer_key)
    }

    fn call_contract(
        &self,
        contract_address: &str,
        function_name: &str,
        arguments: Vec<serde_json::Value>,
        signer_key: &str,
    ) -> CanvasResult<TransactionResult> {
        self.client
            .call_contract(contract_address, function_name, arguments, signer_key)
    }

    fn query_contract(
        &self,
        contract_address: &str,
        _function_name: &str,
        _arguments: Vec<serde_json::Value>,
    ) -> CanvasResult<serde_json::Value> {
        let state = self.client.get_contract_state(contract_address)?;
        Ok(serde_json::to_value(state.storage).unwrap_or(serde_json::json!({})))
    }

    fn subscribe_events(&self, _channels: Vec<String>) -> CanvasResult<()> {
        Ok(())
    }
}

/// Trait defining verify historical queries and Merkle proof exploration
pub trait ChronoNodeClient: Send + Sync {
    fn get_block(&self, chain_id: &str, height: u64) -> CanvasResult<serde_json::Value>;
    fn get_block_range(
        &self,
        chain_id: &str,
        from: u64,
        to: u64,
    ) -> CanvasResult<serde_json::Value>;
    fn get_proof(&self, chain_id: &str, height: u64) -> CanvasResult<serde_json::Value>;
    fn verify_proof(&self, proof: serde_json::Value) -> CanvasResult<bool>;
    fn get_tx_by_sender(&self, chain_id: &str, sender: &str) -> CanvasResult<serde_json::Value>;
    fn get_tx_by_recipient(
        &self,
        chain_id: &str,
        recipient: &str,
    ) -> CanvasResult<serde_json::Value>;
    fn get_events(&self, chain_id: &str, event_type: &str) -> CanvasResult<serde_json::Value>;
}

/// HTTP implementation of the ChronoNode indexer client
pub struct HttpChronoNodeClient {
    client: reqwest::blocking::Client,
    base_url: String,
}

impl HttpChronoNodeClient {
    /// Create a new HTTP ChronoNode client
    pub fn new(base_url: String) -> Self {
        let client = reqwest::blocking::Client::new();
        Self { client, base_url }
    }
}

impl ChronoNodeClient for HttpChronoNodeClient {
    fn get_block(&self, chain_id: &str, height: u64) -> CanvasResult<serde_json::Value> {
        let url = format!("{}/v1/chains/{}/blocks/{}", self.base_url, chain_id, height);
        let resp = self
            .client
            .get(&url)
            .send()
            .map_err(|e| CanvasError::Internal(format!("ChronoNode query failed: {}", e)))?;
        resp.json().map_err(|e| {
            CanvasError::Internal(format!("Failed to parse ChronoNode response: {}", e))
        })
    }

    fn get_block_range(
        &self,
        chain_id: &str,
        from: u64,
        to: u64,
    ) -> CanvasResult<serde_json::Value> {
        let url = format!(
            "{}/v1/chains/{}/blocks?from={}&to={}",
            self.base_url, chain_id, from, to
        );
        let resp =
            self.client.get(&url).send().map_err(|e| {
                CanvasError::Internal(format!("ChronoNode range query failed: {}", e))
            })?;
        resp.json().map_err(|e| {
            CanvasError::Internal(format!("Failed to parse ChronoNode block range: {}", e))
        })
    }

    fn get_proof(&self, chain_id: &str, height: u64) -> CanvasResult<serde_json::Value> {
        let url = format!(
            "{}/v1/chains/{}/proofs/block/{}",
            self.base_url, chain_id, height
        );
        let resp =
            self.client.get(&url).send().map_err(|e| {
                CanvasError::Internal(format!("ChronoNode proof query failed: {}", e))
            })?;
        resp.json()
            .map_err(|e| CanvasError::Internal(format!("Failed to parse ChronoNode proof: {}", e)))
    }

    fn verify_proof(&self, proof: serde_json::Value) -> CanvasResult<bool> {
        let url = format!("{}/v1/proofs/verify", self.base_url);
        let resp = self.client.post(&url).json(&proof).send().map_err(|e| {
            CanvasError::Internal(format!("ChronoNode verify request failed: {}", e))
        })?;
        let result: serde_json::Value = resp.json().map_err(|e| {
            CanvasError::Internal(format!("Failed to parse verify response: {}", e))
        })?;
        Ok(result
            .get("valid")
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }

    fn get_tx_by_sender(&self, chain_id: &str, sender: &str) -> CanvasResult<serde_json::Value> {
        let url = format!(
            "{}/v1/chains/{}/txs/sender/{}",
            self.base_url, chain_id, sender
        );
        let resp = self.client.get(&url).send().map_err(|e| {
            CanvasError::Internal(format!("ChronoNode tx by sender query failed: {}", e))
        })?;
        resp.json().map_err(|e| {
            CanvasError::Internal(format!("Failed to parse ChronoNode tx response: {}", e))
        })
    }

    fn get_tx_by_recipient(
        &self,
        chain_id: &str,
        recipient: &str,
    ) -> CanvasResult<serde_json::Value> {
        let url = format!(
            "{}/v1/chains/{}/txs/recipient/{}",
            self.base_url, chain_id, recipient
        );
        let resp = self.client.get(&url).send().map_err(|e| {
            CanvasError::Internal(format!("ChronoNode tx by recipient query failed: {}", e))
        })?;
        resp.json().map_err(|e| {
            CanvasError::Internal(format!("Failed to parse ChronoNode tx response: {}", e))
        })
    }

    fn get_events(&self, chain_id: &str, event_type: &str) -> CanvasResult<serde_json::Value> {
        let url = format!(
            "{}/v1/chains/{}/events/{}",
            self.base_url, chain_id, event_type
        );
        let resp =
            self.client.get(&url).send().map_err(|e| {
                CanvasError::Internal(format!("ChronoNode events query failed: {}", e))
            })?;
        resp.json().map_err(|e| {
            CanvasError::Internal(format!("Failed to parse ChronoNode events response: {}", e))
        })
    }
}

/// Mock implementation of the ChronoNode client for test and local use cases
pub struct MockChronoNodeClient;

impl ChronoNodeClient for MockChronoNodeClient {
    fn get_block(&self, chain_id: &str, height: u64) -> CanvasResult<serde_json::Value> {
        Ok(serde_json::json!({
            "chain_id": chain_id,
            "height": height,
            "hash": format!("0x{:064x}", height),
            "timestamp": 1716500000,
            "transactions": ["0xabc123"]
        }))
    }

    fn get_block_range(
        &self,
        chain_id: &str,
        from: u64,
        _to: u64,
    ) -> CanvasResult<serde_json::Value> {
        Ok(serde_json::json!([
            {
                "chain_id": chain_id,
                "height": from,
                "hash": format!("0x{:064x}", from),
                "timestamp": 1716500000
            }
        ]))
    }

    fn get_proof(&self, _chain_id: &str, height: u64) -> CanvasResult<serde_json::Value> {
        Ok(serde_json::json!({
            "height": height,
            "proof_data": "mock_proof_data",
            "root": "0xroot"
        }))
    }

    fn verify_proof(&self, _proof: serde_json::Value) -> CanvasResult<bool> {
        Ok(true)
    }

    fn get_tx_by_sender(&self, chain_id: &str, sender: &str) -> CanvasResult<serde_json::Value> {
        Ok(serde_json::json!([
            {
                "chain_id": chain_id,
                "hash": "0xabc123",
                "sender": sender,
                "recipient": "0xdef456",
                "value": 1000
            }
        ]))
    }

    fn get_tx_by_recipient(
        &self,
        chain_id: &str,
        recipient: &str,
    ) -> CanvasResult<serde_json::Value> {
        Ok(serde_json::json!([
            {
                "chain_id": chain_id,
                "hash": "0xabc123",
                "sender": "0xdef456",
                "recipient": recipient,
                "value": 1000
            }
        ]))
    }

    fn get_events(&self, chain_id: &str, event_type: &str) -> CanvasResult<serde_json::Value> {
        Ok(serde_json::json!([
            {
                "chain_id": chain_id,
                "type": event_type,
                "data": { "param1": "val1" }
            }
        ]))
    }
}

/// Create a ChronoNode client based on Configuration settings
pub fn create_chrononode_client(config: &Config) -> CanvasResult<Box<dyn ChronoNodeClient>> {
    if config.baals.node_url == "mock" || config.baals.node_url.is_empty() {
        Ok(Box::new(MockChronoNodeClient))
    } else {
        Ok(Box::new(HttpChronoNodeClient::new(
            config.baals.node_url.clone(),
        )))
    }
}
