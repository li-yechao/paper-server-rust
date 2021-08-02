use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{user::UserId, Id, OrderBy, Pagination, PaginationList, Result};

#[async_trait]
pub trait PaperService: Send + Sync {
    async fn create_paper(&self, viewer_id: UserId, user_id: UserId) -> Result<Paper>;

    async fn delete_paper(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        paper_id: PaperId,
    ) -> Result<()>;

    async fn select_paper(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        paper_id: PaperId,
    ) -> Result<Paper>;

    async fn select_paper_page_of_repository(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        pagination: Pagination<PaperId>,
        order_by: OrderBy<PaperOrderField>,
        deleted: bool,
    ) -> Result<PaginationList<Paper>>;

    async fn can_viewer_read_paper(
        &self,
        viewer_id: UserId,
        owner_id: UserId,
        paper_id: PaperId,
    ) -> Result<Paper>;

    async fn can_viewer_write_paper(
        &self,
        viewer_id: UserId,
        owner_id: UserId,
        paper_id: PaperId,
    ) -> Result<Paper>;
}

pub enum PaperOrderField {
    Id,

    UpdatedAt,
}

pub type PaperId = Id<Paper>;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paper {
    pub id: PaperId,

    pub user_id: UserId,

    pub created_at: u64,

    pub updated_at: u64,

    pub deleted_at: Option<u64>,

    pub title: Option<String>,

    pub tags: Option<Vec<String>>,
}
