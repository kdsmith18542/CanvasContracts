import React, { useState, useEffect } from 'react'
import { Activity, ExternalLink, RefreshCw, CheckCircle, XCircle } from 'lucide-react'

export interface DeployedContract {
    address: string
    name: string
    status: 'pending' | 'confirmed' | 'failed'
    txHash: string
    gasUsed: number
    timestamp: number
}

interface ContractMonitorProps {
    contracts: DeployedContract[]
    onRefresh: (address: string) => void
    onClose: () => void
}

export const ContractMonitor: React.FC<ContractMonitorProps> = ({ contracts, onRefresh, onClose }) => {
    const [autoRefresh, setAutoRefresh] = useState(true)

    useEffect(() => {
        if (!autoRefresh || contracts.length === 0) return
        const interval = setInterval(() => {
            contracts.forEach(c => {
                if (c.status === 'pending') onRefresh(c.address)
            })
        }, 5000)
        return () => clearInterval(interval)
    }, [autoRefresh, contracts, onRefresh])

    return (
        <div className="w-80 bg-white border-l border-gray-200 flex flex-col">
            <div className="p-4 border-b border-gray-200">
                <div className="flex items-center justify-between mb-2">
                    <h2 className="text-lg font-semibold text-gray-900 flex items-center">
                        <Activity className="w-5 h-5 mr-2" />
                        Contract Monitor
                    </h2>
                    <button onClick={onClose} className="text-gray-400 hover:text-gray-600 text-xl leading-none">&times;</button>
                </div>
                <div className="flex items-center justify-between text-xs text-gray-500">
                    <span>{contracts.length} deployed</span>
                    <button
                        onClick={() => setAutoRefresh(!autoRefresh)}
                        className={`flex items-center ${autoRefresh ? 'text-blue-600' : 'text-gray-400'}`}
                    >
                        <RefreshCw className="w-3 h-3 mr-1" />
                        Auto
                    </button>
                </div>
            </div>

            <div className="flex-1 overflow-y-auto p-4 space-y-3">
                {contracts.length === 0 && (
                    <div className="text-center py-8">
                        <Activity className="w-8 h-8 text-gray-300 mx-auto mb-2" />
                        <p className="text-sm text-gray-500">No deployed contracts</p>
                        <p className="text-xs text-gray-400 mt-1">Compile and deploy from the toolbar</p>
                    </div>
                )}

                {contracts.map((c, i) => (
                    <div key={i} className="p-3 border border-gray-200 rounded-lg">
                        <div className="flex items-center justify-between mb-1">
                            <span className="text-sm font-medium text-gray-900 truncate max-w-[140px]">{c.address.slice(0, 16)}...</span>
                            {c.status === 'confirmed' ? (
                                <CheckCircle className="w-4 h-4 text-green-500" />
                            ) : c.status === 'failed' ? (
                                <XCircle className="w-4 h-4 text-red-500" />
                            ) : (
                                <RefreshCw className="w-4 h-4 text-yellow-500 animate-spin" />
                            )}
                        </div>
                        <div className="text-xs text-gray-500 truncate">Tx: {c.txHash.slice(0, 20)}...</div>
                        <div className="flex items-center justify-between mt-1 text-xs text-gray-400">
                            <span>Gas: {c.gasUsed.toLocaleString()}</span>
                            <button onClick={() => onRefresh(c.address)} className="text-blue-500 hover:text-blue-700">
                                <ExternalLink className="w-3 h-3" />
                            </button>
                        </div>
                    </div>
                ))}
            </div>
        </div>
    )
}
