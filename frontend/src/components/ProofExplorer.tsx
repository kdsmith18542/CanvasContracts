import React, { useState } from 'react'
import { ShieldCheck, CheckCircle2, AlertTriangle, Play, HelpCircle, Key } from 'lucide-react'
import { TauriService } from '../services/tauriService'

export const ProofExplorer: React.FC = () => {
    const [proofJson, setProofJson] = useState(
        JSON.stringify(
            {
                height: 42,
                root: "0x4ecdc4a5b7d1e7fec9b59b68e44ad8e44ad8e44ad8e44ad8e44ad8e44ad8e44a",
                proof_data: "0x12b59f8101a182eceb1d82192b726d1dd6bdf2192b726d1d2b726d1dd6bdf219",
                valid: true
            },
            null,
            2
        )
    )
    const [result, setResult] = useState<'idle' | 'valid' | 'invalid'>('idle')
    const [loading, setLoading] = useState(false)
    const [error, setError] = useState<string | null>(null)

    const handleVerify = async () => {
        setLoading(true)
        setError(null)
        setResult('idle')
        try {
            const parsed = JSON.parse(proofJson)
            const isValid = await TauriService.verifyProof(parsed)
            setResult(isValid ? 'valid' : 'invalid')
        } catch (e: any) {
            setError(e.message || 'Invalid JSON format or verification error')
        } finally {
            setLoading(false)
        }
    }

    return (
        <div className="flex flex-col h-full bg-slate-900 text-slate-100 border-l border-slate-800 w-80">
            <div className="p-4 border-b border-slate-800 flex items-center justify-between">
                <h3 className="text-sm font-semibold flex items-center gap-2">
                    <ShieldCheck className="w-4 h-4 text-sky-400" />
                    Verifiable Proof Explorer
                </h3>
            </div>

            <div className="p-4 flex-1 overflow-y-auto space-y-4">
                <div>
                    <label className="block text-xs font-semibold text-slate-400 uppercase mb-2 flex items-center gap-1">
                        <Key className="w-3 h-3 text-sky-500" />
                        Merkle Proof Payload (JSON)
                    </label>
                    <textarea
                        value={proofJson}
                        onChange={(e) => setProofJson(e.target.value)}
                        rows={12}
                        className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-xs font-mono text-slate-300 focus:outline-none focus:border-sky-500 transition-all resize-none"
                    />
                </div>

                <button
                    onClick={handleVerify}
                    disabled={loading}
                    className="w-full bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white font-medium text-xs py-2 rounded flex items-center justify-center gap-1.5 transition-all shadow-lg shadow-sky-900/35"
                >
                    <Play className="w-3.5 h-3.5 fill-current" />
                    Verify Proof Validity
                </button>

                <div className="border-t border-slate-800 pt-4 space-y-3">
                    <h4 className="text-xs font-semibold text-slate-400 uppercase flex items-center gap-1">
                        <HelpCircle className="w-3.5 h-3.5 text-slate-400" />
                        Verification Status
                    </h4>

                    {loading && (
                        <div className="p-3 bg-slate-950/40 border border-slate-800 rounded flex items-center gap-2 text-xs text-slate-400">
                            <span className="w-2 h-2 rounded-full bg-sky-500 animate-ping" />
                            Checking with ChronoNode...
                        </div>
                    )}

                    {error && (
                        <div className="p-3 bg-red-950/40 border border-red-800 rounded flex items-start gap-2 text-xs text-red-300">
                            <AlertTriangle className="w-4 h-4 text-red-500 shrink-0 mt-0.5" />
                            <div>
                                <span className="font-semibold block">Verification Failed</span>
                                {error}
                            </div>
                        </div>
                    )}

                    {result === 'valid' && (
                        <div className="p-3 bg-emerald-950/40 border border-emerald-800 rounded flex items-start gap-2.5 text-xs text-emerald-300">
                            <CheckCircle2 className="w-4.5 h-4.5 text-emerald-400 shrink-0" />
                            <div>
                                <span className="font-semibold block text-emerald-400 text-sm">Valid Proof</span>
                                Verifiable against ChronoNode anchor block. Target hash verified matching state commitment.
                            </div>
                        </div>
                    )}

                    {result === 'invalid' && (
                        <div className="p-3 bg-red-950/40 border border-red-800 rounded flex items-start gap-2.5 text-xs text-red-300">
                            <AlertTriangle className="w-4.5 h-4.5 text-red-400 shrink-0 animate-bounce" />
                            <div>
                                <span className="font-semibold block text-red-400 text-sm">Invalid Proof</span>
                                Merkle root mismatch. The provided proof is not anchored to a valid block checkpoints chain.
                            </div>
                        </div>
                    )}

                    {result === 'idle' && !loading && (
                        <div className="text-center py-6 text-slate-500 text-xs italic">
                            Enter proof payload and click verify to check status.
                        </div>
                    )}
                </div>
            </div>
        </div>
    )
}
