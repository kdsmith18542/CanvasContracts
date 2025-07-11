//! Tests for ecosystem features

use canvascontract::{
    marketplace::{MarketplaceClient, LocalMarketplace, MarketplaceItem, MarketplaceItemType, CustomNodeItem, TemplateItem},
    sdk::{CanvasSdk, SdkConfig, GraphBuilder, TemplateBuilder, PluginRegistry, PluginCapability},
    community::{CommunityManager, CommunityUser, UserRole, Project, ProjectVisibility, ProjectStatus},
    nodes::custom::CustomNodeBuilder,
    types::{Graph, NodeType},
};

#[test]
fn test_marketplace_client_creation() {
    let client = MarketplaceClient::new("https://api.example.com".to_string());
    assert_eq!(client.api_url, "https://api.example.com");
    assert!(client.api_key.is_none());
    
    let client_with_key = client.with_api_key("test_key".to_string());
    assert_eq!(client_with_key.api_key, Some("test_key".to_string()));
}

#[test]
fn test_local_marketplace_operations() {
    let mut marketplace = LocalMarketplace::new();
    
    // Create a custom node item
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
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        price: None,
        license: "MIT".to_string(),
        dependencies: vec![],
        compatibility: vec!["1.0.0".to_string()],
        size_bytes: 1024,
        hash: "test_hash".to_string(),
    };

    let node_definition = CustomNodeBuilder::new(
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
    let filters = canvascontract::marketplace::SearchFilters {
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
}

#[test]
fn test_sdk_creation_and_usage() {
    let config = SdkConfig {
        api_version: "1.0.0".to_string(),
        features: vec!["custom_nodes".to_string()],
        debug_mode: false,
        log_level: "info".to_string(),
        cache_enabled: true,
        max_cache_size: 1000,
    };

    let sdk = CanvasSdk::new(config);
    assert!(sdk.is_ok());
    
    let sdk = sdk.unwrap();
    let info = sdk.get_info();
    assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(info.plugins_count, 0);
}

#[test]
fn test_graph_builder() {
    let graph = GraphBuilder::new()
        .add_node(NodeType::Start, (0.0, 0.0))
        .add_node(NodeType::Logic, (100.0, 0.0))
        .add_node(NodeType::End, (200.0, 0.0))
        .connect("node_0", "node_1")
        .connect("node_1", "node_2")
        .build();

    assert_eq!(graph.get_nodes().len(), 3);
    assert_eq!(graph.get_edges().len(), 2);
}

#[test]
fn test_template_builder() {
    let graph = Graph::new();
    let template = TemplateBuilder::new(
        "Test Template".to_string(),
        "A test template".to_string(),
    )
    .metadata("difficulty".to_string(), serde_json::json!("beginner"))
    .graph(graph)
    .build();

    assert_eq!(template.name, "Test Template");
    assert_eq!(template.description, "A test template");
    assert_eq!(template.metadata.get("difficulty"), Some(&serde_json::json!("beginner")));
}

#[test]
fn test_plugin_registry() {
    let config = SdkConfig {
        api_version: "1.0.0".to_string(),
        features: vec![],
        debug_mode: false,
        log_level: "info".to_string(),
        cache_enabled: true,
        max_cache_size: 1000,
    };

    let mut registry = PluginRegistry::new(config);
    assert_eq!(registry.get_all_plugins().len(), 0);
    
    // Test getting plugins by capability
    let validators = registry.get_plugins_by_capability(&PluginCapability::Validators);
    assert_eq!(validators.len(), 0);
}

#[test]
fn test_community_manager() {
    let mut manager = CommunityManager::new();
    
    // Register a user
    let user_id = manager.register_user(
        "testuser".to_string(),
        "test@example.com".to_string(),
        "password_hash".to_string(),
    ).unwrap();

    assert!(manager.get_user(&user_id).is_some());
    assert!(manager.get_user_by_username("testuser").is_some());
    
    // Test duplicate registration
    let result = manager.register_user(
        "testuser".to_string(),
        "test2@example.com".to_string(),
        "password_hash".to_string(),
    );
    assert!(result.is_err());
}

#[test]
fn test_project_creation() {
    let mut manager = CommunityManager::new();
    
    let user_id = manager.register_user(
        "testuser".to_string(),
        "test@example.com".to_string(),
        "password_hash".to_string(),
    ).unwrap();

    let graph = Graph::new();
    let project_id = manager.create_project(
        "Test Project".to_string(),
        "A test project".to_string(),
        user_id.clone(),
        graph,
    ).unwrap();

    assert!(manager.get_project(&project_id).is_some());
    
    let project = manager.get_project(&project_id).unwrap();
    assert_eq!(project.name, "Test Project");
    assert_eq!(project.owner_id, user_id);
    assert_eq!(project.visibility, ProjectVisibility::Private);
    assert_eq!(project.status, ProjectStatus::Draft);
}

#[test]
fn test_project_collaboration() {
    let mut manager = CommunityManager::new();
    
    let owner_id = manager.register_user(
        "owner".to_string(),
        "owner@example.com".to_string(),
        "password_hash".to_string(),
    ).unwrap();

    let collaborator_id = manager.register_user(
        "collaborator".to_string(),
        "collaborator@example.com".to_string(),
        "password_hash".to_string(),
    ).unwrap();

    let graph = Graph::new();
    let project_id = manager.create_project(
        "Test Project".to_string(),
        "A test project".to_string(),
        owner_id.clone(),
        graph,
    ).unwrap();

    // Add collaborator
    assert!(manager.add_collaborator(
        &project_id,
        &owner_id,
        &collaborator_id,
        canvascontract::community::CollaboratorRole::Editor,
    ).is_ok());

    let project = manager.get_project(&project_id).unwrap();
    assert_eq!(project.collaborators.len(), 1);
    assert_eq!(project.collaborators[0].user_id, collaborator_id);
}

#[test]
fn test_user_following() {
    let mut manager = CommunityManager::new();
    
    let user1_id = manager.register_user(
        "user1".to_string(),
        "user1@example.com".to_string(),
        "password_hash".to_string(),
    ).unwrap();

    let user2_id = manager.register_user(
        "user2".to_string(),
        "user2@example.com".to_string(),
        "password_hash".to_string(),
    ).unwrap();

    // Follow user
    assert!(manager.follow_user(&user1_id, &user2_id).is_ok());

    let user1 = manager.get_user(&user1_id).unwrap();
    let user2 = manager.get_user(&user2_id).unwrap();

    assert!(user1.following.contains(&user2_id));
    assert!(user2.followers.contains(&user1_id));
    
    // Unfollow user
    assert!(manager.unfollow_user(&user1_id, &user2_id).is_ok());
    
    let user1 = manager.get_user(&user1_id).unwrap();
    let user2 = manager.get_user(&user2_id).unwrap();

    assert!(!user1.following.contains(&user2_id));
    assert!(!user2.followers.contains(&user1_id));
}

#[test]
fn test_user_stats() {
    let mut manager = CommunityManager::new();
    
    let user_id = manager.register_user(
        "testuser".to_string(),
        "test@example.com".to_string(),
        "password_hash".to_string(),
    ).unwrap();

    let stats = manager.get_user_stats(&user_id).unwrap();
    assert_eq!(stats.user_id, user_id);
    assert_eq!(stats.projects_count, 0);
    assert_eq!(stats.comments_count, 0);
    assert_eq!(stats.posts_count, 0);
    assert_eq!(stats.tutorials_count, 0);
    assert_eq!(stats.followers_count, 0);
    assert_eq!(stats.following_count, 0);
    assert_eq!(stats.badges_count, 0);
    assert_eq!(stats.reputation, 0.0);
}

#[test]
fn test_comment_system() {
    let mut manager = CommunityManager::new();
    
    let user_id = manager.register_user(
        "testuser".to_string(),
        "test@example.com".to_string(),
        "password_hash".to_string(),
    ).unwrap();

    // Add comment
    let comment_id = manager.add_comment(
        &user_id,
        "This is a test comment".to_string(),
        None,
    ).unwrap();

    // Get comments
    let comments = manager.get_comments(None);
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].id, comment_id);
    assert_eq!(comments[0].content, "This is a test comment");
    assert_eq!(comments[0].author_id, user_id);
}

