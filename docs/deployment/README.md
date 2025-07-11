# Deployment Guide

This guide covers deploying Canvas Contracts to production environments, including local development, staging, and production deployments.

## Table of Contents

- [Deployment Overview](overview.md)
- [Local Development](local.md)
- [Staging Environment](staging.md)
- [Production Setup](production.md)
- [Blue-Green Deployment](blue-green.md)
- [Canary Deployment](canary.md)
- [Monitoring](monitoring.md)
- [Scaling](scaling.md)
- [Troubleshooting](troubleshooting.md)

## Quick Start

### Local Deployment

```bash
# Start local BaaLS node
canvas-contracts baals start

# Deploy contract
canvas-contracts deploy --contract my_contract.wasm --local
```

### Production Deployment

```bash
# Deploy with production config
canvas-contracts deploy \
  --contract my_contract.wasm \
  --config production.yaml \
  --replicas 3 \
  --auto-scale
```

## Deployment Strategies

### 1. Local Development

For development and testing:

```yaml
# deployment/local.yaml
environment: local
baals:
  node_url: "http://localhost:8080"
  enable_local_node: true
  local_node_port: 8080

deployment:
  replicas: 1
  resources:
    cpu_requests: "100m"
    cpu_limits: "500m"
    memory_requests: "128Mi"
    memory_limits: "512Mi"
  
monitoring:
  enabled: false
  metrics_endpoint: "/metrics"
```

### 2. Staging Environment

For pre-production testing:

```yaml
# deployment/staging.yaml
environment: staging
baals:
  node_url: "https://staging-baals.example.com"
  auth_token: "${BAALS_AUTH_TOKEN}"

deployment:
  replicas: 2
  resources:
    cpu_requests: "200m"
    cpu_limits: "1000m"
    memory_requests: "256Mi"
    memory_limits: "1Gi"
  
scaling:
  min_replicas: 1
  max_replicas: 5
  target_cpu_utilization: 70
  target_memory_utilization: 80

monitoring:
  enabled: true
  metrics_endpoint: "/metrics"
  alert_rules:
    - name: "high_error_rate"
      condition: "error_rate > 0.05"
      threshold: 0.05
      duration: 300
      severity: "warning"
```

### 3. Production Environment

For production workloads:

```yaml
# deployment/production.yaml
environment: production
baals:
  node_url: "https://production-baals.example.com"
  auth_token: "${BAALS_AUTH_TOKEN}"
  retry_attempts: 5
  connection_timeout: 30

deployment:
  replicas: 5
  resources:
    cpu_requests: "500m"
    cpu_limits: "2000m"
    memory_requests: "512Mi"
    memory_limits: "2Gi"
    storage_requests: "10Gi"
  
scaling:
  min_replicas: 3
  max_replicas: 20
  target_cpu_utilization: 70
  target_memory_utilization: 80
  scale_up_cooldown: 300
  scale_down_cooldown: 600

health_check:
  initial_delay_seconds: 30
  period_seconds: 10
  timeout_seconds: 5
  failure_threshold: 3
  success_threshold: 1
  health_check_path: "/health"

monitoring:
  enabled: true
  metrics_endpoint: "/metrics"
  log_level: "info"
  enable_tracing: true
  enable_profiling: false
  alert_rules:
    - name: "high_error_rate"
      condition: "error_rate > 0.01"
      threshold: 0.01
      duration: 60
      severity: "critical"
      notification:
        email: "alerts@example.com"
        webhook: "https://hooks.slack.com/..."
    
    - name: "high_latency"
      condition: "response_time > 1000"
      threshold: 1000
      duration: 300
      severity: "warning"

security:
  enable_tls: true
  certificate_path: "/etc/ssl/certs/contract.crt"
  key_path: "/etc/ssl/private/contract.key"
  allowed_origins:
    - "https://app.example.com"
    - "https://api.example.com"
  rate_limiting:
    requests_per_second: 1000
    burst_size: 100
    window_size: 60
```

## Deployment Commands

### Basic Deployment

```bash
# Deploy a contract
canvas-contracts deploy \
  --contract contract.wasm \
  --config deployment.yaml \
  --name "my-contract"

# Deploy with specific replicas
canvas-contracts deploy \
  --contract contract.wasm \
  --replicas 3 \
  --name "my-contract"

# Deploy with auto-scaling
canvas-contracts deploy \
  --contract contract.wasm \
  --auto-scale \
  --min-replicas 2 \
  --max-replicas 10 \
  --name "my-contract"
```

