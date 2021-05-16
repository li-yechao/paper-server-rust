use crate::logger::LogLevelFilter;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub address: std::net::IpAddr,

    pub port: u16,

    pub cors: bool,

    #[serde(with = "LogLevelFilter")]
    pub log_level: log::LevelFilter,

    pub access_token: ConfigAccessToken,

    pub refresh_token: ConfigAccessToken,

    pub storage: ConfigStorage,

    pub github_auth: Vec<ConfigGithubAuth>,

    pub google_auth: Vec<ConfigGoogleAuth>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigAccessToken {
    pub expires_in_sec: u64,

    pub secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigStorage {
    pub uri: String,

    pub database: String,

    pub collection_user: String,

    pub collection_paper: String,

    pub collection_paper_content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigGithubAuth {
    pub client_id: String,

    pub client_secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigGoogleAuth {
    pub client_id: String,

    pub client_secret: String,

    pub redirect_uri: String,
}
