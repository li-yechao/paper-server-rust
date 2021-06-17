use std::ops::Deref;

use async_trait::async_trait;
use bson::{doc, Document};
use lazy_static::lazy_static;
use mongodb::options::{FindOneAndUpdateOptions, FindOneOptions, FindOptions};
use paper::{paper::*, user::*, Error, OrderBy, Pagination, PaginationList, Result};
use shaku::Provider;

use crate::utils::*;

#[derive(Provider)]
#[shaku(interface = PaperService)]
pub struct PaperServiceImpl {
    #[shaku(provide)]
    pub paper_collection: Box<dyn PaperCollection>,

    #[shaku(provide)]
    pub user_service: Box<dyn UserService>,
}

pub trait PaperCollection: Deref<Target = mongodb::Collection> + Send + Sync {}

#[async_trait]
impl PaperService for PaperServiceImpl {
    async fn create_paper(&self, viewer_id: UserId, user_id: UserId) -> Result<Paper> {
        self.user_service
            .can_viewer_write_user(viewer_id, user_id.to_owned())
            .await?;

        let now = now_msec();

        let paper = Paper {
            user_id,
            id: new_id().into(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            title: None,
        };

        self.paper_collection
            .insert_one(to_doc(&paper)?, None)
            .await
            .map_err(|e| Error::unknown(e.to_string()))?;

        Ok(paper.into())
    }

    async fn delete_paper(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        paper_id: PaperId,
    ) -> Result<()> {
        self.can_viewer_write_paper(viewer_id, user_id.to_owned(), paper_id.to_owned())
            .await?;

        self.paper_collection
            .find_one_and_update(
                doc! { "_id": paper_id.to_string() },
                doc! { "$set": { "deleted_at": now_msec() } },
                FindOneAndUpdateOptions::builder()
                    .projection(doc! {})
                    .build(),
            )
            .await
            .map_err(|e| Error::unknown(e.to_string()))?;

        Ok(())
    }

    async fn select_paper(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        paper_id: PaperId,
    ) -> Result<Paper> {
        self.can_viewer_read_paper(viewer_id, user_id, paper_id)
            .await
    }

    async fn select_paper_page_of_repository(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        pagination: Pagination<PaperId>,
        order_by: OrderBy<PaperOrderField>,
        deleted: bool,
    ) -> Result<PaginationList<Paper>> {
        self.user_service
            .can_viewer_read_user(viewer_id, user_id.to_owned())
            .await?;

        let mut filter = doc! { "user_id": user_id.to_string() };
        match deleted {
            true => filter.insert("deleted_at", doc! { "$exists": true, "$ne": null }),
            false => filter.insert("deleted_at", bson::Bson::Null),
        };

        mongodb_select_pagination(
            &self.paper_collection,
            pagination,
            filter,
            paper_order_to_str(order_by),
            FindOptions::builder()
                .projection(PAPER_PROJECTION.to_owned())
                .build(),
        )
        .await
    }

    async fn can_viewer_read_paper(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        paper_id: PaperId,
    ) -> Result<Paper> {
        self.user_service
            .can_viewer_read_user(viewer_id, user_id.to_owned())
            .await?;

        self.find_paper(user_id, paper_id).await
    }

    async fn can_viewer_write_paper(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        paper_id: PaperId,
    ) -> Result<Paper> {
        self.user_service
            .can_viewer_write_user(viewer_id, user_id.to_owned())
            .await?;

        self.find_paper(user_id, paper_id).await
    }
}

impl PaperServiceImpl {
    async fn find_paper(&self, user_id: UserId, paper_id: PaperId) -> Result<Paper> {
        self.paper_collection
            .find_one(
                doc! { "_id": paper_id.to_string(), "user_id": user_id.to_string() },
                FindOneOptions::builder()
                    .projection(PAPER_PROJECTION.to_owned())
                    .build(),
            )
            .await
            .map_err(|e| Error::unknown(e.to_string()))?
            .map(|doc| from_doc::<Paper>(doc))
            .ok_or(Error::not_found("Paper not found".to_owned()))?
    }
}

fn paper_order_to_str(order_by: OrderBy<PaperOrderField>) -> OrderBy<&'static str> {
    OrderBy {
        field: match order_by.field {
            PaperOrderField::Id => "_id",
            PaperOrderField::UpdatedAt => "updated_at",
        },
        direction: order_by.direction,
    }
}

lazy_static! {
    static ref PAPER_PROJECTION: Document = doc! {
        "_id": 1,
        "user_id": 1,
        "created_at": 1,
        "updated_at": 1,
        "deleted_at": 1,
        "title": 1,
    };
}
