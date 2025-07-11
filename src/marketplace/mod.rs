//! Marketplace system for Canvas Contracts ecosystem

use crate::{
    error::{CanvasError, CanvasResult},
    types::{Graph, Node, NodeId},
    nodes::custom::CustomNodeDefinition,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Marketplace item types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketplaceItemType {
    CustomNode,
    Template,
    Component,
    Tutorial,
}

/// Marketplace item metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub item_type: MarketplaceItemType,
    pub tags: Vec<String>,
    pub rating: f64,
    pub downloads: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub price: Option<f64>, // None for free items
    pub license: String,
    pub dependencies: Vec<String>,
    pub compatibility: Vec<String>, // Supported versions
    pub size_bytes: u64,
    pub hash: String, // Content hash for verification
}

/// Custom node marketplace item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomNodeItem {
    pub metadata: MarketplaceItem,
    pub node_definition: CustomNodeDefinition,
    pub examples: Vec<NodeExample>,
    pub documentation: String,
}

/// Template marketplace item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateItem {
    pub metadata: MarketplaceItem,
    pub graph: Graph,
    pub description: String,
    pub use_cases: Vec<String>,
    pub difficulty: TemplateDifficulty,
    pub estimated_gas: u64,
}

/// Component marketplace item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentItem {
    pub metadata: MarketplaceItem,
    pub components: Vec<Graph>,
    pub architecture: String,
    pub integration_guide: String,
}

/// Tutorial marketplace item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialItem {
    pub metadata: MarketplaceItem,
    pub content: String,
    pub difficulty: TutorialDifficulty,
    pub duration_minutes: u32,
    pub prerequisites: Vec<String>,
    pub resources: Vec<TutorialResource>,
}

/// Node example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExample {
    pub name: String,
    pub description: String,
    pub input_data: HashMap<String, serde_json::Value>,
    pub expected_output: HashMap<String, serde_json::Value>,
    pub graph_snippet: String, // JSON snippet showing usage
}

/// Template difficulty levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Tutorial difficulty levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TutorialDifficulty {
    Beginner,
    Intermediate,
    Advanced,
}

/// Tutorial resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialResource {
    pub name: String,
    pub url: String,
    pub resource_type: String, // "video", "documentation", "code", etc.
}

/// User profile for marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub bio: String,
    pub location: Option<String>,
    pub website: Option<String>,
    pub social_links: HashMap<String, String>,
    pub reputation_score: f64,
    pub items_published: u32,
    pub total_downloads: u64,
    pub member_since: DateTime<Utc>,
    pub verified: bool,
}

/// Review for marketplace items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: String,
    pub item_id: String,
    pub user_id: String,
    pub rating: u8, // 1-5 stars
    pub title: String,
    pub content: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub helpful_votes: u32,
    pub verified_purchase: bool,
}

/// Marketplace search filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub item_type: Option<MarketplaceItemType>,
    pub tags: Vec<String>,
    pub min_rating: Option<f64>,
    pub max_price: Option<f64>,
    pub free_only: bool,
    pub author: Option<String>,
    pub compatibility: Option<String>,
    pub difficulty: Option<String>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

/// Marketplace client
pub struct MarketplaceClient {
    api_url: String,
    api_key: Option<String>,
    cache: HashMap<String, MarketplaceItem>,
}

impl MarketplaceClient {
    /// Create a new marketplace client
    pub fn new(api_url: String) -> Self {
        Self {
            api_url,
            api_key: None,
            cache: HashMap::new(),
        }
    }

    /// Set API key for authenticated requests
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Search for marketplace items
    pub async fn search_items(
        &self,
        query: &str,
        filters: &SearchFilters,
        page: u32,
        limit: u32,
    ) -> CanvasResult<Vec<MarketplaceItem>> {
        // TODO: Implement actual API call
        log::info!("Searching marketplace for: {}", query);
        
        // Mock response for now
        Ok(vec![])
    }

    /// Get item details
    pub async fn get_item(&mut self, item_id: &str) -> CanvasResult<MarketplaceItem> {
        // Check cache first
        if let Some(item) = self.cache.get(item_id) {
            return Ok(item.clone());
        }

        // TODO: Implement actual API call
        log::info!("Fetching item details for: {}", item_id);
        
        // Mock response for now
        let item = MarketplaceItem {
            id: item_id.to_string(),
            name: "Sample Item".to_string(),
            description: "A sample marketplace item".to_string(),
            author: "sample_author".to_string(),
            version: "1.0.0".to_string(),
            item_type: MarketplaceItemType::CustomNode,
            tags: vec!["sample".to_string()],
            rating: 4.5,
            downloads: 100,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            price: None,
            license: "MIT".to_string(),
            dependencies: vec![],
            compatibility: vec!["1.0.0".to_string()],
            size_bytes: 1024,
            hash: "sample_hash".to_string(),
        };

        // Cache the item
        self.cache.insert(item_id.to_string(), item.clone());
        Ok(item)
    }

