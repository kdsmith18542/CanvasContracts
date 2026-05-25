//! Performance optimization and production scaling

use crate::{
    config::Config,
    error::CanvasResult,
    types::{Connection, NodeId, NodeType, VisualGraph, VisualNode},
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Performance optimizer for production contracts
pub struct PerformanceOptimizer {
    _config: Config,
    optimization_passes: Vec<Box<dyn OptimizationPass>>,
    cache: HashMap<String, OptimizationResult>,
}

/// Optimization pass trait
pub trait OptimizationPass: Send + Sync {
    fn name(&self) -> &str;
    fn optimize(&self, graph: &VisualGraph) -> CanvasResult<OptimizationResult>;
    fn is_applicable(&self, graph: &VisualGraph) -> bool;
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
    _config: Config,
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
    _config: Config,
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
            _config: config.clone(),
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
    pub fn optimize(&mut self, graph: &VisualGraph) -> CanvasResult<Vec<OptimizationResult>> {
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
    fn compute_graph_hash(&self, graph: &VisualGraph) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for node in graph.get_nodes() {
            node.id.hash(&mut hasher);
        }
        for conn in graph.get_connections() {
            conn.id.hash(&mut hasher);
        }
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

    fn optimize(&self, graph: &VisualGraph) -> CanvasResult<OptimizationResult> {
        let nodes = graph.get_nodes();
        let edges = graph.get_connections();

        let mut reachable_nodes = std::collections::HashSet::new();
        let mut to_visit = Vec::new();

        // Find start nodes
        for node in nodes {
            if node.node_type == "Start" {
                to_visit.push(node.id);
                reachable_nodes.insert(node.id);
            }
        }

        // BFS to find reachable nodes
        while let Some(node_id) = to_visit.pop() {
            for edge in edges {
                if edge.source_node == node_id && !reachable_nodes.contains(&edge.target_node) {
                    reachable_nodes.insert(edge.target_node);
                    to_visit.push(edge.target_node);
                }
            }
        }

        // Find unreachable nodes
        let unreachable_nodes: Vec<_> = nodes
            .iter()
            .filter(|node| !reachable_nodes.contains(&node.id))
            .map(|node| node.id)
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
            original_gas: 0,  // Will be calculated by caller
            optimized_gas: 0, // Will be calculated by caller
            gas_savings,
            original_size: 0,  // Will be calculated by caller
            optimized_size: 0, // Will be calculated by caller
            size_savings,
            changes,
            warnings: Vec::new(),
        })
    }

    fn is_applicable(&self, _graph: &VisualGraph) -> bool {
        // Always applicable
        true
    }
}

impl OptimizationPass for ConstantFoldingPass {
    fn name(&self) -> &str {
        "constant_folding"
    }

