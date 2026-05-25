import { invoke } from '@tauri-apps/api/tauri'
import { VisualGraph, CompilationResult, ValidationResult, NodeDefinition, ExecutionStep } from '../types'

export interface ExecutionTrace {
    steps: ExecutionStep[]
    total_gas: number
    success: boolean
}

export interface DebugState {
    state: 'running' | 'paused' | 'stepping' | 'finished' | 'error'
    current_step: number
    total_steps: number
    variables: Record<string, any>
}

export interface AuditBundleResult {
    wasm_bytes: string
    abi: any
    manifest: any
    safety_report: any
    canonical_graph: any
    wit_files: Array<{ name: string; content: string }>
    lock: any
    validation_report: any
    archive: {
        status: string
        storage_pointer: string | null
        content_hash: string | null
    }
}

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

    static async debugStart(graph: VisualGraph): Promise<DebugState> {
        try {
            const result = await invoke('debug_start', { graph })
            return {
                state: 'paused',
                current_step: 0,
                total_steps: (result as any).total_steps || 0,
                variables: {}
            }
        } catch (error) {
            throw new Error(`Debug start failed: ${error}`)
        }
    }

    static async debugStep(): Promise<DebugState> {
        try {
            const result = await invoke('debug_step')
            return {
                state: 'stepping',
                current_step: (result as any).current_step || 0,
                total_steps: (result as any).total_steps || 0,
                variables: (result as any).variables || {}
            }
        } catch (error) {
            throw new Error(`Debug step failed: ${error}`)
        }
    }

    static async debugContinue(): Promise<DebugState> {
        try {
            const result = await invoke('debug_continue')
            return {
                state: 'finished',
                current_step: (result as any).current_step || 0,
                total_steps: (result as any).total_steps || 0,
                variables: (result as any).variables || {}
            }
        } catch (error) {
            throw new Error(`Debug continue failed: ${error}`)
        }
    }

    static async debugGetTrace(): Promise<ExecutionTrace> {
        try {
            const result = await invoke('debug_get_trace')
            return {
                steps: (result as any).trace || [],
                total_gas: (result as any).total_gas || 0,
                success: (result as any).success_at_end || false
            }
        } catch (error) {
            throw new Error(`Debug get trace failed: ${error}`)
        }
    }

    static async queryHistory(contractAddress: string, queryType: 'blocks' | 'transactions' | 'events'): Promise<any> {
        try {
            return await invoke('query_history', { contractAddress, queryType })
        } catch (error) {
            throw new Error(`Query history failed: ${error}`)
        }
    }

    static async verifyProof(proof: any): Promise<boolean> {
        try {
            return await invoke('verify_proof', { proof }) as boolean
        } catch (error) {
            throw new Error(`Proof verification failed: ${error}`)
        }
    }

    static async exportAuditBundle(graph: VisualGraph): Promise<AuditBundleResult> {
        try {
            return await invoke('export_audit_bundle', { graph }) as AuditBundleResult
        } catch (error) {
            throw new Error(`Audit bundle export failed: ${error}`)
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
