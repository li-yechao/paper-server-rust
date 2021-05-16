use std::sync::Arc;

use async_trait::async_trait;
use paper::{auth::*, user::*, Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use shaku::{Component, Provider};

#[derive(Provider)]
#[shaku(interface = AuthService)]
pub struct AuthServiceImpl {
    #[shaku(provide)]
    pub user_service: Box<dyn UserService>,

    #[shaku(inject)]
    pub access_token_config: Arc<dyn AccessTokenConfigInterface>,

    #[shaku(inject)]
    pub refresh_token_config: Arc<dyn RefreshTokenConfigInterface>,

    #[shaku(inject)]
    pub github_auth_config: Arc<dyn GithubAuthConfigInterface>,

    #[shaku(inject)]
    pub google_auth_config: Arc<dyn GoogleAuthConfigInterface>,
}

#[async_trait]
impl AuthService for AuthServiceImpl {
    async fn create_access_token(&self, input: CreateAccessTokenInput) -> Result<AccessToken> {
        let user = match &input {
            CreateAccessTokenInput::Github { client_id, code } => {
                let config = self
                    .github_auth_config
                    .list
                    .iter()
                    .find(|x| x.client_id == *client_id)
                    .ok_or(Error::unknown("Invalid client_id".to_owned()))?;

                let github_access_token = github::auth(&client_id, &config.client_secret, &code)
                    .await
                    .map_err(|e| Error::unknown(e.to_string()))?;

                let github_user = github::user(&github_access_token)
                    .await
                    .map_err(|e| Error::unknown(e.to_string()))?;

                let github_user_id = github_user
                    .as_object()
                    .map(|obj| obj.get("id").map(|id| id.as_u64()))
                    .flatten()
                    .flatten()
                    .ok_or_else(|| Error::unknown("Invalid github user".to_owned()))?;

                match self
                    .user_service
                    .select_user(UserIdentifier::GithubUserId(github_user_id))
                    .await
                {
                    Ok(user) => user,
                    Err(e) => {
                        if e.kind == ErrorKind::NotFound {
                            self.user_service
                                .create_user(CreateUserInput::Github { github_user })
                                .await?
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
            CreateAccessTokenInput::Google { .. }
            | CreateAccessTokenInput::GoogleAccessToken { .. } => {
                let google_access_token = match input {
                    CreateAccessTokenInput::Google { client_id, code } => {
                        let config = self
                            .google_auth_config
                            .list
                            .iter()
                            .find(|x| x.client_id == *client_id)
                            .ok_or(Error::unknown("Invalid client_id".to_owned()))?;

                        google::auth(
                            &client_id,
                            &config.client_secret,
                            &code,
                            &config.redirect_uri,
                        )
                        .await
                        .map_err(|e| Error::unknown(e.to_string()))?
                    }
                    CreateAccessTokenInput::GoogleAccessToken { access_token } => access_token,
                    _ => unreachable!(),
                };

                let google_user = google::user(&google_access_token)
                    .await
                    .map_err(|e| Error::unknown(e.to_string()))?;

                let google_user_id = google_user
                    .as_object()
                    .map(|obj| obj.get("id").map(|id| id.as_str()))
                    .flatten()
                    .flatten()
                    .ok_or_else(|| Error::unknown("Invalid google user".to_owned()))?;

                match self
                    .user_service
                    .select_user(UserIdentifier::GoogleUserId(google_user_id.to_owned()))
                    .await
                {
                    Ok(user) => user,
                    Err(e) => {
                        if e.kind == ErrorKind::NotFound {
                            self.user_service
                                .create_user(CreateUserInput::Google { google_user })
                                .await?
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
            CreateAccessTokenInput::RefreshToken { refresh_token } => {
                let user_id = RefreshTokenPayload::decode(
                    &refresh_token,
                    &(*self.refresh_token_config).secret,
                )
                .map_err(|e| Error::unauthorized(e.to_string()))?
                .sub;

                self.user_service
                    .select_user(UserIdentifier::Id(user_id.into()))
                    .await?
            }
        };

        Ok(AccessToken::new(
            user,
            (
                self.access_token_config.expires_in_sec,
                self.access_token_config.secret.as_ref(),
            ),
            (
                self.refresh_token_config.expires_in_sec,
                self.refresh_token_config.secret.as_ref(),
            ),
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
#[shaku(interface = AccessTokenConfigInterface)]
pub struct AccessTokenConfig {
    pub expires_in_sec: u64,

    pub secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
#[shaku(interface = RefreshTokenConfigInterface)]
pub struct RefreshTokenConfig {
    pub expires_in_sec: u64,

    pub secret: String,
}

crate::shaku_deref_self_interface!(AccessTokenConfigInterface, AccessTokenConfig);
crate::shaku_deref_self_interface!(RefreshTokenConfigInterface, RefreshTokenConfig);

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
#[shaku(interface = GithubAuthConfigInterface)]
pub struct GithubAuthConfig {
    pub list: Vec<GithubAuthConfigItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubAuthConfigItem {
    pub client_id: String,

    pub client_secret: String,
}

crate::shaku_deref_self_interface!(GithubAuthConfigInterface, GithubAuthConfig);

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
#[shaku(interface = GoogleAuthConfigInterface)]
pub struct GoogleAuthConfig {
    pub list: Vec<GoogleAuthConfigItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAuthConfigItem {
    pub client_id: String,

    pub client_secret: String,

    pub redirect_uri: String,
}

crate::shaku_deref_self_interface!(GoogleAuthConfigInterface, GoogleAuthConfig);

mod github {
    use std::{error::Error, result::Result};

    use hyper::*;
    use hyper_tls::HttpsConnector;

    pub async fn auth(
        client_id: &str,
        client_secret: &str,
        code: &str,
    ) -> Result<String, Box<dyn Error>> {
        let client = Client::builder().build::<_, Body>(HttpsConnector::new());
        let uri = "https://github.com/login/oauth/access_token";
        let request = Request::builder()
            .method(Method::POST)
            .uri(format!(
                "{}?client_id={}&client_secret={}&code={}",
                uri, client_id, client_secret, code,
            ))
            .header("Accept", "application/json")
            .body(Body::empty())?;
        let response = client.request(request).await?;
        if response.status() != StatusCode::OK {
            return Err(response.status().to_string().into());
        }
        let body = body::to_bytes(response).await?;
        let body = String::from_utf8(body.to_vec())?;

        #[derive(Debug, serde::Deserialize)]
        struct Response {
            access_token: Option<String>,
            scope: Option<String>,
            token_type: Option<String>,
            error: Option<String>,
            error_description: Option<String>,
            error_uri: Option<String>,
        }

        let json = serde_json::from_str::<Response>(&body)?;

        if let Some(access_token) = json.access_token {
            return Ok(access_token);
        } else if let Some(message) = json.error_description {
            return Err(message.into());
        }

        Err(body.into())
    }

    pub async fn user(access_token: &str) -> Result<serde_json::Value, Box<dyn Error>> {
        let client = Client::builder().build::<_, Body>(HttpsConnector::new());
        let request = Request::builder()
            .method(Method::GET)
            .uri("https://api.github.com/user")
            .header("Accept", "application/json")
            .header("Authorization", format!("token {}", access_token))
            .header("User-Agent", "hyper")
            .body(Body::empty())?;
        let response = client.request(request).await?;
        if response.status() != StatusCode::OK {
            return Err(response.status().to_string().into());
        }
        let body = body::to_bytes(response).await?;
        let body = String::from_utf8(body.to_vec())?;
        let json = serde_json::from_str(&body)?;

        if let serde_json::Value::Object(obj) = &json {
            if let Some(serde_json::Value::Number(_)) = obj.get("id") {
                return Ok(json);
            }
        }

        Err("Invalid github user response".into())
    }
}

mod google {
    use std::{error::Error, result::Result};

    use hyper::*;
    use hyper_tls::HttpsConnector;

    pub async fn auth(
        client_id: &str,
        client_secret: &str,
        code: &str,
        redirect_uri: &str,
    ) -> Result<String, Box<dyn Error>> {
        let client = Client::builder().build::<_, Body>(HttpsConnector::new());
        let uri = "https://oauth2.googleapis.com/token";
        let request = Request::builder()
            .method(Method::POST)
            .uri(format!(
                "{}?grant_type=authorization_code&client_id={}&client_secret={}&code={}&\
                 redirect_uri={}",
                uri, client_id, client_secret, code, redirect_uri,
            ))
            .header("Accept", "application/json")
            .header("Content-Length", 0)
            .body(Body::empty())?;
        let response = client.request(request).await?;
        if response.status() != StatusCode::OK {
            return Err(response.status().to_string().into());
        }
        let body = body::to_bytes(response).await?;
        let body = String::from_utf8(body.to_vec())?;

        #[derive(Debug, serde::Deserialize)]
        struct Response {
            access_token: String,
            expires_in: u64,
            scope: String,
            token_type: String,
            id_token: String,
        }

        let json = serde_json::from_str::<Response>(&body)?;

        Ok(json.access_token)
    }

    pub async fn user(access_token: &str) -> Result<serde_json::Value, Box<dyn Error>> {
        let client = Client::builder().build::<_, Body>(HttpsConnector::new());
        let request = Request::builder()
            .method(Method::GET)
            .uri("https://www.googleapis.com/oauth2/v1/userinfo")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "hyper")
            .header("Content-Length", 0)
            .body(Body::empty())?;
        let response = client.request(request).await?;
        if response.status() != StatusCode::OK {
            return Err(response.status().to_string().into());
        }
        let body = body::to_bytes(response).await?;
        let body = String::from_utf8(body.to_vec())?;
        let json = serde_json::from_str(&body)?;

        if let serde_json::Value::Object(obj) = &json {
            if let Some(serde_json::Value::String(_)) = obj.get("id") {
                return Ok(json);
            }
        }

        Err("Invalid google user response".into())
    }
}
