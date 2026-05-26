import { useState, useEffect } from 'react'
import { useParams, Link } from 'react-router-dom'

// 1. Projects Page
export function ProjectsPage() {
  const [projects, setProjects] = useState<any[]>([])
  
  useEffect(() => {
    // Generate some mock Canvas visual contract projects
    setProjects([
      {
        name: "Resurgence Reward Distributor",
        desc: "Visual contract logic for staking rewards, token minting role governance, and legacy fallback price feeds.",
        graphHash: "0x3f1e0400fb8f19fefa8aa6b8d23468949e73a7b5",
        wasmHash: "0x12b909ce63794aecb8f86b93147562dbfd7c4156b0b784020e2d95cfc0663584",
        updatedAt: "2026-05-24 14:12"
      },
      {
        name: "DeadCoin Pool Custody Manager",
        desc: "Staking TVL manager, dynamic emission rates calculation, and Arbitrum Sepolia spoke contract relays.",
        graphHash: "0x71c56x917088d3745f3f4f19c8b8f1041bc73a9f",
        wasmHash: "0xfc0663584d610bad57026bbabe97c6a477d9ebee9b52ea26c2f9a47b988d3112",
        updatedAt: "2026-05-23 09:24"
      }
    ])
  }, [])

  return (
    <div className="space-y-6 text-left">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Contract Projects Registry</h1>
        <p className="text-gray-400 text-sm mt-1">Browse visual contract source graphs and compiled WASM bytecode artifacts.</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {projects.map((p) => (
          <div key={p.graphHash} className="bg-gray-800 border border-gray-700/60 rounded-xl p-6 flex flex-col justify-between hover:border-gray-500 transition-colors">
            <div>
              <div className="flex justify-between items-start mb-2">
                <h3 className="text-lg font-semibold text-white">{p.name}</h3>
                <span className="text-xs font-bold px-2 py-0.5 rounded bg-blue-900/30 text-blue-400 border border-blue-800/20">WASM</span>
              </div>
              <p className="text-gray-400 text-sm leading-relaxed mb-4">{p.desc}</p>
            </div>

            <div className="space-y-2 border-t border-gray-700/40 pt-4 text-xs text-gray-500">
              <div className="flex justify-between">
                <span>Source Graph Hash</span>
                <Link to={`/canvas/graphs/${p.graphHash}`} className="font-mono text-blue-400 hover:underline">{p.graphHash.slice(0, 12)}...</Link>
              </div>
              <div className="flex justify-between">
                <span>WASM Bytecode Hash</span>
                <Link to={`/canvas/artifacts/${p.wasmHash}`} className="font-mono text-blue-400 hover:underline">{p.wasmHash.slice(0, 12)}...</Link>
              </div>
              <div className="flex justify-between mt-2 pt-2 border-t border-gray-700/20 text-[10px]">
                <span>Last Updated</span>
                <span>{p.updatedAt}</span>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

// 2. Graph Detail Page
export function GraphDetailPage() {
  const { graphHash } = useParams()
  
  // Deterministic mock nodes based on graphHash
  let sum = 0;
  for (let i = 0; i < (graphHash || '').length; i++) {
    sum += (graphHash || '').charCodeAt(i)
  }

  const nodes = [
    { id: "1", type: "Input", label: "staker (Address)", x: 80, y: 120 },
    { id: "2", type: "Input", label: "amount (Uint256)", x: 80, y: 220 },
    { id: "3", type: "Logic", label: "verifySigner (PoA)", x: 260, y: 150 },
    { id: "4", type: "Action", label: "distributeRewards (State Update)", x: 440, y: 180 }
  ]

  return (
    <div className="space-y-6 text-left">
      <div className="text-sm text-gray-400 flex items-center gap-2">
        <Link to="/canvas/projects" className="hover:text-white transition-colors">Registry</Link>
        <span>/</span>
        <span className="text-white font-medium">Source Graph</span>
      </div>

      <div className="flex flex-col sm:flex-row justify-between items-start gap-4">
        <div>
          <span className="text-xs font-semibold text-blue-400 uppercase tracking-widest">Source Logic Graph</span>
          <h1 className="text-2xl font-bold text-white mt-1 break-all bg-gray-950 px-3 py-2 rounded-lg border border-gray-800 font-mono">
            {graphHash}
          </h1>
        </div>
      </div>

      {/* Visual Canvas Representation */}
      <div className="bg-gray-950 border border-gray-800 rounded-xl p-8 h-[360px] relative overflow-hidden flex items-center justify-center">
        <div className="absolute inset-0 bg-[linear-gradient(to_right,#1f2937_1px,transparent_1px),linear-gradient(to_bottom,#1f2937_1px,transparent_1px)] bg-[size:24px_24px] opacity-15" />
        
        {/* Render mockup of nodes connection */}
        <svg className="absolute inset-0 w-full h-full pointer-events-none">
          <path d="M 180 135 L 260 165" stroke="#3b82f6" strokeWidth="2" fill="none" strokeDasharray="4" />
          <path d="M 180 235 L 260 180" stroke="#3b82f6" strokeWidth="2" fill="none" strokeDasharray="4" />
          <path d="M 370 165 L 440 195" stroke="#10b981" strokeWidth="2" fill="none" />
        </svg>

        {nodes.map((node) => (
          <div
            key={node.id}
            style={{ left: `${node.x}px`, top: `${node.y}px` }}
            className={`absolute px-4 py-2.5 rounded-lg border text-xs font-semibold select-none shadow-lg ${
              node.type === 'Input' ? 'bg-blue-950/80 border-blue-800 text-blue-300' :
              node.type === 'Logic' ? 'bg-purple-950/80 border-purple-800 text-purple-300' :
              'bg-emerald-950/80 border-emerald-800 text-emerald-300'
            }`}
          >
            <span className="block text-[9px] uppercase opacity-60 font-bold mb-0.5">{node.type}</span>
            {node.label}
          </div>
        ))}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-gray-800 rounded-xl p-6 border border-gray-700 space-y-4">
          <h3 className="text-lg font-semibold text-white">Graph Details</h3>
          <div className="text-sm text-gray-400 space-y-3">
            <div className="flex justify-between border-b border-gray-700/40 pb-2">
              <span>Node Count</span>
              <span className="text-white font-semibold">4 Nodes</span>
            </div>
            <div className="flex justify-between border-b border-gray-700/40 pb-2">
              <span>Logical Connections</span>
              <span className="text-white font-semibold">3 Edges</span>
            </div>
            <div className="flex justify-between pb-2">
              <span>Visual Editor Version</span>
              <span className="text-white font-semibold">v0.1.0</span>
            </div>
          </div>
        </div>

        <div className="bg-gray-800 rounded-xl p-6 border border-gray-700 flex flex-col justify-between">
          <div>
            <h3 className="text-lg font-semibold text-white mb-2">Build Target Code</h3>
            <p className="text-gray-400 text-sm leading-relaxed mb-4">
              Compile this visual logic graph directly to Rust WIT assembly exports.
            </p>
          </div>
          <Link to={`/canvas/artifacts/0x12b909ce63794aecb8f86b93147562dbfd7c4156b0b784020e2d95cfc0663584`} className="glow-btn text-center">
            Inspect Compiled WASM Bytecode
          </Link>
        </div>
      </div>
    </div>
  )
}

// 3. Artifact Detail Page
export function ArtifactDetailPage() {
  const { wasmHash } = useParams()

  const witInterface = `interface reward-distributor {
  record-claim: func(staker: address, amount: u64, nonce: string) -> result<u64, string>;
  set-oracle: func(oracle: address) -> result<bool, string>;
  calculate-dynamic-rate: func(tvl: u64) -> u32;
}`;

  return (
    <div className="space-y-6 text-left">
      <div className="text-sm text-gray-400 flex items-center gap-2">
        <Link to="/canvas/projects" className="hover:text-white transition-colors">Registry</Link>
        <span>/</span>
        <span className="text-white font-medium">WASM Artifact</span>
      </div>

      <div className="flex flex-col sm:flex-row justify-between items-start gap-4">
        <div>
          <span className="text-xs font-semibold text-blue-400 uppercase tracking-widest">WASM Compiled Bytecode</span>
          <h1 className="text-2xl font-bold text-white mt-1 break-all bg-gray-950 px-3 py-2 rounded-lg border border-gray-800 font-mono">
            {wasmHash}
          </h1>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="space-y-6">
          {/* Metadata */}
          <div className="bg-gray-800 rounded-xl p-6 border border-gray-700 space-y-4">
            <h3 className="text-lg font-semibold text-white">Compilation Specs</h3>
            <div className="text-sm text-gray-400 space-y-3">
              <div className="flex justify-between border-b border-gray-700/40 pb-2">
                <span>Optimizer Level</span>
                <span className="text-white font-semibold">-O3 (zstd compressed)</span>
              </div>
              <div className="flex justify-between border-b border-gray-700/40 pb-2">
                <span>Compiler Target</span>
                <span className="text-white font-semibold">wasm32-wasi (wit-bindgen)</span>
              </div>
              <div className="flex justify-between border-b border-gray-700/40 pb-2">
                <span>Gas Metering Overhead</span>
                <span className="text-green-400 font-bold">1.2% (HostState Metred)</span>
              </div>
              <div className="flex justify-between">
                <span>ChronoNode Archive Pointer</span>
                <a href="https://chrono.baals.network/proofs" className="text-blue-400 font-mono hover:underline" target="_blank" rel="noopener noreferrer">chrono://artifacts/rwd_dist.wasm</a>
              </div>
            </div>
          </div>

          {/* Validation report */}
          <div className="bg-gray-800 rounded-xl p-6 border border-gray-700 space-y-3">
            <div className="flex justify-between items-center">
              <h3 className="text-lg font-semibold text-white">Security Validation Report</h3>
              <span className="text-xs font-semibold px-2 py-0.5 bg-green-500/10 text-green-400 border border-green-500/20 rounded-full">Secure ✓</span>
            </div>
            <div className="text-sm text-gray-400 space-y-2 pt-2">
              <div className="flex justify-between"><span>Compiler Warnings</span><span className="text-white font-mono">0 warnings</span></div>
              <div className="flex justify-between"><span>Unsafe Blocks Count</span><span className="text-white font-mono">0 unsafe</span></div>
              <div className="flex justify-between"><span>Heap Memory Limits</span><span className="text-white font-mono">16 MB max</span></div>
            </div>
          </div>
        </div>

        {/* WIT Interface Definition */}
        <div className="bg-gray-800 rounded-xl p-6 border border-gray-700 flex flex-col">
          <h3 className="text-lg font-semibold text-white mb-2">WIT Export Interface</h3>
          <p className="text-gray-400 text-xs leading-relaxed mb-4">
            Component model WIT exports mapping to WASM runtime entry points.
          </p>
          <pre className="flex-1 bg-gray-950 p-4 rounded-xl border border-gray-700 font-mono text-xs text-green-400 whitespace-pre overflow-x-auto">
            {witInterface}
          </pre>
        </div>
      </div>
    </div>
  )
}

// 4. Manifest Detail Page
export function ManifestDetailPage() {
  const { manifestHash } = useParams()

  return (
    <div className="space-y-6 text-left">
      <div className="text-sm text-gray-400 flex items-center gap-2">
        <Link to="/canvas/projects" className="hover:text-white transition-colors">Registry</Link>
        <span>/</span>
        <span className="text-white font-medium">Manifest Details</span>
      </div>

      <div className="flex flex-col sm:flex-row justify-between items-start gap-4">
        <div>
          <span className="text-xs font-semibold text-blue-400 uppercase tracking-widest">Deployment Manifest Schema</span>
          <h1 className="text-2xl font-bold text-white mt-1 break-all bg-gray-950 px-3 py-2 rounded-lg border border-gray-800 font-mono">
            {manifestHash}
          </h1>
        </div>
      </div>

      <div className="bg-gray-800 rounded-xl p-6 border border-gray-700 space-y-4">
        <h2 className="text-lg font-semibold text-white">Manifest Metadata</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-gray-400">
          <div className="flex justify-between py-2 border-b border-gray-700/60">
            <span>Compiler Engine</span>
            <span className="text-white">Canvas Compiler v0.1.0</span>
          </div>
          <div className="flex justify-between py-2 border-b border-gray-700/60">
            <span>WIT Bindgen Version</span>
            <span className="text-white font-mono">v0.12.0</span>
          </div>
          <div className="flex justify-between py-2 border-b border-gray-700/60">
            <span>Verification Signature</span>
            <span className="text-white font-mono">SIG_ED25519_88bc7a...</span>
          </div>
          <div className="flex justify-between py-2 border-b border-gray-700/60">
            <span>Target ABI Version</span>
            <span className="text-white font-mono">BaaLS WASM ABI v1</span>
          </div>
        </div>
      </div>
    </div>
  )
}

// 5. Deployment Detail Page
export function DeploymentDetailPage() {
  const { txHash } = useParams()

  return (
    <div className="space-y-6 text-left">
      <div className="text-sm text-gray-400 flex items-center gap-2">
        <Link to="/canvas/projects" className="hover:text-white transition-colors">Registry</Link>
        <span>/</span>
        <span className="text-white font-medium">Deployment Log</span>
      </div>

      <div className="flex flex-col sm:flex-row justify-between items-start gap-4">
        <div>
          <span className="text-xs font-semibold text-blue-400 uppercase tracking-widest">WASM Contract Deployment Transaction</span>
          <h1 className="text-2xl font-bold text-white mt-1 break-all bg-gray-950 px-3 py-2 rounded-lg border border-gray-800 font-mono">
            {txHash}
          </h1>
        </div>
      </div>

      <div className="bg-gray-800 rounded-xl p-6 border border-gray-700 space-y-4">
        <h2 className="text-lg font-semibold text-white">Execution Metrics</h2>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-gray-400">
          <div className="flex justify-between py-2 border-b border-gray-700/60">
            <span>Target Contract ID</span>
            <span className="text-white font-mono">contract_reward_distributor</span>
          </div>
          <div className="flex justify-between py-2 border-b border-gray-700/60">
            <span>Deployer Account</span>
            <span className="text-white font-mono">0x201624cBa366250D08bCdA95e6eF64151687A447</span>
          </div>
          <div className="flex justify-between py-2 border-b border-gray-700/60">
            <span>Included Block Height</span>
            <span className="text-white font-mono">#14,105</span>
          </div>
          <div className="flex justify-between py-2 border-b border-gray-700/60">
            <span>Gas Metering Fees</span>
            <span className="text-green-400 font-mono">42,500 gas</span>
          </div>
        </div>

        <div className="pt-4 flex flex-col gap-2">
          <span className="text-xs uppercase text-gray-500 font-mono">BaaLS Block Explorer</span>
          <div className="flex justify-between items-center bg-gray-950 p-4 rounded-xl border border-gray-700">
            <span className="text-xs text-gray-400 font-mono">Verify state and events recorded inside local ledger:</span>
            <a href="http://127.0.0.1:4173#explorer" target="_blank" rel="noopener noreferrer" className="glow-btn text-xs font-semibold py-1.5 px-4">
              Open BaaLS Explorer ↗
            </a>
          </div>
        </div>
      </div>
    </div>
  )
}

// 6. Project Detail Page
export function ProjectDetailPage() {
  const { projectId } = useParams()

  return (
    <div className="space-y-6 text-left">
      <div className="text-sm text-gray-400 flex items-center gap-2">
        <Link to="/canvas/projects" className="hover:text-white transition-colors">Registry</Link>
        <span>/</span>
        <span className="text-white font-medium">Project Detail</span>
      </div>

      <div>
        <span className="text-xs font-semibold text-blue-400 uppercase tracking-widest">Project ID</span>
        <h1 className="text-2xl font-bold text-white mt-1 break-all bg-gray-950 px-3 py-2 rounded-lg border border-gray-800 font-mono">
          {projectId}
        </h1>
      </div>

      <div className="bg-gray-800 rounded-xl p-6 border border-gray-700 space-y-4">
        <h2 className="text-lg font-semibold text-white">Project Specs</h2>
        <div className="text-sm text-gray-400 space-y-2">
          <div className="flex justify-between"><span>Owner Identity</span><span className="text-white">Local Developer Node</span></div>
          <div className="flex justify-between"><span>Graph Versions</span><span className="text-white">v0.1.0</span></div>
          <div className="flex justify-between"><span>Validation Status</span><span className="text-green-400 font-semibold">Verified</span></div>
        </div>
      </div>
    </div>
  )
}

// 7. WIT Interface Detail Page
export function WitInterfaceDetailPage() {
  const { witPackage } = useParams()

  return (
    <div className="space-y-6 text-left">
      <div className="text-sm text-gray-400 flex items-center gap-2">
        <Link to="/canvas/projects" className="hover:text-white transition-colors">Registry</Link>
        <span>/</span>
        <span className="text-white font-medium">WIT Interface</span>
      </div>

      <div>
        <span className="text-xs font-semibold text-blue-400 uppercase tracking-widest">WIT Package Name</span>
        <h1 className="text-2xl font-bold text-white mt-1 break-all bg-gray-950 px-3 py-2 rounded-lg border border-gray-800 font-mono">
          {witPackage}
        </h1>
      </div>

      <div className="bg-gray-800 rounded-xl p-6 border border-gray-700 space-y-4">
        <h2 className="text-lg font-semibold text-white">WIT Export Definition</h2>
        <pre className="bg-gray-950 p-4 rounded-xl border border-gray-700 font-mono text-xs text-green-400 whitespace-pre overflow-x-auto">
{`interface ${witPackage || 'witness'} {
  execute-trigger: func(payload: list<u8>) -> result<list<u8>, string>;
}`}
        </pre>
      </div>
    </div>
  )
}

// 8. Security Report Detail Page
export function SecurityReportDetailPage() {
  const { reportId } = useParams()

  return (
    <div className="space-y-6 text-left">
      <div className="text-sm text-gray-400 flex items-center gap-2">
        <Link to="/canvas/projects" className="hover:text-white transition-colors">Registry</Link>
        <span>/</span>
        <span className="text-white font-medium">Security Report</span>
      </div>

      <div>
        <span className="text-xs font-semibold text-blue-400 uppercase tracking-widest">Report Hash</span>
        <h1 className="text-2xl font-bold text-white mt-1 break-all bg-gray-950 px-3 py-2 rounded-lg border border-gray-800 font-mono">
          {reportId}
        </h1>
      </div>

      <div className="bg-gray-800 rounded-xl p-6 border border-gray-700 space-y-4">
        <div className="flex justify-between items-center">
          <h2 className="text-lg font-semibold text-white">Validation Results</h2>
          <span className="text-xs font-semibold px-2 py-0.5 bg-green-500/10 text-green-400 border border-green-500/20 rounded-full">Secure ✓</span>
        </div>
        <div className="text-sm text-gray-400 space-y-2">
          <div className="flex justify-between"><span>Compiler Warnings</span><span className="text-white font-mono">0 warnings</span></div>
          <div className="flex justify-between"><span>Unsafe Blocks Count</span><span className="text-white font-mono">0 unsafe</span></div>
          <div className="flex justify-between"><span>Heap Memory Limits</span><span className="text-white font-mono">16 MB max</span></div>
        </div>
      </div>
    </div>
  )
}

// 9. Templates Page
export function TemplatesPage() {
  return (
    <div className="space-y-6 text-left">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Ecosystem Logic Templates</h1>
        <p className="text-gray-400 text-sm mt-1">Ready-to-use visual templates for deploying custom BaaLS oracle adapters.</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-gray-800 border border-gray-700/60 rounded-xl p-6 flex flex-col justify-between hover:border-gray-500 transition-colors">
          <div>
            <h3 className="text-lg font-semibold text-white mb-2">UTXO Dormancy Monitor</h3>
            <p className="text-gray-400 text-sm leading-relaxed mb-4">
              Pre-wired logic template to watched UTXO inputs, verifying block timestamps against time thresholds.
            </p>
          </div>
          <div className="text-xs text-blue-400 font-semibold uppercase tracking-wider">Load Template →</div>
        </div>

        <div className="bg-gray-800 border border-gray-700/60 rounded-xl p-6 flex flex-col justify-between hover:border-gray-500 transition-colors">
          <div>
            <h3 className="text-lg font-semibold text-white mb-2">ERC20 Bridge Custody</h3>
            <p className="text-gray-400 text-sm leading-relaxed mb-4">
              Cross-chain CCIP staking reward manager template for handling deposits on remote L2 chains.
            </p>
          </div>
          <div className="text-xs text-blue-400 font-semibold uppercase tracking-wider">Load Template →</div>
        </div>
      </div>
    </div>
  )
}

// 10. Node Packs Page
export function NodePacksPage() {
  return (
    <div className="space-y-6 text-left">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Custom Node Packs</h1>
        <p className="text-gray-400 text-sm mt-1">Ecosystem logic packages extending the visual canvas palettes.</p>
      </div>

      <div className="grid grid-cols-1 gap-4">
        <div className="bg-gray-800 rounded-xl p-6 border border-gray-700 flex justify-between items-center">
          <div>
            <h3 className="text-base font-semibold text-white">ChronoNode Ingestion Pack</h3>
            <p className="text-sm text-gray-400">Nodes for querying indexer endpoints, Merkle checkpoints, and fetching CAS pointer payloads.</p>
          </div>
          <span className="text-xs font-bold px-2 py-1 rounded bg-blue-900/30 text-blue-400 border border-blue-800/20">Installed</span>
        </div>

        <div className="bg-gray-800 rounded-xl p-6 border border-gray-700 flex justify-between items-center">
          <div>
            <h3 className="text-base font-semibold text-white">CCIP Bridge Node Pack</h3>
            <p className="text-sm text-gray-400">Nodes for encoding CCIP messaging payloads, target routers mapping, and gas metering.</p>
          </div>
          <span className="text-xs font-bold px-2 py-1 rounded bg-blue-900/30 text-blue-400 border border-blue-800/20">Installed</span>
        </div>
      </div>
    </div>
  )
}
