//! Production deployment and scaling system

use crate::{
    compiler::Compiler,
    config::Config,
    error::{CanvasError, CanvasResult},
    monitoring::{CircuitBreaker, HealthChecker, MetricsCollector},
    optimization::PerformanceOptimizer,
    types::VisualGraph,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    pub graph: VisualGraph,
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
    _config: Config,
    templates: Arc<Mutex<HashMap<String, InfrastructureTemplate>>>,
    active_deployments: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
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

fn unix_timestamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
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
    pub async fn deploy(
        &self,
        name: &str,
        graph: &VisualGraph,
        config: DeploymentConfig,
    ) -> CanvasResult<String> {
        let deployment_id = self.generate_deployment_id(name);

        // Optimize the graph
        let optimization_results = {
            let mut optimizer = self.optimizer.lock().unwrap();
            optimizer.optimize(graph)?
        };
        log::debug!(
            "Optimization applied for deployment {}: {} improvements",
            deployment_id,
            optimization_results.len()
        );

        // Compile to WASM
        let wasm_bytes = self.compile_graph(graph)?;
        self.validate_scaling_bounds(&config)?;

        // Create deployment info
        let deployment_info = DeploymentInfo {
            id: deployment_id.clone(),
            name: name.to_string(),
            status: DeploymentStatus::Pending,
            graph: graph.clone(),
            wasm_bytes,
            config,
            metrics: DeploymentMetrics::default(),
            created_at: unix_timestamp_secs(),
            updated_at: unix_timestamp_secs(),
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
        let deployment_name = {
            let deployments = self.deployments.lock().unwrap();
            let deployment = deployments.get(deployment_id).ok_or_else(|| {
                CanvasError::NotFound(format!("Deployment '{}' not found", deployment_id))
            })?;
            deployment.name.clone()
        };

        {
            let mut breakpoints = self.circuit_breakers.lock().unwrap();
            breakpoints
                .entry(deployment_id.to_string())
                .or_insert_with(|| {
                    CircuitBreaker::new(
                        deployment_id,
                        5,
                        Duration::from_secs(self.config.runtime.timeout.max(1)),
                    )
                });
        }

        let overall_health = {
            let checker = self.health_checker.lock().unwrap();
            checker.get_overall_health()
        };

        let mut deployments = self.deployments.lock().unwrap();
        let deployment = deployments.get_mut(deployment_id).ok_or_else(|| {
            CanvasError::NotFound(format!("Deployment '{}' not found", deployment_id))
        })?;

        deployment.status = DeploymentStatus::Deploying;
        deployment.metrics.availability = 100.0;
        deployment.metrics.throughput = deployment.config.replicas as f64;
        deployment.metrics.error_count = 0;

        match overall_health {
            crate::monitoring::HealthStatus::Healthy => {
                deployment.status = DeploymentStatus::Running;
            }
            crate::monitoring::HealthStatus::Degraded(reason) => {
                log::warn!(
                    "Deployment {} started in degraded mode: {}",
                    deployment_id,
                    reason
                );
                deployment.status = DeploymentStatus::Degraded;
                deployment.metrics.availability = 95.0;
            }
            crate::monitoring::HealthStatus::Unhealthy(reason) => {
                deployment.status = DeploymentStatus::Failed(format!(
                    "health checks failed during startup: {}",
                    reason
                ));
                deployment.metrics.availability = 0.0;
                deployment.updated_at = unix_timestamp_secs();
                return Err(CanvasError::InvalidState(format!(
                    "Deployment '{}' failed startup: {}",
                    deployment_name, reason
                )));
            }
        }

        deployment.updated_at = unix_timestamp_secs();
        drop(deployments);

        let metrics = self.metrics.lock().unwrap();
        metrics.increment_counter("deployments_started_total", 1)?;
        metrics.set_gauge(
            "deployments_running",
            self.running_deployments_count() as f64,
        )?;

        Ok(())
    }

    /// Scale deployment
    pub async fn scale(&self, deployment_id: &str, replicas: u32) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        let deployment = deployments.get_mut(deployment_id).ok_or_else(|| {
            CanvasError::NotFound(format!("Deployment '{}' not found", deployment_id))
        })?;

        if matches!(deployment.status, DeploymentStatus::Stopped) {
            return Err(CanvasError::InvalidState(format!(
                "Cannot scale stopped deployment '{}'",
                deployment_id
            )));
        }

        let min = deployment.config.scaling.min_replicas;
        let max = deployment.config.scaling.max_replicas;
        if replicas < min || replicas > max {
            return Err(CanvasError::Validation(format!(
                "Replica count {} is outside allowed range [{}..={}]",
                replicas, min, max
            )));
        }

        deployment.status = DeploymentStatus::Scaling;
        deployment.config.replicas = replicas;
        deployment.metrics.throughput = replicas as f64;
        deployment.metrics.cpu_usage = (deployment.metrics.cpu_usage + 5.0).min(100.0);
        deployment.metrics.memory_usage = (deployment.metrics.memory_usage + 2.0).min(100.0);
        deployment.status = DeploymentStatus::Running;
        deployment.updated_at = unix_timestamp_secs();
        drop(deployments);

        let metrics = self.metrics.lock().unwrap();
        metrics.increment_counter("deployments_scaled_total", 1)?;
        metrics.set_gauge("last_scaled_replicas", replicas as f64)?;

        Ok(())
    }

    /// Update deployment
    pub async fn update(&self, deployment_id: &str, graph: &VisualGraph) -> CanvasResult<()> {
        let wasm_bytes = self.compile_graph(graph)?;
        let mut deployments = self.deployments.lock().unwrap();
        let deployment = deployments.get_mut(deployment_id).ok_or_else(|| {
            CanvasError::NotFound(format!("Deployment '{}' not found", deployment_id))
        })?;

        deployment.status = DeploymentStatus::Deploying;
        deployment.graph = graph.clone();
        deployment.wasm_bytes = wasm_bytes;

        // Rolling update simulation: availability dip then recovery.
        deployment.metrics.availability = 99.5;
        deployment.metrics.response_time += 2.0;
        deployment.status = DeploymentStatus::Running;
        deployment.updated_at = unix_timestamp_secs();
        drop(deployments);

        let metrics = self.metrics.lock().unwrap();
        metrics.increment_counter("deployments_updated_total", 1)?;

        Ok(())
    }

    /// Stop deployment
    pub async fn stop(&self, deployment_id: &str) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        let deployment = deployments.get_mut(deployment_id).ok_or_else(|| {
            CanvasError::NotFound(format!("Deployment '{}' not found", deployment_id))
        })?;

        deployment.status = DeploymentStatus::Stopped;
        deployment.metrics.throughput = 0.0;
        deployment.metrics.availability = 0.0;
        deployment.updated_at = unix_timestamp_secs();
        drop(deployments);

        let mut breakers = self.circuit_breakers.lock().unwrap();
        breakers.remove(deployment_id);
        drop(breakers);

        let metrics = self.metrics.lock().unwrap();
        metrics.increment_counter("deployments_stopped_total", 1)?;
        metrics.set_gauge(
            "deployments_running",
            self.running_deployments_count() as f64,
        )?;

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
    fn compile_graph(&self, graph: &VisualGraph) -> CanvasResult<Vec<u8>> {
        let compiler = Compiler::new(&self.config)?;
        let compilation = compiler.compile(graph)?;
        Ok(compilation.wasm_bytes)
    }

    fn validate_scaling_bounds(&self, config: &DeploymentConfig) -> CanvasResult<()> {
        if config.scaling.min_replicas == 0 {
            return Err(CanvasError::Validation(
                "Minimum replicas must be at least 1".to_string(),
            ));
        }
        if config.scaling.max_replicas < config.scaling.min_replicas {
            return Err(CanvasError::Validation(format!(
                "max_replicas ({}) must be >= min_replicas ({})",
                config.scaling.max_replicas, config.scaling.min_replicas
            )));
        }
        if config.replicas < config.scaling.min_replicas
            || config.replicas > config.scaling.max_replicas
        {
            return Err(CanvasError::Validation(format!(
                "Initial replicas {} must be within [{}..={}]",
                config.replicas, config.scaling.min_replicas, config.scaling.max_replicas
            )));
        }
        Ok(())
    }

    fn running_deployments_count(&self) -> usize {
        let deployments = self.deployments.lock().unwrap();
        deployments
            .values()
            .filter(|d| {
                matches!(
                    d.status,
                    DeploymentStatus::Running | DeploymentStatus::Degraded
                )
            })
            .count()
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
    pub async fn create_deployment(
        &self,
        id: &str,
        graph: &VisualGraph,
        config: DeploymentConfig,
    ) -> CanvasResult<()> {
        if id.trim().is_empty() {
            return Err(CanvasError::Validation(
                "Blue/green deployment id cannot be empty".to_string(),
            ));
        }
        if config.scaling.min_replicas > config.scaling.max_replicas {
            return Err(CanvasError::Validation(
                "Invalid scaling bounds for blue/green deployment".to_string(),
            ));
        }

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
        log::info!(
            "Created blue/green deployment '{}' for graph '{}' with {} nodes",
            id,
            graph.name,
            graph.nodes.len()
        );

        Ok(())
    }

    /// Deploy to blue environment
    pub async fn deploy_blue(
        &self,
        id: &str,
        graph: &VisualGraph,
        config: DeploymentConfig,
    ) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        let deployment = deployments.get_mut(id).ok_or_else(|| {
            CanvasError::NotFound(format!("Blue/green deployment '{}' not found", id))
        })?;
        deployment.blue_deployment =
            Some(self.create_environment_deployment(id, "blue", graph, config)?);
        if matches!(deployment.active_environment, ActiveEnvironment::Blue) {
            deployment.switchover_config.automatic_switchover = true;
        }

        Ok(())
    }

    /// Deploy to green environment
    pub async fn deploy_green(
        &self,
        id: &str,
        graph: &VisualGraph,
        config: DeploymentConfig,
    ) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        let deployment = deployments.get_mut(id).ok_or_else(|| {
            CanvasError::NotFound(format!("Blue/green deployment '{}' not found", id))
        })?;
        deployment.green_deployment =
            Some(self.create_environment_deployment(id, "green", graph, config)?);

        Ok(())
    }

    /// Switch traffic to green environment
    pub async fn switch_to_green(&self, id: &str) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        let deployment = deployments.get_mut(id).ok_or_else(|| {
            CanvasError::NotFound(format!("Blue/green deployment '{}' not found", id))
        })?;
        if deployment.green_deployment.is_none() {
            return Err(CanvasError::InvalidState(format!(
                "Cannot switch '{}' to green before green environment is deployed",
                id
            )));
        }
        deployment.active_environment = ActiveEnvironment::Green;
        log::info!("Switched '{}' traffic to green environment", id);

        Ok(())
    }

    /// Switch traffic to blue environment
    pub async fn switch_to_blue(&self, id: &str) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        let deployment = deployments.get_mut(id).ok_or_else(|| {
            CanvasError::NotFound(format!("Blue/green deployment '{}' not found", id))
        })?;
        if deployment.blue_deployment.is_none() {
            return Err(CanvasError::InvalidState(format!(
                "Cannot switch '{}' to blue before blue environment is deployed",
                id
            )));
        }
        deployment.active_environment = ActiveEnvironment::Blue;
        log::info!("Switched '{}' traffic to blue environment", id);

        Ok(())
    }

    /// Rollback to previous environment
    pub async fn rollback(&self, id: &str) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        let deployment = deployments.get_mut(id).ok_or_else(|| {
            CanvasError::NotFound(format!("Blue/green deployment '{}' not found", id))
        })?;
        deployment.active_environment = match deployment.active_environment {
            ActiveEnvironment::Blue => {
                if deployment.green_deployment.is_none() {
                    return Err(CanvasError::InvalidState(format!(
                        "Rollback requested for '{}', but no green deployment exists",
                        id
                    )));
                }
                ActiveEnvironment::Green
            }
            ActiveEnvironment::Green => {
                if deployment.blue_deployment.is_none() {
                    return Err(CanvasError::InvalidState(format!(
                        "Rollback requested for '{}', but no blue deployment exists",
                        id
                    )));
                }
                ActiveEnvironment::Blue
            }
        };
        log::warn!("Rollback performed for '{}'", id);

        Ok(())
    }

    fn create_environment_deployment(
        &self,
        id: &str,
        environment: &str,
        graph: &VisualGraph,
        config: DeploymentConfig,
    ) -> CanvasResult<DeploymentInfo> {
        let compiler = Compiler::new(&self.config)?;
        let result = compiler.compile(graph)?;
        Ok(DeploymentInfo {
            id: format!("{}-{}", id, environment),
            name: format!("{} {}", id, environment.to_uppercase()),
            status: DeploymentStatus::Running,
            graph: graph.clone(),
            wasm_bytes: result.wasm_bytes,
            config,
            metrics: DeploymentMetrics::default(),
            created_at: unix_timestamp_secs(),
            updated_at: unix_timestamp_secs(),
        })
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
    pub async fn create_deployment(
        &self,
        id: &str,
        stable_deployment: DeploymentInfo,
        config: DeploymentConfig,
    ) -> CanvasResult<()> {
        if id.trim().is_empty() {
            return Err(CanvasError::Validation(
                "Canary deployment id cannot be empty".to_string(),
            ));
        }
        if config.scaling.min_replicas > config.scaling.max_replicas {
            return Err(CanvasError::Validation(
                "Invalid canary scaling bounds".to_string(),
            ));
        }

        let mut canary_wasm = stable_deployment.wasm_bytes.clone();
        if canary_wasm.is_empty() {
            let compiler = Compiler::new(&self.config)?;
            canary_wasm = compiler.compile(&stable_deployment.graph)?.wasm_bytes;
        }

        let deployment = CanaryDeployment {
            id: id.to_string(),
            stable_deployment: stable_deployment.clone(),
            canary_deployment: DeploymentInfo {
                id: format!("{}-canary", id),
                name: format!("{} Canary", id),
                status: DeploymentStatus::Pending,
                graph: stable_deployment.graph.clone(),
                wasm_bytes: canary_wasm,
                config,
                metrics: DeploymentMetrics::default(),
                created_at: unix_timestamp_secs(),
                updated_at: unix_timestamp_secs(),
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
        log::info!("Created canary deployment '{}'", id);

        Ok(())
    }

    /// Update traffic split
    pub async fn update_traffic_split(
        &self,
        id: &str,
        stable_percentage: f64,
        canary_percentage: f64,
    ) -> CanvasResult<()> {
        if stable_percentage < 0.0 || canary_percentage < 0.0 {
            return Err(CanvasError::Validation(
                "Traffic percentages must be non-negative".to_string(),
            ));
        }
        let total = stable_percentage + canary_percentage;
        if (total - 100.0).abs() > 0.001 {
            return Err(CanvasError::Validation(format!(
                "Traffic split must total 100.0, got {:.3}",
                total
            )));
        }

        let mut deployments = self.deployments.lock().unwrap();
        let deployment = deployments.get_mut(id).ok_or_else(|| {
            CanvasError::NotFound(format!("Canary deployment '{}' not found", id))
        })?;

        deployment.traffic_split.stable_percentage = stable_percentage;
        deployment.traffic_split.canary_percentage = canary_percentage;
        deployment.canary_deployment.status = if canary_percentage > 0.0 {
            DeploymentStatus::Running
        } else {
            DeploymentStatus::Stopped
        };
        deployment.canary_deployment.updated_at = unix_timestamp_secs();

        Ok(())
    }

    /// Promote canary to stable
    pub async fn promote_canary(&self, id: &str) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        let deployment = deployments.get_mut(id).ok_or_else(|| {
            CanvasError::NotFound(format!("Canary deployment '{}' not found", id))
        })?;

        deployment.canary_deployment.status = DeploymentStatus::Running;
        deployment.stable_deployment = deployment.canary_deployment.clone();
        deployment.stable_deployment.id = format!("{}-stable", id);
        deployment.stable_deployment.name = format!("{} Stable", id);
        deployment.stable_deployment.updated_at = unix_timestamp_secs();
        deployment.traffic_split.stable_percentage = 100.0;
        deployment.traffic_split.canary_percentage = 0.0;
        deployment.canary_deployment.status = DeploymentStatus::Stopped;
        deployment.canary_deployment.updated_at = unix_timestamp_secs();

        Ok(())
    }

    /// Rollback canary deployment
    pub async fn rollback_canary(&self, id: &str) -> CanvasResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        let deployment = deployments.get_mut(id).ok_or_else(|| {
            CanvasError::NotFound(format!("Canary deployment '{}' not found", id))
        })?;

        deployment.traffic_split.stable_percentage = 100.0;
        deployment.traffic_split.canary_percentage = 0.0;
        deployment.canary_deployment.status =
            DeploymentStatus::Failed("Rolled back by deployment manager".to_string());
        deployment.canary_deployment.updated_at = unix_timestamp_secs();

        Ok(())
    }
}

