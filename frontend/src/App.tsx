import React, { useState } from 'react'
import { CanvasEditor } from './components/CanvasEditor'
import { NodePalette } from './components/NodePalette'
import { Toolbar } from './components/Toolbar'
import { AiAssistant } from './components/AiAssistant'
import { Debugger } from './components/Debugger'
import { CustomNodeCreator } from './components/CustomNodeCreator'
import { Marketplace } from './components/Marketplace'

function App() {
    const [showDebugger, setShowDebugger] = useState(false)
    const [showCustomNodeCreator, setShowCustomNodeCreator] = useState(false)
    const [showMarketplace, setShowMarketplace] = useState(false)

    return (
        <div className="h-screen flex flex-col bg-gray-50">
            <Toolbar
                onDebugToggle={() => setShowDebugger(!showDebugger)}
                onCustomNodeToggle={() => setShowCustomNodeCreator(!showCustomNodeCreator)}
                onMarketplaceToggle={() => setShowMarketplace(!showMarketplace)}
            />
            <div className="flex-1 flex">
                <NodePalette />
                <CanvasEditor />
                <AiAssistant />
                {showDebugger && <Debugger />}
                {showMarketplace && <Marketplace />}
            </div>
            {showCustomNodeCreator && <CustomNodeCreator />}
        </div>
    )
}

export default App 