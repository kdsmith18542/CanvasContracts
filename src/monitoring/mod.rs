//! Production monitoring and observability system

use crate::{
    config::Config,
    error::{CanvasError, CanvasResult},
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

/// Metrics collector for production monitoring
pub struct MetricsCollector {
    config: Config,
    metrics: Arc<Mutex<MetricsStore>>,
    exporters: Vec<Box<dyn MetricsExporter>>,
    tx: mpsc::UnboundedSender<MetricEvent>,
}

/// Metrics store
#[derive(Debug, Clone)]
pub struct MetricsStore {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub histograms: HashMap<String, Vec<f64>>,
    pub timers: HashMap<String, Vec<Duration>>,
}

/// Metric event
#[derive(Debug, Clone)]
pub enum MetricEvent {
    IncrementCounter(String, u64),
    SetGauge(String, f64),
    RecordHistogram(String, f64),
    RecordTimer(String, Duration),
}

/// Metrics exporter trait
pub trait MetricsExporter: Send + Sync {
    fn export(&self, metrics: &MetricsStore) -> CanvasResult<()>;
    fn name(&self) -> &str;
}

/// Prometheus exporter
pub struct PrometheusExporter {
    endpoint: String,
}

/// InfluxDB exporter
pub struct InfluxDbExporter {
    url: String,
    database: String,
    token: String,
}

/// Performance profiler
pub struct PerformanceProfiler {
    config: Config,
    profiles: Arc<Mutex<HashMap<String, ProfileData>>>,
}

/// Profile data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileData {
    pub operation: String,
    pub duration: Duration,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub gas_consumed: u64,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

/// Health checker
pub struct HealthChecker {
    config: Config,
    checks: Vec<Box<dyn HealthCheck>>,
}

/// Health check trait
pub trait HealthCheck: Send + Sync {
    fn check(&self) -> CanvasResult<HealthStatus>;
    fn name(&self) -> &str;
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Circuit breaker for fault tolerance
pub struct CircuitBreaker {
    name: String,
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: Arc<Mutex<CircuitState>>,
}

/// Circuit state
#[derive(Debug, Clone)]
pub enum CircuitState {
    Closed { failure_count: u32 }, // Normal operation
    Open { opened_at: Instant, failure_count: u32 }, // Failing, reject requests
    HalfOpen,                      // Testing if recovered
}

/// Load balancer for distributed deployment
pub struct LoadBalancer {
    config: Config,
    nodes: Arc<Mutex<Vec<NodeInfo>>>,
    strategy: LoadBalancingStrategy,
    selection_counter: Arc<Mutex<usize>>,
}

/// Node information
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: String,
    pub url: String,
    pub health: HealthStatus,
    pub load: f64,
    pub last_seen: Instant,
}

/// Load balancing strategy
#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin(Vec<f64>),
    HealthBased,
}

/// Auto-scaling manager
pub struct AutoScalingManager {
    config: Config,
    metrics: Arc<Mutex<MetricsStore>>,
    scaling_rules: Vec<ScalingRule>,
    current_replicas: Arc<Mutex<u32>>,
    last_rule_execution: Arc<Mutex<HashMap<String, Instant>>>,
}

/// Scaling rule
#[derive(Debug, Clone)]
pub struct ScalingRule {
    pub name: String,
    pub metric: String,
    pub threshold: f64,
    pub action: ScalingAction,
    pub cooldown: Duration,
}

/// Scaling action
#[derive(Debug, Clone)]
pub enum ScalingAction {
    ScaleUp(u32),
    ScaleDown(u32),
    ScaleTo(u32),
}

