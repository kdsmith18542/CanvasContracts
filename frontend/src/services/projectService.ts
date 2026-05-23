import { VisualGraph } from '../types'

export class ProjectService {
    static async saveProject(graph: VisualGraph, filePath?: string): Promise<string> {
        const json = JSON.stringify(graph, null, 2)

        if (filePath) {
            try {
                const { writeTextFile } = await import('@tauri-apps/api/fs')
                await writeTextFile(filePath, json)
                return filePath
            } catch {
                // fall through to browser download
            }
        }

        // Try Tauri save dialog first, fall back to browser download
        try {
            const { save } = await import('@tauri-apps/api/dialog')
            const path = await save({
                filters: [{ name: 'Canvas Contract', extensions: ['canvas.json'] }]
            })
            if (path) {
                const { writeTextFile } = await import('@tauri-apps/api/fs')
                await writeTextFile(path as string, json)
                return path as string
            }
        } catch {
            // fall through to browser download
        }

        const blob = new Blob([json], { type: 'application/json' })
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url
        a.download = `contract-${Date.now()}.canvas.json`
        a.click()
        URL.revokeObjectURL(url)
        return 'downloaded'
    }

    static async loadProject(): Promise<VisualGraph | null> {
        try {
            const { open } = await import('@tauri-apps/api/dialog')
            const { readTextFile } = await import('@tauri-apps/api/fs')
            const path = await open({
                filters: [{ name: 'Canvas Contract', extensions: ['canvas.json'] }]
            })
            if (path) {
                const content = await readTextFile(path as string)
                return JSON.parse(content) as VisualGraph
            }
        } catch {
            // fall through to browser file input
        }

        return new Promise((resolve) => {
            const input = document.createElement('input')
            input.type = 'file'
            input.accept = '.canvas.json,.json'
            input.onchange = async (e) => {
                const file = (e.target as HTMLInputElement).files?.[0]
                if (file) {
                    const text = await file.text()
                    resolve(JSON.parse(text) as VisualGraph)
                } else {
                    resolve(null)
                }
            }
            input.click()
        })
    }
}
