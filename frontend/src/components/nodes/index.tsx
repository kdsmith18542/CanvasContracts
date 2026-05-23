import React from 'react'
import { Handle, Position } from '@xyflow/react'

export const StartNode: React.FC<{ data: any; selected: boolean }> = ({ data, selected }) => (
    <div className={`px-4 py-2 rounded-lg border-2 ${selected ? 'border-blue-500 bg-blue-50' : 'border-green-500 bg-green-50'}`}>
        <div className="text-sm font-bold text-green-700">START</div>
        <div className="text-xs text-gray-600">{data.label}</div>
        <Handle type="source" position={Position.Right} />
    </div>
)

export const EndNode: React.FC<{ data: any; selected: boolean }> = ({ data, selected }) => (
    <div className={`px-4 py-2 rounded-lg border-2 ${selected ? 'border-blue-500 bg-blue-50' : 'border-red-500 bg-red-50'}`}>
        <div className="text-sm font-bold text-red-700">END</div>
        <div className="text-xs text-gray-600">{data.label}</div>
        <Handle type="target" position={Position.Left} />
    </div>
)

export const DefaultNode: React.FC<{ data: any; selected: boolean }> = ({ data, selected }) => (
    <div className={`px-4 py-2 rounded border-2 ${selected ? 'border-blue-500 bg-blue-50' : 'border-gray-300 bg-white'}`}>
        <Handle type="target" position={Position.Left} />
        <div className="text-sm font-medium text-gray-800">{data.label}</div>
        <div className="text-xs text-gray-500">{data.nodeType || ''}</div>
        <Handle type="source" position={Position.Right} />
    </div>
)

export const nodeTypes = {
    start: StartNode,
    end: EndNode,
    default: DefaultNode,
}