#[test]
fn test_forum_posts() {
    let mut manager = CommunityManager::new();
    
    let user_id = manager.register_user(
        "testuser".to_string(),
        "test@example.com".to_string(),
        "password_hash".to_string(),
    ).unwrap();

    // Create forum post
    let post_id = manager.create_forum_post(
        "Test Post".to_string(),
        "This is a test forum post".to_string(),
        user_id.clone(),
        "general".to_string(),
        vec!["test".to_string()],
    ).unwrap();

    // Get forum posts
    let posts = manager.get_forum_posts(None);
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, post_id);
    assert_eq!(posts[0].title, "Test Post");
    assert_eq!(posts[0].content, "This is a test forum post");
    assert_eq!(posts[0].author_id, user_id);
    assert_eq!(posts[0].category, "general");
}

#[test]
fn test_tutorials() {
    let mut manager = CommunityManager::new();
    
    let user_id = manager.register_user(
        "testuser".to_string(),
        "test@example.com".to_string(),
        "password_hash".to_string(),
    ).unwrap();

    // Create tutorial
    let tutorial_id = manager.create_tutorial(
        "Test Tutorial".to_string(),
        "This is a test tutorial".to_string(),
        user_id.clone(),
        canvascontract::community::TutorialDifficulty::Beginner,
        30,
        vec![],
        vec!["tutorial".to_string()],
    ).unwrap();

    // Get tutorials
    let tutorials = manager.get_tutorials(None);
    assert_eq!(tutorials.len(), 1);
    assert_eq!(tutorials[0].id, tutorial_id);
    assert_eq!(tutorials[0].title, "Test Tutorial");
    assert_eq!(tutorials[0].content, "This is a test tutorial");
    assert_eq!(tutorials[0].author_id, user_id);
    assert_eq!(tutorials[0].duration_minutes, 30);
}

