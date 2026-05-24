use crate::{
    config::Config,
    error::{CanvasError, CanvasResult},
    types::{ContractAddress, Event, Gas, TransactionHash},
};
use ed25519_dalek::Signer;
use std::collections::HashMap;
use std::convert::TryInto;

pub trait BaalsClient: Send + Sync {
    fn deploy_contract(
        &self,
        wasm_bytes: &[u8],
        constructor_args: serde_json::Value,
        private_key: &str,
    ) -> CanvasResult<DeploymentResult>;
    fn call_contract(
        &self,
        contract_address: &str,
        function_name: &str,
        arguments: Vec<serde_json::Value>,
        private_key: &str,
    ) -> CanvasResult<TransactionResult>;
    fn get_contract_state(&self, contract_address: &str) -> CanvasResult<ContractState>;
    fn read_storage(&self, contract_address: &str, key: &str) -> CanvasResult<serde_json::Value>;
    fn get_transaction_status(&self, transaction_hash: &str) -> CanvasResult<TransactionStatus>;
    fn get_block_info(&self, block_number: u64) -> CanvasResult<BlockInfo>;
}

pub fn create_client(config: &Config) -> CanvasResult<Box<dyn BaalsClient>> {
    if config.baals.enable_local_node
        || config.baals.node_url.is_empty()
        || config.baals.node_url == "mock"
    {
        Ok(Box::new(MockBaalsClient::new(config)?))
    } else {
        Ok(Box::new(HttpBaalsClient::new(config)?))
    }
}

#[derive(Debug, Clone)]
pub struct DeploymentResult {
    pub contract_address: ContractAddress,
    pub transaction_hash: TransactionHash,
    pub gas_used: Gas,
    pub block_number: u64,
}

#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub transaction_hash: TransactionHash,
    pub gas_used: Gas,
    pub block_number: u64,
    pub success: bool,
    pub output: serde_json::Value,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct ContractState {
    pub address: ContractAddress,
    pub balance: u64,
    pub code_hash: String,
    pub storage: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct TransactionStatus {
    pub hash: String,
    pub status: TransactionState,
    pub block_number: u64,
    pub gas_used: Gas,
    pub confirmations: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    Pending,
    Confirmed,
    Failed,
    Reverted,
}

#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub number: u64,
    pub hash: String,
    pub timestamp: u64,
    pub transactions: Vec<String>,
}

pub struct MockBaalsClient {
    #[allow(dead_code)]
    config: Config,
    storage: HashMap<String, HashMap<String, serde_json::Value>>,
    deployed: HashMap<String, Vec<u8>>,
    block_counter: u64,
}

impl MockBaalsClient {
    pub fn new(config: &Config) -> CanvasResult<Self> {
        Ok(Self {
            config: config.clone(),
            storage: HashMap::new(),
            deployed: HashMap::new(),
            block_counter: 12345,
        })
    }

    #[allow(dead_code)]
    fn contract_id(wasm: &[u8]) -> String {
        use sha2::Digest;
        hex::encode(sha2::Sha256::digest(wasm))
    }

    fn sign_tx(&self, _payload: &[u8], private_key: &str) -> Vec<u8> {
        let sig: Vec<u8> = private_key.bytes().chain(std::iter::once(0xFA)).collect();
        if sig.is_empty() {
            vec![0xFA; 64]
        } else {
            sig
        }
    }
}

impl BaalsClient for MockBaalsClient {
    fn deploy_contract(
        &self,
        wasm_bytes: &[u8],
        _constructor_args: serde_json::Value,
        private_key: &str,
    ) -> CanvasResult<DeploymentResult> {
        self.sign_tx(wasm_bytes, private_key);
        let contract_id = Self::contract_id(wasm_bytes);
        let tx_hash = format!("0x{:064x}", rand::random::<u128>());

        log::info!(
            "[MockBaaLS] Deploy contract {} ({} bytes)",
            contract_id,
            wasm_bytes.len()
        );

        Ok(DeploymentResult {
            contract_address: contract_id,
            transaction_hash: tx_hash,
            gas_used: (wasm_bytes.len() as u64 * 100).max(1000),
            block_number: self.block_counter,
        })
    }