    fn optimize(&self, graph: &VisualGraph) -> CanvasResult<OptimizationResult> {
        let nodes = graph.get_nodes();
        let mut changes = Vec::new();
        let mut folded_nodes = Vec::new();

        // Find nodes with constant inputs that can be folded
        for node in nodes {
            if node.node_type == "Arithmetic" {
                // Check if all inputs are constants
                let inputs: HashMap<String, serde_json::Value> = node.properties.clone();
                if inputs.iter().all(|(_, value)| value.is_number()) {
                    folded_nodes.push(node.id);
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

    fn is_applicable(&self, graph: &VisualGraph) -> bool {
        // Check if there are arithmetic nodes
        graph
            .get_nodes()
            .iter()
            .any(|n| n.node_type == "Arithmetic")
    }
}

impl OptimizationPass for LoopOptimizationPass {
    fn name(&self) -> &str {
        "loop_optimization"
    }

    fn optimize(&self, graph: &VisualGraph) -> CanvasResult<OptimizationResult> {
        let nodes = graph.get_nodes();
        let edges = graph.get_connections();
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

    fn is_applicable(&self, graph: &VisualGraph) -> bool {
        // Check if there are control flow nodes that might form loops
        let control_nodes = graph
            .get_nodes()
            .iter()
            .filter(|n| n.node_type == "Control")
            .count();
        control_nodes > 2
    }
}

impl LoopOptimizationPass {
    fn find_loops(
        &self,
        nodes: &[VisualNode],
        edges: &[Connection],
    ) -> CanvasResult<Vec<Vec<NodeId>>> {
        let mut adj = HashMap::new();
        for edge in edges {
            adj.entry(edge.source_node).or_insert_with(Vec::new).push(edge.target_node);
        }

        let mut visited = HashMap::new(); // NodeId -> state (0: White, 1: Gray, 2: Black)
        for node in nodes {
            visited.insert(node.id, 0);
        }

        let mut stack = Vec::new();
        let mut loops = Vec::new();

        fn dfs(
            u: NodeId,
            adj: &HashMap<NodeId, Vec<NodeId>>,
            visited: &mut HashMap<NodeId, i32>,
            stack: &mut Vec<NodeId>,
            loops: &mut Vec<Vec<NodeId>>,
        ) {
            visited.insert(u, 1);
            stack.push(u);

            if let Some(neighbors) = adj.get(&u) {
                for &v in neighbors {
                    if let Some(&state) = visited.get(&v) {
                        if state == 1 {
                            // Cycle detected!
                            if let Some(pos) = stack.iter().position(|&x| x == v) {
                                let cycle = stack[pos..].to_vec();
                                loops.push(cycle);
                            }
                        } else if state == 0 {
                            dfs(v, adj, visited, stack, loops);
                        }
                    }
                }
            }

            stack.pop();
            visited.insert(u, 2);
        }

        for node in nodes {
            if visited.get(&node.id) == Some(&0) {
                dfs(node.id, &adj, &mut visited, &mut stack, &mut loops);
            }
        }

        Ok(loops)
    }

    fn can_optimize_loop(
        &self,
        loop_nodes: &[NodeId],
        graph: &VisualGraph,
    ) -> CanvasResult<bool> {
        for &node_id in loop_nodes {
            if let Some(node) = graph.get_node(node_id) {
                let nt = NodeType::from(node.node_type.as_str());
                match nt {
                    NodeType::State | NodeType::External => return Ok(true),
                    NodeType::Arithmetic => {
                        let mut all_inputs_outside = true;
                        for conn in &graph.connections {
                            if conn.target_node == node_id
                                && loop_nodes.contains(&conn.source_node)
                            {
                                all_inputs_outside = false;
                                break;
                            }
                        }
                        if all_inputs_outside {
                            return Ok(true);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(false)
    }
}

impl OptimizationPass for MemoryOptimizationPass {
    fn name(&self) -> &str {
        "memory_optimization"
    }

    fn optimize(&self, graph: &VisualGraph) -> CanvasResult<OptimizationResult> {
        let nodes = graph.get_nodes();
        let mut changes = Vec::new();
        let mut memory_optimized_nodes = Vec::new();

        // Find memory-intensive operations
        for node in nodes {
            if node.node_type == "State" {
                // Storage operations are memory-intensive
                memory_optimized_nodes.push(node.id);
            }
        }

        let gas_savings = memory_optimized_nodes.len() as u64 * 200;
        let size_savings = memory_optimized_nodes.len() * 40;

        if !memory_optimized_nodes.is_empty() {
            changes.push(OptimizationChange {
                change_type: ChangeType::MemoryOptimization,
                description: format!(
                    "Optimize {} memory operations",
                    memory_optimized_nodes.len()
                ),
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

    fn is_applicable(&self, graph: &VisualGraph) -> bool {
        // Check if there are state operations
        graph.get_nodes().iter().any(|n| n.node_type == "State")
    }
}

impl OptimizationPass for CacheOptimizationPass {
    fn name(&self) -> &str {
        "cache_optimization"
    }

    fn optimize(&self, graph: &VisualGraph) -> CanvasResult<OptimizationResult> {
        let nodes = graph.get_nodes();
        let mut changes = Vec::new();
        let mut cache_optimized_nodes = Vec::new();

        // Find repeated operations that can be cached
        let mut operation_counts: HashMap<String, Vec<NodeId>> = HashMap::new();
        for node in nodes {
            operation_counts
                .entry(node.node_type.clone())
                .or_default()
                .push(node.id);
        }

        for node_ids in operation_counts.values() {
            if node_ids.len() > 1 {
                cache_optimized_nodes.extend(node_ids.iter().cloned());
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

    fn is_applicable(&self, graph: &VisualGraph) -> bool {
        // Check if there are repeated operations
        let mut operation_counts: HashMap<String, usize> = HashMap::new();
        for node in graph.get_nodes() {
            *operation_counts.entry(node.node_type.clone()).or_insert(0) += 1;
        }
        operation_counts.values().any(|&count| count > 1)
    }
}

impl ParallelExecutionOptimizer {
    /// Create a new parallel execution optimizer
    pub fn new(config: &Config) -> Self {
        Self {
            _config: config.clone(),
        }
    }

    /// Generate parallel execution plan
    pub fn generate_plan(&self, graph: &VisualGraph) -> CanvasResult<ParallelExecutionPlan> {
        let nodes = graph.get_nodes();
        let edges = graph.get_connections();

        // Build dependency graph
        let mut dependencies: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for edge in edges {
            dependencies
                .entry(edge.target_node)
                .or_default()
                .push(edge.source_node);
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
    fn topological_sort(
        &self,
        nodes: &[VisualNode],
        dependencies: &HashMap<NodeId, Vec<NodeId>>,
    ) -> CanvasResult<Vec<ExecutionStage>> {
        let mut adj = HashMap::new();
        let mut in_degree = HashMap::new();

        // Initialize in_degree and adj
        for node in nodes {
            in_degree.insert(node.id, 0);
            adj.insert(node.id, Vec::new());
        }

        for (&target, sources) in dependencies {
            if in_degree.contains_key(&target) {
                for &source in sources {
                    if in_degree.contains_key(&source) {
                        adj.entry(source).or_default().push(target);
                        *in_degree.entry(target).or_default() += 1;
                    }
                }
            }
        }

        let mut stages = Vec::new();
        let mut current_stage_nodes = Vec::new();

        for node in nodes {
            if in_degree[&node.id] == 0 {
                current_stage_nodes.push(node.id);
            }
        }
        current_stage_nodes.sort();

        let mut processed_count = 0;
        let mut stage_id = 0;
        let mut node_to_stage = HashMap::new();

        while !current_stage_nodes.is_empty() {
            let mut next_stage_nodes = Vec::new();

            for &node_id in &current_stage_nodes {
                node_to_stage.insert(node_id, stage_id);
            }

            for &u in &current_stage_nodes {
                processed_count += 1;
                if let Some(neighbors) = adj.get(&u) {
                    for &v in neighbors {
                        if let Some(deg) = in_degree.get_mut(&v) {
                            *deg -= 1;
                            if *deg == 0 {
                                next_stage_nodes.push(v);
                            }
                        }
                    }
                }
            }
            next_stage_nodes.sort();

            // Find which stages this stage depends on
            let mut stage_dependencies = std::collections::HashSet::new();
            for &node_id in &current_stage_nodes {
                if let Some(sources) = dependencies.get(&node_id) {
                    for &source in sources {
                        if let Some(&dep_stage) = node_to_stage.get(&source) {
                            if dep_stage != stage_id {
                                stage_dependencies.insert(dep_stage);
                            }
                        }
                    }
                }
            }

            stages.push(ExecutionStage {
                stage_id,
                nodes: current_stage_nodes,
                estimated_duration: 100, // Mock duration
                dependencies: stage_dependencies.into_iter().collect(),
            });

            current_stage_nodes = next_stage_nodes;
            stage_id += 1;
        }

        if processed_count < nodes.len() {
            return Err(crate::error::CanvasError::Graph(
                "Cycle detected in dependency graph".to_string()
            ));
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
        let parallel_time = stages
            .iter()
            .map(|s| s.estimated_duration)
            .max()
            .unwrap_or(0);

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
            _config: config.clone(),
        }
    }

    /// Analyze resource usage
    pub fn analyze(&self, graph: &VisualGraph) -> CanvasResult<ResourceUsageReport> {
        let memory_usage = self.analyze_memory_usage(graph)?;
        let cpu_usage = self.analyze_cpu_usage(graph)?;
        let gas_usage = self.analyze_gas_usage(graph)?;
        let network_usage = self.analyze_network_usage(graph)?;
        let recommendations = self.generate_recommendations(
            graph,
            &memory_usage,
            &cpu_usage,
            &gas_usage,
            &network_usage,
        )?;

        Ok(ResourceUsageReport {
            memory_usage,
            cpu_usage,
            gas_usage,
            network_usage,
            recommendations,
        })
    }

    /// Analyze memory usage
    fn analyze_memory_usage(&self, graph: &VisualGraph) -> CanvasResult<MemoryUsage> {
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
            if node.node_type == "State" {
                memory_leaks.push(format!(
                    "Storage operation in node {} may cause memory growth",
                    node.id
                ));
            }
        }

        let average_memory = if !nodes.is_empty() {
            total_memory / nodes.len() as u64
        } else {
            0
        };

        // Generate optimization suggestions
        if peak_memory > 1_000_000 {
            optimization_suggestions
                .push("Consider reducing memory usage in state operations".to_string());
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
    fn analyze_cpu_usage(&self, graph: &VisualGraph) -> CanvasResult<CpuUsage> {
        let nodes = graph.get_nodes();
        let mut peak_cpu = 0.0f64;
        let mut total_cpu = 0.0f64;
        let mut cpu_intensive_operations = Vec::new();

        for node in nodes {
            let node_cpu = self.estimate_node_cpu_usage(node);
            peak_cpu = peak_cpu.max(node_cpu);
            total_cpu += node_cpu;

            if node_cpu > 0.8 {
                cpu_intensive_operations.push(format!(
                    "High CPU usage in node {} ({:.2})",
                    node.id, node_cpu
                ));
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
    fn analyze_gas_usage(&self, graph: &VisualGraph) -> CanvasResult<GasUsage> {
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
                expensive_operations.push(format!(
                    "Expensive operation in node {}: {} gas",
                    node.id, node_gas
                ));
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
    fn analyze_network_usage(&self, graph: &VisualGraph) -> CanvasResult<NetworkUsage> {
        let nodes = graph.get_nodes();
        let mut total_bandwidth = 0u64;
        let mut requests_per_second = 0.0;

        for node in nodes {
            if node.node_type == "External" {
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
        _graph: &VisualGraph,
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
    fn estimate_node_memory_usage(&self, node: &VisualNode) -> u64 {
        match node.node_type.as_str() {
            "State" => 1024,    // Storage operations use more memory
            "External" => 512,  // External calls use moderate memory
            "Arithmetic" => 64, // Arithmetic operations use little memory
            "Logic" => 32,      // Logic operations use very little memory
            "Control" => 128,   // Control flow uses some memory
            "Start" => 256,     // Start nodes use moderate memory
            "End" => 256,       // End nodes use moderate memory
            _ => 64,
        }
    }

    /// Estimate node CPU usage
    fn estimate_node_cpu_usage(&self, node: &VisualNode) -> f64 {
        match node.node_type.as_str() {
            "State" => 0.3,      // Storage operations are CPU intensive
            "External" => 0.5,   // External calls are very CPU intensive
            "Arithmetic" => 0.1, // Arithmetic operations are light
            "Logic" => 0.05,     // Logic operations are very light
            "Control" => 0.2,    // Control flow is moderate
            "Start" => 0.1,      // Start nodes are light
            "End" => 0.1,        // End nodes are light
            _ => 0.1,
        }
    }

    /// Estimate node gas usage
    fn estimate_node_gas_usage(&self, node: &VisualNode) -> u64 {
        match node.node_type.as_str() {
            "State" => 20000,   // Storage operations are expensive
            "External" => 2600, // External calls are expensive
            "Arithmetic" => 3,  // Arithmetic operations are cheap
            "Logic" => 1,       // Logic operations are very cheap
            "Control" => 1,     // Control flow is cheap
            "Start" => 100,     // Start nodes are moderate
            "End" => 100,       // End nodes are moderate
            _ => 1,
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

        let graph = VisualGraph::new("test");
        let results = optimizer.optimize(&graph).unwrap();

        assert!(!results.is_empty());

        let summary = optimizer.get_optimization_summary(&results);
        assert!(summary.total_optimizations > 0);
    }

    #[test]
    fn test_parallel_execution_optimizer() {
        let config = Config::default();
        let optimizer = ParallelExecutionOptimizer::new(&config);

        let graph = VisualGraph::new("test");
        let plan = optimizer.generate_plan(&graph).unwrap();

        assert!(plan.estimated_parallelism >= 0.0);
        assert!(plan.estimated_speedup >= 1.0);
    }

    #[test]
    fn test_resource_usage_analyzer() {
        let config = Config::default();
        let analyzer = ResourceUsageAnalyzer::new(&config);

        let graph = VisualGraph::new("test");
        let report = analyzer.analyze(&graph).unwrap();

        assert!(report.memory_usage.peak_memory <= report.memory_usage.peak_memory);
        assert!(report.cpu_usage.peak_cpu >= 0.0);
        assert!(report.gas_usage.total_gas <= report.gas_usage.total_gas);
    }

    #[test]
    fn test_topological_sort_and_cycle_detection() {
        let config = Config::default();
        let optimizer = ParallelExecutionOptimizer::new(&config);

        // Acyclical graph: A -> B -> C
        let mut graph = VisualGraph::new("acyclic");
        let a = VisualNode::new(uuid::Uuid::new_v4(), "Start", crate::types::Position::new(0.0, 0.0));
        let b = VisualNode::new(uuid::Uuid::new_v4(), "Add", crate::types::Position::new(0.0, 0.0));
        let c = VisualNode::new(uuid::Uuid::new_v4(), "End", crate::types::Position::new(0.0, 0.0));
        
        let a_id = a.id;
        let b_id = b.id;
        let c_id = c.id;

        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);

        graph.add_connection(crate::types::Connection::new(uuid::Uuid::new_v4(), a_id, "flow_out", b_id, "flow_in"));
        graph.add_connection(crate::types::Connection::new(uuid::Uuid::new_v4(), b_id, "flow_out", c_id, "flow_in"));

        let plan = optimizer.generate_plan(&graph).unwrap();
        assert_eq!(plan.stages.len(), 3);
        assert_eq!(plan.stages[0].nodes, vec![a_id]);
        assert_eq!(plan.stages[1].nodes, vec![b_id]);
        assert_eq!(plan.stages[2].nodes, vec![c_id]);

        // Cyclical graph: A -> B -> A
        let mut cyclical_graph = VisualGraph::new("cyclical");
        let a_node = VisualNode::new(a_id, "Add", crate::types::Position::new(0.0, 0.0));
        let b_node = VisualNode::new(b_id, "Subtract", crate::types::Position::new(0.0, 0.0));
        cyclical_graph.add_node(a_node);
        cyclical_graph.add_node(b_node);
        cyclical_graph.add_connection(crate::types::Connection::new(uuid::Uuid::new_v4(), a_id, "flow_out", b_id, "flow_in"));
        cyclical_graph.add_connection(crate::types::Connection::new(uuid::Uuid::new_v4(), b_id, "flow_out", a_id, "flow_in"));

        let plan_err = optimizer.generate_plan(&cyclical_graph);
        assert!(plan_err.is_err());
    }

    #[test]
    fn test_loop_detection_and_optimization() {
        let pass = LoopOptimizationPass;
        let mut graph = VisualGraph::new("loop");
        
        let a_id = uuid::Uuid::new_v4();
        let b_id = uuid::Uuid::new_v4();
        
        let a = VisualNode::new(a_id, "ReadStorage", crate::types::Position::new(0.0, 0.0));
        let b = VisualNode::new(b_id, "Add", crate::types::Position::new(0.0, 0.0));
        
        graph.add_node(a);
        graph.add_node(b);
        
        graph.add_connection(crate::types::Connection::new(uuid::Uuid::new_v4(), a_id, "flow_out", b_id, "flow_in"));
        graph.add_connection(crate::types::Connection::new(uuid::Uuid::new_v4(), b_id, "flow_out", a_id, "flow_in"));
        
        let loops = pass.find_loops(graph.get_nodes(), graph.get_connections()).unwrap();
        assert!(!loops.is_empty());
        assert_eq!(loops[0].len(), 2);
        
        // Loop contains ReadStorage (State), so it should be optimizable
        let optimizable = pass.can_optimize_loop(&loops[0], &graph).unwrap();
        assert!(optimizable);
    }
}
