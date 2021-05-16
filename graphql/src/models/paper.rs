use juniper::{GraphQLEnum, GraphQLInputObject};
use paper::{
    paper::{PaperId, PaperService},
    user::{UserId, UserIdentifier, UserService},
    ErrorKind, Pagination, PaginationList,
};
use serde::{Deserialize, Serialize};
use shaku::HasProvider;

use crate::{models::user::User, *};

#[derive(GraphQLInputObject)]
pub struct PaperOrder {
    field: PaperOrderField,

    direction: OrderDirection,
}

#[derive(GraphQLEnum)]
pub enum PaperOrderField {
    Id,

    UpdatedAt,
}

impl Into<paper::OrderBy<paper::paper::PaperOrderField>> for PaperOrder {
    fn into(self) -> paper::OrderBy<paper::paper::PaperOrderField> {
        paper::OrderBy {
            field: match self.field {
                PaperOrderField::Id => paper::paper::PaperOrderField::Id,
                PaperOrderField::UpdatedAt => paper::paper::PaperOrderField::UpdatedAt,
            },
            direction: self.direction.into(),
        }
    }
}

#[derive(GraphQLInputObject)]
pub struct CreatePaperInput {
    pub title: Option<String>,

    pub content: Option<PaperContent>,
}

impl Into<paper::paper::CreatePaperInput> for CreatePaperInput {
    fn into(self) -> paper::paper::CreatePaperInput {
        paper::paper::CreatePaperInput {
            title: self.title,
            content: self.content.map(|x| x.into()),
        }
    }
}

#[derive(GraphQLInputObject)]
pub struct UpdatePaperInput {
    pub title: Option<String>,

    pub content: Option<PaperContent>,
}

impl Into<paper::paper::UpdatePaperInput> for UpdatePaperInput {
    fn into(self) -> paper::paper::UpdatePaperInput {
        paper::paper::UpdatePaperInput {
            title: self.title,
            content: self.content.map(|x| x.into()),
        }
    }
}

#[derive(Clone)]
pub struct Paper(paper::paper::Paper);

impl From<paper::paper::Paper> for Paper {
    fn from(v: paper::paper::Paper) -> Self {
        Self(v)
    }
}

#[juniper::graphql_object(context = Context)]
impl Paper {
    fn id(&self) -> String {
        self.0.id.to_string()
    }

    async fn user(&self, ctx: &Context) -> Result<User> {
        let user_service: Box<dyn UserService> = ctx.module.provide().unwrap();

        user_service
            .select_user(UserIdentifier::Id(self.0.user_id.to_owned()))
            .await
            .map(|x| x.into())
            .map_err(|e| e.into())
    }

    fn created_at(&self) -> String {
        self.0.created_at.to_string()
    }

    fn updated_at(&self) -> String {
        self.0.updated_at.to_string()
    }

    fn title(&self) -> Option<&String> {
        self.0.title.as_ref()
    }

    async fn content(&self, ctx: &Context) -> Result<Option<PaperContent>> {
        let paper_service: Box<dyn PaperService> = ctx.module.provide().unwrap();

        paper_service
            .select_paper_content(
                ctx.access_token()?.sub,
                self.0.user_id.to_owned(),
                self.0.id.to_owned(),
            )
            .await
            .map(|x| x.content.map(|x| x.into()))
            .map_err(|e| e.into())
    }

    async fn can_viewer_write_paper(&self, ctx: &Context) -> Result<bool> {
        let paper_service: Box<dyn PaperService> = ctx.module.provide().unwrap();

        paper_service
            .can_viewer_write_paper(
                ctx.access_token()?.sub,
                self.0.user_id.to_owned(),
                self.0.id.to_owned(),
            )
            .await
            .map_or_else(
                |e| match e.kind {
                    ErrorKind::Forbidden => Ok(false),
                    _ => Err(e.into()),
                },
                |_| Ok(true),
            )
    }
}

pub struct PaperContent(Vec<paper::paper::content::Block>);

impl From<Vec<paper::paper::content::Block>> for PaperContent {
    fn from(v: Vec<paper::paper::content::Block>) -> Self {
        Self(v)
    }
}

impl Into<Vec<paper::paper::content::Block>> for PaperContent {
    fn into(self) -> Vec<paper::paper::content::Block> {
        self.0
    }
}

