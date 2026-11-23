use figment::{
    Figment,
    providers::{Env, Format, Yaml},
};
use figment_file_provider_adapter::FileAdapter;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;
use thiserror::Error;
use url::Url;

use ::hams::hams::config::HamsConfig;

use crate::mcp::{McpConfig, McpError};
use crate::tokio_tools::ThreadRuntime;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Database URL cannot be empty")]
    EmptyDatabaseUrl,
    #[error("Failed to set database username on database.url")]
    SetDatabaseUsernameFailed,
    #[error("Failed to set database password on database.url")]
    SetDatabasePasswordFailed,
    #[error("Webservice api_prefix must be non-empty and start with '/'")]
    InvalidApiPrefix,
    #[error("LLM Ollama URL cannot be empty")]
    EmptyLlmUrl,
    #[error("LLM Model cannot be empty")]
    EmptyLlmModel,
    #[error("LLM timeout must be greater than 0")]
    ZeroLlmTimeout,
    #[error("Runtime metrics_interval must be greater than 0")]
    ZeroMetricsInterval,
    #[error(transparent)]
    Mcp(#[from] McpError),
}

/// A URL structure that optionally embeds credential placeholders for file-based secret resolution.
#[derive(Serialize, Debug, Clone)]
pub struct UrlWithUsernamePassword {
    /// Base target URL.
    pub url: Url,
    /// Optional username credential.
    pub username: Option<String>,
    /// Optional password credential.
    pub password: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum UrlHelper {
    Full {
        url: Url,
        username: Option<String>,
        password: Option<String>,
    },
    Simple(Url),
}

impl<'de> Deserialize<'de> for UrlWithUsernamePassword {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper = UrlHelper::deserialize(deserializer)?;
        match helper {
            UrlHelper::Full {
                url,
                username,
                password,
            } => Ok(Self {
                url,
                username,
                password,
            }),
            UrlHelper::Simple(url) => Ok(Self {
                url,
                username: None,
                password: None,
            }),
        }
    }
}

impl UrlWithUsernamePassword {
    pub fn connection_url(&self) -> Result<Url, ConfigError> {
        let mut return_url = self.url.clone();
        if let Some(ref username) = self.username {
            return_url
                .set_username(username)
                .map_err(|()| ConfigError::SetDatabaseUsernameFailed)?;
        }
        if let Some(ref password) = self.password {
            return_url
                .set_password(Some(password))
                .map_err(|()| ConfigError::SetDatabasePasswordFailed)?;
        }
        Ok(return_url)
    }
}

impl TryFrom<UrlWithUsernamePassword> for Url {
    type Error = ConfigError;

