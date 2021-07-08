use juniper::{GraphQLEnum, GraphQLInputObject};
use paper::{
    paper::{PaperId, PaperService},
    user::{UserId, UserIdentifier, UserService},
    ErrorKind, Pagination, PaginationList,
};
use serde::{Deserialize, Serialize};
use shaku::{Component, HasComponent, HasProvider};

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

    async fn can_viewer_write_paper(&self, ctx: &Context) -> Result<bool> {
        self._can_viewer_write_paper(ctx).await
    }

    async fn token(&self, ctx: &Context) -> Result<PaperToken> {
        let read_only = self._can_viewer_write_paper(ctx).await?;

        let config: &dyn PaperTokenConfigInterface = ctx.module.resolve_ref();

        let access_token = PaperTokenPayload::new(
            ctx.access_token()?.sub,
            config.expires_in_sec,
            self.0.id.to_owned(),
            Some(read_only),
        )
        .encode(&config.secret);

        Ok(PaperToken {
            access_token,
            token_type: String::from("Bearer"),
            expires_in: config.expires_in_sec.to_string(),
        })
    }
}

impl Paper {
    async fn _can_viewer_write_paper(&self, ctx: &Context) -> Result<bool> {
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

#[derive(juniper::GraphQLObject)]
pub struct PaperToken {
    pub access_token: String,

    pub token_type: String,

    pub expires_in: String,
}

#[derive(Serialize)]
pub struct PaperTokenPayload {
    pub iat: u64,

    pub exp: u64,

    pub sub: UserId,

    pub paper_id: PaperId,

    pub read_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
#[shaku(interface = PaperTokenConfigInterface)]
pub struct PaperTokenConfig {
    pub expires_in_sec: u64,

    pub secret: String,
}

paper_impl::shaku_deref_self_interface!(PaperTokenConfigInterface, PaperTokenConfig);

impl PaperTokenPayload {
    pub fn new(
        user_id: UserId,
        expires_in_sec: u64,
        paper_id: PaperId,
        read_only: Option<bool>,
    ) -> Self {
        let now_sec = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Invalid system time")
            .as_secs();

        Self {
            iat: now_sec,
            exp: now_sec + expires_in_sec,
            sub: user_id,
            paper_id,
            read_only,
        }
    }

    pub fn encode(&self, secret: impl AsRef<[u8]>) -> String {
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &self,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_ref()),
        )
        .expect("Encode jsonwebtoken error")
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
