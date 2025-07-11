import React, { useState, useEffect } from 'react'
import {
    Play,
    Pause,
    StepForward,
    StepInto,
    StepOut,
    Square,
    Bug,
    Clock,
    Zap,
    Eye,
    EyeOff
} from 'lucide-react'
import { useCanvasStore } from '../store/useCanvasStore'
import { TauriService } from '../services/tauriService'

interface ExecutionStep {
    step_number: number
    node_id: string
    node_type: string
    timestamp: number
    inputs: Record<string, any>
    outputs: Record<string, any>
    gas_consumed: number
    duration_ms: number
    error?: string
}

interface Breakpoint {
    node_id: string
    condition?: string
    enabled: boolean
    hit_count: number
}

interface DebugState {
    state: 'running' | 'paused' | 'stepping' | 'finished' | 'error'
    current_step: number
    total_steps: number
    variables: Record<string, any>
    call_stack: Array<{
        node_id: string
        function_name: string
        line_number?: number
        variables: Record<string, any>
    }>
}

export const Debugger: React.FC = () => {
    const [isDebugging, setIsDebugging] = useState(false)
    const [debugState, setDebugState] = useState<DebugState | null>(null)
    const [executionTrace, setExecutionTrace] = useState<ExecutionStep[]>([])
    const [breakpoints, setBreakpoints] = useState<Breakpoint[]>([])
    const [showVariables, setShowVariables] = useState(true)
    const [showCallStack, setShowCallStack] = useState(true)
    const [selectedStep, setSelectedStep] = useState<number | null>(null)
    const { graph } = useCanvasStore()

    const handleStartDebug = async () => {
        if (graph.nodes.length === 0) {
            alert('Please add some nodes to the canvas first')
            return
        }

        setIsDebugging(true)
        try {
            // TODO: Implement actual debug start
            // const result = await TauriService.startDebug(graph)
            // setDebugState(result)

            // Mock debug state for now
            setDebugState({
                state: 'running',
                current_step: 0,
                total_steps: 0,
                variables: {},
                call_stack: []
            })
        } catch (error) {
            console.error('Debug start failed:', error)
            alert('Failed to start debugging')
            setIsDebugging(false)
        }
    }

    const handleStopDebug = () => {
        setIsDebugging(false)
        setDebugState(null)
        setExecutionTrace([])
        setSelectedStep(null)
    }

    const handleContinue = async () => {
        if (!debugState) return

        try {
            // TODO: Implement continue execution
            // const result = await TauriService.continueDebug()
            // setDebugState(result)

            setDebugState(prev => prev ? { ...prev, state: 'running' } : null)
        } catch (error) {
            console.error('Continue failed:', error)
        }
    }

    const handleStepNext = async () => {
        if (!debugState) return

        try {
            // TODO: Implement step next
            // const result = await TauriService.stepNext()
            // setDebugState(result)

            setDebugState(prev => prev ? {
                ...prev,
                state: 'stepping',
                current_step: prev.current_step + 1
            } : null)
        } catch (error) {
            console.error('Step next failed:', error)
        }
    }

    const handleStepInto = async () => {
        if (!debugState) return

        try {
            // TODO: Implement step into
            // const result = await TauriService.stepInto()
            // setDebugState(result)

            setDebugState(prev => prev ? { ...prev, state: 'stepping' } : null)
        } catch (error) {
            console.error('Step into failed:', error)
        }
    }

    const handleStepOut = async () => {
        if (!debugState) return

        try {
            // TODO: Implement step out
            // const result = await TauriService.stepOut()
            // setDebugState(result)

            setDebugState(prev => prev ? { ...prev, state: 'stepping' } : null)
        } catch (error) {
            console.error('Step out failed:', error)
        }
    }

    const handleAddBreakpoint = (nodeId: string) => {
        const newBreakpoint: Breakpoint = {
            node_id: nodeId,
            enabled: true,
            hit_count: 0
        }
        setBreakpoints(prev => [...prev, newBreakpoint])
    }

    const handleRemoveBreakpoint = (nodeId: string) => {
        setBreakpoints(prev => prev.filter(bp => bp.node_id !== nodeId))
    }

    const handleToggleBreakpoint = (nodeId: string) => {
        setBreakpoints(prev => prev.map(bp =>
            bp.node_id === nodeId
                ? { ...bp, enabled: !bp.enabled }
                : bp
        ))
    }

    const getStateColor = (state: string) => {
        switch (state) {
            case 'running':
                return 'text-green-600'
            case 'paused':
                return 'text-yellow-600'
            case 'stepping':
                return 'text-blue-600'
            case 'finished':
                return 'text-gray-600'
            case 'error':
                return 'text-red-600'
            default:
                return 'text-gray-600'
        }
    }

    const formatTimestamp = (timestamp: number) => {
        return new Date(timestamp).toLocaleTimeString()
    }

    const formatDuration = (duration: number) => {
        return `${duration}ms`
    }

    const formatGas = (gas: number) => {
        return gas.toLocaleString()
    }

    return (
        <div className="w-96 bg-white border-l border-gray-200 flex flex-col">
            {/* Debugger Header */}
            <div className="p-4 border-b border-gray-200">
                <div className="flex items-center justify-between mb-4">
                    <h2 className="text-lg font-semibold text-gray-900 flex items-center">
                        <Bug className="w-5 h-5 mr-2" />
                        Debugger
                    </h2>
                    <div className="flex items-center space-x-2">
                        <button
                            onClick={() => setShowVariables(!showVariables)}
                            className="p-1 text-gray-500 hover:text-gray-700"
                            title={showVariables ? "Hide Variables" : "Show Variables"}
                        >
                            {showVariables ? <Eye className="w-4 h-4" /> : <EyeOff className="w-4 h-4" />}
                        </button>
                    </div>
                </div>

                {/* Debug Controls */}
                <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-2">
                        {!isDebugging ? (
                            <button
                                onClick={handleStartDebug}
                                className="flex items-center px-3 py-1.5 text-sm font-medium text-white bg-green-600 border border-transparent rounded-md hover:bg-green-700"
                            >
                                <Play className="w-4 h-4 mr-1" />
                                Start
                            </button>
                        ) : (
                            <>
                                <button
                                    onClick={handleContinue}
                                    disabled={debugState?.state === 'running'}
                                    className="flex items-center px-2 py-1 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 disabled:opacity-50"
                                >
                                    <Play className="w-4 h-4" />
                                </button>
                                <button
                                    onClick={handleStepNext}
                                    className="flex items-center px-2 py-1 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                                >
                                    <StepForward className="w-4 h-4" />
                                </button>
                                <button
                                    onClick={handleStepInto}
                                    className="flex items-center px-2 py-1 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                                >
                                    <StepInto className="w-4 h-4" />
                                </button>
                                <button
                                    onClick={handleStepOut}
                                    className="flex items-center px-2 py-1 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                                >
                                    <StepOut className="w-4 h-4" />
                                </button>
                                <button
                                    onClick={handleStopDebug}
                                    className="flex items-center px-2 py-1 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                                >
                                    <Square className="w-4 h-4" />
                                </button>
                            </>
                        )}
                    </div>

                    {debugState && (
                        <div className={`text-sm font-medium ${getStateColor(debugState.state)}`}>
                            {debugState.state.toUpperCase()}
                        </div>
                    )}
                </div>

                {/* Debug Info */}
                {debugState && (
                    <div className="mt-3 text-xs text-gray-500">
                        <div className="flex items-center justify-between">
                            <span>Step: {debugState.current_step + 1} / {debugState.total_steps}</span>
                            <span>Variables: {Object.keys(debugState.variables).length}</span>
                        </div>
                    </div>
                )}
            </div>

            {/* Debug Content */}
            <div className="flex-1 overflow-y-auto">
                {debugState && (
                    <>
                        {/* Variables Panel */}
                        {showVariables && (
                            <div className="p-4 border-b border-gray-200">
                                <h3 className="text-sm font-medium text-gray-900 mb-2">Variables</h3>
                                <div className="space-y-1">
                                    {Object.entries(debugState.variables).map(([key, value]) => (
                                        <div key={key} className="flex justify-between text-xs">
                                            <span className="font-mono text-gray-700">{key}:</span>
                                            <span className="font-mono text-gray-900">
                                                {typeof value === 'object' ? JSON.stringify(value) : String(value)}
                                            </span>
                                        </div>
                                    ))}
                                    {Object.keys(debugState.variables).length === 0 && (
                                        <span className="text-xs text-gray-500">No variables</span>
                                    )}
                                </div>
                            </div>
                        )}

                        {/* Call Stack Panel */}
                        {showCallStack && debugState.call_stack.length > 0 && (
                            <div className="p-4 border-b border-gray-200">
                                <h3 className="text-sm font-medium text-gray-900 mb-2">Call Stack</h3>
                                <div className="space-y-1">
                                    {debugState.call_stack.map((frame, index) => (
                                        <div key={index} className="text-xs text-gray-700">
                                            {frame.function_name} ({frame.node_id})
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}

                        {/* Execution Trace */}
                        <div className="p-4">
                            <h3 className="text-sm font-medium text-gray-900 mb-2">Execution Trace</h3>
                            <div className="space-y-2 max-h-64 overflow-y-auto">
                                {executionTrace.map((step, index) => (
                                    <div
                                        key={index}
                                        className={`p-2 border rounded-md cursor-pointer text-xs ${selectedStep === index
                                                ? 'border-blue-500 bg-blue-50'
                                                : 'border-gray-200 hover:border-gray-300'
                                            }`}
                                        onClick={() => setSelectedStep(selectedStep === index ? null : index)}
                                    >
                                        <div className="flex items-center justify-between mb-1">
                                            <span className="font-medium">Step {step.step_number + 1}</span>
                                            <span className="text-gray-500">{formatTimestamp(step.timestamp)}</span>
                                        </div>
                                        <div className="text-gray-700">{step.node_id}</div>
                                        <div className="flex items-center justify-between text-gray-500">
                                            <span className="flex items-center">
                                                <Clock className="w-3 h-3 mr-1" />
                                                {formatDuration(step.duration_ms)}
                                            </span>
                                            <span className="flex items-center">
                                                <Zap className="w-3 h-3 mr-1" />
                                                {formatGas(step.gas_consumed)}
                                            </span>
                                        </div>
                                        {step.error && (
                                            <div className="mt-1 text-red-600 text-xs">{step.error}</div>
                                        )}
                                    </div>
                                ))}
                                {executionTrace.length === 0 && (
                                    <span className="text-xs text-gray-500">No execution trace</span>
                                )}
                            </div>
                        </div>
                    </>
                )}

                {!debugState && (
                    <div className="flex-1 flex items-center justify-center p-8">
                        <div className="text-center">
                            <Bug className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                            <p className="text-sm text-gray-500">
                                Click "Start" to begin debugging your contract
                            </p>
                        </div>
                    </div>
                )}
            </div>

            {/* Breakpoints Panel */}
            <div className="p-4 border-t border-gray-200">
                <div className="flex items-center justify-between mb-2">
                    <h3 className="text-sm font-medium text-gray-900">Breakpoints</h3>
                    <span className="text-xs text-gray-500">{breakpoints.length}</span>
                </div>
                <div className="space-y-1 max-h-32 overflow-y-auto">
                    {breakpoints.map((breakpoint, index) => (
                        <div key={index} className="flex items-center justify-between p-2 bg-gray-50 rounded text-xs">
                            <span className="font-mono text-gray-700">{breakpoint.node_id}</span>
                            <div className="flex items-center space-x-1">
                                <button
                                    onClick={() => handleToggleBreakpoint(breakpoint.node_id)}
                                    className={`w-2 h-2 rounded-full ${breakpoint.enabled ? 'bg-red-500' : 'bg-gray-300'
                                        }`}
                                    title={breakpoint.enabled ? "Disable" : "Enable"}
                                />
                                <button
                                    onClick={() => handleRemoveBreakpoint(breakpoint.node_id)}
                                    className="text-gray-400 hover:text-red-500"
                                    title="Remove"
                                >
                                    ×
                                </button>
                            </div>
                        </div>
                    ))}
                    {breakpoints.length === 0 && (
                        <span className="text-xs text-gray-500">No breakpoints set</span>
                    )}
                </div>
            </div>
        </div>
    )
} 