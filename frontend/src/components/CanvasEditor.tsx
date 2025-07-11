import React, { useState, useCallback } from 'react'
import ReactFlow, {
    Node,
    Edge,
    addEdge,
    Connection,
    useNodesState,
    useEdgesState,
    Controls,
    Background,
    MiniMap,
} from 'react-flow-renderer'
import 'react-flow-renderer/dist/style.css'

const initialNodes: Node[] = [
    {
        id: '1',
        type: 'input',
        data: { label: 'Start' },
        position: { x: 250, y: 25 },
    },
]

const initialEdges: Edge[] = []

export const CanvasEditor: React.FC = () => {
    const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes)
    const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges)

    const onConnect = useCallback(
        (params: Connection) => setEdges((eds) => addEdge(params, eds)),
        [setEdges],
    )

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
                data: { label: type },
            }

            setNodes((nds) => nds.concat(newNode))
        },
        [setNodes],
    )

    return (
        <div className="flex-1 bg-gray-50">
            <ReactFlow
                nodes={nodes}
                edges={edges}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
                onConnect={onConnect}
                onDragOver={onDragOver}
                onDrop={onDrop}
                fitView
            >
                <Controls />
                <Background />
                <MiniMap />
            </ReactFlow>
        </div>
    )
} 