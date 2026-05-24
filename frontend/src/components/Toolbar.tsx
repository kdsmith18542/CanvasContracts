import React, { useState } from 'react'
import {
    Play,
    CheckCircle,
    Upload,
    Save,
    Settings,
    Bug,
    Plus,
    Store,
    Download,
    Rocket,
    Activity,
    History,
    ShieldCheck,
    FileDown,
    Brain
} from 'lucide-react'
import { useCanvasStore } from '../store/useCanvasStore'
import { TauriService, DeployResult } from '../services/tauriService'

interface ToolbarProps {
    activeSidebar: 'none' | 'ai' | 'debugger' | 'monitor' | 'history' | 'proofs' | 'audit'
    onSidebarToggle: (sidebar: 'none' | 'ai' | 'debugger' | 'monitor' | 'history' | 'proofs' | 'audit') => void
    showMarketplace: boolean
    onMarketplaceToggle: () => void
    onCustomNodeToggle: () => void
    onSave: () => void
    onLoad: () => void
    onDeploy: (result: DeployResult) => void
}

export const Toolbar: React.FC<ToolbarProps> = ({
    activeSidebar,
    onSidebarToggle,
    showMarketplace,
    onMarketplaceToggle,
    onCustomNodeToggle,
    onSave,
    onLoad,
    onDeploy
}) => {
    const { graph, setCompilationResult, setValidationResult, setLoading, isLoading, setError, compilationResult } = useCanvasStore()
    const [isCompiling, setIsCompiling] = useState(false)
    const [isValidating, setIsValidating] = useState(false)
    const [isDeploying, setIsDeploying] = useState(false)

    const handleCompile = async () => {
        setIsCompiling(true)
        setLoading(true)
        setError(null)
        try {
            const result = await TauriService.compileContract(graph)
            setCompilationResult(result)
            if (result.success) {
                const gasMsg = result.gas_estimate !== undefined ? ` Gas estimate: ${result.gas_estimate}` : ''
                alert(`Compilation successful!${gasMsg}`)
            } else {
                alert(`Compilation failed: ${result.error || 'Unknown error'}`)
            }
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Compilation failed'
            setError(msg)
            alert(msg)
        } finally {
            setIsCompiling(false)
            setLoading(false)
        }
    }

    const handleValidate = async () => {
        setIsValidating(true)
        setLoading(true)
        setError(null)
        try {
            const result = await TauriService.validateGraph(graph)
            setValidationResult(result)
            if (result.is_valid) {
                alert('Validation passed!')
            } else {
                const msgs = [...result.errors, ...result.warnings].join('\n')
                alert(`Validation issues:\n${msgs}`)
            }
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Validation failed'
            setError(msg)
            alert(msg)
        } finally {
            setIsValidating(false)
            setLoading(false)
        }
    }

    const handleSimulate = async () => {
        setLoading(true)
        setError(null)
        try {
            const result = await TauriService.compileContract(graph)
            setCompilationResult(result)
            if (result.success) {
                alert(`Simulation complete. Wasm size: ${result.wasm_size || 0} bytes, Gas: ${result.gas_estimate || 0}`)
            } else {
                alert(`Simulation failed: ${result.error || 'Unknown error'}`)
            }
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Simulation failed'
            setError(msg)
            alert(msg)
        } finally {
            setLoading(false)
        }
    }

    const handleDeploy = async () => {
        setIsDeploying(true)
        setLoading(true)
        setError(null)
        try {
            const privateKey = prompt('Enter private key for deployment:', 'dev-key') || 'dev-key'
            let result = compilationResult
            if (!result || !result.success) {
                result = await TauriService.compileContract(graph)
                setCompilationResult(result)
            }
            if (!result.success) {
                throw new Error(result.error || 'Compilation failed')
            }
            const wasmHex = result.wasm_bytes
            if (!wasmHex || typeof wasmHex !== 'string') {
                throw new Error('No WASM bytes in compilation result - compile first')
            }
            // Convert hex string to byte array
            const wasmBytes = new Array(0)
            for (let i = 0; i < wasmHex.length; i += 2) {
                wasmBytes.push(parseInt(wasmHex.substring(i, i + 2), 16))
            }
            const deployResult = await TauriService.deployContract(wasmBytes, privateKey)
            if (deployResult.success) {
                onDeploy(deployResult)
                alert(`Contract deployed at ${deployResult.contract_address}`)
            } else {
                throw new Error(deployResult.error || 'Deployment failed')
            }
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Deployment failed'
            setError(msg)
            alert(msg)
        } finally {
            setIsDeploying(false)
            setLoading(false)
        }
    }

    const handleSave = () => {
        onSave()
    }

    const handleLoad = () => {
        onLoad()
    }

    return (
        <div className="bg-white border-b border-gray-200 px-4 py-2 flex items-center justify-between">
            <div className="flex items-center space-x-2">
                <h1 className="text-xl font-semibold text-gray-900">Canvas Contracts</h1>
            </div>

            <div className="flex items-center space-x-2">
                <button
                    onClick={onMarketplaceToggle}
                    className={`flex items-center px-3 py-1.5 text-sm font-medium rounded-md border transition-colors duration-150 ${
                        showMarketplace
                            ? 'bg-blue-50 text-blue-600 border-blue-200 hover:bg-blue-100/75'
                            : 'text-gray-700 bg-white border-gray-300 hover:bg-gray-50'
                    }`}
                >
                    <Store className="w-4 h-4 mr-1" />
                    Marketplace
                </button>

                <button
                    onClick={onCustomNodeToggle}
                    className="flex items-center px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                >
                    <Plus className="w-4 h-4 mr-1" />
                    Custom Node
                </button>

                <button
                    onClick={() => onSidebarToggle(activeSidebar === 'ai' ? 'none' : 'ai')}
                    className={`flex items-center px-3 py-1.5 text-sm font-medium rounded-md border transition-colors duration-150 ${
                        activeSidebar === 'ai'
                            ? 'bg-blue-50 text-blue-600 border-blue-200 hover:bg-blue-100/75'
                            : 'text-gray-700 bg-white border-gray-300 hover:bg-gray-50'
                    }`}
                >
                    <Brain className="w-4 h-4 mr-1" />
                    AI Assistant
                </button>

                <button
                    onClick={() => onSidebarToggle(activeSidebar === 'debugger' ? 'none' : 'debugger')}
                    className={`flex items-center px-3 py-1.5 text-sm font-medium rounded-md border transition-colors duration-150 ${
                        activeSidebar === 'debugger'
                            ? 'bg-blue-50 text-blue-600 border-blue-200 hover:bg-blue-100/75'
                            : 'text-gray-700 bg-white border-gray-300 hover:bg-gray-50'
                    }`}
                >
                    <Bug className="w-4 h-4 mr-1" />
                    Debug
                </button>

                <button
                    onClick={() => onSidebarToggle(activeSidebar === 'monitor' ? 'none' : 'monitor')}
                    className={`flex items-center px-3 py-1.5 text-sm font-medium rounded-md border transition-colors duration-150 ${
                        activeSidebar === 'monitor'
                            ? 'bg-blue-50 text-blue-600 border-blue-200 hover:bg-blue-100/75'
                            : 'text-gray-700 bg-white border-gray-300 hover:bg-gray-50'
                    }`}
                >
                    <Activity className="w-4 h-4 mr-1" />
                    Monitor
                </button>

                <button
                    onClick={() => onSidebarToggle(activeSidebar === 'history' ? 'none' : 'history')}
                    className={`flex items-center px-3 py-1.5 text-sm font-medium rounded-md border transition-colors duration-150 ${
                        activeSidebar === 'history'
                            ? 'bg-blue-50 text-blue-600 border-blue-200 hover:bg-blue-100/75'
                            : 'text-gray-700 bg-white border-gray-300 hover:bg-gray-50'
                    }`}
                >
                    <History className="w-4 h-4 mr-1" />
                    History
                </button>

                <button
                    onClick={() => onSidebarToggle(activeSidebar === 'proofs' ? 'none' : 'proofs')}
                    className={`flex items-center px-3 py-1.5 text-sm font-medium rounded-md border transition-colors duration-150 ${
                        activeSidebar === 'proofs'
                            ? 'bg-blue-50 text-blue-600 border-blue-200 hover:bg-blue-100/75'
                            : 'text-gray-700 bg-white border-gray-300 hover:bg-gray-50'
                    }`}
                >
                    <ShieldCheck className="w-4 h-4 mr-1" />
                    Proofs
                </button>

                <button
                    onClick={() => onSidebarToggle(activeSidebar === 'audit' ? 'none' : 'audit')}
                    className={`flex items-center px-3 py-1.5 text-sm font-medium rounded-md border transition-colors duration-150 ${
                        activeSidebar === 'audit'
                            ? 'bg-blue-50 text-blue-600 border-blue-200 hover:bg-blue-100/75'
                            : 'text-gray-700 bg-white border-gray-300 hover:bg-gray-50'
                    }`}
                >
                    <FileDown className="w-4 h-4 mr-1" />
                    Audit
                </button>

                <button
                    onClick={handleValidate}
                    disabled={isValidating || isLoading}
                    className="flex items-center px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50"
                >
                    <CheckCircle className="w-4 h-4 mr-1" />
                    {isValidating ? 'Validating...' : 'Validate'}
                </button>

                <button
                    onClick={handleSimulate}
                    disabled={isLoading}
                    className="flex items-center px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50"
                >
                    <Play className="w-4 h-4 mr-1" />
                    Simulate
                </button>

                <button
                    onClick={handleCompile}
                    disabled={isCompiling || isLoading}
                    className="flex items-center px-3 py-1.5 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 disabled:opacity-50"
                >
                    <Upload className="w-4 h-4 mr-1" />
                    {isCompiling ? 'Compiling...' : 'Compile'}
                </button>

                <button
                    onClick={handleDeploy}
                    disabled={isDeploying || isLoading}
                    className="flex items-center px-3 py-1.5 text-sm font-medium text-white bg-green-600 border border-transparent rounded-md hover:bg-green-700 disabled:opacity-50"
                >
                    <Rocket className="w-4 h-4 mr-1" />
                    {isDeploying ? 'Deploying...' : 'Deploy'}
                </button>

                <button
                    onClick={handleSave}
                    className="flex items-center px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                >
                    <Save className="w-4 h-4 mr-1" />
                    Save
                </button>

                <button
                    onClick={handleLoad}
                    className="flex items-center px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                >
                    <Download className="w-4 h-4 mr-1" />
                    Load
                </button>

                <button className="flex items-center px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50">
                    <Settings className="w-4 h-4" />
                </button>
            </div>
        </div>
    )
}