    fn try_from(value: UrlWithUsernamePassword) -> Result<Self, Self::Error> {
        value.connection_url()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub webservice: WebServiceConfig,
    pub llm: LlmConfig,
    pub mcp: McpConfig,
    #[serde(serialize_with = "serialize_hams")]
    pub hams: HamsConfig,
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
    pub ollama_url: Url,
    pub model: String,
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    pub default_max_turns: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DebuggingConfig {
    pub environment: String,
    pub log_level: String,
    #[serde(with = "humantime_serde")]
    pub fail_debug_delay: Duration,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DatabaseConfig {
    pub url: UrlWithUsernamePassword,
    pub max_connections: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WebServiceConfig {
    pub address: SocketAddr,
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
    pub fn validate(&self) -> Result<(), ConfigError> {
        let db_url = self.database.url.connection_url()?;
        if db_url.as_str().trim().is_empty() {
            return Err(ConfigError::EmptyDatabaseUrl);
        }
        let api_prefix = self.webservice.api_prefix.trim();
        if api_prefix.is_empty() || !api_prefix.starts_with('/') {
            return Err(ConfigError::InvalidApiPrefix);
        }
        if self.llm.ollama_url.as_str().trim().is_empty() {
            return Err(ConfigError::EmptyLlmUrl);
        }
        if self.llm.model.trim().is_empty() {
            return Err(ConfigError::EmptyLlmModel);
        }
        if self.llm.timeout.is_zero() {
            return Err(ConfigError::ZeroLlmTimeout);
        }
        self.mcp.validate()?;
        if self.runtime.metrics_interval.is_zero() {
            return Err(ConfigError::ZeroMetricsInterval);
        }
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
                address: "0.0.0.0:8080".parse().unwrap(),
                api_prefix: "/api".to_string(),
            },
            llm: LlmConfig {
                ollama_url: Url::parse("http://localhost:11434").unwrap(),
                model: "llama3".to_string(),
                timeout: Duration::from_secs(15),
                default_max_turns: 5,
            },
            mcp: McpConfig::default(),
            hams: ::hams::hams::config::HamsConfig::default(),
            runtime: ThreadRuntime::default(),
            debugging: DebuggingConfig {
                environment: "development".to_string(),
                log_level: "info".to_string(),
                fail_debug_delay: Duration::from_secs(0),
            },
        };

        assert!(config.validate().is_ok());
        let db_url = config.database.url.connection_url().unwrap();
        assert_eq!(
            db_url.as_str(),
            "postgres://postgres:mysecretpassword@localhost:5432/aaddb"
        );
    }

    #[test]
    fn test_config_validation_invalid_api_prefix() {
        let mut config = AppConfig {
            database: DatabaseConfig {
                url: UrlWithUsernamePassword {
                    url: Url::parse("postgres://localhost:5432/aaddb").unwrap(),
                    username: Some("postgres".to_string()),
                    password: Some("mysecretpassword".to_string()),
                },
                max_connections: 5,
            },
            webservice: WebServiceConfig {
                address: "0.0.0.0:8080".parse().unwrap(),
                api_prefix: "".to_string(),
            },
            llm: LlmConfig {
                ollama_url: Url::parse("http://localhost:11434").unwrap(),
                model: "llama3".to_string(),
                timeout: Duration::from_secs(15),
                default_max_turns: 5,
            },
            mcp: McpConfig::default(),
            hams: ::hams::hams::config::HamsConfig::default(),
            runtime: ThreadRuntime::default(),
            debugging: DebuggingConfig {
                environment: "development".to_string(),
                log_level: "info".to_string(),
                fail_debug_delay: Duration::from_secs(0),
            },
        };

        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidApiPrefix)
        ));

