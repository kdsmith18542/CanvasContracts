import React from 'react'
import {
    Play,
    Square,
    Plus,
    Minus,
    X,
    Divide,
    Database,
    Settings,
    Zap,
    Shield,
    Clock
} from 'lucide-react'

interface NodeCategory {
    name: string
    icon: React.ReactNode
    nodes: NodeDefinition[]
}

interface NodeDefinition {
    type: string
    name: string
    description: string
    icon: React.ReactNode
}

const nodeCategories: NodeCategory[] = [
    {
        name: 'Control',
        icon: <Play className="w-4 h-4" />,
        nodes: [
            {
                type: 'Start',
                name: 'Start',
                description: 'Contract entry point',
                icon: <Play className="w-4 h-4" />
            },
            {
                type: 'End',
                name: 'End',
                description: 'Contract exit point',
                icon: <Square className="w-4 h-4" />
            }
        ]
    },
    {
        name: 'Logic',
        icon: <Settings className="w-4 h-4" />,
        nodes: [
            {
                type: 'If',
                name: 'If Condition',
                description: 'Conditional execution',
                icon: <Settings className="w-4 h-4" />
            },
            {
                type: 'And',
                name: 'AND',
                description: 'Logical AND operation',
                icon: <Settings className="w-4 h-4" />
            },
            {
                type: 'Or',
                name: 'OR',
                description: 'Logical OR operation',
                icon: <Settings className="w-4 h-4" />
            },
            {
                type: 'Not',
                name: 'NOT',
                description: 'Logical NOT operation',
                icon: <Settings className="w-4 h-4" />
            }
        ]
    },
    {
        name: 'Arithmetic',
        icon: <Plus className="w-4 h-4" />,
        nodes: [
            {
                type: 'Add',
                name: 'Add',
                description: 'Add two numbers',
                icon: <Plus className="w-4 h-4" />
            },
            {
                type: 'Subtract',
                name: 'Subtract',
                description: 'Subtract two numbers',
                icon: <Minus className="w-4 h-4" />
            },
            {
                type: 'Multiply',
                name: 'Multiply',
                description: 'Multiply two numbers',
                icon: <X className="w-4 h-4" />
            },
            {
                type: 'Divide',
                name: 'Divide',
                description: 'Divide two numbers',
                icon: <Divide className="w-4 h-4" />
            }
        ]
    },
    {
        name: 'Storage',
        icon: <Database className="w-4 h-4" />,
        nodes: [
            {
                type: 'ReadStorage',
                name: 'Read Storage',
                description: 'Read from contract storage',
                icon: <Database className="w-4 h-4" />
            },
            {
                type: 'WriteStorage',
                name: 'Write Storage',
                description: 'Write to contract storage',
                icon: <Database className="w-4 h-4" />
            }
        ]
    }
]

export const NodePalette: React.FC = () => {
    const handleNodeDrag = (nodeType: string, event: React.DragEvent) => {
        event.dataTransfer.setData('application/reactflow', nodeType)
        event.dataTransfer.effectAllowed = 'move'
    }

    return (
        <div className="w-64 bg-white border-r border-gray-200 overflow-y-auto">
            <div className="p-4 border-b border-gray-200">
                <h2 className="text-lg font-semibold text-gray-900">Node Palette</h2>
                <p className="text-sm text-gray-500 mt-1">Drag nodes to the canvas</p>
            </div>

            <div className="p-4 space-y-6">
                {nodeCategories.map((category) => (
                    <div key={category.name}>
                        <div className="flex items-center mb-3">
                            {category.icon}
                            <h3 className="ml-2 text-sm font-medium text-gray-900">
                                {category.name}
                            </h3>
                        </div>

                        <div className="space-y-2">
                            {category.nodes.map((node) => (
                                <div
                                    key={node.type}
                                    draggable
                                    onDragStart={(e) => handleNodeDrag(node.type, e)}
                                    className="flex items-center p-2 text-sm text-gray-700 bg-gray-50 border border-gray-200 rounded-md cursor-move hover:bg-gray-100"
                                >
                                    {node.icon}
                                    <div className="ml-2">
                                        <div className="font-medium">{node.name}</div>
                                        <div className="text-xs text-gray-500">{node.description}</div>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>
                ))}
            </div>
        </div>
    )
} 