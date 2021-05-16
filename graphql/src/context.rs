use std::sync::Arc;

use juniper::{EmptySubscription, RootNode};
use paper::auth::AccessTokenPayload;

use crate::*;

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub struct Context {
    pub module: Arc<Module>,
    pub access_token: Option<String>,
}

impl juniper::Context for Context {}

impl Context {
    pub fn access_token(&self) -> Result<AccessTokenPayload> {
        use paper_impl::auth::AccessTokenConfigInterface;
        use shaku::HasComponent;

        let access_token_config: &dyn AccessTokenConfigInterface = self.module.resolve_ref();
        self.access_token
            .as_ref()
            .map(String::as_ref)
            .ok_or_else(|| Error::unauthorized("AccessToken is not present".to_owned()))
            .and_then(|x| {
                AccessTokenPayload::decode(x, &access_token_config.secret)
                    .map_err(|e| Error::unauthorized(e.to_string()))
            })
    }
}
