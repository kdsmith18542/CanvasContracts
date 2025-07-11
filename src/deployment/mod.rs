//! Production deployment and scaling system

use crate::{
    error::CanvasResult,
    types::{Graph, NodeId},
    config::Config,
    monitoring::{MetricsCollector, HealthChecker, CircuitBreaker},
    optimization::PerformanceOptimizer,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Production deployment manager
pub struct DeploymentManager {
    config: Config,
    metrics: Arc<Mutex<MetricsCollector>>,
    health_checker: Arc<Mutex<HealthChecker>>,
    optimizer: Arc<Mutex<PerformanceOptimizer>>,
    deployments: Arc<Mutex<HashMap<String, DeploymentInfo>>>,
    circuit_breakers: Arc<Mutex<HashMap<String, CircuitBreaker>>>,
}

/// Deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub id: String,
    pub name: String,
    pub status: DeploymentStatus,
    pub graph: Graph,
    pub wasm_bytes: Vec<u8>,
    pub config: DeploymentConfig,
    pub metrics: DeploymentMetrics,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    Deploying,
    Running,
    Scaling,
    Degraded,
    Failed(String),
    Stopped,
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub replicas: u32,
    pub resources: ResourceRequirements,
    pub scaling: ScalingConfig,
    pub health_check: HealthCheckConfig,
    pub monitoring: MonitoringConfig,
    pub security: SecurityConfig,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_requests: String,
    pub cpu_limits: String,
    pub memory_requests: String,
    pub memory_limits: String,
    pub storage_requests: String,
}

/// Scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_utilization: f64,
    pub target_memory_utilization: f64,
    pub scale_up_cooldown: u64,
    pub scale_down_cooldown: u64,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub initial_delay_seconds: u32,
    pub period_seconds: u32,
    pub timeout_seconds: u32,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub health_check_path: String,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_endpoint: String,
    pub log_level: String,
    pub enable_tracing: bool,
    pub enable_profiling: bool,
    pub alert_rules: Vec<AlertRule>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_tls: bool,
    pub certificate_path: Option<String>,
    pub key_path: Option<String>,
    pub allowed_origins: Vec<String>,
    pub rate_limiting: RateLimitingConfig,
}

/// Alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub condition: String,
    pub threshold: f64,
    pub duration: u64,
    pub severity: AlertSeverity,
    pub notification: NotificationConfig,
}

/// Alert severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub email: Option<String>,
    pub webhook: Option<String>,
    pub slack: Option<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
    pub window_size: u64,
}

/// Deployment metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub request_count: u64,
    pub error_count: u64,
    pub response_time: f64,
    pub throughput: f64,
    pub availability: f64,
}

/// Blue-green deployment manager
pub struct BlueGreenDeploymentManager {
    config: Config,
    deployments: Arc<Mutex<HashMap<String, BlueGreenDeployment>>>,
}

/// Blue-green deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueGreenDeployment {
    pub id: String,
    pub blue_deployment: Option<DeploymentInfo>,
    pub green_deployment: Option<DeploymentInfo>,
    pub active_environment: ActiveEnvironment,
    pub switchover_config: SwitchoverConfig,
}

/// Active environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActiveEnvironment {
    Blue,
    Green,
}

/// Switchover configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchoverConfig {
    pub automatic_switchover: bool,
    pub health_check_threshold: f64,
    pub rollback_threshold: f64,
    pub switchover_delay: u64,
}

/// Canary deployment manager
pub struct CanaryDeploymentManager {
    config: Config,
    deployments: Arc<Mutex<HashMap<String, CanaryDeployment>>>,
}

/// Canary deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryDeployment {
    pub id: String,
    pub stable_deployment: DeploymentInfo,
    pub canary_deployment: DeploymentInfo,
    pub traffic_split: TrafficSplit,
    pub promotion_config: PromotionConfig,
}

