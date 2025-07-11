import { create } from 'zustand'
import { VisualGraph, CompilationResult, ValidationResult } from '../types'

interface CanvasState {
    graph: VisualGraph
    compilationResult: CompilationResult | null
    validationResult: ValidationResult | null
    isLoading: boolean
    error: string | null

    // Actions
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
}

export const useCanvasStore = create<CanvasState>((set, get) => ({
    graph: {
        nodes: [],
        edges: []
    },
    compilationResult: null,
    validationResult: null,
    isLoading: false,
    error: null,

    updateGraph: (graph) => set({ graph }),

    addNode: (node) => set((state) => ({
        graph: {
            ...state.graph,
            nodes: [...state.graph.nodes, node]
        }
    })),

    removeNode: (nodeId) => set((state) => ({
        graph: {
            ...state.graph,
            nodes: state.graph.nodes.filter(n => n.id !== nodeId),
            edges: state.graph.edges.filter(e => e.source !== nodeId && e.target !== nodeId)
        }
    })),

    addEdge: (edge) => set((state) => ({
        graph: {
            ...state.graph,
            edges: [...state.graph.edges, edge]
        }
    })),

    removeEdge: (edgeId) => set((state) => ({
        graph: {
            ...state.graph,
            edges: state.graph.edges.filter(e => e.id !== edgeId)
        }
    })),

    setCompilationResult: (result) => set({ compilationResult: result }),
    setValidationResult: (result) => set({ validationResult: result }),
    setLoading: (loading) => set({ isLoading: loading }),
    setError: (error) => set({ error }),
    clearError: () => set({ error: null })
})) 