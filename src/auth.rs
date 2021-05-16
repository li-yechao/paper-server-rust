use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::Result;

use super::user::{User, UserId};

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn create_access_token(&self, input: CreateAccessTokenInput) -> Result<AccessToken>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreateAccessTokenInput {
    Github { client_id: String, code: String },

    Google { client_id: String, code: String },

    GoogleAccessToken { access_token: String },

    RefreshToken { refresh_token: String },
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessToken {
    pub access_token: String,

    pub token_type: String,

    pub expires_in: u64,

    pub refresh_token: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessTokenPayload {
    pub iat: u64,

    pub exp: u64,

    pub sub: UserId,
}

pub type RefreshTokenPayload = AccessTokenPayload;

impl AccessToken {
    pub fn new(
        user: User,
        access_token_config: (u64, &[u8]),
        refresh_token_config: (u64, &[u8]),
    ) -> Self {
        let access_token = AccessTokenPayload::new(user.id.to_owned(), access_token_config.0);
        let refresh_token = RefreshTokenPayload::new(user.id.to_owned(), refresh_token_config.0);

        Self {
            access_token: access_token.encode(access_token_config.1),
            token_type: "Bearer".to_owned(),
            expires_in: access_token_config.0,
            refresh_token: refresh_token.encode(refresh_token_config.1),
        }
    }
}

impl AccessTokenPayload {
    pub fn new(user_id: UserId, expires_in_sec: u64) -> Self {
        let now_sec = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Invalid system time")
            .as_secs();

        Self {
            iat: now_sec,
            exp: now_sec + expires_in_sec,
            sub: user_id,
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

    pub fn decode(token: &str, secret: impl AsRef<[u8]>) -> jsonwebtoken::errors::Result<Self> {
        jsonwebtoken::decode::<Self>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(secret.as_ref()),
            &jsonwebtoken::Validation::default(),
        )
        .map(|x| x.claims)
    }
}