/// Traffic split
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSplit {
    pub stable_percentage: f64,
    pub canary_percentage: f64,
    pub routing_rules: Vec<RoutingRule>,
}

/// Routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub condition: String,
    pub target: RoutingTarget,
    pub weight: f64,
}

/// Routing target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingTarget {
    Stable,
    Canary,
}

/// Promotion configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionConfig {
    pub automatic_promotion: bool,
    pub success_threshold: f64,
    pub evaluation_period: u64,
    pub metrics: Vec<String>,
}

/// Infrastructure as Code manager
pub struct InfrastructureManager {
    config: Config,
    templates: Arc<Mutex<HashMap<String, InfrastructureTemplate>>>,
}

/// Infrastructure template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureTemplate {
    pub name: String,
    pub description: String,
    pub provider: CloudProvider,
    pub resources: Vec<ResourceDefinition>,
    pub variables: HashMap<String, String>,
}

/// Cloud provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudProvider {
    Aws,
    Gcp,
    Azure,
    Kubernetes,
    Docker,
}

/// Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDefinition {
    pub resource_type: String,
    pub name: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub dependencies: Vec<String>,
}

impl DeploymentManager {
    /// Create a new deployment manager
    pub fn new(config: &Config) -> CanvasResult<Self> {
        let metrics = Arc::new(Mutex::new(MetricsCollector::new(config)?));
        let health_checker = Arc::new(Mutex::new(HealthChecker::new(config)));
        let optimizer = Arc::new(Mutex::new(PerformanceOptimizer::new(config)));

        Ok(Self {
            config: config.clone(),
            metrics,
            health_checker,
            optimizer,
            deployments: Arc::new(Mutex::new(HashMap::new())),
            circuit_breakers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Deploy a contract
    pub async fn deploy(&self, name: &str, graph: &Graph, config: DeploymentConfig) -> CanvasResult<String> {
        let deployment_id = self.generate_deployment_id(name);
        
        // Optimize the graph
        let optimization_results = {
            let mut optimizer = self.optimizer.lock().unwrap();
            optimizer.optimize(graph)?
        };

        // Compile to WASM
        let wasm_bytes = self.compile_graph(graph)?;

        // Create deployment info
        let deployment_info = DeploymentInfo {
            id: deployment_id.clone(),
            name: name.to_string(),
            status: DeploymentStatus::Pending,
            graph: graph.clone(),
            wasm_bytes,
            config,
            metrics: DeploymentMetrics::default(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Store deployment
        {
            let mut deployments = self.deployments.lock().unwrap();
            deployments.insert(deployment_id.clone(), deployment_info);
        }

        // Start deployment process
        self.start_deployment(&deployment_id).await?;

        Ok(deployment_id)
    }

    /// Start deployment process
    async fn start_deployment(&self, deployment_id: &str) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        
        if let Some(deployment) = deployments.get_mut(deployment_id) {
            deployment.status = DeploymentStatus::Deploying;
            
            // TODO: Implement actual deployment logic
            // - Provision infrastructure
            // - Deploy containers/pods
            // - Configure load balancers
            // - Set up monitoring
            
            deployment.status = DeploymentStatus::Running;
            deployment.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }

        Ok(())
    }

    /// Scale deployment
    pub async fn scale(&self, deployment_id: &str, replicas: u32) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        
        if let Some(deployment) = deployments.get_mut(deployment_id) {
            deployment.status = DeploymentStatus::Scaling;
            deployment.config.replicas = replicas;
            
            // TODO: Implement actual scaling logic
            // - Scale up/down containers/pods
            // - Update load balancer configuration
            
            deployment.status = DeploymentStatus::Running;
            deployment.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }

        Ok(())
    }

    /// Update deployment
    pub async fn update(&self, deployment_id: &str, graph: &Graph) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        
        if let Some(deployment) = deployments.get_mut(deployment_id) {
            deployment.status = DeploymentStatus::Deploying;
            deployment.graph = graph.clone();
            
            // Recompile with new graph
            deployment.wasm_bytes = self.compile_graph(graph)?;
            
            // TODO: Implement rolling update logic
            // - Deploy new version alongside old version
            // - Gradually shift traffic
            // - Remove old version
            
            deployment.status = DeploymentStatus::Running;
            deployment.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }

        Ok(())
    }

    /// Stop deployment
    pub async fn stop(&self, deployment_id: &str) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        
        if let Some(deployment) = deployments.get_mut(deployment_id) {
            deployment.status = DeploymentStatus::Stopped;
            
            // TODO: Implement actual stop logic
            // - Stop containers/pods
            // - Remove from load balancer
            // - Clean up resources
            
            deployment.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }

        Ok(())
    }

    /// Get deployment status
    pub fn get_status(&self, deployment_id: &str) -> Option<DeploymentStatus> {
        let deployments = self.deployments.lock().unwrap();
        deployments.get(deployment_id).map(|d| d.status.clone())
    }

    /// Get deployment metrics
    pub fn get_metrics(&self, deployment_id: &str) -> Option<DeploymentMetrics> {
        let deployments = self.deployments.lock().unwrap();
        deployments.get(deployment_id).map(|d| d.metrics.clone())
    }

    /// List all deployments
    pub fn list_deployments(&self) -> Vec<DeploymentInfo> {
        let deployments = self.deployments.lock().unwrap();
        deployments.values().cloned().collect()
    }

    /// Generate deployment ID
    fn generate_deployment_id(&self, name: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        std::time::SystemTime::now().hash(&mut hasher);
        
        format!("{}-{:x}", name, hasher.finish())
    }

    /// Compile graph to WASM
    fn compile_graph(&self, graph: &Graph) -> CanvasResult<Vec<u8>> {
        // TODO: Implement actual compilation
        // For now, return mock WASM bytes
        Ok(vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00])
    }
}