#[juniper::graphql_scalar(description = "Paper Content")]
impl<S> GraphQLScalar for PaperContent
where
    S: ScalarValue,
{
    fn resolve(&self) -> Value {
        juniper::Value::scalar(
            serde_json::to_string(&self.0).expect("Serialize paper content error"),
        )
    }

    fn from_input_value(v: &InputValue) -> Option<Self> {
        v.as_scalar_value()
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str::<Vec<paper::paper::content::Block>>(s).ok())
            .map(|x| Self(x))
    }

    fn from_str<'a>(value: ScalarToken<'a>) -> juniper::ParseScalarResult<'a, S> {
        <String as juniper::ParseScalarValue<S>>::from_str(value)
    }
}

#[derive(Serialize, Deserialize)]
pub struct PaperCursor {
    pub id: PaperId,
}

impl Cursor for PaperCursor {}

#[juniper::graphql_scalar(description = "Paper Cursor")]
impl<S> GraphQLScalar for PaperCursor
where
    S: ScalarValue,
{
    fn resolve(&self) -> Value {
        juniper::Value::scalar(self.encode())
    }

    fn from_input_value(v: &InputValue) -> Option<Self> {
        v.as_scalar_value()
            .and_then(|v| v.as_str())
            .and_then(|s| Self::decode(s))
    }

    fn from_str<'a>(value: ScalarToken<'a>) -> juniper::ParseScalarResult<'a, S> {
        <String as juniper::ParseScalarValue<S>>::from_str(value)
    }
}

pub struct PaperEdge(paper::paper::Paper);

impl From<paper::paper::Paper> for PaperEdge {
    fn from(v: paper::paper::Paper) -> Self {
        Self(v)
    }
}

#[juniper::graphql_object(context = Context)]
impl PaperEdge {
    fn node(&self) -> Paper {
        self.0.clone().into()
    }

    fn cursor(&self) -> PaperCursor {
        PaperCursor {
            id: self.0.id.to_owned(),
        }
    }
}

pub struct PaperConnection(PaginationList<paper::paper::Paper>);

pub enum PaperConnectionKind {
    User { user_id: UserId },
}

impl PaperConnection {
    pub async fn new(
        ctx: &Context,
        kind: PaperConnectionKind,
        pagination: Pagination<PaperId>,
        order_by: Option<PaperOrder>,
        deleted: Option<bool>,
    ) -> Result<Self> {
        let paper_service: Box<dyn PaperService> = ctx.module.provide().unwrap();

        let page_list = match &kind {
            PaperConnectionKind::User { user_id } => {
                paper_service
                    .select_paper_page_of_repository(
                        ctx.access_token()?.sub,
                        user_id.to_owned(),
                        pagination,
                        order_by
                            .unwrap_or(PaperOrder {
                                field: PaperOrderField::Id,
                                direction: OrderDirection::Asc,
                            })
                            .into(),
                        deleted.unwrap_or(false),
                    )
                    .await?
            }
        };
        Ok(Self(page_list))
    }
}

#[juniper::graphql_object(context = Context)]
impl PaperConnection {
    async fn edges(&self) -> Vec<PaperEdge> {
        self.0.list.iter().map(|x| x.to_owned().into()).collect()
    }

    async fn nodes(&self) -> Vec<Paper> {
        self.0.list.iter().map(|x| x.to_owned().into()).collect()
    }

    async fn page_info(&self) -> PageInfo {
        PageInfo {
            start_cursor: self.0.list.first().map(|x| {
                PaperCursor {
                    id: x.id.to_owned(),
                }
                .encode()
            }),
            end_cursor: self.0.list.last().map(|x| {
                PaperCursor {
                    id: x.id.to_owned(),
                }
                .encode()
            }),
            has_next_page: self.0.has_next_page,
        }
    }

    async fn total(&self) -> i32 {
        self.0.total as i32
    }
}

pub struct DeletePaperPayload(paper::paper::Paper);

impl From<paper::paper::Paper> for DeletePaperPayload {
    fn from(v: paper::paper::Paper) -> Self {
        Self(v)
    }
}

#[juniper::graphql_object(context = Context)]
impl DeletePaperPayload {
    fn id(&self) -> String {
        self.0.id.to_string()
    }

    async fn user(&self, ctx: &Context) -> Result<User> {
        let user_service: Box<dyn UserService> = ctx.module.provide().unwrap();

        user_service
            .select_user(UserIdentifier::Id(self.0.user_id.to_owned()))
            .await
            .map(|x| x.into())
            .map_err(|e| e.into())
    }
}
