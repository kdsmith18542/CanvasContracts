import { invoke } from '@tauri-apps/api/tauri'
import { VisualGraph, CompilationResult, ValidationResult, NodeDefinition } from '../types'

export class TauriService {
    static async compileContract(graph: VisualGraph, optimizationLevel: number = 1): Promise<CompilationResult> {
        try {
            const result = await invoke('compile_contract', {
                request: {
                    graph,
                    optimization_level: optimizationLevel
                }
            })
            return result as CompilationResult
        } catch (error) {
            throw new Error(`Compilation failed: ${error}`)
        }
    }

    static async validateGraph(graph: VisualGraph): Promise<ValidationResult> {
        try {
            const result = await invoke('validate_graph', { graph })
            return result as ValidationResult
        } catch (error) {
            throw new Error(`Validation failed: ${error}`)
        }
    }

    static async analyzePatterns(graph: VisualGraph): Promise<any> {
        try {
            const result = await invoke('analyze_patterns', { graph })
            return result
        } catch (error) {
            throw new Error(`Pattern analysis failed: ${error}`)
        }
    }

    static async deployContract(wasmBytes: number[], privateKey: string): Promise<DeployResult> {
        try {
            const result = await invoke('deploy_contract', {
                request: {
                    wasm_bytes: wasmBytes,
                    private_key: privateKey,
                }
            })
            return result as DeployResult
        } catch (error) {
            throw new Error(`Deployment failed: ${error}`)
        }
    }

    static async getNodeDefinitions(): Promise<NodeDefinition[]> {
        try {
            const result = await invoke('get_node_definitions')
            return result as NodeDefinition[]
        } catch (error) {
            throw new Error(`Failed to get node definitions: ${error}`)
        }
    }
}

export interface DeployResult {
    success: boolean
    contract_address: string
    transaction_hash: string
    gas_used: number
    error?: string | null
}
