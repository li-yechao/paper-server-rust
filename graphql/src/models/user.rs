use std::convert::TryInto;

use juniper::GraphQLInputObject;
use paper::{paper::PaperService, user::UserService, Pagination};
use shaku::HasProvider;

use crate::{
    models::paper::{Paper, PaperConnection, PaperConnectionKind, PaperCursor, PaperOrder},
    *,
};

#[derive(GraphQLInputObject)]
pub struct UserIdentifier {
    pub id: Option<String>,

    pub name: Option<String>,
}

impl TryInto<paper::user::UserIdentifier> for UserIdentifier {
    type Error = Error;

    fn try_into(self) -> Result<paper::user::UserIdentifier> {
        if let Some(id) = self.id {
            Ok(paper::user::UserIdentifier::Id(id.into()))
        } else if let Some(name) = self.name {
            Ok(paper::user::UserIdentifier::Name(name))
        } else {
            Err(Error::unknown("Invalid UserIdentifier".to_owned()))
        }
    }
}

#[derive(GraphQLInputObject)]
pub struct UpdateUserInput {
    pub name: Option<String>,
}

impl Into<paper::user::UpdateUserInput> for UpdateUserInput {
    fn into(self) -> paper::user::UpdateUserInput {
        paper::user::UpdateUserInput { name: self.name }
    }
}

pub struct User(paper::user::User);

impl From<paper::user::User> for User {
    fn from(v: paper::user::User) -> Self {
        Self(v)
    }
}

#[juniper::graphql_object(context = Context)]
impl User {
    fn id(&self) -> &str {
        self.0.id.as_ref()
    }

    fn created_at(&self) -> String {
        self.0.created_at.to_string()
    }

    fn name(&self) -> &str {
        &self.0.name
    }

    async fn papers(
        &self,
        ctx: &Context,
        skip: Option<i32>,
        after: Option<PaperCursor>,
        first: Option<i32>,
        before: Option<PaperCursor>,
        last: Option<i32>,
        order_by: Option<PaperOrder>,
        deleted: Option<bool>,
    ) -> Result<PaperConnection> {
        let pagination = match (first, last) {
            (Some(first), _) => Pagination::After {
                after: after.map(|x| x.id),
                skip: skip.map(|x| x as u64),
                first: first as u64,
            },
            (_, Some(last)) => Pagination::Before {
                before: before.map(|x| x.id),
                skip: skip.map(|x| x as u64),
                last: last as u64,
            },
            _ => {
                return Err(Error::unknown(
                    "Missing required parameter first or last".to_owned(),
                ))
            }
        };

        PaperConnection::new(
            ctx,
            PaperConnectionKind::User {
                user_id: self.0.id.to_owned(),
            },
            pagination,
            order_by,
            deleted,
        )
        .await
        .map_err(|e| e.into())
    }

    async fn paper(&self, ctx: &Context, paper_id: String) -> Result<Paper> {
        let paper_service: Box<dyn PaperService> = ctx.module.provide().unwrap();

        paper_service
            .select_paper(
                ctx.access_token()?.sub,
                self.0.id.to_owned(),
                paper_id.into(),
            )
            .await
            .map(|x| x.into())
            .map_err(|e| e.into())
    }

    async fn can_viewer_read_user(&self, ctx: &Context) -> Result<bool> {
        let user_service: Box<dyn UserService> = ctx.module.provide().unwrap();

        user_service
            .can_viewer_read_user(ctx.access_token()?.sub, self.0.id.to_owned())
            .await
            .map_or_else(
                |e| match e.kind {
                    paper::ErrorKind::Forbidden => Ok(false),
                    _ => Err(e.into()),
                },
                |_| Ok(true),
            )
    }

    async fn can_viewer_write_user(&self, ctx: &Context) -> Result<bool> {
        let user_service: Box<dyn UserService> = ctx.module.provide().unwrap();

        user_service
            .can_viewer_write_user(ctx.access_token()?.sub, self.0.id.to_owned())
            .await
            .map_or_else(
                |e| match e.kind {
                    paper::ErrorKind::Forbidden => Ok(false),
                    _ => Err(e.into()),
                },
                |_| Ok(true),
            )
    }

    async fn can_viewer_administer_user(&self, ctx: &Context) -> Result<bool> {
        let user_service: Box<dyn UserService> = ctx.module.provide().unwrap();

        user_service
            .can_viewer_administer_user(ctx.access_token()?.sub, self.0.id.to_owned())
            .await
            .map_or_else(
                |e| match e.kind {
                    paper::ErrorKind::Forbidden => Ok(false),
                    _ => Err(e.into()),
                },
                |_| Ok(true),
            )
    }
}