impl Default for DeploymentMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            request_count: 0,
            error_count: 0,
            response_time: 0.0,
            throughput: 0.0,
            availability: 100.0,
        }
    }
}

impl BlueGreenDeploymentManager {
    /// Create a new blue-green deployment manager
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            deployments: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create blue-green deployment
    pub async fn create_deployment(&self, id: &str, graph: &Graph, config: DeploymentConfig) -> CanvasResult<()> {
        let deployment = BlueGreenDeployment {
            id: id.to_string(),
            blue_deployment: None,
            green_deployment: None,
            active_environment: ActiveEnvironment::Blue,
            switchover_config: SwitchoverConfig {
                automatic_switchover: true,
                health_check_threshold: 0.95,
                rollback_threshold: 0.8,
                switchover_delay: 30,
            },
        };

        let mut deployments = self.deployments.lock().unwrap();
        deployments.insert(id.to_string(), deployment);

        Ok(())
    }

    /// Deploy to blue environment
    pub async fn deploy_blue(&self, id: &str, graph: &Graph, config: DeploymentConfig) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        
        if let Some(deployment) = deployments.get_mut(id) {
            // TODO: Implement actual blue deployment
            deployment.blue_deployment = Some(DeploymentInfo {
                id: format!("{}-blue", id),
                name: format!("{} Blue", id),
                status: DeploymentStatus::Running,
                graph: graph.clone(),
                wasm_bytes: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
                config,
                metrics: DeploymentMetrics::default(),
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }

        Ok(())
    }

    /// Deploy to green environment
    pub async fn deploy_green(&self, id: &str, graph: &Graph, config: DeploymentConfig) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        
        if let Some(deployment) = deployments.get_mut(id) {
            // TODO: Implement actual green deployment
            deployment.green_deployment = Some(DeploymentInfo {
                id: format!("{}-green", id),
                name: format!("{} Green", id),
                status: DeploymentStatus::Running,
                graph: graph.clone(),
                wasm_bytes: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
                config,
                metrics: DeploymentMetrics::default(),
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }

        Ok(())
    }

    /// Switch traffic to green environment
    pub async fn switch_to_green(&self, id: &str) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        
        if let Some(deployment) = deployments.get_mut(id) {
            if deployment.green_deployment.is_some() {
                deployment.active_environment = ActiveEnvironment::Green;
                
                // TODO: Implement actual traffic switching
                // - Update load balancer configuration
                // - Gradually shift traffic
                // - Monitor health metrics
            }
        }

        Ok(())
    }

    /// Switch traffic to blue environment
    pub async fn switch_to_blue(&self, id: &str) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        
        if let Some(deployment) = deployments.get_mut(id) {
            deployment.active_environment = ActiveEnvironment::Blue;
            
            // TODO: Implement actual traffic switching
        }

        Ok(())
    }

