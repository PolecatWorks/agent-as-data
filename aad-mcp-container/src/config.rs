use figment::{
    Figment,
    providers::{Env, Format, Yaml},
};
use figment_file_provider_adapter::FileAdapter;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use ::hams::hams::config::HamsConfig;

use crate::tokio_tools::ThreadRuntime;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AppConfig {
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
    #[serde(default = "default_fail_debug_delay", with = "humantime_serde")]
    pub fail_debug_delay: Duration,
}

fn default_fail_debug_delay() -> Duration {
    Duration::from_secs(0)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WebServiceConfig {
    pub address: String,
    pub api_prefix: String,
}

impl AppConfig {
    pub fn load(
        config_path: &std::path::Path,
        secrets_dir: &std::path::Path,
    ) -> Result<Self, Box<figment::Error>> {
        let adapter = FileAdapter::wrap(Yaml::file(config_path)).relative_to_dir(secrets_dir);

        Figment::new()
            .merge(adapter)
            .merge(Env::prefixed("AAD_MCP__").split("__").lowercase(true))
            .extract()
            .map_err(Box::new)
    }

    /// Fail-Fast early validation of loaded configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.webservice.address.trim().is_empty() {
            return Err("Webservice address cannot be empty".to_string());
        }
        if self.webservice.api_prefix.trim().is_empty() {
            return Err("Webservice api_prefix cannot be empty".to_string());
        }
        Ok(())
    }
}