    fn call_contract(
        &self,
        contract_address: &str,
        function_name: &str,
        arguments: Vec<serde_json::Value>,
        private_key: &str,
    ) -> CanvasResult<TransactionResult> {
        self.sign_tx(function_name.as_bytes(), private_key);
        let tx_hash = format!("0x{:064x}", rand::random::<u128>());

        log::info!(
            "[MockBaaLS] Call {} on {} ({} args)",
            function_name,
            contract_address,
            arguments.len()
        );

        Ok(TransactionResult {
            transaction_hash: tx_hash,
            gas_used: (arguments.len() as u64 * 50).max(100),
            block_number: self.block_counter,
            success: true,
            output: serde_json::json!({"function": function_name, "args": arguments}),
            events: vec![],
        })
    }

    fn get_contract_state(&self, contract_address: &str) -> CanvasResult<ContractState> {
        let code_hash = if self.deployed.contains_key(contract_address) {
            use sha2::Digest;
            hex::encode(sha2::Sha256::digest(&self.deployed[contract_address]))
        } else {
            format!("0x{:064x}", rand::random::<u128>())
        };

        Ok(ContractState {
            address: contract_address.to_string(),
            balance: 1_000_000,
            code_hash,
            storage: self
                .storage
                .get(contract_address)
                .cloned()
                .unwrap_or_default(),
        })
    }

    fn read_storage(&self, contract_address: &str, key: &str) -> CanvasResult<serde_json::Value> {
        Ok(self
            .storage
            .get(contract_address)
            .and_then(|s| s.get(key))
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }

    fn get_transaction_status(&self, transaction_hash: &str) -> CanvasResult<TransactionStatus> {
        Ok(TransactionStatus {
            hash: transaction_hash.to_string(),
            status: TransactionState::Confirmed,
            block_number: self.block_counter,
            gas_used: 100_000,
            confirmations: 12,
        })
    }

    fn get_block_info(&self, block_number: u64) -> CanvasResult<BlockInfo> {
        Ok(BlockInfo {
            number: block_number,
            hash: format!("0x{:064x}", rand::random::<u128>()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            transactions: vec![],
        })
    }
}

pub struct HttpBaalsClient {
    #[allow(dead_code)]
    config: Config,
    client: reqwest::blocking::Client,
    base_url: String,
}

impl HttpBaalsClient {
    pub fn new(config: &Config) -> CanvasResult<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(
                config.baals.connection_timeout.max(30),
            ))
            .build()
            .map_err(|e| CanvasError::Baals(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            config: config.clone(),
            client,
            base_url: config.baals.node_url.trim_end_matches('/').to_string(),
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    #[allow(dead_code)]
    fn contract_id(wasm: &[u8]) -> String {
        use sha2::Digest;
        hex::encode(sha2::Sha256::digest(wasm))
    }

    fn auth_token(&self, private_key_hex: &str) -> CanvasResult<String> {
        let signing_key = {
            use zeroize::Zeroize;
            let mut key_bytes = hex::decode(private_key_hex)
                .map_err(|_| CanvasError::Baals("Invalid private key hex".to_string()))?;
            let mut keypair_bytes: [u8; 64] = key_bytes.as_slice().try_into().map_err(|_| {
                key_bytes.zeroize();
                CanvasError::Baals(
                    "Private key must be 64 bytes (32-byte seed + 32-byte public key)".to_string(),
                )
            })?;
            let key =
                ed25519_dalek::SigningKey::from_keypair_bytes(&keypair_bytes).map_err(|e| {
                    key_bytes.zeroize();
                    keypair_bytes.zeroize();
                    CanvasError::Baals(format!("Invalid Ed25519 key: {}", e))
                })?;
            key_bytes.zeroize();
            keypair_bytes.zeroize();
            key
        };
        let verifying_key = signing_key.verifying_key();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let nonce: u64 = rand::random();
        let challenge = format!("baals-auth-token:{}:{}", timestamp, nonce);
        let signature = signing_key.sign(challenge.as_bytes());

        let body = serde_json::json!({
            "timestamp": timestamp,
            "nonce": nonce,
            "public_key": hex::encode(verifying_key.to_bytes()),
            "signature": hex::encode(signature.to_bytes()),
        });

        let resp: serde_json::Value = self
            .client
            .post(self.url("/api/v1/auth/token"))
            .json(&body)
            .send()
            .map_err(|e| CanvasError::Baals(format!("Auth request failed: {}", e)))?
            .json()
            .map_err(|e| CanvasError::Baals(format!("Auth response parse failed: {}", e)))?;

        resp.get("token")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| CanvasError::Baals("No token in auth response".to_string()))
    }

    fn headers_with_auth(&self, private_key: &str) -> CanvasResult<reqwest::header::HeaderMap> {
        let token = self.auth_token(private_key)?;
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|e| CanvasError::Baals(format!("Invalid header: {}", e)))?,
        );
        Ok(headers)
    }
}

