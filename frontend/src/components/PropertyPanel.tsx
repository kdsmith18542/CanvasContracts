import React from 'react'
import { Settings, X } from 'lucide-react'

interface PropertyPanelProps {
    node: { id: string; type: string; data: { label: string; properties?: Record<string, any> } } | null
    onClose: () => void
    onPropertyChange: (nodeId: string, propertyName: string, value: any) => void
}

export const PropertyPanel: React.FC<PropertyPanelProps> = ({ node, onClose, onPropertyChange }) => {
    if (!node) return null

    const properties = node.data.properties || {}

    const handleChange = (name: string, value: any) => {
        onPropertyChange(node.id, name, value)
    }

    return (
        <div className="w-72 bg-white border-l border-gray-200 p-4 overflow-y-auto">
            <div className="flex items-center justify-between mb-4">
                <h3 className="text-sm font-semibold text-gray-900 flex items-center">
                    <Settings className="w-4 h-4 mr-2" />
                    Properties
                </h3>
                <button onClick={onClose} className="text-gray-400 hover:text-gray-600">
                    <X className="w-4 h-4" />
                </button>
            </div>
            <div className="mb-4">
                <label className="text-xs text-gray-500 uppercase">Type</label>
                <p className="text-sm font-medium text-gray-900">{node.type}</p>
            </div>
            <div className="mb-4">
                <label className="text-xs text-gray-500 uppercase">Label</label>
                <p className="text-sm text-gray-700">{node.data.label}</p>
            </div>
            <div className="space-y-3">
                {Object.entries(properties).map(([key, value]) => (
                    <div key={key}>
                        <label className="block text-xs text-gray-500 mb-1">{key}</label>
                        <input
                            type="text"
                            value={String(value)}
                            onChange={(e) => handleChange(key, e.target.value)}
                            className="w-full px-2 py-1 text-sm border border-gray-300 rounded"
                        />
                    </div>
                ))}
                {Object.keys(properties).length === 0 && (
                    <p className="text-xs text-gray-400 italic">No editable properties</p>
                )}
            </div>
            <div className="mt-4 pt-4 border-t border-gray-200">
                <h4 className="text-xs font-medium text-gray-500 uppercase mb-2">Node Info</h4>
                <p className="text-xs text-gray-500">ID: {node.id}</p>
            </div>
        </div>
    )
}