fn read_process_usage() -> (u64, f64) {
    #[cfg(target_os = "linux")]
    {
        // RSS pages from /proc/self/statm converted to bytes.
        let memory_bytes = std::fs::read_to_string("/proc/self/statm")
            .ok()
            .and_then(|content| {
                let pages = content.split_whitespace().nth(1)?.parse::<u64>().ok()?;
                Some(pages.saturating_mul(4096))
            })
            .unwrap_or(0);

        // utime + stime ticks from /proc/self/stat, used as a monotonic CPU proxy.
        let cpu_ticks = std::fs::read_to_string("/proc/self/stat")
            .ok()
            .and_then(|content| {
                let fields: Vec<&str> = content.split_whitespace().collect();
                if fields.len() > 15 {
                    let utime = fields[13].parse::<f64>().ok()?;
                    let stime = fields[14].parse::<f64>().ok()?;
                    Some(utime + stime)
                } else {
                    None
                }
            })
            .unwrap_or(0.0);

        (memory_bytes, cpu_ticks)
    }

    #[cfg(not(target_os = "linux"))]
    {
        (0, 0.0)
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: &Config) -> CanvasResult<Self> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let metrics = Arc::new(Mutex::new(MetricsStore {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
            timers: HashMap::new(),
        }));

        let metrics_clone = metrics.clone();

        // Start metrics processing task
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let mut store = metrics_clone.lock().unwrap();
                match event {
                    MetricEvent::IncrementCounter(name, value) => {
                        *store.counters.entry(name).or_insert(0) += value;
                    }
                    MetricEvent::SetGauge(name, value) => {
                        store.gauges.insert(name, value);
                    }
                    MetricEvent::RecordHistogram(name, value) => {
                        store
                            .histograms
                            .entry(name)
                            .or_insert_with(Vec::new)
                            .push(value);
                    }
                    MetricEvent::RecordTimer(name, duration) => {
                        store
                            .timers
                            .entry(name)
                            .or_insert_with(Vec::new)
                            .push(duration);
                    }
                }
            }
        });

        Ok(Self {
            config: config.clone(),
            metrics,
            exporters: Vec::new(),
            tx,
        })
    }

    /// Increment a counter
    pub fn increment_counter(&self, name: &str, value: u64) -> CanvasResult<()> {
        self.tx
            .send(MetricEvent::IncrementCounter(name.to_string(), value))
            .map_err(|e| crate::error::CanvasError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Set a gauge
    pub fn set_gauge(&self, name: &str, value: f64) -> CanvasResult<()> {
        self.tx
            .send(MetricEvent::SetGauge(name.to_string(), value))
            .map_err(|e| crate::error::CanvasError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Record a histogram value
    pub fn record_histogram(&self, name: &str, value: f64) -> CanvasResult<()> {
        self.tx
            .send(MetricEvent::RecordHistogram(name.to_string(), value))
            .map_err(|e| crate::error::CanvasError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Record a timer
    pub fn record_timer(&self, name: &str, duration: Duration) -> CanvasResult<()> {
        self.tx
            .send(MetricEvent::RecordTimer(name.to_string(), duration))
            .map_err(|e| crate::error::CanvasError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Add an exporter
    pub fn add_exporter(&mut self, exporter: Box<dyn MetricsExporter>) {
        self.exporters.push(exporter);
    }

    /// Export metrics to all registered exporters
    pub fn export_metrics(&self) -> CanvasResult<()> {
        let metrics = self.metrics.lock().unwrap();
        log::debug!(
            "Exporting metrics with {} exporters at app log level {}",
            self.exporters.len(),
            self.config.app.log_level
        );

        for exporter in &self.exporters {
            if let Err(e) = exporter.export(&metrics) {
                log::error!("Failed to export metrics to {}: {}", exporter.name(), e);
            }
        }

        Ok(())
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> MetricsStore {
        self.metrics.lock().unwrap().clone()
    }
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
        }
    }
}

impl MetricsExporter for PrometheusExporter {
    fn export(&self, metrics: &MetricsStore) -> CanvasResult<()> {
        log::info!("Exporting metrics to Prometheus at {}", self.endpoint);

        // Format metrics in Prometheus format
        let mut prometheus_metrics = String::new();

        // Counters
        for (name, value) in &metrics.counters {
            prometheus_metrics.push_str(&format!("canvas_{} {}\n", name, value));
        }

        // Gauges
        for (name, value) in &metrics.gauges {
            prometheus_metrics.push_str(&format!("canvas_{} {}\n", name, value));
        }

        // Histograms
        for (name, values) in &metrics.histograms {
            if !values.is_empty() {
                let sum: f64 = values.iter().sum();
                let count = values.len() as f64;
                let avg = sum / count;
                prometheus_metrics.push_str(&format!("canvas_{}_sum {}\n", name, sum));
                prometheus_metrics.push_str(&format!("canvas_{}_count {}\n", name, count));
                prometheus_metrics.push_str(&format!("canvas_{}_avg {}\n", name, avg));
            }
        }

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&self.endpoint)
            .header("Content-Type", "text/plain; version=0.0.4")
            .body(prometheus_metrics)
            .send()
            .map_err(|e| {
                CanvasError::Network(format!("Failed to send metrics to Prometheus endpoint: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(CanvasError::Network(format!(
                "Prometheus endpoint returned non-success status {}",
                response.status()
            )));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "prometheus"
    }
}

impl InfluxDbExporter {
    /// Create a new InfluxDB exporter
    pub fn new(url: &str, database: &str, token: &str) -> Self {
        Self {
            url: url.to_string(),
            database: database.to_string(),
            token: token.to_string(),
        }
    }
}

impl MetricsExporter for InfluxDbExporter {
    fn export(&self, metrics: &MetricsStore) -> CanvasResult<()> {
        log::info!("Exporting metrics to InfluxDB at {}", self.url);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let mut influx_lines = Vec::new();

        // Counters
        for (name, value) in &metrics.counters {
            influx_lines.push(format!(
                "canvas_counters,metric={} value={} {}",
                name, value, timestamp
            ));
        }

        // Gauges
        for (name, value) in &metrics.gauges {
            influx_lines.push(format!(
                "canvas_gauges,metric={} value={} {}",
                name, value, timestamp
            ));
        }

        // Timers (milliseconds)
        for (name, values) in &metrics.timers {
            if values.is_empty() {
                continue;
            }
            let total_ms: f64 = values.iter().map(|d| d.as_secs_f64() * 1000.0).sum();
            let avg_ms = total_ms / values.len() as f64;
            influx_lines.push(format!(
                "canvas_timers,metric={} avg_ms={},count={} {}",
                name,
                avg_ms,
                values.len(),
                timestamp
            ));
        }

        if influx_lines.is_empty() {
            return Ok(());
        }

        let endpoint = format!(
            "{}/write?db={}",
            self.url.trim_end_matches('/'),
            self.database
        );
        let client = reqwest::blocking::Client::new();
        let mut request = client.post(endpoint).body(influx_lines.join("\n"));
        if !self.token.is_empty() {
            request = request.header("Authorization", format!("Token {}", self.token));
        }
        let response = request.send().map_err(|e| {
            CanvasError::Network(format!("Failed to send metrics to InfluxDB endpoint: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(CanvasError::Network(format!(
                "InfluxDB endpoint returned non-success status {}",
                response.status()
            )));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "influxdb"
    }
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            profiles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start profiling an operation
    pub fn start_profile(&self, operation: &str) -> ProfileHandle {
        let start_memory = self.get_memory_usage();
        let start_cpu = self.get_cpu_usage();
        if self.config.development.profiling {
            log::debug!("Starting profiler for operation '{}'", operation);
        }

        ProfileHandle {
            operation: operation.to_string(),
            start_time: Instant::now(),
            start_memory,
            start_cpu,
            profiler: self.profiles.clone(),
        }
    }

    /// Get memory usage
    fn get_memory_usage(&self) -> u64 {
        let (memory, _) = read_process_usage();
        memory
    }

    /// Get CPU usage
    fn get_cpu_usage(&self) -> f64 {
        let (_, cpu) = read_process_usage();
        cpu
    }

    /// Get profile data
    pub fn get_profiles(&self) -> HashMap<String, ProfileData> {
        self.profiles.lock().unwrap().clone()
    }

    /// Clear old profiles
    pub fn clear_old_profiles(&self, max_age: Duration) -> CanvasResult<()> {
        let mut profiles = self.profiles.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        profiles.retain(|_, profile| (now - profile.timestamp) < max_age.as_secs());

        Ok(())
    }
}

/// Profile handle for tracking operation performance
pub struct ProfileHandle {
    operation: String,
    start_time: Instant,
    start_memory: u64,
    start_cpu: f64,
    profiler: Arc<Mutex<HashMap<String, ProfileData>>>,
}

impl ProfileHandle {
    /// Finish profiling and record the data
    pub fn finish(self, gas_consumed: u64, metadata: HashMap<String, String>) -> CanvasResult<()> {
        let duration = self.start_time.elapsed();
        let (end_memory, end_cpu) = read_process_usage();

        let profile_data = ProfileData {
            operation: self.operation.clone(),
            duration,
            memory_usage: end_memory.saturating_sub(self.start_memory),
            cpu_usage: end_cpu - self.start_cpu,
            gas_consumed,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata,
        };

        let mut profiles = self.profiler.lock().unwrap();
        profiles.insert(self.operation, profile_data);

        Ok(())
    }
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            checks: Vec::new(),
        }
    }

    /// Add a health check
    pub fn add_check(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }

    /// Run all health checks
    pub fn check_health(&self) -> Vec<HealthCheckResult> {
        let mut results = Vec::new();
        if self.config.app.debug {
            log::debug!("Running {} health checks", self.checks.len());
        }

        for check in &self.checks {
            let result = HealthCheckResult {
                name: check.name().to_string(),
                status: check
                    .check()
                    .unwrap_or(HealthStatus::Unhealthy("Check failed".to_string())),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            results.push(result);
        }

        results
    }

    /// Get overall health status
    pub fn get_overall_health(&self) -> HealthStatus {
        let results = self.check_health();

        if results.is_empty() {
            return HealthStatus::Healthy;
        }

        let unhealthy_count = results
            .iter()
            .filter(|r| matches!(r.status, HealthStatus::Unhealthy(_)))
            .count();

        let degraded_count = results
            .iter()
            .filter(|r| matches!(r.status, HealthStatus::Degraded(_)))
            .count();

        if unhealthy_count > 0 {
            HealthStatus::Unhealthy(format!("{} unhealthy checks", unhealthy_count))
        } else if degraded_count > 0 {
            HealthStatus::Degraded(format!("{} degraded checks", degraded_count))
        } else {
            HealthStatus::Healthy
        }
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub name: String,
    pub status: HealthStatus,
    pub timestamp: u64,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(name: &str, failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            name: name.to_string(),
            failure_threshold,
            recovery_timeout,
            state: Arc::new(Mutex::new(CircuitState::Closed { failure_count: 0 })),
        }
    }

    /// Execute a function with circuit breaker protection
    pub fn execute<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Display,
    {
        let mut state_guard = self.state.lock().unwrap();

        if let CircuitState::Open {
            opened_at,
            failure_count,
        } = &*state_guard
        {
            if opened_at.elapsed() < self.recovery_timeout {
                return Err(CircuitBreakerError::CircuitOpen);
            }
            log::info!(
                "Circuit breaker '{}' moving to half-open after timeout (failures={})",
                self.name,
                failure_count
            );
            *state_guard = CircuitState::HalfOpen;
        }
        let current_state = state_guard.clone();
        drop(state_guard);

        match f() {
            Ok(result) => {
                let mut state = self.state.lock().unwrap();
                *state = CircuitState::Closed { failure_count: 0 };
                Ok(result)
            }
            Err(e) => {
                let mut state = self.state.lock().unwrap();
                match current_state {
                    CircuitState::HalfOpen => {
                        *state = CircuitState::Open {
                            opened_at: Instant::now(),
                            failure_count: self.failure_threshold,
                        };
                    }
                    CircuitState::Closed { failure_count } => {
                        let next = failure_count.saturating_add(1);
                        if next >= self.failure_threshold {
                            *state = CircuitState::Open {
                                opened_at: Instant::now(),
                                failure_count: next,
                            };
                        } else {
                            *state = CircuitState::Closed { failure_count: next };
                        }
                    }
                    CircuitState::Open { .. } => {}
                }
                log::warn!("Circuit breaker {}: operation failed: {}", self.name, e);
                Err(CircuitBreakerError::OperationFailed(e.to_string()))
            }
        }
    }

    /// Get current state
    pub fn get_state(&self) -> CircuitState {
        self.state.lock().unwrap().clone()
    }
}

/// Circuit breaker error
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open")]
    CircuitOpen,
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

impl LoadBalancer {
    /// Create a new load balancer
    pub fn new(config: &Config, strategy: LoadBalancingStrategy) -> Self {
        Self {
            config: config.clone(),
            nodes: Arc::new(Mutex::new(Vec::new())),
            strategy,
            selection_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Add a node
    pub fn add_node(&self, node: NodeInfo) -> CanvasResult<()> {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.push(node);
        Ok(())
    }

    /// Remove a node
    pub fn remove_node(&self, node_id: &str) -> CanvasResult<()> {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.retain(|n| n.id != node_id);
        Ok(())
    }

    /// Get next node based on strategy
    pub fn get_next_node(&self) -> Option<NodeInfo> {
        let mut nodes = self.nodes.lock().unwrap();
        if self.config.app.debug {
            log::debug!("Selecting node from {} candidates", nodes.len());
        }

        // Remove unhealthy nodes
        nodes.retain(|n| matches!(n.health, HealthStatus::Healthy));

        if nodes.is_empty() {
            return None;
        }

        match &self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                let mut counter = self.selection_counter.lock().unwrap();
                let index = *counter % nodes.len();
                *counter = counter.wrapping_add(1);
                nodes.get(index).cloned()
            }
            LoadBalancingStrategy::LeastConnections => {
                // Return node with lowest load
                nodes
                    .iter()
                    .min_by(|a, b| a.load.total_cmp(&b.load))
                    .cloned()
            }
            LoadBalancingStrategy::WeightedRoundRobin(weights) => {
                let normalized = (0..nodes.len())
                    .map(|index| weights.get(index).copied().unwrap_or(1.0).max(0.0))
                    .collect::<Vec<_>>();
                let total_weight: f64 = normalized.iter().sum();
                if total_weight <= f64::EPSILON {
                    return nodes.first().cloned();
                }
                let mut counter = self.selection_counter.lock().unwrap();
                let ticket = (*counter as f64) % total_weight;
                *counter = counter.wrapping_add(1);

                let mut cumulative = 0.0;
                for (index, weight) in normalized.iter().enumerate() {
                    cumulative += *weight;
                    if ticket < cumulative {
                        return nodes.get(index).cloned();
                    }
                }
                nodes.last().cloned()
            }
            LoadBalancingStrategy::HealthBased => {
                // Return healthiest node
                nodes.sort_by(|a, b| match (&a.health, &b.health) {
                    (HealthStatus::Healthy, HealthStatus::Healthy) => {
                        a.load.total_cmp(&b.load)
                    }
                    (HealthStatus::Healthy, _) => std::cmp::Ordering::Less,
                    (_, HealthStatus::Healthy) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                });
                nodes.first().cloned()
            }
        }
    }

    /// Update node health
    pub fn update_node_health(&self, node_id: &str, health: HealthStatus) -> CanvasResult<()> {
        let mut nodes = self.nodes.lock().unwrap();

        if let Some(node) = nodes.iter_mut().find(|n| n.id == node_id) {
            node.health = health;
            node.last_seen = Instant::now();
        }

        Ok(())
    }
}

impl AutoScalingManager {
    /// Create a new auto-scaling manager
    pub fn new(config: &Config, metrics: Arc<Mutex<MetricsStore>>) -> Self {
        Self {
            config: config.clone(),
            metrics,
            scaling_rules: Vec::new(),
            current_replicas: Arc::new(Mutex::new(1)),
            last_rule_execution: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a scaling rule
    pub fn add_rule(&mut self, rule: ScalingRule) {
        self.scaling_rules.push(rule);
    }

    /// Evaluate scaling rules
    pub fn evaluate_scaling(&self) -> CanvasResult<Vec<ScalingAction>> {
        let metrics = self.metrics.lock().unwrap();
        let now = Instant::now();
        let mut last_execution = self.last_rule_execution.lock().unwrap();
        let mut actions = Vec::new();

        for rule in &self.scaling_rules {
            if let Some(value) = metrics.gauges.get(&rule.metric) {
                let cooldown_ok = last_execution
                    .get(&rule.name)
                    .map_or(true, |last| now.duration_since(*last) >= rule.cooldown);
                if *value > rule.threshold && cooldown_ok {
                    actions.push(rule.action.clone());
                    last_execution.insert(rule.name.clone(), now);
                }
            }
        }

        Ok(actions)
    }

    /// Execute scaling actions
    pub fn execute_scaling(&self, actions: &[ScalingAction]) -> CanvasResult<()> {
        let mut replicas = self.current_replicas.lock().unwrap();
        for action in actions {
            match action {
                ScalingAction::ScaleUp(count) => {
                    log::info!("Scaling up by {} instances", count);
                    *replicas = replicas.saturating_add(*count);
                }
                ScalingAction::ScaleDown(count) => {
                    log::info!("Scaling down by {} instances", count);
                    *replicas = replicas.saturating_sub(*count).max(1);
                }
                ScalingAction::ScaleTo(count) => {
                    log::info!("Scaling to {} instances", count);
                    *replicas = (*count).max(1);
                }
            }
        }
        drop(replicas);

        if self.config.app.debug {
            log::debug!("Autoscaling actions executed: {}", actions.len());
        }

        Ok(())
    }

    pub fn current_replicas(&self) -> u32 {
        *self.current_replicas.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let config = Config::default();
        let collector = MetricsCollector::new(&config).unwrap();

        collector.increment_counter("test_counter", 1).unwrap();
        collector.set_gauge("test_gauge", 42.0).unwrap();
        collector.record_histogram("test_histogram", 10.5).unwrap();
        collector
            .record_timer("test_timer", Duration::from_millis(100))
            .unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;

        let metrics = collector.get_metrics();
        assert_eq!(metrics.counters.get("test_counter"), Some(&1));
        assert_eq!(metrics.gauges.get("test_gauge"), Some(&42.0));
    }

    #[test]
    fn test_performance_profiler() {
        let config = Config::default();
        let profiler = PerformanceProfiler::new(&config);

        let handle = profiler.start_profile("test_operation");
        std::thread::sleep(Duration::from_millis(10));

        let mut metadata = HashMap::new();
        metadata.insert("test_key".to_string(), "test_value".to_string());
        handle.finish(100, metadata).unwrap();

        let profiles = profiler.get_profiles();
        assert!(profiles.contains_key("test_operation"));
    }

    #[test]
    fn test_health_checker() {
        let config = Config::default();
        let mut checker = HealthChecker::new(&config);

        struct MockHealthCheck;
        impl HealthCheck for MockHealthCheck {
            fn check(&self) -> CanvasResult<HealthStatus> {
                Ok(HealthStatus::Healthy)
            }
            fn name(&self) -> &str {
                "mock_check"
            }
        }

        checker.add_check(Box::new(MockHealthCheck));

        let results = checker.check_health();
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, HealthStatus::Healthy));
    }

    #[test]
    fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new("test", 2, Duration::from_millis(10));

        // Test successful operation
        let result = breaker.execute(|| Ok::<i32, String>(42));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Two failures open the circuit.
        assert!(breaker
            .execute(|| Err::<i32, String>("test error 1".to_string()))
            .is_err());
        assert!(breaker
            .execute(|| Err::<i32, String>("test error 2".to_string()))
            .is_err());
        assert!(matches!(breaker.get_state(), CircuitState::Open { .. }));
    }

    #[test]
    fn test_load_balancer() {
        let config = Config::default();
        let balancer = LoadBalancer::new(&config, LoadBalancingStrategy::RoundRobin);

        let node = NodeInfo {
            id: "node1".to_string(),
            url: "http://localhost:8080".to_string(),
            health: HealthStatus::Healthy,
            load: 0.5,
            last_seen: Instant::now(),
        };

        balancer.add_node(node).unwrap();

        let next_node = balancer.get_next_node();
        assert!(next_node.is_some());
    }

    #[test]
    fn test_auto_scaling_manager_executes_actions() {
        let config = Config::default();
        let metrics = Arc::new(Mutex::new(MetricsStore {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
            timers: HashMap::new(),
        }));
        let manager = AutoScalingManager::new(&config, metrics);
        manager
            .execute_scaling(&[
                ScalingAction::ScaleUp(2),
                ScalingAction::ScaleDown(1),
                ScalingAction::ScaleTo(5),
            ])
            .unwrap();
        assert_eq!(manager.current_replicas(), 5);
    }
}