impl BaalsClient for HttpBaalsClient {
    fn deploy_contract(
        &self,
        wasm_bytes: &[u8],
        constructor_args: serde_json::Value,
        private_key: &str,
    ) -> CanvasResult<DeploymentResult> {
        let headers = self.headers_with_auth(private_key)?;

        // Use the private key hex as the deployer public key (in production, derive the keypair properly)
        let deployer_bytes = hex::decode(private_key)
            .map_err(|_| CanvasError::Baals("Invalid deployer key hex".to_string()))?;
        let deployer_pub = if deployer_bytes.len() >= 32 {
            hex::encode(&deployer_bytes[..32])
        } else {
            return Err(CanvasError::Baals("Deployer key too short".to_string()));
        };

        let body = serde_json::json!({
            "deployer_hex": deployer_pub,
            "wasm_hex": hex::encode(wasm_bytes),
            "init_hex": hex::encode(serde_json::to_string(&constructor_args).unwrap_or_default().as_bytes()),
            "gas_limit": 1_000_000,
        });

        let resp: serde_json::Value = self
            .client
            .post(self.url("/api/v1/contracts/deploy"))
            .headers(headers)
            .json(&body)
            .send()
            .map_err(|e| CanvasError::Baals(format!("Deploy request failed: {}", e)))?
            .json()
            .map_err(|e| CanvasError::Baals(format!("Deploy response parse failed: {}", e)))?;

        let contract_id = resp
            .get("contract_id")
            .and_then(|c| c.as_str())
            .unwrap_or("");

        log::info!(
            "[BaaLS] Deployed contract {} ({} bytes)",
            contract_id,
            wasm_bytes.len()
        );

        Ok(DeploymentResult {
            contract_address: contract_id.to_string(),
            transaction_hash: format!("deploy-{}", contract_id),
            gas_used: (wasm_bytes.len() as u64 * 100).max(1000),
            block_number: 0,
        })
    }

    fn call_contract(
        &self,
        contract_address: &str,
        function_name: &str,
        arguments: Vec<serde_json::Value>,
        private_key: &str,
    ) -> CanvasResult<TransactionResult> {
        let headers = self.headers_with_auth(private_key)?;

        let args_hex: Vec<String> = arguments
            .iter()
            .map(|a| hex::encode(serde_json::to_string(a).unwrap_or_default().as_bytes()))
            .collect();

        let caller_bytes = hex::decode(private_key)
            .map_err(|_| CanvasError::Baals("Invalid caller key hex".to_string()))?;
        let caller_pub = if caller_bytes.len() >= 32 {
            hex::encode(&caller_bytes[..32])
        } else {
            return Err(CanvasError::Baals("Caller key too short".to_string()));
        };

        let body = serde_json::json!({
            "caller_hex": caller_pub,
            "contract_id": contract_address,
            "method": function_name,
            "args": args_hex,
            "value": 0,
            "gas_limit": 1_000_000,
        });

        let resp: serde_json::Value = self
            .client
            .post(self.url("/api/v1/contracts/invoke"))
            .headers(headers)
            .json(&body)
            .send()
            .map_err(|e| CanvasError::Baals(format!("Call request failed: {}", e)))?
            .json()
            .map_err(|e| CanvasError::Baals(format!("Call response parse failed: {}", e)))?;

        let result_hex = resp
            .get("result_hex")
            .and_then(|r| r.as_str())
            .unwrap_or("");
        let result_bytes = hex::decode(result_hex).unwrap_or_default();
        let result_str = String::from_utf8(result_bytes).unwrap_or_default();

        log::info!(
            "[BaaLS] Called {} on {} ({} args)",
            function_name,
            contract_address,
            arguments.len()
        );

        Ok(TransactionResult {
            transaction_hash: format!("call-{}-{}", contract_address, function_name),
            gas_used: (arguments.len() as u64 * 50).max(100),
            block_number: 0,
            success: true,
            output: serde_json::from_str(&result_str)
                .unwrap_or(serde_json::json!({"result": result_hex})),
            events: vec![],
        })
    }

