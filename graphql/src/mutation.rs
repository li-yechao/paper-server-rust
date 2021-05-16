use std::convert::TryInto;

use paper::{auth::AuthService, paper::PaperService, user::UserService};
use shaku::HasProvider;

use crate::{
    models::{
        auth::{AccessToken, CreateAccessTokenInput},
        paper::{CreatePaperInput, DeletePaperPayload, Paper, UpdatePaperInput},
        user::{UpdateUserInput, User},
    },
    *,
};

pub struct Mutation;

#[juniper::graphql_object(context = Context)]
impl Mutation {
    async fn create_access_token(
        ctx: &Context,
        input: CreateAccessTokenInput,
    ) -> Result<AccessToken> {
        let auth_service: Box<dyn AuthService> = ctx.module.provide().unwrap();

        auth_service
            .create_access_token(input.try_into()?)
            .await
            .map(|x| x.into())
            .map_err(|e| e.into())
    }

    async fn update_user(ctx: &Context, user_id: String, input: UpdateUserInput) -> Result<User> {
        let user_service: Box<dyn UserService> = ctx.module.provide().unwrap();

        user_service
            .update_user(ctx.access_token()?.sub, user_id.into(), input.into())
            .await
            .map(|x| x.into())
            .map_err(|e| e.into())
    }

    async fn create_paper(
        ctx: &Context,
        user_id: String,
        input: CreatePaperInput,
    ) -> Result<Paper> {
        let paper_service: Box<dyn PaperService> = ctx.module.provide().unwrap();

        paper_service
            .create_paper(
                ctx.access_token()?.sub,
                user_id.to_owned().into(),
                input.into(),
            )
            .await
            .map(|(x, _)| x.into())
            .map_err(|e| e.into())
    }

    async fn update_paper(
        ctx: &Context,
        user_id: String,
        paper_id: String,
        input: UpdatePaperInput,
    ) -> Result<Paper> {
        let paper_service: Box<dyn PaperService> = ctx.module.provide().unwrap();

        paper_service
            .update_paper(
                ctx.access_token()?.sub,
                user_id.to_owned().into(),
                paper_id.into(),
                input.into(),
            )
            .await
            .map(|(x, _)| x.into())
            .map_err(|e| e.into())
    }

    async fn delete_paper(
        ctx: &Context,
        user_id: String,
        paper_id: String,
    ) -> Result<DeletePaperPayload> {
        let paper_service: Box<dyn PaperService> = ctx.module.provide().unwrap();

        let payload = paper_service
            .select_paper(
                ctx.access_token()?.sub,
                user_id.to_owned().into(),
                paper_id.to_owned().into(),
            )
            .await
            .map(|x| x.into())?;

        paper_service
            .delete_paper(
                ctx.access_token()?.sub,
                user_id.into(),
                paper_id.to_owned().into(),
            )
            .await?;

        Ok(payload)
    }
}