#[test]
fn test_badge_system() {
    let mut manager = CommunityManager::new();
    
    let user_id = manager.register_user(
        "testuser".to_string(),
        "test@example.com".to_string(),
        "password_hash".to_string(),
    ).unwrap();

    let badge = canvascontract::community::Badge {
        id: "first_project".to_string(),
        name: "First Project".to_string(),
        description: "Created your first project".to_string(),
        icon_url: "badge.png".to_string(),
        earned_at: chrono::Utc::now(),
        rarity: canvascontract::community::BadgeRarity::Common,
    };

    // Award badge
    assert!(manager.award_badge(&user_id, badge.clone()).is_ok());
    
    let user = manager.get_user(&user_id).unwrap();
    assert_eq!(user.badges.len(), 1);
    assert_eq!(user.badges[0].id, "first_project");
    
    // Try to award same badge again (should not duplicate)
    assert!(manager.award_badge(&user_id, badge).is_ok());
    let user = manager.get_user(&user_id).unwrap();
    assert_eq!(user.badges.len(), 1); // Should still be 1, not 2
}

#[test]
fn test_project_updates() {
    let mut manager = CommunityManager::new();
    
    let user_id = manager.register_user(
        "testuser".to_string(),
        "test@example.com".to_string(),
        "password_hash".to_string(),
    ).unwrap();

    let graph = Graph::new();
    let project_id = manager.create_project(
        "Test Project".to_string(),
        "A test project".to_string(),
        user_id.clone(),
        graph,
    ).unwrap();

    // Update project
    let updates = canvascontract::community::ProjectUpdate {
        name: Some("Updated Project".to_string()),
        description: Some("An updated test project".to_string()),
        visibility: Some(ProjectVisibility::Public),
        status: Some(ProjectStatus::InProgress),
        graph: None,
    };

    assert!(manager.update_project(&project_id, &user_id, updates).is_ok());
    
    let project = manager.get_project(&project_id).unwrap();
    assert_eq!(project.name, "Updated Project");
    assert_eq!(project.description, "An updated test project");
    assert_eq!(project.visibility, ProjectVisibility::Public);
    assert_eq!(project.status, ProjectStatus::InProgress);
}