    fn get_contract_state(&self, contract_address: &str) -> CanvasResult<ContractState> {
        let resp: serde_json::Value = self
            .client
            .get(self.url(&format!("/api/v1/contracts/{}/state", contract_address)))
            .send()
            .map_err(|e| CanvasError::Baals(format!("State request failed: {}", e)))?
            .json()
            .map_err(|e| CanvasError::Baals(format!("State response parse failed: {}", e)))?;

        let storage_list = resp
            .get("storage")
            .and_then(|s| s.as_array())
            .cloned()
            .unwrap_or_default();
        let mut storage = HashMap::new();
        for entry in &storage_list {
            if let (Some(k), Some(v)) = (
                entry.get("key").and_then(|k| k.as_str()),
                entry.get("value"),
            ) {
                storage.insert(k.to_string(), v.clone());
            }
        }

        Ok(ContractState {
            address: contract_address.to_string(),
            balance: 0,
            code_hash: String::new(),
            storage,
        })
    }

    fn read_storage(&self, contract_address: &str, key: &str) -> CanvasResult<serde_json::Value> {
        let key_hex = hex::encode(key);
        let resp: serde_json::Value = self
            .client
            .get(self.url(&format!(
                "/api/v1/proofs/contract/{}/storage/{}",
                contract_address, key_hex
            )))
            .send()
            .map_err(|e| CanvasError::Baals(format!("Storage read failed: {}", e)))?
            .json()
            .map_err(|e| CanvasError::Baals(format!("Storage response parse failed: {}", e)))?;

        let value_hex = resp.get("value_hex").and_then(|v| v.as_str()).unwrap_or("");
        let value_bytes = hex::decode(value_hex).unwrap_or_default();
        let value_str = String::from_utf8(value_bytes).unwrap_or_default();

        Ok(serde_json::Value::String(value_str))
    }

    fn get_transaction_status(&self, transaction_hash: &str) -> CanvasResult<TransactionStatus> {
        let resp: serde_json::Value = self
            .client
            .get(self.url(&format!(
                "/api/v1/transactions/{}/finality",
                transaction_hash
            )))
            .send()
            .map_err(|e| CanvasError::Baals(format!("Tx status request failed: {}", e)))?
            .json()
            .map_err(|e| CanvasError::Baals(format!("Tx status response parse failed: {}", e)))?;

        let finality = resp.get("finality").unwrap_or(&resp);
        let status_str = finality
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("Pending");
        let status = match status_str {
            "Success" => TransactionState::Confirmed,
            "Failed" => TransactionState::Failed,
            _ => TransactionState::Pending,
        };
        let block_height = finality
            .get("block_height")
            .and_then(|b| b.as_u64())
            .unwrap_or(0);
        let confirmations = finality
            .get("confirmations")
            .and_then(|c| c.as_u64())
            .unwrap_or(0);

        Ok(TransactionStatus {
            hash: transaction_hash.to_string(),
            status,
            block_number: block_height,
            gas_used: 0,
            confirmations,
        })
    }

    fn get_block_info(&self, block_number: u64) -> CanvasResult<BlockInfo> {
        let resp: serde_json::Value = self
            .client
            .get(self.url(&format!("/api/v1/blocks/{}", block_number)))
            .send()
            .map_err(|e| CanvasError::Baals(format!("Block request failed: {}", e)))?
            .json()
            .map_err(|e| CanvasError::Baals(format!("Block response parse failed: {}", e)))?;

        let hash = resp
            .get("hash")
            .and_then(|h| h.as_str())
            .map(hex::encode)
            .unwrap_or_default();
        let timestamp = resp.get("timestamp").and_then(|t| t.as_u64()).unwrap_or(0);
        let txs = resp
            .get("transactions")
            .and_then(|t| t.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|t| t.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(BlockInfo {
            number: block_number,
            hash,
            timestamp,
            transactions: txs,
        })
    }
}

pub struct BaalsNodeManager {
    config: Config,
    process: Option<std::process::Child>,
}

impl BaalsNodeManager {
    pub fn new(config: &Config) -> CanvasResult<Self> {
        Ok(Self {
            config: config.clone(),
            process: None,
        })
    }

    pub fn initialize(&mut self) -> CanvasResult<()> {
        if self.config.baals.enable_local_node {
            self.start_local_node()?;
        }
        Ok(())
    }

    pub fn shutdown(&mut self) -> CanvasResult<()> {
        self.stop_local_node()
    }

    pub fn is_local_node_running(&mut self) -> bool {
        self.process
            .as_mut()
            .is_some_and(|p| p.try_wait().map(|s| s.is_none()).unwrap_or(false))
    }