    /// Download item content
    pub async fn download_item(&self, item_id: &str) -> CanvasResult<Vec<u8>> {
        // TODO: Implement actual download
        log::info!("Downloading item: {}", item_id);
        
        // Mock response for now
        Ok(vec![0u8; 1024])
    }

    /// Upload item to marketplace
    pub async fn upload_item(
        &self,
        item: &MarketplaceItem,
        content: &[u8],
    ) -> CanvasResult<String> {
        // TODO: Implement actual upload
        log::info!("Uploading item: {}", item.name);
        
        // Mock response for now
        Ok("uploaded_item_id".to_string())
    }

    /// Get user profile
    pub async fn get_user_profile(&self, username: &str) -> CanvasResult<UserProfile> {
        // TODO: Implement actual API call
        log::info!("Fetching user profile for: {}", username);
        
        // Mock response for now
        Ok(UserProfile {
            username: username.to_string(),
            display_name: "Sample User".to_string(),
            email: "sample@example.com".to_string(),
            avatar_url: None,
            bio: "A sample user".to_string(),
            location: None,
            website: None,
            social_links: HashMap::new(),
            reputation_score: 4.5,
            items_published: 5,
            total_downloads: 1000,
            member_since: Utc::now(),
            verified: false,
        })
    }

    /// Get item reviews
    pub async fn get_item_reviews(
        &self,
        item_id: &str,
        page: u32,
        limit: u32,
    ) -> CanvasResult<Vec<Review>> {
        // TODO: Implement actual API call
        log::info!("Fetching reviews for item: {}", item_id);
        
        // Mock response for now
        Ok(vec![])
    }

    /// Submit a review
    pub async fn submit_review(&self, review: &Review) -> CanvasResult<()> {
        // TODO: Implement actual API call
        log::info!("Submitting review for item: {}", review.item_id);
        Ok(())
    }

    /// Get trending items
    pub async fn get_trending_items(&self, limit: u32) -> CanvasResult<Vec<MarketplaceItem>> {
        // TODO: Implement actual API call
        log::info!("Fetching trending items");
        
        // Mock response for now
        Ok(vec![])
    }

    /// Get recommended items
    pub async fn get_recommended_items(
        &self,
        user_id: &str,
        limit: u32,
    ) -> CanvasResult<Vec<MarketplaceItem>> {
        // TODO: Implement actual API call
        log::info!("Fetching recommended items for user: {}", user_id);
        
        // Mock response for now
        Ok(vec![])
    }
}

/// Local marketplace manager
pub struct LocalMarketplace {
    items: HashMap<String, MarketplaceItem>,
    custom_nodes: HashMap<String, CustomNodeItem>,
    templates: HashMap<String, TemplateItem>,
    components: HashMap<String, ComponentItem>,
    tutorials: HashMap<String, TutorialItem>,
}

