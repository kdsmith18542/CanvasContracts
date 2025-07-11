import React from 'react'
import { CanvasEditor } from './components/CanvasEditor'
import { NodePalette } from './components/NodePalette'
import { Toolbar } from './components/Toolbar'
import { AiAssistant } from './components/AiAssistant'

function App() {
    return (
        <div className="h-screen flex flex-col bg-gray-50">
            <Toolbar />
            <div className="flex-1 flex">
                <NodePalette />
                <CanvasEditor />
                <AiAssistant />
            </div>
        </div>
    )
}

export default App 