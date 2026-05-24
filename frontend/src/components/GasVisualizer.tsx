import { Flame } from 'lucide-react'

export const getNodeGasCost = (nodeType: string): number => {
    switch (nodeType) {
        // Low Gas nodes (<= 10)
        case 'Start':
        case 'End':
            return 0
        case 'Not':
            return 3
        case 'And':
        case 'Or':
        case 'Add':
        case 'Subtract':
            return 5
        case 'Multiply':
        case 'Divide':
            return 5
        case 'If':
        case 'GetSender':
        case 'GetContractId':
        case 'GetBlockTimestamp':
        case 'GetBlockHeight':
        case 'Revert':
        case 'ReadCallResult':
            return 10

        // Medium Gas nodes (11 - 150)
        case 'EmitDormancyOracleResult':
        case 'NormalizeDeadCoinRisk':
            return 30
        case 'DecodeProof':
        case 'CalculateDormancyScore':
        case 'EmitEvent':
            return 50
        case 'CheckTokenAge':
            return 50
        case 'VerifySignature':
        case 'HashSha256':
        case 'ReadStorage':
        case 'CheckTokenActivityWindow':
        case 'CheckLiquidityDormancy':
        case 'CheckGovernanceDormancy':
            return 100
        case 'ExtractChronoEvent':
            return 100
        case 'TransferValue':
            return 150

        // High Gas nodes (> 150)
        case 'ExtractTxBySender':
        case 'ExtractTxByRecipient':
        case 'GenerateDormancyProof':
            return 200
        case 'FetchCheckpoint':
            return 300
        case 'CallContract':
        case 'FetchChronoBlock':
        case 'VerifyChronoProof':
        case 'VerifyArchiveRange':
            return 500

        default:
            return 10
    }
}

export const getGasCostColor = (gas: number): { border: string; bg: string; text: string; label: string } => {
    if (gas <= 10) {
        return {
            border: 'border-emerald-500',
            bg: 'bg-emerald-950/20 hover:bg-emerald-950/30',
            text: 'text-emerald-400',
            label: 'Low Gas'
        }
    } else if (gas <= 150) {
        return {
            border: 'border-amber-500',
            bg: 'bg-amber-950/20 hover:bg-amber-950/30',
            text: 'text-amber-400',
            label: 'Medium Gas'
        }
    } else {
        return {
            border: 'border-rose-500',
            bg: 'bg-rose-950/20 hover:bg-rose-950/30',
            text: 'text-rose-400',
            label: 'High Gas'
        }
    }
}

interface GasVisualizerProps {
    active: boolean
    onToggle: () => void
}

export const GasVisualizer: React.FC<GasVisualizerProps> = ({ active, onToggle }) => {
    return (
        <div className="absolute bottom-4 left-4 z-50 bg-slate-900 border border-slate-800 rounded-lg p-3 w-56 shadow-2xl">
            <div className="flex items-center justify-between mb-2">
                <span className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
                    <Flame className={`w-4 h-4 ${active ? 'text-amber-500 fill-amber-500 animate-pulse' : 'text-slate-400'}`} />
                    Gas Heatmap Visualizer
                </span>
                <label className="relative inline-flex items-center cursor-pointer">
                    <input 
                        type="checkbox" 
                        checked={active} 
                        onChange={onToggle} 
                        className="sr-only peer" 
                    />
                    <div className="w-7 h-4 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-slate-400 after:border-slate-300 after:border after:rounded-full after:h-3 after:w-3 after:transition-all peer-checked:bg-amber-600 peer-checked:after:bg-slate-100"></div>
                </label>
            </div>

            {active && (
                <div className="space-y-1.5 pt-1.5 border-t border-slate-800/60 text-[10px]">
                    <div className="flex items-center justify-between text-slate-400">
                        <span className="flex items-center gap-1">
                            <span className="w-2.5 h-2.5 rounded bg-emerald-500/20 border border-emerald-500" />
                            Low (≤ 10 Gas)
                        </span>
                        <span className="font-mono">e.g., Arithmetic, If</span>
                    </div>
                    <div className="flex items-center justify-between text-slate-400">
                        <span className="flex items-center gap-1">
                            <span className="w-2.5 h-2.5 rounded bg-amber-500/20 border border-amber-500" />
                            Medium (11 - 150 Gas)
                        </span>
                        <span className="font-mono">e.g., Read/Write Storage</span>
                    </div>
                    <div className="flex items-center justify-between text-slate-400">
                        <span className="flex items-center gap-1">
                            <span className="w-2.5 h-2.5 rounded bg-rose-500/20 border border-rose-500" />
                            High (&gt; 150 Gas)
                        </span>
                        <span className="font-mono">e.g., CallContract, Proofs</span>
                    </div>
                </div>
            )}
        </div>
    )
}
