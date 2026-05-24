import React, { useState, useCallback, useEffect } from 'react'
import {
    ReactFlow,
    Node,
    Edge,
    addEdge,
    Connection,
    useNodesState,
    useEdgesState,
    Controls,
    Background,
    MiniMap,
    ReactFlowProvider,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { nodeTypes } from './nodes'
import { useCanvasStore } from '../store/useCanvasStore'
import { GasVisualizer } from './GasVisualizer'

const initialNodes: Node[] = [
    {
        id: '1',
        type: 'start',
        data: { label: 'Start', gasVisualizerActive: false },
        position: { x: 250, y: 25 },
    },
]

const initialEdges: Edge[] = []

interface CanvasEditorInnerProps {
    onNodeSelect: (node: Node | null) => void
}

const CanvasEditorInner: React.FC<CanvasEditorInnerProps> = ({ onNodeSelect }) => {
    const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes)
    const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges)
    const [gasActive, setGasActive] = useState(false)
    const { addNode: storeAddNode, addEdge: storeAddEdge, undo, redo } = useCanvasStore()

    const toggleGas = useCallback(() => {
        setGasActive(active => {
            const next = !active;
            setNodes(nds => nds.map(n => ({
                ...n,
                data: {
                    ...n.data,
                    gasVisualizerActive: next
                }
            })));
            return next;
        });
    }, [setNodes]);

    const onConnect = useCallback(
        (params: Connection) => {
            setEdges((eds) => {
                const newEdges = addEdge(params, eds)
                storeAddEdge(params)
                return newEdges
            })
        },
        [setEdges, storeAddEdge],
    )

    const onNodeClick = useCallback(
        (_event: React.MouseEvent, node: Node) => {
            onNodeSelect(node)
        },
        [onNodeSelect],
    )

    const onPaneClick = useCallback(() => {
        onNodeSelect(null)
    }, [onNodeSelect])

    const onDragOver = useCallback((event: React.DragEvent) => {
        event.preventDefault()
        event.dataTransfer.dropEffect = 'move'
    }, [])

    const onDrop = useCallback(
        (event: React.DragEvent) => {
            event.preventDefault()

            const type = event.dataTransfer.getData('application/reactflow')
            if (typeof type === 'undefined' || !type) {
                return
            }

            const position = {
                x: event.clientX - 250,
                y: event.clientY - 100,
            }

            const newNode: Node = {
                id: `${type}-${Date.now()}`,
                type: 'default',
                position,
                data: { label: type, gasVisualizerActive: gasActive },
            }

            setNodes((nds) => {
                storeAddNode(newNode)
                return nds.concat(newNode)
            })
        },
        [setNodes, storeAddNode, gasActive],
    )

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

    return (
        <div className="flex-1 bg-gray-50 relative h-full">
            <ReactFlow
                nodes={nodes}
                edges={edges}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
                onConnect={onConnect}
                onNodeClick={onNodeClick}
                onPaneClick={onPaneClick}
                onDragOver={onDragOver}
                onDrop={onDrop}
                nodeTypes={nodeTypes}
                fitView
            >
                <Controls />
                <Background />
                <MiniMap />
            </ReactFlow>
            <GasVisualizer active={gasActive} onToggle={toggleGas} />
        </div>
    )
}

interface CanvasEditorProps {
    onNodeSelect: (node: Node | null) => void
}

export const CanvasEditor: React.FC<CanvasEditorProps> = ({ onNodeSelect }) => {
    return (
        <ReactFlowProvider>
            <CanvasEditorInner onNodeSelect={onNodeSelect} />
        </ReactFlowProvider>
    )
}