impl LocalMarketplace {
    /// Create a new local marketplace
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            custom_nodes: HashMap::new(),
            templates: HashMap::new(),
            components: HashMap::new(),
            tutorials: HashMap::new(),
        }
    }

    /// Add a custom node to local marketplace
    pub fn add_custom_node(&mut self, item: CustomNodeItem) -> CanvasResult<()> {
        let item_id = item.metadata.id.clone();
        self.custom_nodes.insert(item_id.clone(), item.clone());
        self.items.insert(item_id, item.metadata);
        Ok(())
    }

    /// Add a template to local marketplace
    pub fn add_template(&mut self, item: TemplateItem) -> CanvasResult<()> {
        let item_id = item.metadata.id.clone();
        self.templates.insert(item_id.clone(), item.clone());
        self.items.insert(item_id, item.metadata);
        Ok(())
    }

    /// Add a component to local marketplace
    pub fn add_component(&mut self, item: ComponentItem) -> CanvasResult<()> {
        let item_id = item.metadata.id.clone();
        self.components.insert(item_id.clone(), item.clone());
        self.items.insert(item_id, item.metadata);
        Ok(())
    }

    /// Add a tutorial to local marketplace
    pub fn add_tutorial(&mut self, item: TutorialItem) -> CanvasResult<()> {
        let item_id = item.metadata.id.clone();
        self.tutorials.insert(item_id.clone(), item.clone());
        self.items.insert(item_id, item.metadata);
        Ok(())
    }

    /// Get all items
    pub fn get_all_items(&self) -> Vec<&MarketplaceItem> {
        self.items.values().collect()
    }

    /// Get custom nodes
    pub fn get_custom_nodes(&self) -> Vec<&CustomNodeItem> {
        self.custom_nodes.values().collect()
    }

    /// Get templates
    pub fn get_templates(&self) -> Vec<&TemplateItem> {
        self.templates.values().collect()
    }

    /// Get components
    pub fn get_components(&self) -> Vec<&ComponentItem> {
        self.components.values().collect()
    }

    /// Get tutorials
    pub fn get_tutorials(&self) -> Vec<&TutorialItem> {
        self.tutorials.values().collect()
    }

    /// Search items
    pub fn search_items(&self, query: &str, filters: &SearchFilters) -> Vec<&MarketplaceItem> {
        self.items
            .values()
            .filter(|item| {
                // Basic search implementation
                let matches_query = query.is_empty() || 
                    item.name.to_lowercase().contains(&query.to_lowercase()) ||
                    item.description.to_lowercase().contains(&query.to_lowercase()) ||
                    item.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase()));

                let matches_type = filters.item_type.as_ref().map_or(true, |t| std::mem::discriminant(&item.item_type) == std::mem::discriminant(t));
                let matches_rating = filters.min_rating.map_or(true, |r| item.rating >= r);
                let matches_price = filters.free_only.map_or(true, |free| !free || item.price.is_none());

                matches_query && matches_type && matches_rating && matches_price
            })
            .collect()
    }

    /// Get item by ID
    pub fn get_item(&self, item_id: &str) -> Option<&MarketplaceItem> {
        self.items.get(item_id)
    }

    /// Get custom node by ID
    pub fn get_custom_node(&self, item_id: &str) -> Option<&CustomNodeItem> {
        self.custom_nodes.get(item_id)
    }

    /// Get template by ID
    pub fn get_template(&self, item_id: &str) -> Option<&TemplateItem> {
        self.templates.get(item_id)
    }

    /// Get component by ID
    pub fn get_component(&self, item_id: &str) -> Option<&ComponentItem> {
        self.components.get(item_id)
    }

    /// Get tutorial by ID
    pub fn get_tutorial(&self, item_id: &str) -> Option<&TutorialItem> {
        self.tutorials.get(item_id)
    }

    /// Remove item
    pub fn remove_item(&mut self, item_id: &str) -> CanvasResult<()> {
        self.items.remove(item_id);
        self.custom_nodes.remove(item_id);
        self.templates.remove(item_id);
        self.components.remove(item_id);
        self.tutorials.remove(item_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_marketplace_operations() {
        let mut marketplace = LocalMarketplace::new();
        
        // Create a sample custom node item
        let metadata = MarketplaceItem {
            id: "test-node".to_string(),
            name: "Test Node".to_string(),
            description: "A test custom node".to_string(),
            author: "test_author".to_string(),
            version: "1.0.0".to_string(),
            item_type: MarketplaceItemType::CustomNode,
            tags: vec!["test".to_string()],
            rating: 4.5,
            downloads: 100,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            price: None,
            license: "MIT".to_string(),
            dependencies: vec![],
            compatibility: vec!["1.0.0".to_string()],
            size_bytes: 1024,
            hash: "test_hash".to_string(),
        };

        let node_definition = crate::nodes::custom::CustomNodeBuilder::new(
            "test-node".to_string(),
            "Test Node".to_string(),
        )
        .composite("{}".to_string())
        .build();

        let custom_node_item = CustomNodeItem {
            metadata,
            node_definition,
            examples: vec![],
            documentation: "Test documentation".to_string(),
        };

        // Add item
        assert!(marketplace.add_custom_node(custom_node_item).is_ok());
        
        // Verify item was added
        assert!(marketplace.get_item("test-node").is_some());
        assert_eq!(marketplace.get_custom_nodes().len(), 1);
        
        // Test search
        let filters = SearchFilters {
            item_type: None,
            tags: vec![],
            min_rating: None,
            max_price: None,
            free_only: false,
            author: None,
            compatibility: None,
            difficulty: None,
            date_range: None,
        };
        
        let results = marketplace.search_items("test", &filters);
        assert_eq!(results.len(), 1);
        
        // Remove item
        assert!(marketplace.remove_item("test-node").is_ok());
        assert!(marketplace.get_item("test-node").is_none());
    }

    #[test]
    fn test_marketplace_client_creation() {
        let client = MarketplaceClient::new("https://api.example.com".to_string());
        assert_eq!(client.api_url, "https://api.example.com");
        assert!(client.api_key.is_none());
        
        let client_with_key = client.with_api_key("test_key".to_string());
        assert_eq!(client_with_key.api_key, Some("test_key".to_string()));
    }
} 