//! Community features for Canvas Contracts

use crate::{
    error::{CanvasError, CanvasResult},
    types::{Graph, Node, NodeId},
    marketplace::{MarketplaceItem, UserProfile},
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// User role in the community
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Guest,
    User,
    Contributor,
    Moderator,
    Admin,
}

/// User permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    pub can_publish: bool,
    pub can_comment: bool,
    pub can_rate: bool,
    pub can_moderate: bool,
    pub can_admin: bool,
}

/// Community user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub permissions: UserPermissions,
    pub profile: UserProfile,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub reputation: f64,
    pub badges: Vec<Badge>,
    pub following: Vec<String>, // User IDs
    pub followers: Vec<String>, // User IDs
}

/// Badge for user achievements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_url: String,
    pub earned_at: DateTime<Utc>,
    pub rarity: BadgeRarity,
}

/// Badge rarity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BadgeRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Project for collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner_id: String,
    pub collaborators: Vec<ProjectCollaborator>,
    pub graph: Graph,
    pub visibility: ProjectVisibility,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: String,
    pub status: ProjectStatus,
}

/// Project collaborator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCollaborator {
    pub user_id: String,
    pub role: CollaboratorRole,
    pub joined_at: DateTime<Utc>,
    pub permissions: CollaboratorPermissions,
}

/// Collaborator role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaboratorRole {
    Viewer,
    Editor,
    Admin,
}

/// Collaborator permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorPermissions {
    pub can_view: bool,
    pub can_edit: bool,
    pub can_comment: bool,
    pub can_invite: bool,
    pub can_delete: bool,
}

/// Project visibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectVisibility {
    Private,
    Public,
    Unlisted,
}

/// Project status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectStatus {
    Draft,
    InProgress,
    Review,
    Published,
    Archived,
}

/// Comment on projects or items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub author_id: String,
    pub content: String,
    pub parent_id: Option<String>, // For replies
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub likes: u32,
    pub dislikes: u32,
    pub is_edited: bool,
    pub is_deleted: bool,
}

/// Forum post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumPost {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
    pub category: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub views: u32,
    pub likes: u32,
    pub replies: u32,
    pub is_pinned: bool,
    pub is_locked: bool,
    pub status: PostStatus,
}

/// Post status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostStatus {
    Active,
    Closed,
    Archived,
    Deleted,
}

/// Tutorial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tutorial {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
    pub difficulty: TutorialDifficulty,
    pub duration_minutes: u32,
    pub prerequisites: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub views: u32,
    pub rating: f64,
    pub status: TutorialStatus,
}

/// Tutorial difficulty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TutorialDifficulty {
    Beginner,
    Intermediate,
    Advanced,
}

/// Tutorial status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TutorialStatus {
    Draft,
    Published,
    Archived,
}

/// Community manager
pub struct CommunityManager {
    users: HashMap<String, CommunityUser>,
    projects: HashMap<String, Project>,
    comments: HashMap<String, Comment>,
    forum_posts: HashMap<String, ForumPost>,
    tutorials: HashMap<String, Tutorial>,
}

