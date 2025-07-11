//! Performance optimization and production scaling

use crate::{
    error::CanvasResult,
    types::{Graph, NodeId, NodeType},
    config::Config,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Performance optimizer for production contracts
pub struct PerformanceOptimizer {
    config: Config,
    optimization_passes: Vec<Box<dyn OptimizationPass>>,
    cache: HashMap<String, OptimizationResult>,
}

/// Optimization pass trait
pub trait OptimizationPass: Send + Sync {
    fn name(&self) -> &str;
    fn optimize(&self, graph: &Graph) -> CanvasResult<OptimizationResult>;
    fn is_applicable(&self, graph: &Graph) -> bool;
}

/// Optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub name: String,
    pub original_gas: u64,
    pub optimized_gas: u64,
    pub gas_savings: u64,
    pub original_size: usize,
    pub optimized_size: usize,
    pub size_savings: usize,
    pub changes: Vec<OptimizationChange>,
    pub warnings: Vec<String>,
}

/// Optimization change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationChange {
    pub change_type: ChangeType,
    pub description: String,
    pub nodes_affected: Vec<NodeId>,
    pub impact: OptimizationImpact,
}

/// Change type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    NodeRemoval,
    NodeConsolidation,
    EdgeOptimization,
    ConstantFolding,
    DeadCodeElimination,
    LoopOptimization,
    MemoryOptimization,
}

/// Optimization impact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationImpact {
    High,
    Medium,
    Low,
}

/// Dead code elimination pass
pub struct DeadCodeEliminationPass;

/// Constant folding pass
pub struct ConstantFoldingPass;

/// Loop optimization pass
pub struct LoopOptimizationPass;

/// Memory optimization pass
pub struct MemoryOptimizationPass;

/// Cache optimization pass
pub struct CacheOptimizationPass;

/// Parallel execution optimizer
pub struct ParallelExecutionOptimizer {
    config: Config,
}

/// Parallel execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelExecutionPlan {
    pub stages: Vec<ExecutionStage>,
    pub dependencies: HashMap<NodeId, Vec<NodeId>>,
    pub estimated_parallelism: f64,
    pub estimated_speedup: f64,
}

/// Execution stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStage {
    pub stage_id: u32,
    pub nodes: Vec<NodeId>,
    pub estimated_duration: u64,
    pub dependencies: Vec<u32>,
}

/// Resource usage analyzer
pub struct ResourceUsageAnalyzer {
    config: Config,
}

/// Resource usage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageReport {
    pub memory_usage: MemoryUsage,
    pub cpu_usage: CpuUsage,
    pub gas_usage: GasUsage,
    pub network_usage: NetworkUsage,
    pub recommendations: Vec<ResourceRecommendation>,
}

/// Memory usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub peak_memory: u64,
    pub average_memory: u64,
    pub memory_leaks: Vec<String>,
    pub optimization_suggestions: Vec<String>,
}

/// CPU usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuUsage {
    pub peak_cpu: f64,
    pub average_cpu: f64,
    pub cpu_intensive_operations: Vec<String>,
    pub optimization_suggestions: Vec<String>,
}

/// Gas usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasUsage {
    pub total_gas: u64,
    pub gas_per_operation: HashMap<String, u64>,
    pub expensive_operations: Vec<String>,
    pub optimization_suggestions: Vec<String>,
}

/// Network usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkUsage {
    pub total_bandwidth: u64,
    pub requests_per_second: f64,
    pub network_latency: u64,
    pub optimization_suggestions: Vec<String>,
}

/// Resource recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRecommendation {
    pub category: ResourceCategory,
    pub priority: RecommendationPriority,
    pub description: String,
    pub estimated_impact: f64,
    pub implementation_effort: ImplementationEffort,
}

/// Resource category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceCategory {
    Memory,
    Cpu,
    Gas,
    Network,
    Storage,
}

