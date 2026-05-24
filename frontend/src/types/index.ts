export interface VisualNode {
    id: string
    type: string
    position: { x: number; y: number }
    data: {
        label: string
        nodeType?: string
        properties?: Record<string, any>
        gasVisualizerActive?: boolean
    }
}

export interface VisualEdge {
    id: string
    source: string
    target: string
    sourceHandle?: string
    targetHandle?: string
}

export interface VisualGraph {
    id?: string
    name: string
    description?: string
    nodes: VisualNode[]
    edges: VisualEdge[]
    metadata?: Record<string, string>
}

export interface NodeDefinition {
    id: string
    name: string
    description: string
    category: string
    inputs: PortInfo[]
    outputs: PortInfo[]
}

export interface PortInfo {
    name: string
    value_type: string
    required: boolean
}

export interface CompilationResult {
    success: boolean
    wasm_size?: number
    wasm_bytes?: string
    gas_estimate?: number
    error?: string | null
}

export interface ValidationResult {
    is_valid: boolean
    errors: string[]
    warnings: string[]
}

export interface ExecutionStep {
    step_number: number
    node_id: string
    node_type: string
    inputs: Record<string, any>
    outputs: Record<string, any>
    gas_consumed: number
    duration_ms: number
    error?: string | null
}
