export interface VisualNode {
    id: string
    type: string
    position: { x: number; y: number }
    data: {
        label: string
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
    type: string
    name: string
    description: string
    category: string
    inputs: NodePort[]
    outputs: NodePort[]
    properties: NodeProperty[]
}

export interface NodePort {
    name: string
    type: 'Flow' | 'Boolean' | 'Number' | 'String' | 'Bytes'
    required: boolean
}

export interface NodeProperty {
    name: string
    type: 'string' | 'number' | 'boolean'
    description: string
    required: boolean
    default?: any
}

export interface CompilationResult {
    success: boolean
    wasm_bytes: Uint8Array
    gas_estimate: number
    error?: string
}

export interface ValidationResult {
    valid: boolean
    errors: ValidationError[]
    warnings: ValidationWarning[]
}

export interface ValidationError {
    node_id: string
    message: string
    severity: 'error' | 'warning'
}

export interface ValidationWarning {
    node_id: string
    message: string
    suggestion?: string
} 