### Blue-Green Deployment

```bash
# Create blue-green deployment
canvas-contracts blue-green create \
  --name "my-contract" \
  --config deployment.yaml

# Deploy to blue environment
canvas-contracts blue-green deploy-blue \
  --name "my-contract" \
  --contract contract.wasm

# Deploy to green environment
canvas-contracts blue-green deploy-green \
  --name "my-contract" \
  --contract contract-v2.wasm

# Switch traffic to green
canvas-contracts blue-green switch-to-green \
  --name "my-contract"

# Rollback to blue if needed
canvas-contracts blue-green rollback \
  --name "my-contract"
```

### Canary Deployment

```bash
# Create canary deployment
canvas-contracts canary create \
  --name "my-contract" \
  --stable-contract stable.wasm \
  --canary-contract canary.wasm

# Update traffic split
canvas-contracts canary update-traffic \
  --name "my-contract" \
  --stable-percentage 90 \
  --canary-percentage 10

# Promote canary to stable
canvas-contracts canary promote \
  --name "my-contract"

# Rollback canary
canvas-contracts canary rollback \
  --name "my-contract"
```

## Monitoring and Observability

### Metrics Collection

```rust
use canvas_contracts::monitoring::{MetricsCollector, PrometheusExporter};

let metrics = MetricsCollector::new(&config)?;

// Add Prometheus exporter
let prometheus = PrometheusExporter::new("http://localhost:9090");
metrics.add_exporter(Box::new(prometheus));

// Record metrics
metrics.increment_counter("contract_calls", 1)?;
metrics.set_gauge("cpu_usage", 0.75)?;
metrics.record_histogram("response_time", 150.0)?;
metrics.record_timer("execution_time", Duration::from_millis(100))?;
```

### Health Checks

```rust
use canvas_contracts::monitoring::{HealthChecker, HealthStatus};

let mut health_checker = HealthChecker::new(&config)?;

// Add custom health checks
struct DatabaseHealthCheck;
impl HealthCheck for DatabaseHealthCheck {
    fn check(&self) -> CanvasResult<HealthStatus> {
        // Check database connectivity
        if database.is_connected() {
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Unhealthy("Database connection failed".to_string()))
        }
    }
    
    fn name(&self) -> &str {
        "database"
    }
}

health_checker.add_check(Box::new(DatabaseHealthCheck));

// Check overall health
let health = health_checker.get_overall_health();
println!("Health status: {:?}", health);
```

### Circuit Breakers

```rust
use canvas_contracts::monitoring::CircuitBreaker;

let breaker = CircuitBreaker::new(
    "external_api",
    5, // failure threshold
    Duration::from_secs(60) // recovery timeout
);

// Use circuit breaker for external calls
let result = breaker.execute(|| {
    external_api.call()
})?;
```

## Scaling

### Auto-Scaling

```rust
use canvas_contracts::deployment::{AutoScalingManager, ScalingRule, ScalingAction};

let scaling_manager = AutoScalingManager::new(&config, metrics.clone())?;

// Add scaling rules
scaling_manager.add_rule(ScalingRule {
    name: "cpu_scaling".to_string(),
    metric: "cpu_usage".to_string(),
    threshold: 0.8,
    action: ScalingAction::ScaleUp(2),
    cooldown: Duration::from_secs(300),
});

scaling_manager.add_rule(ScalingRule {
    name: "memory_scaling".to_string(),
    metric: "memory_usage".to_string(),
    threshold: 0.85,
    action: ScalingAction::ScaleUp(1),
    cooldown: Duration::from_secs(300),
});

// Evaluate and execute scaling
let actions = scaling_manager.evaluate_scaling()?;
scaling_manager.execute_scaling(&actions)?;
```

### Manual Scaling

```bash
# Scale up
canvas-contracts scale --name "my-contract" --replicas 5

# Scale down
canvas-contracts scale --name "my-contract" --replicas 2

# Scale to zero
canvas-contracts scale --name "my-contract" --replicas 0
```

## Infrastructure as Code

### Kubernetes Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: canvas-contract
  labels:
    app: canvas-contract
