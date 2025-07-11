import React, { useState } from 'react'
import {
    Plus,
    Save,
    X,
    Code,
    Database,
    Settings,
    Upload,
    Download
} from 'lucide-react'
import { TauriService } from '../services/tauriService'

interface CustomNodeDefinition {
    id: string
    name: string
    description: string
    category: string
    inputs: CustomNodePort[]
    outputs: CustomNodePort[]
    properties: CustomNodeProperty[]
    implementation: CustomNodeImplementation
}

interface CustomNodePort {
    name: string
    port_type: string
    required: boolean
    description: string
}

interface CustomNodeProperty {
    name: string
    property_type: string
    required: boolean
    default_value?: string
    description: string
}

interface CustomNodeImplementation {
    type: 'composite' | 'wasm' | 'script'
    data: string
    language?: string
}

export const CustomNodeCreator: React.FC = () => {
    const [isOpen, setIsOpen] = useState(false)
    const [nodeDefinition, setNodeDefinition] = useState<CustomNodeDefinition>({
        id: '',
        name: '',
        description: '',
        category: 'Custom',
        inputs: [],
        outputs: [],
        properties: [],
        implementation: {
            type: 'composite',
            data: ''
        }
    })
    const [activeTab, setActiveTab] = useState<'basic' | 'ports' | 'properties' | 'implementation'>('basic')
    const [newInput, setNewInput] = useState({ name: '', port_type: 'any', required: false, description: '' })
    const [newOutput, setNewOutput] = useState({ name: '', port_type: 'any', description: '' })
    const [newProperty, setNewProperty] = useState({ name: '', property_type: 'string', required: false, default_value: '', description: '' })

    const portTypes = ['any', 'number', 'string', 'boolean', 'object', 'array']
    const propertyTypes = ['string', 'number', 'boolean', 'select']
    const categories = ['Custom', 'Logic', 'Math', 'Storage', 'External', 'Utility']

    const handleSave = async () => {
        if (!nodeDefinition.id || !nodeDefinition.name) {
            alert('Please provide an ID and name for the custom node')
            return
        }

        try {
            // TODO: Implement save functionality
            // await TauriService.saveCustomNode(nodeDefinition)
            console.log('Saving custom node:', nodeDefinition)
            alert('Custom node saved successfully!')
            handleClose()
        } catch (error) {
            console.error('Failed to save custom node:', error)
            alert('Failed to save custom node')
        }
    }

    const handleClose = () => {
        setIsOpen(false)
        setNodeDefinition({
            id: '',
            name: '',
            description: '',
            category: 'Custom',
            inputs: [],
            outputs: [],
            properties: [],
            implementation: {
                type: 'composite',
                data: ''
            }
        })
        setActiveTab('basic')
    }

    const handleAddInput = () => {
        if (!newInput.name) {
            alert('Please provide a name for the input')
            return
        }

        setNodeDefinition(prev => ({
            ...prev,
            inputs: [...prev.inputs, { ...newInput }]
        }))
        setNewInput({ name: '', port_type: 'any', required: false, description: '' })
    }

    const handleRemoveInput = (index: number) => {
        setNodeDefinition(prev => ({
            ...prev,
            inputs: prev.inputs.filter((_, i) => i !== index)
        }))
    }

    const handleAddOutput = () => {
        if (!newOutput.name) {
            alert('Please provide a name for the output')
            return
        }

        setNodeDefinition(prev => ({
            ...prev,
            outputs: [...prev.outputs, { ...newOutput, required: false }]
        }))
        setNewOutput({ name: '', port_type: 'any', description: '' })
    }

    const handleRemoveOutput = (index: number) => {
        setNodeDefinition(prev => ({
            ...prev,
            outputs: prev.outputs.filter((_, i) => i !== index)
        }))
    }

    const handleAddProperty = () => {
        if (!newProperty.name) {
            alert('Please provide a name for the property')
            return
        }

        setNodeDefinition(prev => ({
            ...prev,
            properties: [...prev.properties, { ...newProperty }]
        }))
        setNewProperty({ name: '', property_type: 'string', required: false, default_value: '', description: '' })
    }

    const handleRemoveProperty = (index: number) => {
        setNodeDefinition(prev => ({
            ...prev,
            properties: prev.properties.filter((_, i) => i !== index)
        }))
    }

    const handleImportWasm = () => {
        // TODO: Implement WASM file import
        alert('WASM import functionality coming soon')
    }

    const handleExportNode = () => {
        const dataStr = JSON.stringify(nodeDefinition, null, 2)
        const dataBlob = new Blob([dataStr], { type: 'application/json' })
        const url = URL.createObjectURL(dataBlob)
        const link = document.createElement('a')
        link.href = url
        link.download = `${nodeDefinition.id}.json`
        link.click()
        URL.revokeObjectURL(url)
    }

    return (
        <>
            {/* Open Button */}
            <button
                onClick={() => setIsOpen(true)}
                className="flex items-center px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
            >
                <Plus className="w-4 h-4 mr-1" />
                Create Custom Node
            </button>

            {/* Modal */}
            {isOpen && (
                <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                    <div className="bg-white rounded-lg shadow-xl w-full max-w-4xl h-5/6 flex flex-col">
                        {/* Header */}
                        <div className="flex items-center justify-between p-6 border-b border-gray-200">
                            <h2 className="text-xl font-semibold text-gray-900">Create Custom Node</h2>
                            <div className="flex items-center space-x-2">
                                <button
                                    onClick={handleExportNode}
                                    className="flex items-center px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                                >
                                    <Download className="w-4 h-4 mr-1" />
                                    Export
                                </button>
                                <button
                                    onClick={handleClose}
                                    className="flex items-center px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                                >
                                    <X className="w-4 h-4" />
                                </button>
                            </div>
                        </div>

                        {/* Tabs */}
                        <div className="flex border-b border-gray-200">
                            <button
                                onClick={() => setActiveTab('basic')}
                                className={`flex-1 px-4 py-2 text-sm font-medium ${activeTab === 'basic'
                                        ? 'text-blue-600 border-b-2 border-blue-600'
                                        : 'text-gray-500 hover:text-gray-700'
                                    }`}
                            >
                                Basic Info
                            </button>
                            <button
                                onClick={() => setActiveTab('ports')}
                                className={`flex-1 px-4 py-2 text-sm font-medium ${activeTab === 'ports'
                                        ? 'text-blue-600 border-b-2 border-blue-600'
                                        : 'text-gray-500 hover:text-gray-700'
                                    }`}
                            >
                                Ports
                            </button>
                            <button
                                onClick={() => setActiveTab('properties')}
                                className={`flex-1 px-4 py-2 text-sm font-medium ${activeTab === 'properties'
                                        ? 'text-blue-600 border-b-2 border-blue-600'
                                        : 'text-gray-500 hover:text-gray-700'
                                    }`}
                            >
                                Properties
                            </button>
                            <button
                                onClick={() => setActiveTab('implementation')}
                                className={`flex-1 px-4 py-2 text-sm font-medium ${activeTab === 'implementation'
                                        ? 'text-blue-600 border-b-2 border-blue-600'
                                        : 'text-gray-500 hover:text-gray-700'
                                    }`}
                            >
                                Implementation
                            </button>
                        </div>

                        {/* Content */}
                        <div className="flex-1 overflow-y-auto p-6">
                            {activeTab === 'basic' && (
                                <div className="space-y-4">
                                    <div>
                                        <label className="block text-sm font-medium text-gray-700 mb-1">
                                            Node ID *
                                        </label>
                                        <input
                                            type="text"
                                            value={nodeDefinition.id}
                                            onChange={(e) => setNodeDefinition(prev => ({ ...prev, id: e.target.value }))}
                                            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            placeholder="unique-node-id"
                                        />
                                    </div>
                                    <div>
                                        <label className="block text-sm font-medium text-gray-700 mb-1">
                                            Name *
                                        </label>
                                        <input
                                            type="text"
                                            value={nodeDefinition.name}
                                            onChange={(e) => setNodeDefinition(prev => ({ ...prev, name: e.target.value }))}
                                            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            placeholder="Display Name"
                                        />
                                    </div>
                                    <div>
                                        <label className="block text-sm font-medium text-gray-700 mb-1">
                                            Description
                                        </label>
                                        <textarea
                                            value={nodeDefinition.description}
                                            onChange={(e) => setNodeDefinition(prev => ({ ...prev, description: e.target.value }))}
                                            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            rows={3}
                                            placeholder="Describe what this node does..."
                                        />
                                    </div>
                                    <div>
                                        <label className="block text-sm font-medium text-gray-700 mb-1">
                                            Category
                                        </label>
                                        <select
                                            value={nodeDefinition.category}
                                            onChange={(e) => setNodeDefinition(prev => ({ ...prev, category: e.target.value }))}
                                            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                                        >
                                            {categories.map(category => (
                                                <option key={category} value={category}>{category}</option>
                                            ))}
                                        </select>
                                    </div>
                                </div>
                            )}

                            {activeTab === 'ports' && (
                                <div className="space-y-6">
                                    {/* Inputs */}
                                    <div>
                                        <h3 className="text-lg font-medium text-gray-900 mb-4">Input Ports</h3>
                                        <div className="space-y-4">
                                            {nodeDefinition.inputs.map((input, index) => (
                                                <div key={index} className="flex items-center space-x-2 p-3 bg-gray-50 rounded-md">
                                                    <div className="flex-1">
                                                        <div className="flex items-center space-x-2">
                                                            <span className="font-medium">{input.name}</span>
                                                            <span className="text-sm text-gray-500">({input.port_type})</span>
                                                            {input.required && <span className="text-red-500 text-xs">*</span>}
                                                        </div>
                                                        <p className="text-sm text-gray-600">{input.description}</p>
                                                    </div>
                                                    <button
                                                        onClick={() => handleRemoveInput(index)}
                                                        className="text-red-500 hover:text-red-700"
                                                    >
                                                        <X className="w-4 h-4" />
                                                    </button>
                                                </div>
                                            ))}
                                            <div className="border-2 border-dashed border-gray-300 rounded-md p-4">
                                                <div className="grid grid-cols-2 gap-4">
                                                    <input
                                                        type="text"
                                                        value={newInput.name}
                                                        onChange={(e) => setNewInput(prev => ({ ...prev, name: e.target.value }))}
                                                        className="px-3 py-2 border border-gray-300 rounded-md"
                                                        placeholder="Input name"
                                                    />
                                                    <select
                                                        value={newInput.port_type}
                                                        onChange={(e) => setNewInput(prev => ({ ...prev, port_type: e.target.value }))}
                                                        className="px-3 py-2 border border-gray-300 rounded-md"
                                                    >
                                                        {portTypes.map(type_ => (
                                                            <option key={type_} value={type_}>{type_}</option>
                                                        ))}
                                                    </select>
                                                </div>
                                                <div className="mt-2">
                                                    <input
                                                        type="text"
                                                        value={newInput.description}
                                                        onChange={(e) => setNewInput(prev => ({ ...prev, description: e.target.value }))}
                                                        className="w-full px-3 py-2 border border-gray-300 rounded-md"
                                                        placeholder="Description"
                                                    />
                                                </div>
                                                <div className="mt-2 flex items-center space-x-4">
                                                    <label className="flex items-center">
                                                        <input
                                                            type="checkbox"
                                                            checked={newInput.required}
                                                            onChange={(e) => setNewInput(prev => ({ ...prev, required: e.target.checked }))}
                                                            className="mr-2"
                                                        />
                                                        <span className="text-sm">Required</span>
                                                    </label>
                                                    <button
                                                        onClick={handleAddInput}
                                                        className="px-3 py-1 text-sm bg-blue-600 text-white rounded-md hover:bg-blue-700"
                                                    >
                                                        Add Input
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    {/* Outputs */}
                                    <div>
                                        <h3 className="text-lg font-medium text-gray-900 mb-4">Output Ports</h3>
                                        <div className="space-y-4">
                                            {nodeDefinition.outputs.map((output, index) => (
                                                <div key={index} className="flex items-center space-x-2 p-3 bg-gray-50 rounded-md">
                                                    <div className="flex-1">
                                                        <div className="flex items-center space-x-2">
                                                            <span className="font-medium">{output.name}</span>
                                                            <span className="text-sm text-gray-500">({output.port_type})</span>
                                                        </div>
                                                        <p className="text-sm text-gray-600">{output.description}</p>
                                                    </div>
                                                    <button
                                                        onClick={() => handleRemoveOutput(index)}
                                                        className="text-red-500 hover:text-red-700"
                                                    >
                                                        <X className="w-4 h-4" />
                                                    </button>
                                                </div>
                                            ))}
                                            <div className="border-2 border-dashed border-gray-300 rounded-md p-4">
                                                <div className="grid grid-cols-2 gap-4">
                                                    <input
                                                        type="text"
                                                        value={newOutput.name}
                                                        onChange={(e) => setNewOutput(prev => ({ ...prev, name: e.target.value }))}
                                                        className="px-3 py-2 border border-gray-300 rounded-md"
                                                        placeholder="Output name"
                                                    />
                                                    <select
                                                        value={newOutput.port_type}
                                                        onChange={(e) => setNewOutput(prev => ({ ...prev, port_type: e.target.value }))}
                                                        className="px-3 py-2 border border-gray-300 rounded-md"
                                                    >
                                                        {portTypes.map(type_ => (
                                                            <option key={type_} value={type_}>{type_}</option>
                                                        ))}
                                                    </select>
                                                </div>
                                                <div className="mt-2">
                                                    <input
                                                        type="text"
                                                        value={newOutput.description}
                                                        onChange={(e) => setNewOutput(prev => ({ ...prev, description: e.target.value }))}
                                                        className="w-full px-3 py-2 border border-gray-300 rounded-md"
                                                        placeholder="Description"
                                                    />
                                                </div>
                                                <div className="mt-2">
                                                    <button
                                                        onClick={handleAddOutput}
                                                        className="px-3 py-1 text-sm bg-blue-600 text-white rounded-md hover:bg-blue-700"
                                                    >
                                                        Add Output
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            )}

                            {activeTab === 'properties' && (
                                <div className="space-y-6">
                                    <h3 className="text-lg font-medium text-gray-900">Properties</h3>
                                    <div className="space-y-4">
                                        {nodeDefinition.properties.map((property, index) => (
                                            <div key={index} className="flex items-center space-x-2 p-3 bg-gray-50 rounded-md">
                                                <div className="flex-1">
                                                    <div className="flex items-center space-x-2">
                                                        <span className="font-medium">{property.name}</span>
                                                        <span className="text-sm text-gray-500">({property.property_type})</span>
                                                        {property.required && <span className="text-red-500 text-xs">*</span>}
                                                    </div>
                                                    <p className="text-sm text-gray-600">{property.description}</p>
                                                    {property.default_value && (
                                                        <p className="text-xs text-gray-500">Default: {property.default_value}</p>
                                                    )}
                                                </div>
                                                <button
                                                    onClick={() => handleRemoveProperty(index)}
                                                    className="text-red-500 hover:text-red-700"
                                                >
                                                    <X className="w-4 h-4" />
                                                </button>
                                            </div>
                                        ))}
                                        <div className="border-2 border-dashed border-gray-300 rounded-md p-4">
                                            <div className="grid grid-cols-2 gap-4">
                                                <input
                                                    type="text"
                                                    value={newProperty.name}
                                                    onChange={(e) => setNewProperty(prev => ({ ...prev, name: e.target.value }))}
                                                    className="px-3 py-2 border border-gray-300 rounded-md"
                                                    placeholder="Property name"
                                                />
                                                <select
                                                    value={newProperty.property_type}
                                                    onChange={(e) => setNewProperty(prev => ({ ...prev, property_type: e.target.value }))}
                                                    className="px-3 py-2 border border-gray-300 rounded-md"
                                                >
                                                    {propertyTypes.map(type_ => (
                                                        <option key={type_} value={type_}>{type_}</option>
                                                    ))}
                                                </select>
                                            </div>
                                            <div className="mt-2">
                                                <input
                                                    type="text"
                                                    value={newProperty.description}
                                                    onChange={(e) => setNewProperty(prev => ({ ...prev, description: e.target.value }))}
                                                    className="w-full px-3 py-2 border border-gray-300 rounded-md"
                                                    placeholder="Description"
                                                />
                                            </div>
                                            <div className="mt-2">
                                                <input
                                                    type="text"
                                                    value={newProperty.default_value}
                                                    onChange={(e) => setNewProperty(prev => ({ ...prev, default_value: e.target.value }))}
                                                    className="w-full px-3 py-2 border border-gray-300 rounded-md"
                                                    placeholder="Default value (optional)"
                                                />
                                            </div>
                                            <div className="mt-2 flex items-center space-x-4">
                                                <label className="flex items-center">
                                                    <input
                                                        type="checkbox"
                                                        checked={newProperty.required}
                                                        onChange={(e) => setNewProperty(prev => ({ ...prev, required: e.target.checked }))}
                                                        className="mr-2"
                                                    />
                                                    <span className="text-sm">Required</span>
                                                </label>
                                                <button
                                                    onClick={handleAddProperty}
                                                    className="px-3 py-1 text-sm bg-blue-600 text-white rounded-md hover:bg-blue-700"
                                                >
                                                    Add Property
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            )}

                            {activeTab === 'implementation' && (
                                <div className="space-y-6">
                                    <h3 className="text-lg font-medium text-gray-900">Implementation</h3>

                                    {/* Implementation Type Selection */}
                                    <div>
                                        <label className="block text-sm font-medium text-gray-700 mb-2">
                                            Implementation Type
                                        </label>
                                        <div className="grid grid-cols-3 gap-4">
                                            <button
                                                onClick={() => setNodeDefinition(prev => ({
                                                    ...prev,
                                                    implementation: { type: 'composite', data: '' }
                                                }))}
                                                className={`p-4 border rounded-md text-center ${nodeDefinition.implementation.type === 'composite'
                                                        ? 'border-blue-500 bg-blue-50'
                                                        : 'border-gray-300 hover:border-gray-400'
                                                    }`}
                                            >
                                                <Database className="w-8 h-8 mx-auto mb-2 text-gray-600" />
                                                <div className="font-medium">Composite</div>
                                                <div className="text-sm text-gray-500">Sub-graph</div>
                                            </button>
                                            <button
                                                onClick={() => setNodeDefinition(prev => ({
                                                    ...prev,
                                                    implementation: { type: 'wasm', data: '' }
                                                }))}
                                                className={`p-4 border rounded-md text-center ${nodeDefinition.implementation.type === 'wasm'
                                                        ? 'border-blue-500 bg-blue-50'
                                                        : 'border-gray-300 hover:border-gray-400'
                                                    }`}
                                            >
                                                <Code className="w-8 h-8 mx-auto mb-2 text-gray-600" />
                                                <div className="font-medium">WASM</div>
                                                <div className="text-sm text-gray-500">WebAssembly</div>
                                            </button>
                                            <button
                                                onClick={() => setNodeDefinition(prev => ({
                                                    ...prev,
                                                    implementation: { type: 'script', data: '' }
                                                }))}
                                                className={`p-4 border rounded-md text-center ${nodeDefinition.implementation.type === 'script'
                                                        ? 'border-blue-500 bg-blue-50'
                                                        : 'border-gray-300 hover:border-gray-400'
                                                    }`}
                                            >
                                                <Settings className="w-8 h-8 mx-auto mb-2 text-gray-600" />
                                                <div className="font-medium">Script</div>
                                                <div className="text-sm text-gray-500">Code</div>
                                            </button>
                                        </div>
                                    </div>

                                    {/* Implementation Content */}
                                    <div>
                                        <label className="block text-sm font-medium text-gray-700 mb-2">
                                            {nodeDefinition.implementation.type === 'composite' && 'Sub-graph JSON'}
                                            {nodeDefinition.implementation.type === 'wasm' && 'WASM Module'}
                                            {nodeDefinition.implementation.type === 'script' && 'Script Code'}
                                        </label>

                                        {nodeDefinition.implementation.type === 'wasm' && (
                                            <div className="mb-4">
                                                <button
                                                    onClick={handleImportWasm}
                                                    className="flex items-center px-3 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                                                >
                                                    <Upload className="w-4 h-4 mr-1" />
                                                    Import WASM File
                                                </button>
                                            </div>
                                        )}

                                        {nodeDefinition.implementation.type === 'script' && (
                                            <div className="mb-4">
                                                <select
                                                    value={nodeDefinition.implementation.language || 'rust'}
                                                    onChange={(e) => setNodeDefinition(prev => ({
                                                        ...prev,
                                                        implementation: {
                                                            ...prev.implementation,
                                                            language: e.target.value
                                                        }
                                                    }))}
                                                    className="px-3 py-2 border border-gray-300 rounded-md"
                                                >
                                                    <option value="rust">Rust</option>
                                                    <option value="go">Go</option>
                                                    <option value="assemblyscript">AssemblyScript</option>
                                                </select>
                                            </div>
                                        )}

                                        <textarea
                                            value={nodeDefinition.implementation.data}
                                            onChange={(e) => setNodeDefinition(prev => ({
                                                ...prev,
                                                implementation: {
                                                    ...prev.implementation,
                                                    data: e.target.value
                                                }
                                            }))}
                                            className="w-full h-64 px-3 py-2 border border-gray-300 rounded-md font-mono text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            placeholder={
                                                nodeDefinition.implementation.type === 'composite'
                                                    ? '{"nodes": [], "edges": []}'
                                                    : nodeDefinition.implementation.type === 'script'
                                                        ? '// Write your code here...'
                                                        : '// WASM module data...'
                                            }
                                        />
                                    </div>
                                </div>
                            )}
                        </div>

                        {/* Footer */}
                        <div className="flex items-center justify-end space-x-3 p-6 border-t border-gray-200">
                            <button
                                onClick={handleClose}
                                className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={handleSave}
                                className="flex items-center px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700"
                            >
                                <Save className="w-4 h-4 mr-1" />
                                Save Node
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </>
    )
} 