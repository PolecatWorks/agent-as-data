use figment::{
    Figment,
    providers::{Env, Format, Yaml},
};
use figment_file_provider_adapter::FileAdapter;
use serde::{Deserialize, Serialize};
use url::Url;

use ::hams::hams::config::HamsConfig;

use crate::tokio_tools::ThreadRuntime;

/// A URL structure that optionally embeds credential placeholders for file-based secret resolution.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UrlWithUsernamePassword {
    /// Base target URL.
    pub url: Url,
    /// Optional username credential.
    pub username: Option<String>,
    /// Optional password credential.
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
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub webservice: WebServiceConfig,
    pub llm: LlmConfig,
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
pub struct LlmConfig {
    pub ollama_url: String,
    pub model: String,
    pub timeout_secs: u64,
}

use std::time::Duration;

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
pub struct DatabaseConfig {
    pub url: UrlWithUsernamePassword,
    pub max_connections: u32,
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
            .merge(Env::prefixed("AAD_BE__").split("__").lowercase(true))
            .extract()
            .map_err(Box::new)
    }

    /// Fail-Fast early validation of loaded configuration.
    pub fn validate(&self) -> Result<(), String> {
        let db_url: Url = self.database.url.clone().into();
        if db_url.as_str().trim().is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        if self.webservice.address.trim().is_empty() {
            return Err("Webservice address cannot be empty".to_string());
        }
        if self.llm.ollama_url.trim().is_empty() {
            return Err("LLM Ollama URL cannot be empty".to_string());
        }
        if self.llm.model.trim().is_empty() {
            return Err("LLM Model cannot be empty".to_string());
        }
        if self.llm.timeout_secs == 0 {
            return Err("LLM timeout_secs must be greater than 0".to_string());
        }
        Url::parse(&self.llm.ollama_url).map_err(|e| format!("Invalid LLM Ollama URL format: {}", e))?;
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
                url: UrlWithUsernamePassword {
                    url: Url::parse("postgres://localhost:5432/aaddb").unwrap(),
                    username: Some("postgres".to_string()),
                    password: Some("mysecretpassword".to_string()),
                },
                max_connections: 5,
            },
            webservice: WebServiceConfig {
                address: "0.0.0.0:8080".to_string(),
                api_prefix: "/api".to_string(),
            },
            llm: LlmConfig {
                ollama_url: "http://localhost:11434".to_string(),
                model: "llama3".to_string(),
                timeout_secs: 15,
            },
            hams: ::hams::hams::config::HamsConfig::default(),
            runtime: ThreadRuntime::default(),
            debugging: DebuggingConfig {
                environment: "development".to_string(),
                log_level: "info".to_string(),
                fail_debug_delay: Duration::from_secs(0),
            },
        };

        assert!(config.validate().is_ok());
        let db_url: Url = config.database.url.into();
        assert_eq!(db_url.as_str(), "postgres://postgres:mysecretpassword@localhost:5432/aaddb");
    }

    #[test]
    fn test_config_validation_empty_llm_url() {
        let config = AppConfig {
            database: DatabaseConfig {
                url: UrlWithUsernamePassword {
                    url: Url::parse("postgres://localhost:5432/aaddb").unwrap(),
                    username: Some("postgres".to_string()),
                    password: Some("mysecretpassword".to_string()),
                },
                max_connections: 5,
            },
            webservice: WebServiceConfig {
                address: "0.0.0.0:8080".to_string(),
                api_prefix: "/api".to_string(),
            },
            llm: LlmConfig {
                ollama_url: "".to_string(),
                model: "llama3".to_string(),
                timeout_secs: 15,
            },
            hams: ::hams::hams::config::HamsConfig::default(),
            runtime: ThreadRuntime::default(),
            debugging: DebuggingConfig {
                environment: "development".to_string(),
                log_level: "info".to_string(),
                fail_debug_delay: Duration::from_secs(0),
            },
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_zero_timeout() {
        let config = AppConfig {
            database: DatabaseConfig {
                url: UrlWithUsernamePassword {
                    url: Url::parse("postgres://localhost:5432/aaddb").unwrap(),
                    username: Some("postgres".to_string()),
                    password: Some("mysecretpassword".to_string()),
                },
                max_connections: 5,
            },
            webservice: WebServiceConfig {
                address: "0.0.0.0:8080".to_string(),
                api_prefix: "/api".to_string(),
            },
            llm: LlmConfig {
                ollama_url: "http://localhost:11434".to_string(),
                model: "llama3".to_string(),
                timeout_secs: 0,
            },
            hams: ::hams::hams::config::HamsConfig::default(),
            runtime: ThreadRuntime::default(),
            debugging: DebuggingConfig {
                environment: "development".to_string(),
                log_level: "info".to_string(),
                fail_debug_delay: Duration::from_secs(0),
            },
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_load_with_secrets_dir() {
        use std::fs;
        use std::io::Write;

        let test_dir = std::path::Path::new("test_aad_secrets_dir");
        if test_dir.exists() {
            let _ = fs::remove_dir_all(test_dir);
        }
        fs::create_dir_all(test_dir).unwrap();
        let test_dir = fs::canonicalize(test_dir).unwrap();

        fs::write(test_dir.join("db_user"), "secretuser").unwrap();
        fs::write(test_dir.join("db_pass"), "secretpass").unwrap();

        let test_config_path = test_dir.join("config.yaml");
        let mut file = fs::File::create(&test_config_path).unwrap();
        writeln!(file, "database:").unwrap();
        writeln!(file, "  url:").unwrap();
        writeln!(file, "    url: postgres://localhost:5432/aaddb").unwrap();
        writeln!(file, "    username_file: db_user").unwrap();
        writeln!(file, "    password_file: db_pass").unwrap();
        writeln!(file, "  max_connections: 5").unwrap();
        writeln!(file, "webservice:").unwrap();
        writeln!(file, "  address: '0.0.0.0:8080'").unwrap();
        writeln!(file, "  api_prefix: '/api'").unwrap();
        writeln!(file, "llm:").unwrap();
        writeln!(file, "  ollama_url: 'http://ollama.k8s:80'").unwrap();
        writeln!(file, "  model: 'qwen2.5-coder:14b'").unwrap();
        writeln!(file, "  timeout_secs: 120").unwrap();
        writeln!(file, "hams:").unwrap();
        writeln!(file, "  name: 'aad-be'").unwrap();
        writeln!(file, "  version: '0.1.0'").unwrap();
        writeln!(file, "  port: 8079").unwrap();
        writeln!(file, "runtime:").unwrap();
        writeln!(file, "  threads: 4").unwrap();
        writeln!(file, "  stack_size: 3000000").unwrap();
        writeln!(file, "  name: 'aad-worker'").unwrap();
        writeln!(file, "debugging:").unwrap();
        writeln!(file, "  environment: 'development'").unwrap();
        writeln!(file, "  log_level: 'info'").unwrap();

        let config = AppConfig::load(&test_config_path, &test_dir).unwrap();

        assert_eq!(config.database.url.username.as_deref(), Some("secretuser"));
        assert_eq!(config.database.url.password.as_deref(), Some("secretpass"));

        let db_url: Url = config.database.url.into();
        assert_eq!(db_url.as_str(), "postgres://secretuser:secretpass@localhost:5432/aaddb");

        let _ = fs::remove_dir_all(test_dir);
    }
}