spec:
  replicas: 3
  selector:
    matchLabels:
      app: canvas-contract
  template:
    metadata:
      labels:
        app: canvas-contract
    spec:
      containers:
      - name: canvas-contract
        image: canvas-contracts:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        env:
        - name: BAALS_NODE_URL
          value: "https://production-baals.example.com"
        - name: BAALS_AUTH_TOKEN
          valueFrom:
            secretKeyRef:
              name: baals-secret
              key: auth-token
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: canvas-contract-service
spec:
  selector:
    app: canvas-contract
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: canvas-contract-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: canvas-contract
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  canvas-contract:
    image: canvas-contracts:latest
    ports:
      - "8080:8080"
    environment:
      - BAALS_NODE_URL=http://baals:8080
      - LOG_LEVEL=info
    depends_on:
      - baals
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
        reservations:
          cpus: '0.5'
          memory: 512M
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
        window: 120s

  baals:
    image: baals:latest
    ports:
      - "8081:8080"
    volumes:
      - baals-data:/data
    environment:
      - LOG_LEVEL=info

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-data:/var/lib/grafana

volumes:
  baals-data:
  prometheus-data:
  grafana-data:
```

## Security

### TLS Configuration

```yaml
# security/tls.yaml
tls:
  enabled: true
  certificate:
    path: "/etc/ssl/certs/contract.crt"
    key_path: "/etc/ssl/private/contract.key"
    ca_path: "/etc/ssl/certs/ca.crt"
  
  # Certificate renewal
  auto_renew: true
  renewal_threshold: 30 # days
  
  # Cipher suites
  cipher_suites:
    - "TLS_AES_256_GCM_SHA384"
    - "TLS_CHACHA20_POLY1305_SHA256"
    - "TLS_AES_128_GCM_SHA256"
```

### Rate Limiting

```rust
use canvas_contracts::deployment::RateLimitingConfig;

let rate_limiting = RateLimitingConfig {
    requests_per_second: 1000,
    burst_size: 100,
    window_size: 60,
};
```

### Access Control

```yaml
# security/access-control.yaml
access_control:
  allowed_origins:
    - "https://app.example.com"
    - "https://api.example.com"
  
  authentication:
    type: "jwt"
    issuer: "https://auth.example.com"
    audience: "canvas-contracts"
    
  authorization:
    roles:
      - name: "admin"
        permissions: ["read", "write", "deploy", "scale"]
      - name: "developer"
        permissions: ["read", "write", "deploy"]
      - name: "viewer"
        permissions: ["read"]
```

## Troubleshooting

### Common Issues

**Deployment fails**
```bash
# Check deployment status
canvas-contracts status --name "my-contract"

# View logs
canvas-contracts logs --name "my-contract"

# Check health
canvas-contracts health --name "my-contract"
```

**High resource usage**
```bash
# Check resource metrics
canvas-contracts metrics --name "my-contract"

# Scale up if needed
canvas-contracts scale --name "my-contract" --replicas 5
```

**Connection issues**
```bash
# Test BaaLS connection
canvas-contracts baals test --url "https://baals.example.com"

# Check network connectivity
canvas-contracts network test --endpoint "https://api.example.com"
```

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=debug
canvas-contracts deploy --contract contract.wasm --debug

# Enable profiling
canvas-contracts deploy --contract contract.wasm --profile
```

## Best Practices

### Performance

1. **Resource Planning**
   - Monitor resource usage patterns
   - Set appropriate limits and requests
   - Use auto-scaling for variable loads

2. **Caching**
   - Cache frequently accessed data
   - Use distributed caching for multiple replicas
   - Implement cache invalidation strategies

3. **Optimization**
   - Profile contract execution
   - Optimize gas usage
   - Use efficient algorithms

### Security

1. **Network Security**
   - Use TLS for all communications
   - Implement proper authentication
   - Restrict access with firewalls

2. **Data Protection**
   - Encrypt sensitive data
   - Use secure key management
   - Implement audit logging

3. **Access Control**
   - Use least privilege principle
   - Implement role-based access
   - Regular security audits

### Monitoring

1. **Metrics**
   - Monitor key performance indicators
   - Set up alerting for critical metrics
   - Use dashboards for visualization

2. **Logging**
   - Use structured logging
   - Implement log aggregation
   - Set up log retention policies

3. **Health Checks**
   - Implement comprehensive health checks
   - Use circuit breakers for external dependencies
   - Set up automated recovery procedures

## Next Steps

- **[Monitoring](monitoring.md)**: Set up comprehensive monitoring
- **[Scaling](scaling.md)**: Implement auto-scaling strategies
- **[Security](security.md)**: Secure your deployment
- **[Troubleshooting](troubleshooting.md)**: Common issues and solutions 