    /// Rollback to previous environment
    pub async fn rollback(&self, id: &str) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        
        if let Some(deployment) = deployments.get_mut(id) {
            match deployment.active_environment {
                ActiveEnvironment::Blue => {
                    deployment.active_environment = ActiveEnvironment::Green;
                }
                ActiveEnvironment::Green => {
                    deployment.active_environment = ActiveEnvironment::Blue;
                }
            }
            
            // TODO: Implement actual rollback logic
        }

        Ok(())
    }
}

impl CanaryDeploymentManager {
    /// Create a new canary deployment manager
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            deployments: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create canary deployment
    pub async fn create_deployment(&self, id: &str, stable_deployment: DeploymentInfo, config: DeploymentConfig) -> CanvasResult<()> {
        let deployment = CanaryDeployment {
            id: id.to_string(),
            stable_deployment,
            canary_deployment: DeploymentInfo {
                id: format!("{}-canary", id),
                name: format!("{} Canary", id),
                status: DeploymentStatus::Pending,
                graph: Graph::new("canary"),
                wasm_bytes: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
                config,
                metrics: DeploymentMetrics::default(),
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
            traffic_split: TrafficSplit {
                stable_percentage: 90.0,
                canary_percentage: 10.0,
                routing_rules: Vec::new(),
            },
            promotion_config: PromotionConfig {
                automatic_promotion: true,
                success_threshold: 0.95,
                evaluation_period: 300,
                metrics: vec!["error_rate".to_string(), "response_time".to_string()],
            },
        };

        let mut deployments = self.deployments.lock().unwrap();
        deployments.insert(id.to_string(), deployment);

        Ok(())
    }

    /// Update traffic split
    pub async fn update_traffic_split(&self, id: &str, stable_percentage: f64, canary_percentage: f64) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        
        if let Some(deployment) = deployments.get_mut(id) {
            deployment.traffic_split.stable_percentage = stable_percentage;
            deployment.traffic_split.canary_percentage = canary_percentage;
            
            // TODO: Implement actual traffic splitting
            // - Update load balancer weights
            // - Monitor canary metrics
        }

        Ok(())
    }

    /// Promote canary to stable
    pub async fn promote_canary(&self, id: &str) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        
        if let Some(deployment) = deployments.get_mut(id) {
            // TODO: Implement actual promotion
            // - Replace stable deployment with canary
            // - Update traffic split to 100% stable
            // - Clean up old canary deployment
        }

        Ok(())
    }

    /// Rollback canary deployment
    pub async fn rollback_canary(&self, id: &str) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        
        if let Some(deployment) = deployments.get_mut(id) {
            // TODO: Implement actual rollback
            // - Set traffic split to 100% stable
            // - Stop canary deployment
        }

        Ok(())
    }
}

impl InfrastructureManager {
    /// Create a new infrastructure manager
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            templates: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register infrastructure template
    pub fn register_template(&self, template: InfrastructureTemplate) -> CanvasResult<()> {
        let mut templates = self.templates.lock().unwrap();
        templates.insert(template.name.clone(), template);
        Ok(())
    }

