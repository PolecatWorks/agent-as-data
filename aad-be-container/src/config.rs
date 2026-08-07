use figment::{
    Figment,
    providers::{Env, Format, Yaml},
};
use serde::{Deserialize, Serialize};
use url::Url;

use ::hams::hams::config::HamsConfig;

use crate::tokio_tools::ThreadRuntime;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub webservice: WebServiceConfig,
    #[serde(serialize_with = "serialize_hams")]
    pub hams: HamsConfig,
    #[serde(default)]
    pub runtime: ThreadRuntime,
    pub debugging: DebuggingConfig,
}


fn serialize_hams<S>(hams: &HamsConfig, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&format!("{:?}", hams))
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DebuggingConfig {
    pub environment: String,
    pub log_level: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WebServiceConfig {
    pub address: String,
}

impl AppConfig {
    pub fn load(config_path: &std::path::Path) -> Result<Self, Box<figment::Error>> {
        Figment::new()
            .merge(Yaml::file(config_path))
            .merge(Env::prefixed("AAD_BE__").split("__").lowercase(true))
            .extract()
            .map_err(Box::new)
    }

    /// Fail-Fast early validation of loaded configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.database.url.trim().is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        if self.webservice.address.trim().is_empty() {
            return Err("Webservice address cannot be empty".to_string());
        }
        Url::parse(&self.database.url).map_err(|e| format!("Invalid Database URL format: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation_valid() {
        let config = AppConfig {
            database: DatabaseConfig {
                url: "postgres://postgres:mysecretpassword@localhost:5432/aaddb".to_string(),
                max_connections: 5,
            },
            webservice: WebServiceConfig {
                address: "0.0.0.0:8080".to_string(),
            },
            hams: ::hams::hams::config::HamsConfig::default(),
            runtime: ThreadRuntime::default(),
            debugging: DebuggingConfig {
                environment: "development".to_string(),
                log_level: "info".to_string(),
            },
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_empty_url() {
        let config = AppConfig {
            database: DatabaseConfig {
                url: "".to_string(),
                max_connections: 5,
            },
            webservice: WebServiceConfig {
                address: "0.0.0.0:8080".to_string(),
            },
            hams: ::hams::hams::config::HamsConfig::default(),
            runtime: ThreadRuntime::default(),
            debugging: DebuggingConfig {
                environment: "development".to_string(),
                log_level: "info".to_string(),
            },
        };

        assert!(config.validate().is_err());
    }
}

