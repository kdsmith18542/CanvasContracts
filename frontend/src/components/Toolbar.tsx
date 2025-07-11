import React from 'react'
import { Play, CheckCircle, Upload, Save, Settings } from 'lucide-react'

export const Toolbar: React.FC = () => {
    const handleCompile = async () => {
        // TODO: Implement compile functionality
        console.log('Compiling contract...')
    }

    const handleValidate = async () => {
        // TODO: Implement validation
        console.log('Validating contract...')
    }

    const handleDeploy = async () => {
        // TODO: Implement deployment
        console.log('Deploying contract...')
    }

    return (
        <div className="bg-white border-b border-gray-200 px-4 py-2 flex items-center justify-between">
            <div className="flex items-center space-x-2">
                <h1 className="text-xl font-semibold text-gray-900">Canvas Contracts</h1>
            </div>

            <div className="flex items-center space-x-2">
                <button
                    onClick={handleValidate}
                    className="flex items-center px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                >
                    <CheckCircle className="w-4 h-4 mr-1" />
                    Validate
                </button>

                <button
                    onClick={handleCompile}
                    className="flex items-center px-3 py-1.5 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700"
                >
                    <Play className="w-4 h-4 mr-1" />
                    Compile
                </button>

                <button
                    onClick={handleDeploy}
                    className="flex items-center px-3 py-1.5 text-sm font-medium text-white bg-green-600 border border-transparent rounded-md hover:bg-green-700"
                >
                    <Upload className="w-4 h-4 mr-1" />
                    Deploy
                </button>

                <button className="flex items-center px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50">
                    <Save className="w-4 h-4 mr-1" />
                    Save
                </button>

                <button className="flex items-center px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50">
                    <Settings className="w-4 h-4" />
                </button>
            </div>
        </div>
    )
} 