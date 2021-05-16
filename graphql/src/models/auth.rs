use std::convert::TryInto;

use juniper::GraphQLInputObject;

use crate::*;

#[derive(GraphQLInputObject)]
pub struct CreateAccessTokenInput {
    pub github: Option<CreateAccessTokenGithubInput>,

    pub google: Option<CreateAccessTokenGoogleInput>,

    pub google_access_token: Option<CreateAccessTokenGoogleAccessTokenInput>,

    pub refresh_token: Option<CreateAccessTokenRefreshTokenInput>,
}

impl TryInto<paper::auth::CreateAccessTokenInput> for CreateAccessTokenInput {
    type Error = Error;

    fn try_into(self) -> Result<paper::auth::CreateAccessTokenInput> {
        match (
            self.github,
            self.google,
            self.google_access_token,
            self.refresh_token,
        ) {
            (Some(github), None, None, None) => Ok(paper::auth::CreateAccessTokenInput::Github {
                client_id: github.client_id,
                code: github.code,
            }),
            (None, Some(google), None, None) => Ok(paper::auth::CreateAccessTokenInput::Google {
                client_id: google.client_id,
                code: google.code,
            }),
            (None, None, Some(google), None) => {
                Ok(paper::auth::CreateAccessTokenInput::GoogleAccessToken {
                    access_token: google.access_token,
                })
            }
            (None, None, None, Some(refresh_token)) => Ok({
                paper::auth::CreateAccessTokenInput::RefreshToken {
                    refresh_token: refresh_token.refresh_token,
                }
            }),
            _ => Err(Error::unknown("Invalid CreateAccessTokenInput".to_owned())),
        }
    }
}

#[derive(GraphQLInputObject)]
pub struct CreateAccessTokenGithubInput {
    pub client_id: String,

    pub code: String,
}

#[derive(GraphQLInputObject)]
pub struct CreateAccessTokenGoogleInput {
    pub client_id: String,

    pub code: String,
}

#[derive(GraphQLInputObject)]
pub struct CreateAccessTokenGoogleAccessTokenInput {
    pub access_token: String,
}

#[derive(GraphQLInputObject)]
pub struct CreateAccessTokenRefreshTokenInput {
    pub refresh_token: String,
}

pub struct AccessToken(paper::auth::AccessToken);

impl From<paper::auth::AccessToken> for AccessToken {
    fn from(v: paper::auth::AccessToken) -> Self {
        Self(v)
    }
}

#[juniper::graphql_object(context = Context)]
impl AccessToken {
    fn access_token(&self) -> &str {
        &self.0.access_token
    }

    fn token_type(&self) -> &str {
        &self.0.token_type
    }

    fn expires_in(&self) -> String {
        self.0.expires_in.to_string()
    }

    fn refresh_token(&self) -> &str {
        &self.0.refresh_token
    }
}
