import React, { useState } from 'react'
import { Brain, AlertTriangle, CheckCircle, Info, Zap } from 'lucide-react'
import { useCanvasStore } from '../store/useCanvasStore'
import { TauriService } from '../services/tauriService'

interface AnalysisResult {
    patterns_found: Array<{
        name: string
        description: string
        confidence: number
        category: string
    }>
    anti_patterns: Array<{
        name: string
        description: string
        severity: string
        suggestion: string
    }>
    security_issues: Array<{
        name: string
        description: string
        severity: string
        cve_reference?: string
        mitigation: string
    }>
    suggestions: string[]
}

export const AiAssistant: React.FC = () => {
    const [isAnalyzing, setIsAnalyzing] = useState(false)
    const [analysisResult, setAnalysisResult] = useState<AnalysisResult | null>(null)
    const [activeTab, setActiveTab] = useState<'patterns' | 'issues' | 'suggestions'>('patterns')
    const { graph } = useCanvasStore()

    const handleAnalyze = async () => {
        if (graph.nodes.length === 0) {
            alert('Please add some nodes to the canvas first')
            return
        }

        setIsAnalyzing(true)
        try {
            const result = await TauriService.analyzePatterns(graph)
            setAnalysisResult(result as AnalysisResult)
        } catch (error) {
            console.error('Analysis failed:', error)
            alert('Analysis failed. Please try again.')
        } finally {
            setIsAnalyzing(false)
        }
    }

    const getSeverityColor = (severity: string) => {
        switch (severity.toLowerCase()) {
            case 'critical':
                return 'text-red-600 bg-red-50 border-red-200'
            case 'high':
                return 'text-orange-600 bg-orange-50 border-orange-200'
            case 'medium':
                return 'text-yellow-600 bg-yellow-50 border-yellow-200'
            case 'low':
                return 'text-blue-600 bg-blue-50 border-blue-200'
            default:
                return 'text-gray-600 bg-gray-50 border-gray-200'
        }
    }

    const getSeverityIcon = (severity: string) => {
        switch (severity.toLowerCase()) {
            case 'critical':
            case 'high':
                return <AlertTriangle className="w-4 h-4" />
            case 'medium':
                return <Info className="w-4 h-4" />
            case 'low':
                return <CheckCircle className="w-4 h-4" />
            default:
                return <Info className="w-4 h-4" />
        }
    }

    return (
        <div className="w-80 bg-white border-l border-gray-200 flex flex-col">
            <div className="p-4 border-b border-gray-200">
                <div className="flex items-center justify-between">
                    <h2 className="text-lg font-semibold text-gray-900 flex items-center">
                        <Brain className="w-5 h-5 mr-2" />
                        AI Assistant
                    </h2>
                    <button
                        onClick={handleAnalyze}
                        disabled={isAnalyzing}
                        className="px-3 py-1.5 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 disabled:opacity-50"
                    >
                        {isAnalyzing ? 'Analyzing...' : 'Analyze'}
                    </button>
                </div>
            </div>

            {analysisResult && (
                <div className="flex-1 overflow-y-auto">
                    {/* Tabs */}
                    <div className="flex border-b border-gray-200">
                        <button
                            onClick={() => setActiveTab('patterns')}
                            className={`flex-1 px-3 py-2 text-sm font-medium ${activeTab === 'patterns'
                                    ? 'text-blue-600 border-b-2 border-blue-600'
                                    : 'text-gray-500 hover:text-gray-700'
                                }`}
                        >
                            Patterns
                        </button>
                        <button
                            onClick={() => setActiveTab('issues')}
                            className={`flex-1 px-3 py-2 text-sm font-medium ${activeTab === 'issues'
                                    ? 'text-blue-600 border-b-2 border-blue-600'
                                    : 'text-gray-500 hover:text-gray-700'
                                }`}
                        >
                            Issues
                        </button>
                        <button
                            onClick={() => setActiveTab('suggestions')}
                            className={`flex-1 px-3 py-2 text-sm font-medium ${activeTab === 'suggestions'
                                    ? 'text-blue-600 border-b-2 border-blue-600'
                                    : 'text-gray-500 hover:text-gray-700'
                                }`}
                        >
                            Suggestions
                        </button>
                    </div>

                    {/* Content */}
                    <div className="p-4">
                        {activeTab === 'patterns' && (
                            <div className="space-y-4">
                                <h3 className="text-sm font-medium text-gray-900">Detected Patterns</h3>
                                {analysisResult.patterns_found.length > 0 ? (
                                    analysisResult.patterns_found.map((pattern, index) => (
                                        <div key={index} className="p-3 bg-green-50 border border-green-200 rounded-md">
                                            <div className="flex items-center justify-between">
                                                <h4 className="text-sm font-medium text-green-800">{pattern.name}</h4>
                                                <span className="text-xs text-green-600">
                                                    {Math.round(pattern.confidence * 100)}% confidence
                                                </span>
                                            </div>
                                            <p className="text-sm text-green-700 mt-1">{pattern.description}</p>
                                            <span className="inline-block mt-2 px-2 py-1 text-xs font-medium text-green-800 bg-green-100 rounded">
                                                {pattern.category}
                                            </span>
                                        </div>
                                    ))
                                ) : (
                                    <p className="text-sm text-gray-500">No patterns detected</p>
                                )}
                            </div>
                        )}

                        {activeTab === 'issues' && (
                            <div className="space-y-4">
                                <h3 className="text-sm font-medium text-gray-900">Security Issues & Anti-patterns</h3>

                                {/* Anti-patterns */}
                                {analysisResult.anti_patterns.length > 0 && (
                                    <div>
                                        <h4 className="text-xs font-medium text-gray-700 mb-2">Anti-patterns</h4>
                                        <div className="space-y-2">
                                            {analysisResult.anti_patterns.map((antiPattern, index) => (
                                                <div
                                                    key={index}
                                                    className={`p-3 border rounded-md ${getSeverityColor(antiPattern.severity)}`}
                                                >
                                                    <div className="flex items-start">
                                                        {getSeverityIcon(antiPattern.severity)}
                                                        <div className="ml-2 flex-1">
                                                            <h5 className="text-sm font-medium">{antiPattern.name}</h5>
                                                            <p className="text-sm mt-1">{antiPattern.description}</p>
                                                            <p className="text-sm mt-2 font-medium">Suggestion: {antiPattern.suggestion}</p>
                                                        </div>
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                    </div>
                                )}

                                {/* Security Issues */}
                                {analysisResult.security_issues.length > 0 && (
                                    <div className="mt-4">
                                        <h4 className="text-xs font-medium text-gray-700 mb-2">Security Issues</h4>
                                        <div className="space-y-2">
                                            {analysisResult.security_issues.map((issue, index) => (
                                                <div
                                                    key={index}
                                                    className={`p-3 border rounded-md ${getSeverityColor(issue.severity)}`}
                                                >
                                                    <div className="flex items-start">
                                                        {getSeverityIcon(issue.severity)}
                                                        <div className="ml-2 flex-1">
                                                            <h5 className="text-sm font-medium">{issue.name}</h5>
                                                            <p className="text-sm mt-1">{issue.description}</p>
                                                            {issue.cve_reference && (
                                                                <p className="text-xs mt-1 font-mono">CVE: {issue.cve_reference}</p>
                                                            )}
                                                            <p className="text-sm mt-2 font-medium">Mitigation: {issue.mitigation}</p>
                                                        </div>
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                    </div>
                                )}

                                {analysisResult.anti_patterns.length === 0 && analysisResult.security_issues.length === 0 && (
                                    <p className="text-sm text-gray-500">No issues detected</p>
                                )}
                            </div>
                        )}

                        {activeTab === 'suggestions' && (
                            <div className="space-y-4">
                                <h3 className="text-sm font-medium text-gray-900">Optimization Suggestions</h3>
                                {analysisResult.suggestions.length > 0 ? (
                                    <div className="space-y-2">
                                        {analysisResult.suggestions.map((suggestion, index) => (
                                            <div key={index} className="p-3 bg-blue-50 border border-blue-200 rounded-md">
                                                <div className="flex items-start">
                                                    <Zap className="w-4 h-4 text-blue-600 mt-0.5" />
                                                    <p className="text-sm text-blue-800 ml-2">{suggestion}</p>
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                ) : (
                                    <p className="text-sm text-gray-500">No suggestions available</p>
                                )}
                            </div>
                        )}
                    </div>
                </div>
            )}

            {!analysisResult && (
                <div className="flex-1 flex items-center justify-center p-8">
                    <div className="text-center">
                        <Brain className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                        <p className="text-sm text-gray-500">
                            Click "Analyze" to get AI-powered insights about your contract
                        </p>
                    </div>
                </div>
            )}
        </div>
    )
} 