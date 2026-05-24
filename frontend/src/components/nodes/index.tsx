import React from 'react'
import { Handle, Position } from '@xyflow/react'
import { getNodeGasCost, getGasCostColor } from '../GasVisualizer'

export const StartNode: React.FC<{ data: any; selected: boolean }> = ({ data, selected }) => {
    const isGasActive = data.gasVisualizerActive;
    const gas = getNodeGasCost('Start');
    const gasStyle = getGasCostColor(gas);
    
    const borderClass = selected ? 'border-blue-500' : (isGasActive ? gasStyle.border : 'border-green-500');
    const bgClass = isGasActive ? gasStyle.bg : 'bg-green-50';
    
    return (
        <div className={`px-4 py-2 rounded-lg border-2 ${borderClass} ${bgClass} transition-colors duration-200`}>
            <div className="text-sm font-bold text-green-700">START</div>
            <div className="text-xs text-gray-600">{data.label}</div>
            <Handle type="source" position={Position.Right} />
        </div>
    )
}

export const EndNode: React.FC<{ data: any; selected: boolean }> = ({ data, selected }) => {
    const isGasActive = data.gasVisualizerActive;
    const gas = getNodeGasCost('End');
    const gasStyle = getGasCostColor(gas);
    
    const borderClass = selected ? 'border-blue-500' : (isGasActive ? gasStyle.border : 'border-red-500');
    const bgClass = isGasActive ? gasStyle.bg : 'bg-red-50';

    return (
        <div className={`px-4 py-2 rounded-lg border-2 ${borderClass} ${bgClass} transition-colors duration-200`}>
            <div className="text-sm font-bold text-red-700">END</div>
            <div className="text-xs text-gray-600">{data.label}</div>
            <Handle type="target" position={Position.Left} />
        </div>
    )
}

export const DefaultNode: React.FC<{ data: any; selected: boolean }> = ({ data, selected }) => {
    const isGasActive = data.gasVisualizerActive;
    const nodeType = data.nodeType || data.label || 'Add';
    const gas = getNodeGasCost(nodeType);
    const gasStyle = getGasCostColor(gas);
    
    const borderClass = selected ? 'border-blue-500' : (isGasActive ? gasStyle.border : 'border-gray-300');
    const bgClass = isGasActive ? gasStyle.bg : 'bg-white';

    return (
        <div className={`px-4 py-2 rounded border-2 ${borderClass} ${bgClass} min-w-[110px] transition-colors duration-200`}>
            <Handle type="target" position={Position.Left} />
            <div className="text-sm font-medium text-gray-800">{data.label}</div>
            <div className="text-[10px] text-gray-500 flex justify-between gap-2 mt-0.5">
                <span>{data.nodeType || ''}</span>
                {isGasActive && <span className={`font-mono text-[9px] px-1 rounded bg-slate-950/20 ${gasStyle.text}`}>{gas}g</span>}
            </div>
            <Handle type="source" position={Position.Right} />
        </div>
    )
}

export const nodeTypes = {
    start: StartNode,
    end: EndNode,
    default: DefaultNode,
}