        config.webservice.api_prefix = "api/v1".to_string();
        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidApiPrefix)
        ));
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
                address: "0.0.0.0:8080".parse().unwrap(),
                api_prefix: "/api".to_string(),
            },
            llm: LlmConfig {
                ollama_url: Url::parse("http://localhost:11434").unwrap(),
                model: "llama3".to_string(),
                timeout: Duration::from_secs(0),
                default_max_turns: 5,
            },
            mcp: McpConfig::default(),
            hams: ::hams::hams::config::HamsConfig::default(),
            runtime: ThreadRuntime::default(),
            debugging: DebuggingConfig {
                environment: "development".to_string(),
                log_level: "info".to_string(),
                fail_debug_delay: Duration::from_secs(0),
            },
        };

        assert!(matches!(
            config.validate(),
            Err(ConfigError::ZeroLlmTimeout)
        ));
    }

    #[test]
    fn test_config_validation_zero_metrics_interval() {
        let mut config = AppConfig {
            database: DatabaseConfig {
                url: UrlWithUsernamePassword {
                    url: Url::parse("postgres://localhost:5432/aaddb").unwrap(),
                    username: Some("postgres".to_string()),
                    password: Some("mysecretpassword".to_string()),
                },
                max_connections: 5,
            },
            webservice: WebServiceConfig {
                address: "0.0.0.0:8080".parse().unwrap(),
                api_prefix: "/api".to_string(),
            },
            llm: LlmConfig {
                ollama_url: Url::parse("http://localhost:11434").unwrap(),
                model: "llama3".to_string(),
                timeout: Duration::from_secs(15),
                default_max_turns: 5,
            },
            mcp: McpConfig::default(),
            hams: ::hams::hams::config::HamsConfig::default(),
            runtime: ThreadRuntime::default(),
            debugging: DebuggingConfig {
                environment: "development".to_string(),
                log_level: "info".to_string(),
                fail_debug_delay: Duration::from_secs(0),
            },
        };
        config.runtime.metrics_interval = Duration::from_secs(0);

        let res = config.validate();
        assert!(matches!(res, Err(ConfigError::ZeroMetricsInterval)));
        assert!(res.unwrap_err().to_string().contains("metrics_interval"));
    }

    #[test]
    fn test_config_validation_delegates_to_mcp() {
        let mut config = AppConfig {
            database: DatabaseConfig {
                url: UrlWithUsernamePassword {
                    url: Url::parse("postgres://localhost:5432/aaddb").unwrap(),
                    username: Some("postgres".to_string()),
                    password: Some("mysecretpassword".to_string()),
                },
                max_connections: 5,
            },
            webservice: WebServiceConfig {
                address: "0.0.0.0:8080".parse().unwrap(),
                api_prefix: "/api".to_string(),
            },
            llm: LlmConfig {
                ollama_url: Url::parse("http://localhost:11434").unwrap(),
                model: "llama3".to_string(),
                timeout: Duration::from_secs(15),
                default_max_turns: 5,
            },
            mcp: McpConfig::default(),
            hams: ::hams::hams::config::HamsConfig::default(),
            runtime: ThreadRuntime::default(),
            debugging: DebuggingConfig {
                environment: "development".to_string(),
                log_level: "info".to_string(),
                fail_debug_delay: Duration::from_secs(0),
            },
        };
        config.mcp.server_name = "   ".to_string();

        let res = config.validate();
        assert!(matches!(
            res,
            Err(ConfigError::Mcp(McpError::EmptyServerName))
        ));
        assert!(res.unwrap_err().to_string().contains("server_name"));
    }

    #[test]
    fn test_url_with_username_password_rejects_cannot_be_a_base_credentials() {
        let bad_creds = UrlWithUsernamePassword {
            url: Url::parse("mailto:someone@example.com").unwrap(),
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
        };
        assert!(matches!(
            bad_creds.connection_url(),
            Err(ConfigError::SetDatabaseUsernameFailed)
        ));
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
        writeln!(file, "  timeout: 120s").unwrap();
        writeln!(file, "  default_max_turns: 5").unwrap();
        writeln!(file, "mcp:").unwrap();
        writeln!(file, "  enabled: true").unwrap();
        writeln!(file, "  server_name: 'aad-be-mcp'").unwrap();
        writeln!(file, "  instructions: 'AAD MCP Server'").unwrap();
        writeln!(file, "hams:").unwrap();
        writeln!(file, "  name: 'aad-be'").unwrap();
        writeln!(file, "  version: '0.1.0'").unwrap();
        writeln!(file, "  port: 8079").unwrap();
        writeln!(file, "runtime:").unwrap();
        writeln!(file, "  threads: 4").unwrap();
        writeln!(file, "  stack_size: 3000000").unwrap();
        writeln!(file, "  name: 'aad-worker'").unwrap();
        writeln!(file, "  metrics_interval: 5s").unwrap();
        writeln!(file, "debugging:").unwrap();
        writeln!(file, "  environment: 'development'").unwrap();
        writeln!(file, "  log_level: 'info'").unwrap();
        writeln!(file, "  fail_debug_delay: 0s").unwrap();

        let config = AppConfig::load(&test_config_path, &test_dir).unwrap();

        assert_eq!(config.database.url.username.as_deref(), Some("secretuser"));
        assert_eq!(config.database.url.password.as_deref(), Some("secretpass"));

        let db_url = config.database.url.connection_url().unwrap();
        assert_eq!(
            db_url.as_str(),
            "postgres://secretuser:secretpass@localhost:5432/aaddb"
        );

        let _ = fs::remove_dir_all(test_dir);
    }

    #[test]
    fn test_url_deserialization_from_string_and_object() {
        let yaml_str = "url: 'postgres://localhost:5432/testdb'";
        #[derive(serde::Deserialize)]
        struct Wrapper {
            url: UrlWithUsernamePassword,
        }
        let parsed_str: Wrapper = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(
            parsed_str.url.url.as_str(),
            "postgres://localhost:5432/testdb"
        );
        assert!(parsed_str.url.username.is_none());

        let yaml_obj =
            "url:\n  url: 'postgres://localhost:5432/testdb'\n  username: 'user'\n  password: 'pw'";
        let parsed_obj: Wrapper = serde_yaml::from_str(yaml_obj).unwrap();
        assert_eq!(
            parsed_obj.url.url.as_str(),
            "postgres://localhost:5432/testdb"
        );
        assert_eq!(parsed_obj.url.username.as_deref(), Some("user"));
    }
}