    /// Deploy infrastructure
    pub async fn deploy_infrastructure(&self, template_name: &str, variables: HashMap<String, String>) -> CanvasResult<()> {
        let templates = self.templates.lock().unwrap();
        
        if let Some(template) = templates.get(template_name) {
            // TODO: Implement actual infrastructure deployment
            // - Generate configuration files
            // - Execute deployment commands
            // - Monitor deployment progress
            log::info!("Deploying infrastructure template: {}", template_name);
        }

        Ok(())
    }

    /// Destroy infrastructure
    pub async fn destroy_infrastructure(&self, template_name: &str) -> CanvasResult<()> {
        // TODO: Implement actual infrastructure destruction
        log::info!("Destroying infrastructure template: {}", template_name);
        Ok(())
    }

    /// List available templates
    pub fn list_templates(&self) -> Vec<String> {
        let templates = self.templates.lock().unwrap();
        templates.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deployment_manager() {
        let config = Config::default();
        let manager = DeploymentManager::new(&config).unwrap();
        
        let graph = Graph::new("test");
        let config = DeploymentConfig {
            replicas: 3,
            resources: ResourceRequirements {
                cpu_requests: "100m".to_string(),
                cpu_limits: "500m".to_string(),
                memory_requests: "128Mi".to_string(),
                memory_limits: "512Mi".to_string(),
                storage_requests: "1Gi".to_string(),
            },
            scaling: ScalingConfig {
                min_replicas: 1,
                max_replicas: 10,
                target_cpu_utilization: 70.0,
                target_memory_utilization: 80.0,
                scale_up_cooldown: 300,
                scale_down_cooldown: 300,
            },
            health_check: HealthCheckConfig {
                initial_delay_seconds: 30,
                period_seconds: 10,
                timeout_seconds: 5,
                failure_threshold: 3,
                success_threshold: 1,
                health_check_path: "/health".to_string(),
            },
            monitoring: MonitoringConfig {
                metrics_endpoint: "/metrics".to_string(),
                log_level: "info".to_string(),
                enable_tracing: true,
                enable_profiling: false,
                alert_rules: Vec::new(),
            },
            security: SecurityConfig {
                enable_tls: false,
                certificate_path: None,
                key_path: None,
                allowed_origins: vec!["*".to_string()],
                rate_limiting: RateLimitingConfig {
                    requests_per_second: 1000,
                    burst_size: 100,
                    window_size: 60,
                },
            },
        };
        
        let deployment_id = manager.deploy("test-deployment", &graph, config).await.unwrap();
        assert!(!deployment_id.is_empty());
        
        let status = manager.get_status(&deployment_id);
        assert!(status.is_some());
    }

    #[tokio::test]
    async fn test_blue_green_deployment() {
        let config = Config::default();
        let manager = BlueGreenDeploymentManager::new(&config);
        
        let graph = Graph::new("test");
        let config = DeploymentConfig {
            replicas: 2,
            resources: ResourceRequirements {
                cpu_requests: "100m".to_string(),
                cpu_limits: "500m".to_string(),
                memory_requests: "128Mi".to_string(),
                memory_limits: "512Mi".to_string(),
                storage_requests: "1Gi".to_string(),
            },
            scaling: ScalingConfig {
                min_replicas: 1,
                max_replicas: 5,
                target_cpu_utilization: 70.0,
                target_memory_utilization: 80.0,
                scale_up_cooldown: 300,
                scale_down_cooldown: 300,
            },
            health_check: HealthCheckConfig {
                initial_delay_seconds: 30,
                period_seconds: 10,
                timeout_seconds: 5,
                failure_threshold: 3,
                success_threshold: 1,
                health_check_path: "/health".to_string(),
            },
            monitoring: MonitoringConfig {
                metrics_endpoint: "/metrics".to_string(),
                log_level: "info".to_string(),
                enable_tracing: true,
                enable_profiling: false,
                alert_rules: Vec::new(),
            },
            security: SecurityConfig {
                enable_tls: false,
                certificate_path: None,
                key_path: None,
                allowed_origins: vec!["*".to_string()],
                rate_limiting: RateLimitingConfig {
                    requests_per_second: 1000,
                    burst_size: 100,
                    window_size: 60,
                },
            },
        };
        
        manager.create_deployment("test-bg", &graph, config.clone()).await.unwrap();
        manager.deploy_blue("test-bg", &graph, config.clone()).await.unwrap();
        manager.deploy_green("test-bg", &graph, config).await.unwrap();
        manager.switch_to_green("test-bg").await.unwrap();
    }

    #[tokio::test]
    async fn test_canary_deployment() {
        let config = Config::default();
        let manager = CanaryDeploymentManager::new(&config);
        
        let stable_deployment = DeploymentInfo {
            id: "stable".to_string(),
            name: "Stable".to_string(),
            status: DeploymentStatus::Running,
            graph: Graph::new("stable"),
            wasm_bytes: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
            config: DeploymentConfig {
                replicas: 3,
                resources: ResourceRequirements {
                    cpu_requests: "100m".to_string(),
                    cpu_limits: "500m".to_string(),
                    memory_requests: "128Mi".to_string(),
                    memory_limits: "512Mi".to_string(),
                    storage_requests: "1Gi".to_string(),
                },
                scaling: ScalingConfig {
                    min_replicas: 1,
                    max_replicas: 10,
                    target_cpu_utilization: 70.0,
                    target_memory_utilization: 80.0,
                    scale_up_cooldown: 300,
                    scale_down_cooldown: 300,
                },
                health_check: HealthCheckConfig {
                    initial_delay_seconds: 30,
                    period_seconds: 10,
                    timeout_seconds: 5,
                    failure_threshold: 3,
                    success_threshold: 1,
                    health_check_path: "/health".to_string(),
                },
                monitoring: MonitoringConfig {
                    metrics_endpoint: "/metrics".to_string(),
                    log_level: "info".to_string(),
                    enable_tracing: true,
                    enable_profiling: false,
                    alert_rules: Vec::new(),
                },
                security: SecurityConfig {
                    enable_tls: false,
                    certificate_path: None,
                    key_path: None,
                    allowed_origins: vec!["*".to_string()],
                    rate_limiting: RateLimitingConfig {
                        requests_per_second: 1000,
                        burst_size: 100,
                        window_size: 60,
                    },
                },
            },
            metrics: DeploymentMetrics::default(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        let canary_config = DeploymentConfig {
            replicas: 1,
            resources: ResourceRequirements {
                cpu_requests: "100m".to_string(),
                cpu_limits: "500m".to_string(),
                memory_requests: "128Mi".to_string(),
                memory_limits: "512Mi".to_string(),
                storage_requests: "1Gi".to_string(),
            },
            scaling: ScalingConfig {
                min_replicas: 1,
                max_replicas: 5,
                target_cpu_utilization: 70.0,
                target_memory_utilization: 80.0,
                scale_up_cooldown: 300,
                scale_down_cooldown: 300,
            },
            health_check: HealthCheckConfig {
                initial_delay_seconds: 30,
                period_seconds: 10,
                timeout_seconds: 5,
                failure_threshold: 3,
                success_threshold: 1,
                health_check_path: "/health".to_string(),
            },
            monitoring: MonitoringConfig {
                metrics_endpoint: "/metrics".to_string(),
                log_level: "info".to_string(),
                enable_tracing: true,
                enable_profiling: false,
                alert_rules: Vec::new(),
            },
            security: SecurityConfig {
                enable_tls: false,
                certificate_path: None,
                key_path: None,
                allowed_origins: vec!["*".to_string()],
                rate_limiting: RateLimitingConfig {
                    requests_per_second: 1000,
                    burst_size: 100,
                    window_size: 60,
                },
            },
        };
        
        manager.create_deployment("test-canary", stable_deployment, canary_config).await.unwrap();
        manager.update_traffic_split("test-canary", 80.0, 20.0).await.unwrap();
    }
} 