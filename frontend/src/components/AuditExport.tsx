import { useState } from 'react'
import { FileDown, Download, CheckCircle, Loader, Shield } from 'lucide-react'
import { AuditBundleResult, TauriService } from '../services/tauriService'
import { VisualGraph } from '../types'

interface AuditExportProps {
    graph: VisualGraph
}

export const AuditExport: React.FC<AuditExportProps> = ({ graph }) => {
    const [loading, setLoading] = useState(false)
    const [bundle, setBundle] = useState<AuditBundleResult | null>(null)
    const [error, setError] = useState<string | null>(null)

    const handleGenerate = async () => {
        setLoading(true)
        setError(null)
        setBundle(null)
        try {
            const res = await TauriService.exportAuditBundle(graph)
            setBundle(res)
        } catch (e: any) {
            setError(e.message || 'Failed to export bundle')
        } finally {
            setLoading(false)
        }
    }

    const downloadJsonFile = (filename: string, data: any) => {
        const jsonStr = JSON.stringify(data, null, 2)
        const blob = new Blob([jsonStr], { type: 'application/json' })
        const url = URL.createObjectURL(blob)
        const link = document.createElement('a')
        link.href = url
        link.download = filename
        document.body.appendChild(link)
        link.click()
        document.body.removeChild(link)
        URL.revokeObjectURL(url)
    }

    const downloadWasmFile = (filename: string, hexStr: string) => {
        const bytes = new Uint8Array(hexStr.match(/.{1,2}/g)!.map(byte => parseInt(byte, 16)))
        const blob = new Blob([bytes], { type: 'application/wasm' })
        const url = URL.createObjectURL(blob)
        const link = document.createElement('a')
        link.href = url
        link.download = filename
        document.body.appendChild(link)
        link.click()
        document.body.removeChild(link)
        URL.revokeObjectURL(url)
    }

    const downloadTextFile = (filename: string, content: string) => {
        const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
        const url = URL.createObjectURL(blob)
        const link = document.createElement('a')
        link.href = url
        link.download = filename
        document.body.appendChild(link)
        link.click()
        document.body.removeChild(link)
        URL.revokeObjectURL(url)
    }

    return (
        <div className="flex flex-col h-full bg-slate-900 text-slate-100 border-l border-slate-800 w-80">
            <div className="p-4 border-b border-slate-800 flex items-center justify-between">
                <h3 className="text-sm font-semibold flex items-center gap-2">
                    <Shield className="w-4 h-4 text-purple-400" />
                    Audit & Manifest Export
                </h3>
            </div>

            <div className="p-4 flex-1 overflow-y-auto space-y-4">
                <p className="text-xs text-slate-400 leading-relaxed">
                    Generate the official audit manifest bundle containing the visual graph’s compilation lock, verification proof logs, and generated WASM bytecode.
                </p>

                <button
                    onClick={handleGenerate}
                    disabled={loading}
                    className="w-full bg-purple-600 hover:bg-purple-500 disabled:opacity-50 text-white font-medium text-xs py-2 rounded flex items-center justify-center gap-1.5 transition-all shadow-lg shadow-purple-900/35"
                >
                    <FileDown className="w-3.5 h-3.5" />
                    Generate Audit Bundle
                </button>

                {loading && (
                    <div className="flex items-center justify-center py-12 text-slate-400 gap-2">
                        <Loader className="w-4 h-4 animate-spin text-purple-500" />
                        <span className="text-xs">Packaging artifacts...</span>
                    </div>
                )}

                {error && (
                    <div className="p-3 bg-red-950/40 border border-red-800 text-red-300 rounded text-xs">
                        {error}
                    </div>
                )}

                {bundle && (
                    <div className="space-y-4">
                        <div className="bg-slate-950/40 border border-slate-800 rounded p-3 text-xs space-y-2">
                            <h4 className="font-semibold text-slate-300 flex items-center gap-1">
                                <CheckCircle className="w-4 h-4 text-emerald-400" />
                                Audit Bundle Generated
                            </h4>
                            <div className="text-[10px] text-slate-500 space-y-1">
                                <div>Compiler version: {bundle.manifest?.compiler?.version || bundle.lock?.compiler?.version}</div>
                                <div className="truncate">Graph Hash: {bundle.manifest?.source?.graph_hash || bundle.lock?.graph_hash}</div>
                                <div className="truncate">WASM Hash: {bundle.manifest?.artifact?.wasm_hash || bundle.lock?.wasm_hash}</div>
                                <div className="truncate">WIT Hash: {bundle.manifest?.abi?.wit_hash}</div>
                                <div className="text-emerald-400 font-semibold mt-1">Validation Status: {(bundle.manifest?.validation?.status || 'unknown').toUpperCase()}</div>
                            </div>
                        </div>

                        <div className="bg-slate-950/40 border border-slate-800 rounded p-3 text-xs space-y-2">
                            <h4 className="font-semibold text-slate-300">Contract Safety Report</h4>
                            <div className="text-[10px] text-slate-500 space-y-1">
                                <div>Status: {bundle.safety_report?.status || 'unknown'}</div>
                                <div>Runtime Profile: {bundle.safety_report?.target_profile || bundle.manifest?.runtime?.profile || 'n/a'}</div>
                                <div>Imports: {(bundle.safety_report?.wasm?.imports || []).length}</div>
                                <div>Storage Writes: {bundle.safety_report?.graph?.storage_writes ?? 'n/a'}</div>
                                <div>Gas Estimate: {bundle.safety_report?.gas?.estimate ?? 'n/a'}</div>
                                {(bundle.safety_report?.warnings || []).length > 0 && (
                                    <div className="text-amber-400">Warnings: {(bundle.safety_report?.warnings || []).join('; ')}</div>
                                )}
                                {(bundle.safety_report?.errors || []).length > 0 && (
                                    <div className="text-red-400">Errors: {(bundle.safety_report?.errors || []).join('; ')}</div>
                                )}
                            </div>
                        </div>

                        <div className="bg-slate-950/40 border border-slate-800 rounded p-3 text-xs space-y-2">
                            <h4 className="font-semibold text-slate-300">ChronoNode Archive</h4>
                            <div className="text-[10px] text-slate-500 space-y-1">
                                <div>Status: {bundle.archive?.status || 'not_archived'}</div>
                                <div className="truncate">Pointer: {bundle.archive?.storage_pointer || bundle.manifest?.archive?.chrononode_pointer || 'not archived'}</div>
                                <div className="truncate">Content Hash: {bundle.archive?.content_hash || 'not available'}</div>
                            </div>
                        </div>

                        <div className="bg-slate-950/40 border border-slate-800 rounded p-3 text-xs space-y-2">
                            <h4 className="font-semibold text-slate-300">WIT ABI Package</h4>
                            <div className="text-[10px] text-slate-500">
                                Package: {bundle.manifest?.abi?.wit_package || 'baals:contract@1.0.0'}
                            </div>
                            <div className="space-y-1">
                                {bundle.wit_files.map((file) => (
                                    <button
                                        key={file.name}
                                        onClick={() => downloadTextFile(file.name, file.content)}
                                        className="w-full bg-slate-900 hover:bg-slate-800 border border-slate-700 text-slate-200 font-medium text-xs py-1.5 px-2 rounded flex items-center justify-between transition-all"
                                    >
                                        <span className="font-mono text-[10px]">{file.name}</span>
                                        <Download className="w-3.5 h-3.5 text-slate-400" />
                                    </button>
                                ))}
                            </div>
                        </div>

                        <div className="space-y-2">
                            <button
                                onClick={() => downloadJsonFile(`${graph.name.toLowerCase()}.lock.json`, bundle.lock)}
                                className="w-full bg-slate-950 hover:bg-slate-900 border border-slate-800 text-slate-200 font-medium text-xs py-2 px-3 rounded flex items-center justify-between transition-all"
                            >
                                <span className="font-mono text-[10px]">graph.lock.json</span>
                                <Download className="w-3.5 h-3.5 text-slate-400" />
                            </button>

                            <button
                                onClick={() => downloadJsonFile(`${graph.name.toLowerCase()}.validation-report.json`, bundle.validation_report)}
                                className="w-full bg-slate-950 hover:bg-slate-900 border border-slate-800 text-slate-200 font-medium text-xs py-2 px-3 rounded flex items-center justify-between transition-all"
                            >
                                <span className="font-mono text-[10px]">validation-report.json</span>
                                <Download className="w-3.5 h-3.5 text-slate-400" />
                            </button>

                            <button
                                onClick={() => downloadJsonFile(`${graph.name.toLowerCase()}.abi.json`, bundle.abi)}
                                className="w-full bg-slate-950 hover:bg-slate-900 border border-slate-800 text-slate-200 font-medium text-xs py-2 px-3 rounded flex items-center justify-between transition-all"
                            >
                                <span className="font-mono text-[10px]">contract.abi.json</span>
                                <Download className="w-3.5 h-3.5 text-slate-400" />
                            </button>

                            <button
                                onClick={() => downloadWasmFile(`${graph.name.toLowerCase()}.wasm`, bundle.wasm_bytes)}
                                className="w-full bg-slate-950 hover:bg-slate-900 border border-slate-800 text-slate-200 font-medium text-xs py-2 px-3 rounded flex items-center justify-between transition-all"
                            >
                                <span className="font-mono text-[10px] text-purple-400">contract.wasm</span>
                                <Download className="w-3.5 h-3.5 text-slate-400" />
                            </button>

                            <button
                                onClick={() => downloadJsonFile(`${graph.name.toLowerCase()}.manifest.json`, bundle.manifest)}
                                className="w-full bg-slate-950 hover:bg-slate-900 border border-slate-800 text-slate-200 font-medium text-xs py-2 px-3 rounded flex items-center justify-between transition-all"
                            >
                                <span className="font-mono text-[10px] text-emerald-400">canvas.contract.json</span>
                                <Download className="w-3.5 h-3.5 text-slate-400" />
                            </button>

                            <button
                                onClick={() => downloadJsonFile(`${graph.name.toLowerCase()}.safety-report.json`, bundle.safety_report)}
                                className="w-full bg-slate-950 hover:bg-slate-900 border border-slate-800 text-slate-200 font-medium text-xs py-2 px-3 rounded flex items-center justify-between transition-all"
                            >
                                <span className="font-mono text-[10px]">safety-report.json</span>
                                <Download className="w-3.5 h-3.5 text-slate-400" />
                            </button>
                        </div>
                    </div>
                )}
            </div>
        </div>
    )
}