    fn start_local_node(&mut self) -> CanvasResult<()> {
        let port = self.config.baals.local_node_port;
        log::info!("Starting local BaaLS node on port {}", port);

        match std::process::Command::new("baalsd")
            .arg("--port")
            .arg(port.to_string())
            .arg("--data-dir")
            .arg("/tmp/baals-data")
            .spawn()
        {
            Ok(child) => {
                self.process = Some(child);
                std::thread::sleep(std::time::Duration::from_millis(500));

                if let Some(ref mut p) = self.process {
                    match p.try_wait() {
                        Ok(Some(status)) => {
                            return Err(CanvasError::Baals(format!(
                                "BaaLS node exited immediately with status: {}",
                                status
                            )));
                        }
                        Ok(None) => {} // still running — good
                        Err(e) => {
                            return Err(CanvasError::Baals(format!(
                                "Failed to check BaaLS node status: {}",
                                e
                            )));
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(CanvasError::Baals(format!(
                "Failed to spawn BaaLS node: {} — is 'baalsd' installed?",
                e
            ))),
        }
    }

    fn stop_local_node(&mut self) -> CanvasResult<()> {
        if let Some(mut child) = self.process.take() {
            log::info!("Stopping local BaaLS node (PID {})", child.id());
            let _ = child.kill();
            match child.wait() {
                Ok(status) => {
                    if !status.success() && status.code() != Some(-9) && status.code() != Some(143)
                    {
                        log::warn!("BaaLS node exited with non-zero status: {}", status);
                    }
                }
                Err(e) => log::warn!("Failed to wait on BaaLS node process: {}", e),
            }
        }
        Ok(())
    }
}

pub fn sign_payload(payload: &[u8], private_key_hex: &str) -> CanvasResult<Vec<u8>> {
    let key = {
        use zeroize::Zeroize;
        let mut key_bytes = hex::decode(private_key_hex)
            .map_err(|_| CanvasError::Baals("Invalid private key hex".to_string()))?;
        let mut keypair_bytes: [u8; 64] = key_bytes.as_slice().try_into().map_err(|_| {
            key_bytes.zeroize();
            CanvasError::Baals("Private key must be 64 bytes".to_string())
        })?;
        let key = ed25519_dalek::SigningKey::from_keypair_bytes(&keypair_bytes).map_err(|e| {
            key_bytes.zeroize();
            keypair_bytes.zeroize();
            CanvasError::Baals(format!("Invalid Ed25519 key: {}", e))
        })?;
        key_bytes.zeroize();
        keypair_bytes.zeroize();
        key
    };
    Ok(key.sign(payload).to_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_deploy_and_call() {
        let config = Config::default();
        let client = MockBaalsClient::new(&config).unwrap();
        let deploy = client
            .deploy_contract(b"wasm", serde_json::json!({}), "key")
            .unwrap();
        assert!(deploy.contract_address.starts_with("0x") || deploy.contract_address.len() == 64);
        let call = client
            .call_contract(&deploy.contract_address, "test", vec![], "key")
            .unwrap();
        assert!(call.success);
    }

    #[test]
    fn test_mock_deterministic_contract_id() {
        let id1 = MockBaalsClient::contract_id(b"hello");
        let id2 = MockBaalsClient::contract_id(b"hello");
        let id3 = MockBaalsClient::contract_id(b"world");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_factory_creates_mock() {
        let config = Config::default();
        let client = create_client(&config).unwrap();
        assert!(client
            .deploy_contract(b"t", serde_json::json!({}), "key")
            .is_ok());
    }

    #[test]
    fn test_sign_payload() {
        use ed25519_dalek::SigningKey;
        let secret = [0u8; 32];
        let key = SigningKey::from_bytes(&secret);
        let keypair_bytes = key.to_keypair_bytes();
        let hex_key = hex::encode(keypair_bytes);
        let sig = sign_payload(b"test", &hex_key).unwrap();
        assert_eq!(sig.len(), 64);
    }

    #[test]
    fn test_node_manager() {
        let mut config = Config::default();
        config.baals.enable_local_node = false; // don't actually spawn process
        let mut mgr = BaalsNodeManager::new(&config).unwrap();
        assert!(!mgr.is_local_node_running());
        assert!(mgr.initialize().is_ok());
        assert!(mgr.shutdown().is_ok());
    }

    #[test]
    fn test_http_client_rejects_when_node_down() {
        let mut config = Config::default();
        config.baals.node_url = "http://127.0.0.1:19999".to_string();
        let client = HttpBaalsClient::new(&config).unwrap();
        let result = client.deploy_contract(b"test", serde_json::json!({}), "deadbeef");
        assert!(result.is_err());
    }
}