impl CommunityManager {
    /// Create a new community manager
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            projects: HashMap::new(),
            comments: HashMap::new(),
            forum_posts: HashMap::new(),
            tutorials: HashMap::new(),
        }
    }

    /// Register a new user
    pub fn register_user(
        &mut self,
        username: String,
        email: String,
        password_hash: String,
    ) -> CanvasResult<String> {
        // Check if username already exists
        if self.users.values().any(|u| u.username == username) {
            return Err(CanvasError::Validation("Username already exists".to_string()));
        }

        // Check if email already exists
        if self.users.values().any(|u| u.email == email) {
            return Err(CanvasError::Validation("Email already exists".to_string()));
        }

        let user_id = format!("user_{}", uuid::Uuid::new_v4());
        let now = Utc::now();

        let user = CommunityUser {
            id: user_id.clone(),
            username,
            email,
            role: UserRole::User,
            permissions: UserPermissions {
                can_publish: true,
                can_comment: true,
                can_rate: true,
                can_moderate: false,
                can_admin: false,
            },
            profile: UserProfile {
                username: user_id.clone(),
                display_name: "New User".to_string(),
                email,
                avatar_url: None,
                bio: "".to_string(),
                location: None,
                website: None,
                social_links: HashMap::new(),
                reputation_score: 0.0,
                items_published: 0,
                total_downloads: 0,
                member_since: now,
                verified: false,
            },
            created_at: now,
            last_active: now,
            reputation: 0.0,
            badges: vec![],
            following: vec![],
            followers: vec![],
        };

        self.users.insert(user_id.clone(), user);
        Ok(user_id)
    }

    /// Get user by ID
    pub fn get_user(&self, user_id: &str) -> Option<&CommunityUser> {
        self.users.get(user_id)
    }

    /// Get user by username
    pub fn get_user_by_username(&self, username: &str) -> Option<&CommunityUser> {
        self.users.values().find(|u| u.username == username)
    }

    /// Update user profile
    pub fn update_user_profile(
        &mut self,
        user_id: &str,
        profile: UserProfile,
    ) -> CanvasResult<()> {
        if let Some(user) = self.users.get_mut(user_id) {
            user.profile = profile;
            Ok(())
        } else {
            Err(CanvasError::NotFound(format!("User '{}' not found", user_id)))
        }
    }

    /// Create a new project
    pub fn create_project(
        &mut self,
        name: String,
        description: String,
        owner_id: String,
        graph: Graph,
    ) -> CanvasResult<String> {
        if !self.users.contains_key(&owner_id) {
            return Err(CanvasError::NotFound(format!("User '{}' not found", owner_id)));
        }

        let project_id = format!("project_{}", uuid::Uuid::new_v4());
        let now = Utc::now();

        let project = Project {
            id: project_id.clone(),
            name,
            description,
            owner_id,
            collaborators: vec![],
            graph,
            visibility: ProjectVisibility::Private,
            tags: vec![],
            created_at: now,
            updated_at: now,
            version: "1.0.0".to_string(),
            status: ProjectStatus::Draft,
        };

        self.projects.insert(project_id.clone(), project);
        Ok(project_id)
    }

    /// Get project by ID
    pub fn get_project(&self, project_id: &str) -> Option<&Project> {
        self.projects.get(project_id)
    }

    /// Update project
    pub fn update_project(
        &mut self,
        project_id: &str,
        user_id: &str,
        updates: ProjectUpdate,
    ) -> CanvasResult<()> {
        if let Some(project) = self.projects.get_mut(project_id) {
            // Check permissions
            if project.owner_id != user_id && 
               !project.collaborators.iter().any(|c| c.user_id == user_id && c.role == CollaboratorRole::Admin) {
                return Err(CanvasError::PermissionDenied("Insufficient permissions".to_string()));
            }

            // Apply updates
            if let Some(name) = updates.name {
                project.name = name;
            }
            if let Some(description) = updates.description {
                project.description = description;
            }
            if let Some(visibility) = updates.visibility {
                project.visibility = visibility;
            }
            if let Some(status) = updates.status {
                project.status = status;
            }
            if let Some(graph) = updates.graph {
                project.graph = graph;
            }

            project.updated_at = Utc::now();
            Ok(())
        } else {
            Err(CanvasError::NotFound(format!("Project '{}' not found", project_id)))
        }
    }

    /// Add collaborator to project
    pub fn add_collaborator(
        &mut self,
        project_id: &str,
        owner_id: &str,
        collaborator_id: &str,
        role: CollaboratorRole,
    ) -> CanvasResult<()> {
        if let Some(project) = self.projects.get_mut(project_id) {
            if project.owner_id != owner_id {
                return Err(CanvasError::PermissionDenied("Only project owner can add collaborators".to_string()));
            }

            if !self.users.contains_key(collaborator_id) {
                return Err(CanvasError::NotFound(format!("User '{}' not found", collaborator_id)));
            }

            let permissions = match role {
                CollaboratorRole::Viewer => CollaboratorPermissions {
                    can_view: true,
                    can_edit: false,
                    can_comment: true,
                    can_invite: false,
                    can_delete: false,
                },
                CollaboratorRole::Editor => CollaboratorPermissions {
                    can_view: true,
                    can_edit: true,
                    can_comment: true,
                    can_invite: false,
                    can_delete: false,
                },
                CollaboratorRole::Admin => CollaboratorPermissions {
                    can_view: true,
                    can_edit: true,
                    can_comment: true,
                    can_invite: true,
                    can_delete: true,
                },
            };

            let collaborator = ProjectCollaborator {
                user_id: collaborator_id.to_string(),
                role,
                joined_at: Utc::now(),
                permissions,
            };

            project.collaborators.push(collaborator);
            Ok(())
        } else {
            Err(CanvasError::NotFound(format!("Project '{}' not found", project_id)))
        }
    }

    /// Add comment
    pub fn add_comment(
        &mut self,
        author_id: &str,
        content: String,
        parent_id: Option<String>,
    ) -> CanvasResult<String> {
        if !self.users.contains_key(author_id) {
            return Err(CanvasError::NotFound(format!("User '{}' not found", author_id)));
        }

        let comment_id = format!("comment_{}", uuid::Uuid::new_v4());
        let now = Utc::now();

        let comment = Comment {
            id: comment_id.clone(),
            author_id: author_id.to_string(),
            content,
            parent_id,
            created_at: now,
            updated_at: now,
            likes: 0,
            dislikes: 0,
            is_edited: false,
            is_deleted: false,
        };

        self.comments.insert(comment_id.clone(), comment);
        Ok(comment_id)
    }

    /// Get comments for an item
    pub fn get_comments(&self, parent_id: Option<&str>) -> Vec<&Comment> {
        self.comments
            .values()
            .filter(|c| c.parent_id.as_deref() == parent_id && !c.is_deleted)
            .collect()
    }

    /// Create forum post
    pub fn create_forum_post(
        &mut self,
        title: String,
        content: String,
        author_id: String,
        category: String,
        tags: Vec<String>,
    ) -> CanvasResult<String> {
        if !self.users.contains_key(&author_id) {
            return Err(CanvasError::NotFound(format!("User '{}' not found", author_id)));
        }

        let post_id = format!("post_{}", uuid::Uuid::new_v4());
        let now = Utc::now();

        let post = ForumPost {
            id: post_id.clone(),
            title,
            content,
            author_id,
            category,
            tags,
            created_at: now,
            updated_at: now,
            views: 0,
            likes: 0,
            replies: 0,
            is_pinned: false,
            is_locked: false,
            status: PostStatus::Active,
        };

        self.forum_posts.insert(post_id.clone(), post);
        Ok(post_id)
    }

    /// Get forum posts
    pub fn get_forum_posts(&self, category: Option<&str>) -> Vec<&ForumPost> {
        self.forum_posts
            .values()
            .filter(|p| {
                category.map_or(true, |c| p.category == c) && 
                p.status == PostStatus::Active
            })
            .collect()
    }

    /// Create tutorial
    pub fn create_tutorial(
        &mut self,
        title: String,
        content: String,
        author_id: String,
        difficulty: TutorialDifficulty,
        duration_minutes: u32,
        prerequisites: Vec<String>,
        tags: Vec<String>,
    ) -> CanvasResult<String> {
        if !self.users.contains_key(&author_id) {
            return Err(CanvasError::NotFound(format!("User '{}' not found", author_id)));
        }

        let tutorial_id = format!("tutorial_{}", uuid::Uuid::new_v4());
        let now = Utc::now();

        let tutorial = Tutorial {
            id: tutorial_id.clone(),
            title,
            content,
            author_id,
            difficulty,
            duration_minutes,
            prerequisites,
            tags,
            created_at: now,
            updated_at: now,
            views: 0,
            rating: 0.0,
            status: TutorialStatus::Draft,
        };

        self.tutorials.insert(tutorial_id.clone(), tutorial);
        Ok(tutorial_id)
    }

    /// Get tutorials
    pub fn get_tutorials(&self, difficulty: Option<TutorialDifficulty>) -> Vec<&Tutorial> {
        self.tutorials
            .values()
            .filter(|t| {
                difficulty.as_ref().map_or(true, |d| std::mem::discriminant(&t.difficulty) == std::mem::discriminant(d)) &&
                t.status == TutorialStatus::Published
            })
            .collect()
    }

    /// Follow user
    pub fn follow_user(&mut self, follower_id: &str, followed_id: &str) -> CanvasResult<()> {
        if follower_id == followed_id {
            return Err(CanvasError::Validation("Cannot follow yourself".to_string()));
        }

        if let Some(follower) = self.users.get_mut(follower_id) {
            if !follower.following.contains(&followed_id.to_string()) {
                follower.following.push(followed_id.to_string());
            }
        } else {
            return Err(CanvasError::NotFound(format!("User '{}' not found", follower_id)));
        }

        if let Some(followed) = self.users.get_mut(followed_id) {
            if !followed.followers.contains(&follower_id.to_string()) {
                followed.followers.push(follower_id.to_string());
            }
        } else {
            return Err(CanvasError::NotFound(format!("User '{}' not found", followed_id)));
        }

        Ok(())
    }

    /// Unfollow user
    pub fn unfollow_user(&mut self, follower_id: &str, followed_id: &str) -> CanvasResult<()> {
        if let Some(follower) = self.users.get_mut(follower_id) {
            follower.following.retain(|id| id != followed_id);
        }

        if let Some(followed) = self.users.get_mut(followed_id) {
            followed.followers.retain(|id| id != follower_id);
        }

        Ok(())
    }

    /// Award badge to user
    pub fn award_badge(&mut self, user_id: &str, badge: Badge) -> CanvasResult<()> {
        if let Some(user) = self.users.get_mut(user_id) {
            if !user.badges.iter().any(|b| b.id == badge.id) {
                user.badges.push(badge);
            }
            Ok(())
        } else {
            Err(CanvasError::NotFound(format!("User '{}' not found", user_id)))
        }
    }

    /// Get user statistics
    pub fn get_user_stats(&self, user_id: &str) -> Option<UserStats> {
        if let Some(user) = self.users.get(user_id) {
            let projects_count = self.projects.values().filter(|p| p.owner_id == user_id).count();
            let comments_count = self.comments.values().filter(|c| c.author_id == user_id).count();
            let posts_count = self.forum_posts.values().filter(|p| p.author_id == user_id).count();
            let tutorials_count = self.tutorials.values().filter(|t| t.author_id == user_id).count();

            Some(UserStats {
                user_id: user_id.to_string(),
                projects_count,
                comments_count,
                posts_count,
                tutorials_count,
                followers_count: user.followers.len(),
                following_count: user.following.len(),
                badges_count: user.badges.len(),
                reputation: user.reputation,
            })
        } else {
            None
        }
    }
}

