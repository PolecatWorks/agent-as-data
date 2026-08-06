//! Application configuration schema & loader using Figment.

use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use figment_file_provider_adapter::FileAdapter;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UrlWithUsernamePassword {
    pub url: Url,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl From<UrlWithUsernamePassword> for Url {
    fn from(value: UrlWithUsernamePassword) -> Self {
        let mut return_url = value.url;
        if let Some(password) = value.password {
            let _ = return_url.set_password(Some(&password));
        }
        if let Some(username) = value.username {
            let _ = return_url.set_username(&username);
        }
        return_url
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DatabaseConfig {
    pub url: UrlWithUsernamePassword,
    pub max_connections: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WebServiceConfig {
    pub address: String,
    pub cors: CorsConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CorsConfig {
    pub allow_origins: Vec<String>,
    pub allow_methods: Vec<String>,
    pub allow_headers: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub webservice: WebServiceConfig,
}

impl AppConfig {
    pub fn load(
        config_path: &std::path::Path,
        secrets_dir: &std::path::Path,
    ) -> Result<Self, Box<figment::Error>> {
        let adapter = FileAdapter::wrap(Yaml::file(config_path)).relative_to_dir(secrets_dir);

        Figment::new()
            .merge(adapter)
            .merge(Env::prefixed("AAD_BE__").split("__").lowercase(true))
            .extract()
            .map_err(Box::new)
    }
}
