import { useState, useEffect, useCallback } from 'react'
import { CanvasEditor } from './components/CanvasEditor'
import { NodePalette } from './components/NodePalette'
import { Toolbar } from './components/Toolbar'
import { AiAssistant } from './components/AiAssistant'
import { Debugger } from './components/Debugger'
import { CustomNodeCreator } from './components/CustomNodeCreator'
import { Marketplace } from './components/Marketplace'
import { PropertyPanel } from './components/PropertyPanel'
import { ContractMonitor, DeployedContract } from './components/ContractMonitor'
import { ContractHistory } from './components/ContractHistory'
import { ProofExplorer } from './components/ProofExplorer'
import { AuditExport } from './components/AuditExport'
import { useCanvasStore } from './store/useCanvasStore'
import { ProjectService } from './services/projectService'
import { DeployResult } from './services/tauriService'
import { VisualNode } from './types'

function App() {
    const [activeSidebar, setActiveSidebar] = useState<'none' | 'ai' | 'debugger' | 'monitor' | 'history' | 'proofs' | 'audit'>('ai')
    const [showCustomNodeCreator, setShowCustomNodeCreator] = useState(false)
    const [showMarketplace, setShowMarketplace] = useState(false)
    const [deployedContracts, setDeployedContracts] = useState<DeployedContract[]>([])
    const [selectedNode, setSelectedNode] = useState<{ id: string; type: string; data: { label: string; properties?: Record<string, any> } } | null>(null)
    const { graph, updateGraph, undo, redo } = useCanvasStore()

    const handleNodeSelect = useCallback((node: any) => {
        if (!node) {
            setSelectedNode(null)
            return
        }
        setSelectedNode({
            id: node.id,
            type: node.type,
            data: node.data,
        })
    }, [])

    const handlePropertyChange = useCallback((nodeId: string, propertyName: string, value: any) => {
        const updatedNodes = graph.nodes.map((n: VisualNode) => {
            if (n.id === nodeId) {
                return {
                    ...n,
                    data: {
                        ...n.data,
                        properties: {
                            ...(n.data.properties || {}),
                            [propertyName]: value,
                        },
                    },
                }
            }
            return n
        })
        updateGraph({ ...graph, nodes: updatedNodes })
    }, [graph, updateGraph])

    const handleDeploy = useCallback((result: DeployResult) => {
        const contract: DeployedContract = {
            address: result.contract_address,
            name: `Contract ${deployedContracts.length + 1}`,
            status: 'confirmed',
            txHash: result.transaction_hash,
            gasUsed: result.gas_used,
            timestamp: Date.now(),
        }
        setDeployedContracts(prev => [...prev, contract])
    }, [deployedContracts.length])

    const handleContractRefresh = useCallback((address: string) => {
        // Future: query chain for status update
        console.log('Refresh requested for:', address)
    }, [])

    const handleSave = useCallback(async () => {
        await ProjectService.saveProject(graph)
    }, [graph])

    const handleLoad = useCallback(async () => {
        const loaded = await ProjectService.loadProject()
        if (loaded) {
            updateGraph(loaded)
        }
    }, [updateGraph])

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
                e.preventDefault()
                undo()
            }
            if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
                e.preventDefault()
                redo()
            }
        }
        document.addEventListener('keydown', handleKeyDown)
        return () => document.removeEventListener('keydown', handleKeyDown)
    }, [undo, redo])

    const latestContractAddress = deployedContracts.length > 0
        ? deployedContracts[deployedContracts.length - 1].address
        : null

    return (
        <div className="h-screen flex flex-col bg-gray-50">
            <Toolbar
                activeSidebar={activeSidebar}
                onSidebarToggle={(sidebar) => {
                    setActiveSidebar(sidebar)
                    if (sidebar !== 'none') {
                        setShowMarketplace(false)
                    }
                }}
                showMarketplace={showMarketplace}
                onMarketplaceToggle={() => {
                    const next = !showMarketplace
                    setShowMarketplace(next)
                    if (next) {
                        setActiveSidebar('none')
                    }
                }}
                onCustomNodeToggle={() => setShowCustomNodeCreator(!showCustomNodeCreator)}
                onSave={handleSave}
                onLoad={handleLoad}
                onDeploy={handleDeploy}
            />
            <div className="flex-1 flex overflow-hidden">
                {showMarketplace ? (
                    <Marketplace />
                ) : (
                    <>
                        <NodePalette />
                        <CanvasEditor onNodeSelect={handleNodeSelect} />
                        
                        {selectedNode && (
                            <PropertyPanel
                                node={selectedNode}
                                onClose={() => setSelectedNode(null)}
                                onPropertyChange={handlePropertyChange}
                            />
                        )}

                        {activeSidebar === 'ai' && <AiAssistant />}
                        {activeSidebar === 'debugger' && <Debugger />}
                        {activeSidebar === 'monitor' && (
                            <ContractMonitor
                                contracts={deployedContracts}
                                onRefresh={handleContractRefresh}
                                onClose={() => setActiveSidebar('none')}
                            />
                        )}
                        {activeSidebar === 'history' && (
                            <ContractHistory contractAddress={latestContractAddress} />
                        )}
                        {activeSidebar === 'proofs' && <ProofExplorer />}
                        {activeSidebar === 'audit' && <AuditExport graph={graph} />}
                    </>
                )}
            </div>
            {showCustomNodeCreator && <CustomNodeCreator />}
        </div>
    )
}

export default App