#[test]
fn test_marketplace_search() {
    let mut marketplace = LocalMarketplace::new();
    
    // Add multiple items
    let metadata1 = MarketplaceItem {
        id: "node1".to_string(),
        name: "Math Node".to_string(),
        description: "A mathematical operation node".to_string(),
        author: "math_dev".to_string(),
        version: "1.0.0".to_string(),
        item_type: MarketplaceItemType::CustomNode,
        tags: vec!["math".to_string(), "calculation".to_string()],
        rating: 4.5,
        downloads: 100,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        price: None,
        license: "MIT".to_string(),
        dependencies: vec![],
        compatibility: vec!["1.0.0".to_string()],
        size_bytes: 1024,
        hash: "hash1".to_string(),
    };

    let metadata2 = MarketplaceItem {
        id: "template1".to_string(),
        name: "ERC-20 Template".to_string(),
        description: "A token template".to_string(),
        author: "token_dev".to_string(),
        version: "1.0.0".to_string(),
        item_type: MarketplaceItemType::Template,
        tags: vec!["token".to_string(), "erc20".to_string()],
        rating: 4.8,
        downloads: 200,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        price: Some(5.99),
        license: "MIT".to_string(),
        dependencies: vec![],
        compatibility: vec!["1.0.0".to_string()],
        size_bytes: 2048,
        hash: "hash2".to_string(),
    };

    let node_definition = CustomNodeBuilder::new(
        "math-node".to_string(),
        "Math Node".to_string(),
    )
    .composite("{}".to_string())
    .build();

    let custom_node_item = CustomNodeItem {
        metadata: metadata1,
        node_definition,
        examples: vec![],
        documentation: "Math node documentation".to_string(),
    };

    let template_item = TemplateItem {
        metadata: metadata2,
        graph: Graph::new(),
        description: "ERC-20 token template".to_string(),
        use_cases: vec!["Token creation".to_string()],
        difficulty: canvascontract::marketplace::TemplateDifficulty::Beginner,
        estimated_gas: 1000,
    };

    marketplace.add_custom_node(custom_node_item).unwrap();
    marketplace.add_template(template_item).unwrap();

    // Test search by query
    let filters = canvascontract::marketplace::SearchFilters {
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

    let math_results = marketplace.search_items("math", &filters);
    assert_eq!(math_results.len(), 1);
    assert_eq!(math_results[0].name, "Math Node");

    let token_results = marketplace.search_items("token", &filters);
    assert_eq!(token_results.len(), 1);
    assert_eq!(token_results[0].name, "ERC-20 Template");

    // Test search by type
    let node_filters = canvascontract::marketplace::SearchFilters {
        item_type: Some(MarketplaceItemType::CustomNode),
        tags: vec![],
        min_rating: None,
        max_price: None,
        free_only: false,
        author: None,
        compatibility: None,
        difficulty: None,
        date_range: None,
    };

    let node_results = marketplace.search_items("", &node_filters);
    assert_eq!(node_results.len(), 1);
    assert_eq!(node_results[0].item_type, MarketplaceItemType::CustomNode);

    // Test free only filter
    let free_filters = canvascontract::marketplace::SearchFilters {
        item_type: None,
        tags: vec![],
        min_rating: None,
        max_price: None,
        free_only: true,
        author: None,
        compatibility: None,
        difficulty: None,
        date_range: None,
    };

    let free_results = marketplace.search_items("", &free_filters);
    assert_eq!(free_results.len(), 1);
    assert_eq!(free_results[0].price, None);
} 