impl InfrastructureManager {
    /// Create a new infrastructure manager
    pub fn new(config: &Config) -> Self {
        Self {
            _config: config.clone(),
            templates: Arc::new(Mutex::new(HashMap::new())),
            active_deployments: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register infrastructure template
    pub fn register_template(&self, template: InfrastructureTemplate) -> CanvasResult<()> {
        let mut templates = self.templates.lock().unwrap();
        templates.insert(template.name.clone(), template);
        Ok(())
    }

    /// Deploy infrastructure
    pub async fn deploy_infrastructure(
        &self,
        template_name: &str,
        variables: HashMap<String, String>,
    ) -> CanvasResult<()> {
        let templates = self.templates.lock().unwrap();
        let template = templates
            .get(template_name)
            .ok_or_else(|| {
                CanvasError::NotFound(format!("Template '{}' not found", template_name))
            })?
            .clone();
        drop(templates);

        let mut resolved = HashMap::new();
        for (key, default_value) in template.variables {
            let value = variables.get(&key).cloned().unwrap_or(default_value);
            resolved.insert(key, value);
        }
        for (key, value) in variables {
            resolved.entry(key).or_insert(value);
        }

        let mut deployments = self.active_deployments.lock().unwrap();
        deployments.insert(template_name.to_string(), resolved);
        log::info!(
            "Deploying infrastructure template '{}' using provider {:?}",
            template_name,
            template.provider
        );

        Ok(())
    }

    /// Destroy infrastructure
    pub async fn destroy_infrastructure(&self, template_name: &str) -> CanvasResult<()> {
        let mut deployments = self.active_deployments.lock().unwrap();
        if deployments.remove(template_name).is_none() {
            return Err(CanvasError::NotFound(format!(
                "No active infrastructure deployment found for '{}'",
                template_name
            )));
        }
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

        let graph = VisualGraph::new("test");
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

        let deployment_id = manager
            .deploy("test-deployment", &graph, config)
            .await
            .unwrap();
        assert!(!deployment_id.is_empty());

        let status = manager.get_status(&deployment_id);
        assert!(status.is_some());
    }

    #[tokio::test]
    async fn test_blue_green_deployment() {
        let config = Config::default();
        let manager = BlueGreenDeploymentManager::new(&config);

        let graph = VisualGraph::new("test");
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

        manager
            .create_deployment("test-bg", &graph, config.clone())
            .await
            .unwrap();
        manager
            .deploy_blue("test-bg", &graph, config.clone())
            .await
            .unwrap();
        manager
            .deploy_green("test-bg", &graph, config)
            .await
            .unwrap();
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
            graph: VisualGraph::new("stable"),
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

        manager
            .create_deployment("test-canary", stable_deployment, canary_config)
            .await
            .unwrap();
        manager
            .update_traffic_split("test-canary", 80.0, 20.0)
            .await
            .unwrap();
    }
}
