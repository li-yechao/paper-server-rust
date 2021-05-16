use std::convert::TryInto;

use paper::user::{UserIdentifier, UserService};
use shaku::HasProvider;

use crate::{models::user::User, *};

pub struct Query;

#[juniper::graphql_object(context = Context)]
impl super::Query {
    async fn viewer(ctx: &Context) -> Result<User> {
        let user_service: Box<dyn UserService> = ctx.module.provide().unwrap();

        let user_id = ctx.access_token()?.sub;

        user_service
            .select_user(UserIdentifier::Id(user_id))
            .await
            .map(|x| x.into())
            .map_err(|e| e.into())
    }

    async fn user(ctx: &Context, identifier: super::models::user::UserIdentifier) -> Result<User> {
        let user_service: Box<dyn UserService> = ctx.module.provide().unwrap();

        user_service
            .select_user(identifier.try_into()?)
            .await
            .map(|x| x.into())
            .map_err(|e| e.into())
    }
}