/// Project update structure
#[derive(Debug, Clone)]
pub struct ProjectUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<ProjectVisibility>,
    pub status: Option<ProjectStatus>,
    pub graph: Option<Graph>,
}

/// User statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub user_id: String,
    pub projects_count: usize,
    pub comments_count: usize,
    pub posts_count: usize,
    pub tutorials_count: usize,
    pub followers_count: usize,
    pub following_count: usize,
    pub badges_count: usize,
    pub reputation: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_registration() {
        let mut manager = CommunityManager::new();
        
        let user_id = manager.register_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "password_hash".to_string(),
        ).unwrap();

        assert!(manager.get_user(&user_id).is_some());
        assert!(manager.get_user_by_username("testuser").is_some());
    }

    #[test]
    fn test_duplicate_username_registration() {
        let mut manager = CommunityManager::new();
        
        manager.register_user(
            "testuser".to_string(),
            "test1@example.com".to_string(),
            "password_hash".to_string(),
        ).unwrap();

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
    }

    #[test]
    fn test_follow_user() {
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

        manager.follow_user(&user1_id, &user2_id).unwrap();

        let user1 = manager.get_user(&user1_id).unwrap();
        let user2 = manager.get_user(&user2_id).unwrap();

        assert!(user1.following.contains(&user2_id));
        assert!(user2.followers.contains(&user1_id));
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
        assert_eq!(stats.projects_count, 0);
        assert_eq!(stats.followers_count, 0);
        assert_eq!(stats.following_count, 0);
    }
} 