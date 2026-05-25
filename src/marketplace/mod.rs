//! Marketplace system for Canvas Contracts ecosystem

use crate::{
    error::{CanvasError, CanvasResult},
    nodes::custom::CustomNodeDefinition,
    types::Graph,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.api_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn is_mock_backend(&self) -> bool {
        self.api_url.starts_with("mock://")
    }

    fn sample_item(&self, id: &str, name: &str) -> MarketplaceItem {
        MarketplaceItem {
            id: id.to_string(),
            name: name.to_string(),
            description: "Mock marketplace item".to_string(),
            author: "mock_author".to_string(),
            version: "1.0.0".to_string(),
            item_type: MarketplaceItemType::CustomNode,
            tags: vec!["mock".to_string()],
            rating: 4.0,
            downloads: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            price: None,
            license: "MIT".to_string(),
            dependencies: Vec::new(),
            compatibility: vec!["0.1.0".to_string()],
            size_bytes: 0,
            hash: "mock_hash".to_string(),
        }
    }

    async fn send_request(
        &self,
        request: reqwest::RequestBuilder,
    ) -> CanvasResult<reqwest::Response> {
        let request = if let Some(api_key) = &self.api_key {
            request.bearer_auth(api_key)
        } else {
            request
        };

        let response = request.send().await.map_err(|e| {
            CanvasError::Network(format!(
                "Marketplace request to {} failed: {}",
                self.api_url, e
            ))
        })?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(CanvasError::Network(format!(
                "Marketplace API error {}: {}",
                status, body
            )))
        }
    }

    /// Search for marketplace items
    pub async fn search_items(
        &self,
        query: &str,
        filters: &SearchFilters,
        page: u32,
        limit: u32,
    ) -> CanvasResult<Vec<MarketplaceItem>> {
        log::info!("Searching marketplace for: {}", query);
        if self.is_mock_backend() {
            let item = self.sample_item("mock-item-1", query);
            return Ok(vec![item]);
        }

        let client = reqwest::Client::new();
        let mut params: Vec<(&str, String)> = vec![
            ("q", query.to_string()),
            ("page", page.to_string()),
            ("limit", limit.to_string()),
        ];
        if let Some(item_type) = &filters.item_type {
            params.push(("item_type", format!("{:?}", item_type)));
        }
        if !filters.tags.is_empty() {
            params.push(("tags", filters.tags.join(",")));
        }
        if let Some(min_rating) = filters.min_rating {
            params.push(("min_rating", min_rating.to_string()));
        }
        if let Some(max_price) = filters.max_price {
            params.push(("max_price", max_price.to_string()));
        }
        if filters.free_only {
            params.push(("free_only", "true".to_string()));
        }
        if let Some(author) = &filters.author {
            params.push(("author", author.clone()));
        }
        if let Some(compatibility) = &filters.compatibility {
            params.push(("compatibility", compatibility.clone()));
        }
        if let Some(difficulty) = &filters.difficulty {
            params.push(("difficulty", difficulty.clone()));
        }

        let response = self
            .send_request(client.get(self.endpoint("/items/search")).query(&params))
            .await?;
        let payload: serde_json::Value = response.json().await.map_err(|e| {
            CanvasError::Serialization(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse marketplace search response: {}", e),
            )))
        })?;

        if payload.is_array() {
            serde_json::from_value(payload).map_err(CanvasError::from)
        } else if let Some(items) = payload.get("items") {
            serde_json::from_value(items.clone()).map_err(CanvasError::from)
        } else {
            Err(CanvasError::Serialization(serde_json::Error::io(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Marketplace search response missing 'items' payload",
                ),
            )))
        }
    }

    /// Get item details
    pub async fn get_item(&mut self, item_id: &str) -> CanvasResult<MarketplaceItem> {
        // Check cache first
        if let Some(item) = self.cache.get(item_id) {
            return Ok(item.clone());
        }
        log::info!("Fetching item details for: {}", item_id);

        let item = if self.is_mock_backend() {
            self.sample_item(item_id, "Mock Item")
        } else {
            let client = reqwest::Client::new();
            let response = self
                .send_request(client.get(self.endpoint(&format!("/items/{}", item_id))))
                .await?;
            response.json::<MarketplaceItem>().await.map_err(|e| {
                CanvasError::Serialization(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse marketplace item response: {}", e),
                )))
            })?
        };

        // Cache the item
        self.cache.insert(item_id.to_string(), item.clone());
        Ok(item)
    }

    /// Download item content
    pub async fn download_item(&self, item_id: &str) -> CanvasResult<Vec<u8>> {
        log::info!("Downloading item: {}", item_id);
        if self.is_mock_backend() {
            return Ok(vec![0u8; 32]);
        }

        let client = reqwest::Client::new();
        let response = self
            .send_request(client.get(self.endpoint(&format!("/items/{}/download", item_id))))
            .await?;
        response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(|e| {
                CanvasError::Network(format!(
                    "Failed to read downloaded marketplace content: {}",
                    e
                ))
            })
    }

    /// Upload item to marketplace
    pub async fn upload_item(
        &self,
        item: &MarketplaceItem,
        content: &[u8],
    ) -> CanvasResult<String> {
        log::info!("Uploading item: {}", item.name);
        if self.is_mock_backend() {
            return Ok(format!("mock-upload-{}", item.id));
        }

        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "metadata": item,
            "content_hex": hex::encode(content),
        });
        let response = self
            .send_request(client.post(self.endpoint("/items/upload")).json(&payload))
            .await?;
        let body: serde_json::Value = response.json().await.map_err(|e| {
            CanvasError::Serialization(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse upload response: {}", e),
            )))
        })?;

        if let Some(id) = body.get("id").and_then(|v| v.as_str()) {
            Ok(id.to_string())
        } else if let Some(id) = body.get("item_id").and_then(|v| v.as_str()) {
            Ok(id.to_string())
        } else {
            Err(CanvasError::Serialization(serde_json::Error::io(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Upload response missing item id",
                ),
            )))
        }
    }

    /// Get user profile
    pub async fn get_user_profile(&self, username: &str) -> CanvasResult<UserProfile> {
        log::info!("Fetching user profile for: {}", username);
        if self.is_mock_backend() {
            return Ok(UserProfile {
                username: username.to_string(),
                display_name: "Mock User".to_string(),
                email: "mock@example.com".to_string(),
                avatar_url: None,
                bio: "A mock user profile".to_string(),
                location: None,
                website: None,
                social_links: HashMap::new(),
                reputation_score: 4.5,
                items_published: 0,
                total_downloads: 0,
                member_since: Utc::now(),
                verified: false,
            });
        }

        let client = reqwest::Client::new();
        let response = self
            .send_request(client.get(self.endpoint(&format!("/users/{}", username))))
            .await?;
        response.json::<UserProfile>().await.map_err(|e| {
            CanvasError::Serialization(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse user profile response: {}", e),
            )))
        })
    }

    /// Get item reviews
    pub async fn get_item_reviews(
        &self,
        item_id: &str,
        page: u32,
        limit: u32,
    ) -> CanvasResult<Vec<Review>> {
        log::info!("Fetching reviews for item: {}", item_id);
        if self.is_mock_backend() {
            return Ok(Vec::new());
        }

        let client = reqwest::Client::new();
        let response = self
            .send_request(
                client
                    .get(self.endpoint(&format!("/items/{}/reviews", item_id)))
                    .query(&[("page", page.to_string()), ("limit", limit.to_string())]),
            )
            .await?;
        let payload: serde_json::Value = response.json().await.map_err(|e| {
            CanvasError::Serialization(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse reviews response: {}", e),
            )))
        })?;

        if payload.is_array() {
            serde_json::from_value(payload).map_err(CanvasError::from)
        } else if let Some(items) = payload.get("reviews") {
            serde_json::from_value(items.clone()).map_err(CanvasError::from)
        } else {
            Err(CanvasError::Serialization(serde_json::Error::io(
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid reviews payload"),
            )))
        }
    }

    /// Submit a review
    pub async fn submit_review(&self, review: &Review) -> CanvasResult<()> {
        log::info!("Submitting review for item: {}", review.item_id);
        if self.is_mock_backend() {
            return Ok(());
        }
        let client = reqwest::Client::new();
        self.send_request(
            client
                .post(self.endpoint(&format!("/items/{}/reviews", review.item_id)))
                .json(review),
        )
        .await?;
        Ok(())
    }

    /// Get trending items
    pub async fn get_trending_items(&self, limit: u32) -> CanvasResult<Vec<MarketplaceItem>> {
        log::info!("Fetching trending items");
        if self.is_mock_backend() {
            return Ok((0..limit)
                .map(|i| self.sample_item(&format!("trend-{}", i), &format!("Trending {}", i)))
                .collect());
        }

        let client = reqwest::Client::new();
        let response = self
            .send_request(
                client
                    .get(self.endpoint("/items/trending"))
                    .query(&[("limit", limit.to_string())]),
            )
            .await?;
        response.json::<Vec<MarketplaceItem>>().await.map_err(|e| {
            CanvasError::Serialization(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse trending items response: {}", e),
            )))
        })
    }

    /// Get recommended items
    pub async fn get_recommended_items(
        &self,
        user_id: &str,
        limit: u32,
    ) -> CanvasResult<Vec<MarketplaceItem>> {
        log::info!("Fetching recommended items for user: {}", user_id);
        if self.is_mock_backend() {
            return Ok((0..limit)
                .map(|i| {
                    self.sample_item(
                        &format!("recommended-{}-{}", user_id, i),
                        &format!("Recommended {}", i),
                    )
                })
                .collect());
        }

        let client = reqwest::Client::new();
        let response = self
            .send_request(
                client
                    .get(self.endpoint(&format!("/users/{}/recommendations", user_id)))
                    .query(&[("limit", limit.to_string())]),
            )
            .await?;
        response.json::<Vec<MarketplaceItem>>().await.map_err(|e| {
            CanvasError::Serialization(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse recommended items response: {}", e),
            )))
        })
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

impl Default for LocalMarketplace {
    fn default() -> Self {
        Self::new()
    }
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
                let matches_query = query.is_empty()
                    || item.name.to_lowercase().contains(&query.to_lowercase())
                    || item
                        .description
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                    || item
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query.to_lowercase()));

                let matches_type = filters.item_type.as_ref().is_none_or(|t| {
                    std::mem::discriminant(&item.item_type) == std::mem::discriminant(t)
                });
                let matches_rating = filters.min_rating.is_none_or(|r| item.rating >= r);
                let matches_price = !filters.free_only || item.price.is_none();

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
