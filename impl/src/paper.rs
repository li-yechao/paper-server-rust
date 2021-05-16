use std::ops::Deref;

use async_trait::async_trait;
use bson::{doc, Document};
use lazy_static::lazy_static;
use mongodb::options::{FindOneAndUpdateOptions, FindOneOptions, FindOptions, ReturnDocument};
use paper::{paper::*, user::*, Error, OrderBy, Pagination, PaginationList, Result};
use shaku::Provider;

use crate::utils::*;

#[derive(Provider)]
#[shaku(interface = PaperService)]
pub struct PaperServiceImpl {
    #[shaku(provide)]
    pub paper_collection: Box<dyn PaperCollection>,

    #[shaku(provide)]
    pub paper_content_collection: Box<dyn PaperContentCollection>,

    #[shaku(provide)]
    pub user_service: Box<dyn UserService>,
}

pub trait PaperCollection: Deref<Target = mongodb::Collection> + Send + Sync {}

pub trait PaperContentCollection: Deref<Target = mongodb::Collection> + Send + Sync {}

#[async_trait]
impl PaperService for PaperServiceImpl {
    async fn create_paper(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        input: CreatePaperInput,
    ) -> Result<(Paper, PaperContent)> {
        self.user_service
            .can_viewer_write_user(viewer_id, user_id.to_owned())
            .await?;

        let now = now_msec();

        let paper = Paper {
            id: new_id().into(),
            user_id: user_id,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            title: input.title,
        };

        let content = PaperContent {
            id: paper.id.to_owned(),
            content: input.content,
        };

        self.paper_collection
            .insert_one(to_doc(&paper)?, None)
            .await
            .map_err(|e| Error::unknown(e.to_string()))?;

        self.paper_content_collection
            .insert_one(to_doc(&content)?, None)
            .await
            .map_err(|e| Error::unknown(e.to_string()))?;

        Ok((paper.into(), content.into()))
    }

    async fn update_paper(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        paper_id: PaperId,
        input: UpdatePaperInput,
    ) -> Result<(Paper, PaperContent)> {
        self.can_viewer_write_paper(viewer_id, user_id.to_owned(), paper_id.to_owned())
            .await?;

        let mut update_set = doc! { "updated_at": now_msec() };

        if let Some(title) = input.title {
            update_set.insert("title", title);
        }

        let paper = self
            .paper_collection
            .find_one_and_update(
                doc! { "_id": paper_id.to_string() },
                doc! { "$set": update_set },
                FindOneAndUpdateOptions::builder()
                    .return_document(ReturnDocument::After)
                    .projection(PAPER_PROJECTION.to_owned())
                    .build(),
            )
            .await
            .map_err(|e| Error::unknown(e.to_string()))?
            .map(|x| from_doc::<Paper>(x))
            .ok_or(Error::unknown("Paper not found".to_owned()))??;

        let old_content = self.find_paper_content(paper_id.to_owned()).await?;

        let content = match &input.content {
            Some(content) => {
                let content = content
                    .iter()
                    .map(|x| to_doc(x))
                    .collect::<Result<Vec<_>>>()?;

                let mut update = doc! {
                    "$set": { "content": content },
                };

                if old_content.content != input.content {
                    if let Some(old_content) = old_content.content {
                        let old_content = old_content
                            .iter()
                            .map(|x| to_doc(x))
                            .collect::<Result<Vec<_>>>()?;

                        update.insert(
                            "$push",
                            doc! {
                                "history": {
                                    "$each": [ { "content": old_content } ],
                                    "$slice": -10,
                                }
                            },
                        );
                    }
                }

                self.paper_content_collection
                    .find_one_and_update(
                        doc! { "_id": paper.id.to_string() },
                        update,
                        FindOneAndUpdateOptions::builder()
                            .return_document(ReturnDocument::After)
                            .projection(PAPER_CONTENT_PROJECTION.to_owned())
                            .build(),
                    )
                    .await
                    .map_err(|e| Error::unknown(e.to_string()))?
                    .map(|x| from_doc::<PaperContent>(x))
                    .ok_or(Error::not_found("Paper content not found".to_owned()))??
            }
            None => old_content,
        };

        Ok((paper, content))
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

    async fn select_paper_content(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        paper_id: PaperId,
    ) -> Result<PaperContent> {
        self.can_viewer_read_paper(viewer_id, user_id.to_owned(), paper_id.to_owned())
            .await?;

        self.find_paper_content(paper_id).await
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

    async fn find_paper_content(&self, paper_id: PaperId) -> Result<PaperContent> {
        self.paper_content_collection
            .find_one(
                doc! { "_id": paper_id.to_string() },
                FindOneOptions::builder()
                    .projection(PAPER_CONTENT_PROJECTION.to_owned())
                    .build(),
            )
            .await
            .map_err(|e| Error::unknown(e.to_string()))?
            .map(|doc| from_doc::<PaperContent>(doc))
            .ok_or(Error::not_found("Paper content not found".to_owned()))?
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
    static ref PAPER_CONTENT_PROJECTION: Document = doc! {
        "_id": 1,
        "content": 1,
    };
}
