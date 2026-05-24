import { create } from 'zustand'
import { VisualGraph, CompilationResult, ValidationResult } from '../types'

const MAX_UNDO = 50

interface CanvasState {
    graph: VisualGraph
    compilationResult: CompilationResult | null
    validationResult: ValidationResult | null
    isLoading: boolean
    error: string | null
    undoStack: VisualGraph[]
    redoStack: VisualGraph[]

    updateGraph: (graph: VisualGraph) => void
    addNode: (node: any) => void
    removeNode: (nodeId: string) => void
    addEdge: (edge: any) => void
    removeEdge: (edgeId: string) => void
    setCompilationResult: (result: CompilationResult) => void
    setValidationResult: (result: ValidationResult) => void
    setLoading: (loading: boolean) => void
    setError: (error: string | null) => void
    clearError: () => void
    undo: () => void
    redo: () => void
}

export const useCanvasStore = create<CanvasState>((set, get) => {
    const pushUndo = (state: CanvasState): Partial<CanvasState> => {
        const undoStack = [...state.undoStack, state.graph]
        if (undoStack.length > MAX_UNDO) undoStack.shift()
        return { undoStack, redoStack: [] }
    }

    return {
        graph: {
            name: "MyContract",
            nodes: [],
            edges: [],
            metadata: {}
        },
        compilationResult: null,
        validationResult: null,
        isLoading: false,
        error: null,
        undoStack: [],
        redoStack: [],

        updateGraph: (graph) => set((state) => ({
            ...pushUndo(state),
            graph
        })),

        addNode: (node) => set((state) => ({
            ...pushUndo(state),
            graph: {
                ...state.graph,
                nodes: [...state.graph.nodes, node]
            }
        })),

        removeNode: (nodeId) => set((state) => ({
            ...pushUndo(state),
            graph: {
                ...state.graph,
                nodes: state.graph.nodes.filter(n => n.id !== nodeId),
                edges: state.graph.edges.filter(e => e.source !== nodeId && e.target !== nodeId)
            }
        })),

        addEdge: (edge) => set((state) => ({
            ...pushUndo(state),
            graph: {
                ...state.graph,
                edges: [...state.graph.edges, edge]
            }
        })),

        removeEdge: (edgeId) => set((state) => ({
            ...pushUndo(state),
            graph: {
                ...state.graph,
                edges: state.graph.edges.filter(e => e.id !== edgeId)
            }
        })),

        setCompilationResult: (result) => set({ compilationResult: result }),
        setValidationResult: (result) => set({ validationResult: result }),
        setLoading: (loading) => set({ isLoading: loading }),
        setError: (error) => set({ error }),
        clearError: () => set({ error: null }),

        undo: () => {
            const { undoStack, graph, redoStack } = get()
            if (undoStack.length === 0) return
            const previous = undoStack[undoStack.length - 1]
            const newRedoStack = [...redoStack, graph]
            if (newRedoStack.length > MAX_UNDO) newRedoStack.shift()
            set({
                graph: previous,
                undoStack: undoStack.slice(0, -1),
                redoStack: newRedoStack
            })
        },

        redo: () => {
            const { redoStack, graph, undoStack } = get()
            if (redoStack.length === 0) return
            const next = redoStack[redoStack.length - 1]
            const newUndoStack = [...undoStack, graph]
            if (newUndoStack.length > MAX_UNDO) newUndoStack.shift()
            set({
                graph: next,
                undoStack: newUndoStack,
                redoStack: redoStack.slice(0, -1)
            })
        }
    }
})
