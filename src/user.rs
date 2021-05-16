use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{Id, Result};

#[async_trait]
pub trait UserService: Send + Sync {
    async fn create_user(&self, user: CreateUserInput) -> Result<User>;

    async fn select_user(&self, identifier: UserIdentifier) -> Result<User>;

    async fn update_user(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        input: UpdateUserInput,
    ) -> Result<User>;

    async fn can_viewer_read_user(&self, viewer_id: UserId, user_id: UserId) -> Result<User>;

    async fn can_viewer_write_user(&self, viewer_id: UserId, user_id: UserId) -> Result<User>;

    async fn can_viewer_administer_user(&self, viewer_id: UserId, user_id: UserId) -> Result<User>;
}

pub type UserId = Id<User>;

#[derive(Debug, Clone, PartialEq)]
pub enum UserIdentifier {
    Id(UserId),

    Name(String),

    GithubUserId(u64),

    GoogleUserId(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreateUserInput {
    Github { github_user: serde_json::Value },

    Google { google_user: serde_json::Value },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateUserInput {
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,

    pub created_at: u64,

    pub name: String,

    pub github_user: Option<serde_json::Value>,

    pub google_user: Option<serde_json::Value>,
}
