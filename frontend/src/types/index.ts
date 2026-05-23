export interface VisualNode {
    id: string
    type: string
    position: { x: number; y: number }
    data: {
        label: string
        nodeType?: string
        properties?: Record<string, any>
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
    nodes: VisualNode[]
    edges: VisualEdge[]
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
    gas_estimate?: number
    error?: string | null
}

export interface ValidationResult {
    is_valid: boolean
    errors: string[]
    warnings: string[]
}
