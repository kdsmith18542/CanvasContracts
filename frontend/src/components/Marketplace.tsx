import React, { useState, useEffect } from 'react'
import {
    Search,
    Filter,
    Download,
    Star,
    Eye,
    User,
    Calendar,
    Tag,
    Grid,
    List,
    Heart,
    Share2,
    Bookmark
} from 'lucide-react'
import { TauriService } from '../services/tauriService'

interface MarketplaceItem {
    id: string
    name: string
    description: string
    author: string
    version: string
    item_type: 'CustomNode' | 'Template' | 'Component' | 'Tutorial'
    tags: string[]
    rating: number
    downloads: number
    created_at: string
    updated_at: string
    price?: number
    license: string
    dependencies: string[]
    compatibility: string[]
    size_bytes: number
    hash: string
}

interface SearchFilters {
    item_type?: 'CustomNode' | 'Template' | 'Component' | 'Tutorial'
    tags: string[]
    min_rating?: number
    max_price?: number
    free_only: boolean
    author?: string
    compatibility?: string
    difficulty?: string
}

export const Marketplace: React.FC = () => {
    const [items, setItems] = useState<MarketplaceItem[]>([])
    const [filteredItems, setFilteredItems] = useState<MarketplaceItem[]>([])
    const [searchQuery, setSearchQuery] = useState('')
    const [filters, setFilters] = useState<SearchFilters>({
        tags: [],
        free_only: false,
    })
    const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid')
    const [sortBy, setSortBy] = useState<'relevance' | 'rating' | 'downloads' | 'date' | 'name'>('relevance')
    const [loading, setLoading] = useState(false)
    const [selectedItem, setSelectedItem] = useState<MarketplaceItem | null>(null)
    const [showFilters, setShowFilters] = useState(false)

    // Mock data for demonstration
    useEffect(() => {
        const mockItems: MarketplaceItem[] = [
            {
                id: '1',
                name: 'ERC-20 Token Template',
                description: 'A complete ERC-20 token implementation with transfer, approve, and mint functionality.',
                author: 'crypto_dev',
                version: '1.2.0',
                item_type: 'Template',
                tags: ['token', 'erc20', 'defi'],
                rating: 4.8,
                downloads: 1250,
                created_at: '2024-01-15T10:00:00Z',
                updated_at: '2024-02-20T14:30:00Z',
                price: undefined,
                license: 'MIT',
                dependencies: [],
                compatibility: ['1.0.0'],
                size_bytes: 2048,
                hash: 'abc123',
            },
            {
                id: '2',
                name: 'Advanced Math Operations',
                description: 'Custom node for complex mathematical operations including trigonometry and calculus.',
                author: 'math_wizard',
                version: '2.1.0',
                item_type: 'CustomNode',
                tags: ['math', 'calculus', 'trigonometry'],
                rating: 4.6,
                downloads: 890,
                created_at: '2024-01-10T09:00:00Z',
                updated_at: '2024-02-15T16:45:00Z',
                price: 5.99,
                license: 'Commercial',
                dependencies: ['math-core'],
                compatibility: ['1.0.0', '1.1.0'],
                size_bytes: 4096,
                hash: 'def456',
            },
            {
                id: '3',
                name: 'Voting System Component',
                description: 'Complete voting system with proposal creation, voting, and result calculation.',
                author: 'gov_expert',
                version: '1.0.0',
                item_type: 'Component',
                tags: ['governance', 'voting', 'dao'],
                rating: 4.9,
                downloads: 2100,
                created_at: '2024-01-20T11:00:00Z',
                updated_at: '2024-02-25T10:15:00Z',
                price: undefined,
                license: 'GPL-3.0',
                dependencies: [],
                compatibility: ['1.0.0'],
                size_bytes: 3072,
                hash: 'ghi789',
            },
            {
                id: '4',
                name: 'Getting Started with Canvas Contracts',
                description: 'Comprehensive tutorial covering the basics of visual smart contract development.',
                author: 'canvas_team',
                version: '1.0.0',
                item_type: 'Tutorial',
                tags: ['tutorial', 'beginner', 'basics'],
                rating: 4.7,
                downloads: 3400,
                created_at: '2024-01-05T08:00:00Z',
                updated_at: '2024-02-10T12:00:00Z',
                price: undefined,
                license: 'CC-BY-SA',
                dependencies: [],
                compatibility: ['1.0.0'],
                size_bytes: 1024,
                hash: 'jkl012',
            },
        ]

        setItems(mockItems)
        setFilteredItems(mockItems)
    }, [])

    // Filter and sort items
    useEffect(() => {
        let filtered = items.filter(item => {
            // Search query
            const matchesQuery = searchQuery === '' ||
                item.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                item.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
                item.tags.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase()))

            // Item type filter
            const matchesType = !filters.item_type || item.item_type === filters.item_type

            // Tags filter
            const matchesTags = filters.tags.length === 0 ||
                filters.tags.some(tag => item.tags.includes(tag))

            // Rating filter
            const matchesRating = !filters.min_rating || item.rating >= filters.min_rating

            // Price filter
            const matchesPrice = !filters.free_only || item.price === undefined

            // Author filter
            const matchesAuthor = !filters.author ||
                item.author.toLowerCase().includes(filters.author.toLowerCase())

            return matchesQuery && matchesType && matchesTags && matchesRating && matchesPrice && matchesAuthor
        })

        // Sort items
        filtered.sort((a, b) => {
            switch (sortBy) {
                case 'rating':
                    return b.rating - a.rating
                case 'downloads':
                    return b.downloads - a.downloads
                case 'date':
                    return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
                case 'name':
                    return a.name.localeCompare(b.name)
                default:
                    return 0
            }
        })

        setFilteredItems(filtered)
    }, [items, searchQuery, filters, sortBy])

    const handleDownload = async (item: MarketplaceItem) => {
        setLoading(true)
        try {
            // TODO: Implement actual download
            console.log('Downloading item:', item.name)
            await new Promise(resolve => setTimeout(resolve, 1000)) // Simulate download
            alert(`Downloaded ${item.name} successfully!`)
        } catch (error) {
            console.error('Download failed:', error)
            alert('Download failed. Please try again.')
        } finally {
            setLoading(false)
        }
    }

    const handleFavorite = (item: MarketplaceItem) => {
        // TODO: Implement favorite functionality
        console.log('Favorited item:', item.name)
    }

    const handleShare = (item: MarketplaceItem) => {
        // TODO: Implement share functionality
        navigator.clipboard.writeText(`https://marketplace.canvascontracts.com/item/${item.id}`)
        alert('Link copied to clipboard!')
    }

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleDateString()
    }

    const formatFileSize = (bytes: number) => {
        if (bytes < 1024) return `${bytes} B`
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
        return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
    }

    const getItemTypeIcon = (type: string) => {
        switch (type) {
            case 'CustomNode':
                return '🔧'
            case 'Template':
                return '📋'
            case 'Component':
                return '🧩'
            case 'Tutorial':
                return '📚'
            default:
                return '📄'
        }
    }

    const getItemTypeColor = (type: string) => {
        switch (type) {
            case 'CustomNode':
                return 'bg-blue-100 text-blue-800'
            case 'Template':
                return 'bg-green-100 text-green-800'
            case 'Component':
                return 'bg-purple-100 text-purple-800'
            case 'Tutorial':
                return 'bg-orange-100 text-orange-800'
            default:
                return 'bg-gray-100 text-gray-800'
        }
    }

    return (
        <div className="w-full h-full flex flex-col bg-gray-50">
            {/* Header */}
            <div className="bg-white border-b border-gray-200 p-4">
                <div className="flex items-center justify-between mb-4">
                    <h1 className="text-2xl font-bold text-gray-900">Marketplace</h1>
                    <div className="flex items-center space-x-2">
                        <button
                            onClick={() => setViewMode('grid')}
                            className={`p-2 rounded-md ${viewMode === 'grid' ? 'bg-blue-100 text-blue-600' : 'text-gray-500 hover:text-gray-700'}`}
                        >
                            <Grid className="w-5 h-5" />
                        </button>
                        <button
                            onClick={() => setViewMode('list')}
                            className={`p-2 rounded-md ${viewMode === 'list' ? 'bg-blue-100 text-blue-600' : 'text-gray-500 hover:text-gray-700'}`}
                        >
                            <List className="w-5 h-5" />
                        </button>
                    </div>
                </div>

                {/* Search and Filters */}
                <div className="flex items-center space-x-4">
                    <div className="flex-1 relative">
                        <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
                        <input
                            type="text"
                            placeholder="Search marketplace..."
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>
                    <button
                        onClick={() => setShowFilters(!showFilters)}
                        className="flex items-center px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                    >
                        <Filter className="w-4 h-4 mr-2" />
                        Filters
                    </button>
                    <select
                        value={sortBy}
                        onChange={(e) => setSortBy(e.target.value as any)}
                        className="px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    >
                        <option value="relevance">Relevance</option>
                        <option value="rating">Rating</option>
                        <option value="downloads">Downloads</option>
                        <option value="date">Date</option>
                        <option value="name">Name</option>
                    </select>
                </div>

                {/* Filter Panel */}
                {showFilters && (
                    <div className="mt-4 p-4 bg-gray-50 rounded-md">
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                            <div>
                                <label className="block text-sm font-medium text-gray-700 mb-1">Type</label>
                                <select
                                    value={filters.item_type || ''}
                                    onChange={(e) => setFilters(prev => ({ ...prev, item_type: e.target.value as any || undefined }))}
                                    className="w-full px-3 py-1 border border-gray-300 rounded-md text-sm"
                                >
                                    <option value="">All Types</option>
                                    <option value="CustomNode">Custom Nodes</option>
                                    <option value="Template">Templates</option>
                                    <option value="Component">Components</option>
                                    <option value="Tutorial">Tutorials</option>
                                </select>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-gray-700 mb-1">Min Rating</label>
                                <select
                                    value={filters.min_rating || ''}
                                    onChange={(e) => setFilters(prev => ({ ...prev, min_rating: e.target.value ? parseFloat(e.target.value) : undefined }))}
                                    className="w-full px-3 py-1 border border-gray-300 rounded-md text-sm"
                                >
                                    <option value="">Any Rating</option>
                                    <option value="4.5">4.5+ Stars</option>
                                    <option value="4.0">4.0+ Stars</option>
                                    <option value="3.5">3.5+ Stars</option>
                                    <option value="3.0">3.0+ Stars</option>
                                </select>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-gray-700 mb-1">Price</label>
                                <label className="flex items-center">
                                    <input
                                        type="checkbox"
                                        checked={filters.free_only}
                                        onChange={(e) => setFilters(prev => ({ ...prev, free_only: e.target.checked }))}
                                        className="mr-2"
                                    />
                                    <span className="text-sm">Free Only</span>
                                </label>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-gray-700 mb-1">Author</label>
                                <input
                                    type="text"
                                    placeholder="Search by author..."
                                    value={filters.author || ''}
                                    onChange={(e) => setFilters(prev => ({ ...prev, author: e.target.value || undefined }))}
                                    className="w-full px-3 py-1 border border-gray-300 rounded-md text-sm"
                                />
                            </div>
                        </div>
                    </div>
                )}
            </div>

            {/* Content */}
            <div className="flex-1 overflow-y-auto p-4">
                {loading && (
                    <div className="flex items-center justify-center py-8">
                        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    </div>
                )}

                {!loading && filteredItems.length === 0 && (
                    <div className="text-center py-8">
                        <p className="text-gray-500">No items found matching your criteria.</p>
                    </div>
                )}

                {!loading && filteredItems.length > 0 && (
                    <div className={`grid gap-4 ${viewMode === 'grid'
                            ? 'grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'
                            : 'grid-cols-1'
                        }`}>
                        {filteredItems.map((item) => (
                            <div
                                key={item.id}
                                className={`bg-white rounded-lg border border-gray-200 overflow-hidden hover:shadow-md transition-shadow ${viewMode === 'list' ? 'flex' : ''
                                    }`}
                            >
                                {viewMode === 'list' ? (
                                    // List view
                                    <>
                                        <div className="flex-1 p-4">
                                            <div className="flex items-start justify-between mb-2">
                                                <div className="flex items-center space-x-2">
                                                    <span className="text-2xl">{getItemTypeIcon(item.item_type)}</span>
                                                    <div>
                                                        <h3 className="font-semibold text-gray-900">{item.name}</h3>
                                                        <p className="text-sm text-gray-500">by {item.author}</p>
                                                    </div>
                                                </div>
                                                <div className="flex items-center space-x-2">
                                                    <span className={`px-2 py-1 text-xs font-medium rounded-full ${getItemTypeColor(item.item_type)}`}>
                                                        {item.item_type}
                                                    </span>
                                                    {item.price ? (
                                                        <span className="text-sm font-medium text-green-600">${item.price}</span>
                                                    ) : (
                                                        <span className="text-sm font-medium text-green-600">Free</span>
                                                    )}
                                                </div>
                                            </div>
                                            <p className="text-gray-600 text-sm mb-3">{item.description}</p>
                                            <div className="flex items-center space-x-4 text-sm text-gray-500">
                                                <div className="flex items-center">
                                                    <Star className="w-4 h-4 mr-1 text-yellow-400" />
                                                    {item.rating}
                                                </div>
                                                <div className="flex items-center">
                                                    <Download className="w-4 h-4 mr-1" />
                                                    {item.downloads.toLocaleString()}
                                                </div>
                                                <div className="flex items-center">
                                                    <Calendar className="w-4 h-4 mr-1" />
                                                    {formatDate(item.updated_at)}
                                                </div>
                                            </div>
                                        </div>
                                        <div className="flex flex-col justify-center p-4 border-l border-gray-200">
                                            <button
                                                onClick={() => handleDownload(item)}
                                                disabled={loading}
                                                className="flex items-center justify-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 mb-2"
                                            >
                                                <Download className="w-4 h-4 mr-2" />
                                                Download
                                            </button>
                                            <div className="flex space-x-2">
                                                <button
                                                    onClick={() => handleFavorite(item)}
                                                    className="p-2 text-gray-400 hover:text-red-500"
                                                >
                                                    <Heart className="w-4 h-4" />
                                                </button>
                                                <button
                                                    onClick={() => handleShare(item)}
                                                    className="p-2 text-gray-400 hover:text-blue-500"
                                                >
                                                    <Share2 className="w-4 h-4" />
                                                </button>
                                            </div>
                                        </div>
                                    </>
                                ) : (
                                    // Grid view
                                    <>
                                        <div className="p-4">
                                            <div className="flex items-start justify-between mb-2">
                                                <span className="text-2xl">{getItemTypeIcon(item.item_type)}</span>
                                                <span className={`px-2 py-1 text-xs font-medium rounded-full ${getItemTypeColor(item.item_type)}`}>
                                                    {item.item_type}
                                                </span>
                                            </div>
                                            <h3 className="font-semibold text-gray-900 mb-1">{item.name}</h3>
                                            <p className="text-sm text-gray-500 mb-2">by {item.author}</p>
                                            <p className="text-gray-600 text-sm mb-3 line-clamp-2">{item.description}</p>
                                            <div className="flex items-center justify-between text-sm text-gray-500 mb-3">
                                                <div className="flex items-center">
                                                    <Star className="w-4 h-4 mr-1 text-yellow-400" />
                                                    {item.rating}
                                                </div>
                                                <div className="flex items-center">
                                                    <Download className="w-4 h-4 mr-1" />
                                                    {item.downloads.toLocaleString()}
                                                </div>
                                            </div>
                                            <div className="flex flex-wrap gap-1 mb-3">
                                                {item.tags.slice(0, 3).map((tag, index) => (
                                                    <span
                                                        key={index}
                                                        className="px-2 py-1 text-xs bg-gray-100 text-gray-600 rounded"
                                                    >
                                                        {tag}
                                                    </span>
                                                ))}
                                                {item.tags.length > 3 && (
                                                    <span className="px-2 py-1 text-xs bg-gray-100 text-gray-600 rounded">
                                                        +{item.tags.length - 3}
                                                    </span>
                                                )}
                                            </div>
                                        </div>
                                        <div className="px-4 pb-4">
                                            <div className="flex items-center justify-between mb-2">
                                                {item.price ? (
                                                    <span className="text-sm font-medium text-green-600">${item.price}</span>
                                                ) : (
                                                    <span className="text-sm font-medium text-green-600">Free</span>
                                                )}
                                                <span className="text-xs text-gray-500">{formatFileSize(item.size_bytes)}</span>
                                            </div>
                                            <button
                                                onClick={() => handleDownload(item)}
                                                disabled={loading}
                                                className="w-full flex items-center justify-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
                                            >
                                                <Download className="w-4 h-4 mr-2" />
                                                Download
                                            </button>
                                        </div>
                                    </>
                                )}
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    )
} 