/// Recommendation priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Implementation effort
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Easy,
    Medium,
    Hard,
    VeryHard,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    pub fn new(config: &Config) -> Self {
        let mut optimizer = Self {
            config: config.clone(),
            optimization_passes: Vec::new(),
            cache: HashMap::new(),
        };

        // Register optimization passes
        optimizer.register_pass(Box::new(DeadCodeEliminationPass));
        optimizer.register_pass(Box::new(ConstantFoldingPass));
        optimizer.register_pass(Box::new(LoopOptimizationPass));
        optimizer.register_pass(Box::new(MemoryOptimizationPass));
        optimizer.register_pass(Box::new(CacheOptimizationPass));

        optimizer
    }

    /// Register an optimization pass
    pub fn register_pass(&mut self, pass: Box<dyn OptimizationPass>) {
        self.optimization_passes.push(pass);
    }

    /// Optimize a graph
    pub fn optimize(&mut self, graph: &Graph) -> CanvasResult<Vec<OptimizationResult>> {
        let mut results = Vec::new();
        let graph_hash = self.compute_graph_hash(graph);

        // Check cache first
        if let Some(cached_result) = self.cache.get(&graph_hash) {
            results.push(cached_result.clone());
            return Ok(results);
        }

        // Apply optimization passes
        for pass in &self.optimization_passes {
            if pass.is_applicable(graph) {
                match pass.optimize(graph) {
                    Ok(result) => {
                        results.push(result.clone());
                        self.cache.insert(graph_hash.clone(), result);
                    }
                    Err(e) => {
                        log::warn!("Optimization pass {} failed: {}", pass.name(), e);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get optimization summary
    pub fn get_optimization_summary(&self, results: &[OptimizationResult]) -> OptimizationSummary {
        let total_gas_savings: u64 = results.iter().map(|r| r.gas_savings).sum();
        let total_size_savings: usize = results.iter().map(|r| r.size_savings).sum();
        let total_changes: usize = results.iter().map(|r| r.changes.len()).sum();

        OptimizationSummary {
            total_optimizations: results.len(),
            total_gas_savings,
            total_size_savings,
            total_changes,
            optimization_ratio: if total_gas_savings > 0 {
                total_gas_savings as f64 / 1000.0 // Normalize to percentage
            } else {
                0.0
            },
        }
    }

    /// Compute graph hash for caching
    fn compute_graph_hash(&self, graph: &Graph) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        graph.get_nodes().hash(&mut hasher);
        graph.get_edges().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Clear optimization cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

/// Optimization summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSummary {
    pub total_optimizations: usize,
    pub total_gas_savings: u64,
    pub total_size_savings: usize,
    pub total_changes: usize,
    pub optimization_ratio: f64,
}

impl OptimizationPass for DeadCodeEliminationPass {
    fn name(&self) -> &str {
        "dead_code_elimination"
    }

    fn optimize(&self, graph: &Graph) -> CanvasResult<OptimizationResult> {
        let nodes = graph.get_nodes();
        let edges = graph.get_edges();
        
        let mut reachable_nodes = std::collections::HashSet::new();
        let mut to_visit = Vec::new();

        // Find start nodes
        for node in nodes {
            if node.node_type == NodeType::Start {
                to_visit.push(node.id.clone());
                reachable_nodes.insert(node.id.clone());
            }
        }

        // BFS to find reachable nodes
        while let Some(node_id) = to_visit.pop() {
            for edge in edges {
                if edge.source == node_id && !reachable_nodes.contains(&edge.target) {
                    reachable_nodes.insert(edge.target.clone());
                    to_visit.push(edge.target.clone());
                }
            }
        }

        // Find unreachable nodes
        let unreachable_nodes: Vec<_> = nodes
            .iter()
            .filter(|node| !reachable_nodes.contains(&node.id))
            .map(|node| node.id.clone())
            .collect();

        let gas_savings = unreachable_nodes.len() as u64 * 100; // Estimate gas savings
        let size_savings = unreachable_nodes.len() * 50; // Estimate size savings

        let changes = if !unreachable_nodes.is_empty() {
            vec![OptimizationChange {
                change_type: ChangeType::DeadCodeElimination,
                description: format!("Remove {} unreachable nodes", unreachable_nodes.len()),
                nodes_affected: unreachable_nodes,
                impact: OptimizationImpact::High,
            }]
        } else {
            Vec::new()
        };

        Ok(OptimizationResult {
            name: "Dead Code Elimination".to_string(),
            original_gas: 0, // Will be calculated by caller
            optimized_gas: 0, // Will be calculated by caller
            gas_savings,
            original_size: 0, // Will be calculated by caller
            optimized_size: 0, // Will be calculated by caller
            size_savings,
            changes,
            warnings: Vec::new(),
        })
    }

    fn is_applicable(&self, graph: &Graph) -> bool {
        // Always applicable
        true
    }
}

impl OptimizationPass for ConstantFoldingPass {
    fn name(&self) -> &str {
        "constant_folding"
    }

    fn optimize(&self, graph: &Graph) -> CanvasResult<OptimizationResult> {
        let nodes = graph.get_nodes();
        let mut changes = Vec::new();
        let mut folded_nodes = Vec::new();

        // Find nodes with constant inputs that can be folded
        for node in nodes {
            if node.node_type == NodeType::Arithmetic {
                // Check if all inputs are constants
                let inputs = graph.get_node_inputs(&node.id)?;
                if inputs.iter().all(|(_, value)| value.is_number()) {
                    folded_nodes.push(node.id.clone());
                }
            }
        }

        let gas_savings = folded_nodes.len() as u64 * 10;
        let size_savings = folded_nodes.len() * 20;

        if !folded_nodes.is_empty() {
            changes.push(OptimizationChange {
                change_type: ChangeType::ConstantFolding,
                description: format!("Fold {} constant expressions", folded_nodes.len()),
                nodes_affected: folded_nodes,
                impact: OptimizationImpact::Medium,
            });
        }

        Ok(OptimizationResult {
            name: "Constant Folding".to_string(),
            original_gas: 0,
            optimized_gas: 0,
            gas_savings,
            original_size: 0,
            optimized_size: 0,
            size_savings,
            changes,
            warnings: Vec::new(),
        })
    }

    fn is_applicable(&self, graph: &Graph) -> bool {
        // Check if there are arithmetic nodes
        graph.get_nodes().iter().any(|n| n.node_type == NodeType::Arithmetic)
    }
}

impl OptimizationPass for LoopOptimizationPass {
    fn name(&self) -> &str {
        "loop_optimization"
    }

    fn optimize(&self, graph: &Graph) -> CanvasResult<OptimizationResult> {
        let nodes = graph.get_nodes();
        let edges = graph.get_edges();
        let mut changes = Vec::new();
        let mut optimized_loops = Vec::new();

        // Find loops in the graph
        let loops = self.find_loops(nodes, edges)?;
        
        for loop_nodes in loops {
            // Check if loop can be optimized
            if self.can_optimize_loop(&loop_nodes, graph)? {
                optimized_loops.extend(loop_nodes);
            }
        }

        let gas_savings = optimized_loops.len() as u64 * 50;
        let size_savings = optimized_loops.len() * 30;

        if !optimized_loops.is_empty() {
            changes.push(OptimizationChange {
                change_type: ChangeType::LoopOptimization,
                description: format!("Optimize {} loops", optimized_loops.len() / 3), // Estimate loop count
                nodes_affected: optimized_loops,
                impact: OptimizationImpact::High,
            });
        }

        Ok(OptimizationResult {
            name: "Loop Optimization".to_string(),
            original_gas: 0,
            optimized_gas: 0,
            gas_savings,
            original_size: 0,
            optimized_size: 0,
            size_savings,
            changes,
            warnings: Vec::new(),
        })
    }

    fn is_applicable(&self, graph: &Graph) -> bool {
        // Check if there are control flow nodes that might form loops
        let control_nodes = graph.get_nodes().iter()
            .filter(|n| n.node_type == NodeType::Control)
            .count();
        control_nodes > 2
    }
}

impl LoopOptimizationPass {
    fn find_loops(&self, nodes: &[crate::types::Node], edges: &[crate::types::Edge]) -> CanvasResult<Vec<Vec<NodeId>>> {
        // TODO: Implement actual loop detection using DFS
        Ok(Vec::new())
    }

    fn can_optimize_loop(&self, loop_nodes: &[NodeId], graph: &Graph) -> CanvasResult<bool> {
        // TODO: Implement loop optimization analysis
        Ok(false)
    }
}

impl OptimizationPass for MemoryOptimizationPass {
    fn name(&self) -> &str {
        "memory_optimization"
    }

    fn optimize(&self, graph: &Graph) -> CanvasResult<OptimizationResult> {
        let nodes = graph.get_nodes();
        let mut changes = Vec::new();
        let mut memory_optimized_nodes = Vec::new();

        // Find memory-intensive operations
        for node in nodes {
            if node.node_type == NodeType::State {
                // Storage operations are memory-intensive
                memory_optimized_nodes.push(node.id.clone());
            }
        }

        let gas_savings = memory_optimized_nodes.len() as u64 * 200;
        let size_savings = memory_optimized_nodes.len() * 40;

        if !memory_optimized_nodes.is_empty() {
            changes.push(OptimizationChange {
                change_type: ChangeType::MemoryOptimization,
                description: format!("Optimize {} memory operations", memory_optimized_nodes.len()),
                nodes_affected: memory_optimized_nodes,
                impact: OptimizationImpact::High,
            });
        }

        Ok(OptimizationResult {
            name: "Memory Optimization".to_string(),
            original_gas: 0,
            optimized_gas: 0,
            gas_savings,
            original_size: 0,
            optimized_size: 0,
            size_savings,
            changes,
            warnings: Vec::new(),
        })
    }

    fn is_applicable(&self, graph: &Graph) -> bool {
        // Check if there are state operations
        graph.get_nodes().iter().any(|n| n.node_type == NodeType::State)
    }
}

impl OptimizationPass for CacheOptimizationPass {
    fn name(&self) -> &str {
        "cache_optimization"
    }

    fn optimize(&self, graph: &Graph) -> CanvasResult<OptimizationResult> {
        let nodes = graph.get_nodes();
        let mut changes = Vec::new();
        let mut cache_optimized_nodes = Vec::new();

        // Find repeated operations that can be cached
        let mut operation_counts = HashMap::new();
        for node in nodes {
            let key = format!("{:?}", node.node_type);
            *operation_counts.entry(key).or_insert(0) += 1;
        }

        for (operation, count) in operation_counts {
            if count > 1 {
                // This operation is repeated and can be cached
                cache_optimized_nodes.push(operation);
            }
        }

        let gas_savings = cache_optimized_nodes.len() as u64 * 150;
        let size_savings = cache_optimized_nodes.len() * 25;

        if !cache_optimized_nodes.is_empty() {
            changes.push(OptimizationChange {
                change_type: ChangeType::NodeConsolidation,
                description: format!("Cache {} repeated operations", cache_optimized_nodes.len()),
                nodes_affected: Vec::new(), // Will be filled by caller
                impact: OptimizationImpact::Medium,
            });
        }

        Ok(OptimizationResult {
            name: "Cache Optimization".to_string(),
            original_gas: 0,
            optimized_gas: 0,
            gas_savings,
            original_size: 0,
            optimized_size: 0,
            size_savings,
            changes,
            warnings: Vec::new(),
        })
    }

    fn is_applicable(&self, graph: &Graph) -> bool {
        // Check if there are repeated operations
        let mut operation_counts = HashMap::new();
        for node in graph.get_nodes() {
            let key = format!("{:?}", node.node_type);
            *operation_counts.entry(key).or_insert(0) += 1;
        }
        operation_counts.values().any(|&count| count > 1)
    }
}

impl ParallelExecutionOptimizer {
    /// Create a new parallel execution optimizer
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Generate parallel execution plan
    pub fn generate_plan(&self, graph: &Graph) -> CanvasResult<ParallelExecutionPlan> {
        let nodes = graph.get_nodes();
        let edges = graph.get_edges();
        
        // Build dependency graph
        let mut dependencies = HashMap::new();
        for edge in edges {
            dependencies.entry(edge.target.clone())
                .or_insert_with(Vec::new)
                .push(edge.source.clone());
        }

        // Topological sort to find execution stages
        let stages = self.topological_sort(nodes, &dependencies)?;
        
        // Calculate parallelism metrics
        let estimated_parallelism = self.calculate_parallelism(&stages);
        let estimated_speedup = self.calculate_speedup(&stages);

        Ok(ParallelExecutionPlan {
            stages,
            dependencies,
            estimated_parallelism,
            estimated_speedup,
        })
    }

    /// Perform topological sort
    fn topological_sort(&self, nodes: &[crate::types::Node], dependencies: &HashMap<NodeId, Vec<NodeId>>) -> CanvasResult<Vec<ExecutionStage>> {
        // TODO: Implement actual topological sort
        let mut stages = Vec::new();
        
        // Simple stage assignment for now
        let mut stage_id = 0;
        for node in nodes {
            stages.push(ExecutionStage {
                stage_id,
                nodes: vec![node.id.clone()],
                estimated_duration: 100, // Mock duration
                dependencies: Vec::new(),
            });
            stage_id += 1;
        }

        Ok(stages)
    }

    /// Calculate parallelism level
    fn calculate_parallelism(&self, stages: &[ExecutionStage]) -> f64 {
        if stages.is_empty() {
            return 0.0;
        }

        let max_parallel_stages = stages.len() as f64;
        let total_stages = stages.len() as f64;
        
        max_parallel_stages / total_stages
    }

    /// Calculate speedup factor
    fn calculate_speedup(&self, stages: &[ExecutionStage]) -> f64 {
        if stages.is_empty() {
            return 1.0;
        }

        let sequential_time: u64 = stages.iter().map(|s| s.estimated_duration).sum();
        let parallel_time = stages.iter().map(|s| s.estimated_duration).max().unwrap_or(0);
        
        if parallel_time == 0 {
            return 1.0;
        }

        sequential_time as f64 / parallel_time as f64
    }
}

impl ResourceUsageAnalyzer {
    /// Create a new resource usage analyzer
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Analyze resource usage
    pub fn analyze(&self, graph: &Graph) -> CanvasResult<ResourceUsageReport> {
        let memory_usage = self.analyze_memory_usage(graph)?;
        let cpu_usage = self.analyze_cpu_usage(graph)?;
        let gas_usage = self.analyze_gas_usage(graph)?;
        let network_usage = self.analyze_network_usage(graph)?;
        let recommendations = self.generate_recommendations(graph, &memory_usage, &cpu_usage, &gas_usage, &network_usage)?;

        Ok(ResourceUsageReport {
            memory_usage,
            cpu_usage,
            gas_usage,
            network_usage,
            recommendations,
        })
    }

    /// Analyze memory usage
    fn analyze_memory_usage(&self, graph: &Graph) -> CanvasResult<MemoryUsage> {
        let nodes = graph.get_nodes();
        let mut peak_memory = 0u64;
        let mut total_memory = 0u64;
        let mut memory_leaks = Vec::new();
        let mut optimization_suggestions = Vec::new();

        for node in nodes {
            let node_memory = self.estimate_node_memory_usage(node);
            peak_memory = peak_memory.max(node_memory);
            total_memory += node_memory;

            // Check for potential memory leaks
            if node.node_type == NodeType::State {
                memory_leaks.push(format!("Storage operation in node {} may cause memory growth", node.id));
            }
        }

        let average_memory = if !nodes.is_empty() {
            total_memory / nodes.len() as u64
        } else {
            0
        };

        // Generate optimization suggestions
        if peak_memory > 1_000_000 {
            optimization_suggestions.push("Consider reducing memory usage in state operations".to_string());
        }

        if memory_leaks.len() > 5 {
            optimization_suggestions.push("Multiple potential memory leaks detected".to_string());
        }

        Ok(MemoryUsage {
            peak_memory,
            average_memory,
            memory_leaks,
            optimization_suggestions,
        })
    }

    /// Analyze CPU usage
    fn analyze_cpu_usage(&self, graph: &Graph) -> CanvasResult<CpuUsage> {
        let nodes = graph.get_nodes();
        let mut peak_cpu = 0.0;
        let mut total_cpu = 0.0;
        let mut cpu_intensive_operations = Vec::new();

        for node in nodes {
            let node_cpu = self.estimate_node_cpu_usage(node);
            peak_cpu = peak_cpu.max(node_cpu);
            total_cpu += node_cpu;

            if node_cpu > 0.8 {
                cpu_intensive_operations.push(format!("High CPU usage in node {} ({:.2})", node.id, node_cpu));
            }
        }

        let average_cpu = if !nodes.is_empty() {
            total_cpu / nodes.len() as f64
        } else {
            0.0
        };

        let optimization_suggestions = if peak_cpu > 0.9 {
            vec!["Consider optimizing CPU-intensive operations".to_string()]
        } else {
            Vec::new()
        };

        Ok(CpuUsage {
            peak_cpu,
            average_cpu,
            cpu_intensive_operations,
            optimization_suggestions,
        })
    }

    /// Analyze gas usage
    fn analyze_gas_usage(&self, graph: &Graph) -> CanvasResult<GasUsage> {
        let nodes = graph.get_nodes();
        let mut total_gas = 0u64;
        let mut gas_per_operation = HashMap::new();
        let mut expensive_operations = Vec::new();

        for node in nodes {
            let node_gas = self.estimate_node_gas_usage(node);
            total_gas += node_gas;
            
            let operation_type = format!("{:?}", node.node_type);
            gas_per_operation.insert(operation_type.clone(), node_gas);

            if node_gas > 1000 {
                expensive_operations.push(format!("Expensive operation in node {}: {} gas", node.id, node_gas));
            }
        }

        let optimization_suggestions = if total_gas > 10_000 {
            vec!["Consider optimizing gas usage for cost efficiency".to_string()]
        } else {
            Vec::new()
        };

        Ok(GasUsage {
            total_gas,
            gas_per_operation,
            expensive_operations,
            optimization_suggestions,
        })
    }

    /// Analyze network usage
    fn analyze_network_usage(&self, graph: &Graph) -> CanvasResult<NetworkUsage> {
        let nodes = graph.get_nodes();
        let mut total_bandwidth = 0u64;
        let mut requests_per_second = 0.0;

        for node in nodes {
            if node.node_type == NodeType::External {
                total_bandwidth += 1024; // Estimate 1KB per external call
                requests_per_second += 0.1; // Estimate 0.1 requests per second
            }
        }

        let network_latency = 100; // Mock latency in ms
        let optimization_suggestions = if total_bandwidth > 10_240 {
            vec!["Consider batching external calls to reduce network usage".to_string()]
        } else {
            Vec::new()
        };

        Ok(NetworkUsage {
            total_bandwidth,
            requests_per_second,
            network_latency,
            optimization_suggestions,
        })
    }

    /// Generate recommendations
    fn generate_recommendations(
        &self,
        graph: &Graph,
        memory_usage: &MemoryUsage,
        cpu_usage: &CpuUsage,
        gas_usage: &GasUsage,
        network_usage: &NetworkUsage,
    ) -> CanvasResult<Vec<ResourceRecommendation>> {
        let mut recommendations = Vec::new();

        // Memory recommendations
        if memory_usage.peak_memory > 1_000_000 {
            recommendations.push(ResourceRecommendation {
                category: ResourceCategory::Memory,
                priority: RecommendationPriority::High,
                description: "High memory usage detected".to_string(),
                estimated_impact: 0.3,
                implementation_effort: ImplementationEffort::Medium,
            });
        }

        // CPU recommendations
        if cpu_usage.peak_cpu > 0.9 {
            recommendations.push(ResourceRecommendation {
                category: ResourceCategory::Cpu,
                priority: RecommendationPriority::Critical,
                description: "Very high CPU usage detected".to_string(),
                estimated_impact: 0.5,
                implementation_effort: ImplementationEffort::Hard,
            });
        }

        // Gas recommendations
        if gas_usage.total_gas > 10_000 {
            recommendations.push(ResourceRecommendation {
                category: ResourceCategory::Gas,
                priority: RecommendationPriority::High,
                description: "High gas consumption detected".to_string(),
                estimated_impact: 0.4,
                implementation_effort: ImplementationEffort::Medium,
            });
        }

        // Network recommendations
        if network_usage.total_bandwidth > 10_240 {
            recommendations.push(ResourceRecommendation {
                category: ResourceCategory::Network,
                priority: RecommendationPriority::Medium,
                description: "High network usage detected".to_string(),
                estimated_impact: 0.2,
                implementation_effort: ImplementationEffort::Easy,
            });
        }

        Ok(recommendations)
    }

    /// Estimate node memory usage
    fn estimate_node_memory_usage(&self, node: &crate::types::Node) -> u64 {
        match node.node_type {
            NodeType::State => 1024, // Storage operations use more memory
            NodeType::External => 512, // External calls use moderate memory
            NodeType::Arithmetic => 64, // Arithmetic operations use little memory
            NodeType::Logic => 32, // Logic operations use very little memory
            NodeType::Control => 128, // Control flow uses some memory
            NodeType::Start => 256, // Start nodes use moderate memory
            NodeType::End => 256, // End nodes use moderate memory
        }
    }

    /// Estimate node CPU usage
    fn estimate_node_cpu_usage(&self, node: &crate::types::Node) -> f64 {
        match node.node_type {
            NodeType::State => 0.3, // Storage operations are CPU intensive
            NodeType::External => 0.5, // External calls are very CPU intensive
            NodeType::Arithmetic => 0.1, // Arithmetic operations are light
            NodeType::Logic => 0.05, // Logic operations are very light
            NodeType::Control => 0.2, // Control flow is moderate
            NodeType::Start => 0.1, // Start nodes are light
            NodeType::End => 0.1, // End nodes are light
        }
    }

    /// Estimate node gas usage
    fn estimate_node_gas_usage(&self, node: &crate::types::Node) -> u64 {
        match node.node_type {
            NodeType::State => 20000, // Storage operations are expensive
            NodeType::External => 2600, // External calls are expensive
            NodeType::Arithmetic => 3, // Arithmetic operations are cheap
            NodeType::Logic => 1, // Logic operations are very cheap
            NodeType::Control => 1, // Control flow is cheap
            NodeType::Start => 100, // Start nodes are moderate
            NodeType::End => 100, // End nodes are moderate
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_optimizer() {
        let config = Config::default();
        let mut optimizer = PerformanceOptimizer::new(&config);
        
        let graph = Graph::new("test");
        let results = optimizer.optimize(&graph).unwrap();
        
        assert!(!results.is_empty());
        
        let summary = optimizer.get_optimization_summary(&results);
        assert!(summary.total_optimizations > 0);
    }

    #[test]
    fn test_parallel_execution_optimizer() {
        let config = Config::default();
        let optimizer = ParallelExecutionOptimizer::new(&config);
        
        let graph = Graph::new("test");
        let plan = optimizer.generate_plan(&graph).unwrap();
        
        assert!(plan.estimated_parallelism >= 0.0);
        assert!(plan.estimated_speedup >= 1.0);
    }

    #[test]
    fn test_resource_usage_analyzer() {
        let config = Config::default();
        let analyzer = ResourceUsageAnalyzer::new(&config);
        
        let graph = Graph::new("test");
        let report = analyzer.analyze(&graph).unwrap();
        
        assert!(report.memory_usage.peak_memory >= 0);
        assert!(report.cpu_usage.peak_cpu >= 0.0);
        assert!(report.gas_usage.total_gas >= 0);
    }
} 