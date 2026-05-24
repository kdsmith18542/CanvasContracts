import React, { useState, useEffect } from 'react'
import { History, RefreshCw, Layers, Send, Bell, Loader } from 'lucide-react'
import { TauriService } from '../services/tauriService'

interface ContractHistoryProps {
    contractAddress: string | null
}

export const ContractHistory: React.FC<ContractHistoryProps> = ({ contractAddress }) => {
    const [activeTab, setActiveTab] = useState<'blocks' | 'transactions' | 'events'>('blocks')
    const [loading, setLoading] = useState(false)
    const [data, setData] = useState<any[]>([])
    const [error, setError] = useState<string | null>(null)

    const fetchData = async () => {
        if (!contractAddress) return
        setLoading(true)
        setError(null)
        try {
            const res = await TauriService.queryHistory(contractAddress, activeTab)
            if (Array.isArray(res)) {
                setData(res)
            } else if (res && typeof res === 'object') {
                setData(res.blocks || res.transactions || res.events || [res])
            } else {
                setData([])
            }
        } catch (e: any) {
            setError(e.message || 'Failed to fetch history')
        } finally {
            setLoading(false)
        }
    }

    useEffect(() => {
        fetchData()
    }, [contractAddress, activeTab])

    if (!contractAddress) {
        return (
            <div className="flex flex-col items-center justify-center h-full text-gray-400 p-8">
                <History className="w-12 h-12 mb-2 stroke-1" />
                <p className="text-sm">Select or deploy a contract to view history</p>
            </div>
        )
    }

    return (
        <div className="flex flex-col h-full bg-slate-900 text-slate-100 border-l border-slate-800 w-80">
            <div className="p-4 border-b border-slate-800 flex items-center justify-between">
                <h3 className="text-sm font-semibold flex items-center gap-2">
                    <History className="w-4 h-4 text-emerald-400" />
                    Contract History
                </h3>
                <button 
                    onClick={fetchData} 
                    disabled={loading} 
                    className="p-1 text-slate-400 hover:text-slate-200 disabled:opacity-50"
                >
                    <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
                </button>
            </div>

            <div className="flex border-b border-slate-800 bg-slate-950/50">
                <button
                    onClick={() => setActiveTab('blocks')}
                    className={`flex-1 py-2 text-xs font-medium border-b-2 transition-all flex justify-center items-center gap-1 ${
                        activeTab === 'blocks'
                            ? 'border-emerald-500 text-emerald-400 bg-emerald-500/5'
                            : 'border-transparent text-slate-400 hover:text-slate-200'
                    }`}
                >
                    <Layers className="w-3.5 h-3.5" />
                    Blocks
                </button>
                <button
                    onClick={() => setActiveTab('transactions')}
                    className={`flex-1 py-2 text-xs font-medium border-b-2 transition-all flex justify-center items-center gap-1 ${
                        activeTab === 'transactions'
                            ? 'border-emerald-500 text-emerald-400 bg-emerald-500/5'
                            : 'border-transparent text-slate-400 hover:text-slate-200'
                    }`}
                >
                    <Send className="w-3.5 h-3.5" />
                    Txs
                </button>
                <button
                    onClick={() => setActiveTab('events')}
                    className={`flex-1 py-2 text-xs font-medium border-b-2 transition-all flex justify-center items-center gap-1 ${
                        activeTab === 'events'
                            ? 'border-emerald-500 text-emerald-400 bg-emerald-500/5'
                            : 'border-transparent text-slate-400 hover:text-slate-200'
                    }`}
                >
                    <Bell className="w-3.5 h-3.5" />
                    Events
                </button>
            </div>

            <div className="flex-1 overflow-y-auto p-3 space-y-2">
                {loading && (
                    <div className="flex items-center justify-center py-12 text-slate-400 gap-2">
                        <Loader className="w-4 h-4 animate-spin text-emerald-500" />
                        <span className="text-xs">Loading ChronoNode...</span>
                    </div>
                )}

                {error && (
                    <div className="p-3 bg-red-950/50 border border-red-800 text-red-300 rounded text-xs">
                        {error}
                    </div>
                )}

                {!loading && !error && data.length === 0 && (
                    <div className="text-center py-12 text-slate-500 text-xs italic">
                        No historical records found
                    </div>
                )}

                {!loading && !error && data.length > 0 && data.map((item, idx) => (
                    <div key={idx} className="p-2.5 bg-slate-950/40 border border-slate-800 rounded hover:border-slate-700 transition-all text-xs font-mono">
                        {activeTab === 'blocks' && (
                            <div>
                                <div className="flex justify-between font-bold text-slate-300">
                                    <span>Block #{item.height}</span>
                                    <span className="text-emerald-500">{item.chain_id}</span>
                                </div>
                                <div className="text-[10px] text-slate-500 mt-1 truncate">Hash: {item.hash}</div>
                                <div className="text-[10px] text-slate-500">Time: {new Date(item.timestamp * 1000).toLocaleTimeString()}</div>
                            </div>
                        )}

                        {activeTab === 'transactions' && (
                            <div>
                                <div className="flex justify-between font-bold text-slate-300">
                                    <span>Tx Hash</span>
                                    <span className="text-sky-500">{item.chain_id}</span>
                                </div>
                                <div className="text-[10px] text-slate-400 mt-1 truncate">Hash: {item.hash}</div>
                                <div className="text-[10px] text-slate-500 mt-0.5 truncate">To: {item.recipient}</div>
                                <div className="text-[10px] text-slate-500">Value: {item.value} units</div>
                            </div>
                        )}

                        {activeTab === 'events' && (
                            <div>
                                <div className="flex justify-between font-bold text-emerald-400">
                                    <span>{item.type || 'Event'}</span>
                                    <span className="text-slate-500">{item.chain_id}</span>
                                </div>
                                <div className="text-[10px] text-slate-400 mt-1 bg-slate-900/60 p-1.5 rounded border border-slate-800/80">
                                    <pre className="overflow-x-auto whitespace-pre-wrap">{JSON.stringify(item.data, null, 2)}</pre>
                                </div>
                            </div>
                        )}
                    </div>
                ))}
            </div>
            <div className="p-3 border-t border-slate-800 bg-slate-950/20 text-[10px] text-slate-500 truncate">
                Target: {contractAddress}
            </div>
        </div>
    )
}
