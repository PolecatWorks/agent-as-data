use std::fs;
use std::io::Write;
use aad_mcp_container::config::{AppConfig, WebServiceConfig, DebuggingConfig};
use aad_mcp_container::tokio_tools::ThreadRuntime;
use std::time::Duration;

#[test]
fn test_config_validation_valid() {
    let config = AppConfig {
        webservice: WebServiceConfig {
            address: "0.0.0.0:8080".to_string(),
            api_prefix: "/api".to_string(),
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
}

#[test]
fn test_config_validation_empty_address() {
    let config = AppConfig {
        webservice: WebServiceConfig {
            address: "   ".to_string(),
            api_prefix: "/api".to_string(),
        },
        hams: ::hams::hams::config::HamsConfig::default(),
        runtime: ThreadRuntime::default(),
        debugging: DebuggingConfig {
            environment: "development".to_string(),
            log_level: "info".to_string(),
            fail_debug_delay: Duration::from_secs(0),
        },
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Webservice address cannot be empty"));
}

#[test]
fn test_config_load_from_file_and_env_override() {
    let test_dir = std::path::Path::new("target/test_mcp_config");
    if test_dir.exists() {
        let _ = fs::remove_dir_all(test_dir);
    }
    fs::create_dir_all(test_dir).unwrap();
    let test_dir = fs::canonicalize(test_dir).unwrap();

    let config_path = test_dir.join("default.yaml");
    let mut f = fs::File::create(&config_path).unwrap();
    writeln!(f, "webservice:").unwrap();
    writeln!(f, "  address: '0.0.0.0:8080'").unwrap();
    writeln!(f, "  api_prefix: '/api'").unwrap();
    writeln!(f, "hams:").unwrap();
    writeln!(f, "  name: 'aad-mcp'").unwrap();
    writeln!(f, "  version: '0.1.0'").unwrap();
    writeln!(f, "  address: '0.0.0.0:8079'").unwrap();
    writeln!(f, "runtime:").unwrap();
    writeln!(f, "  threads: 2").unwrap();
    writeln!(f, "  stack_size: 3000000").unwrap();
    writeln!(f, "  name: 'aad-mcp-worker'").unwrap();
    writeln!(f, "debugging:").unwrap();
    writeln!(f, "  environment: 'development'").unwrap();
    writeln!(f, "  log_level: 'info'").unwrap();
    writeln!(f, "  fail_debug_delay: '0s'").unwrap();

    // Test standard loading
    let loaded = AppConfig::load(&config_path, &test_dir).expect("Failed to load config");
    assert_eq!(loaded.webservice.address, "0.0.0.0:8080");
    assert_eq!(loaded.hams.address.port(), 8079);

    // Test env override
    unsafe {
        std::env::set_var("AAD_MCP__WEBSERVICE__ADDRESS", "127.0.0.1:9090");
        std::env::set_var("AAD_MCP__HAMS__ADDRESS", "127.0.0.1:9079");
    }

    let loaded_overridden = AppConfig::load(&config_path, &test_dir).expect("Failed to load overridden config");
    assert_eq!(loaded_overridden.webservice.address, "127.0.0.1:9090");
    assert_eq!(loaded_overridden.hams.address.port(), 9079);

    unsafe {
        std::env::remove_var("AAD_MCP__WEBSERVICE__ADDRESS");
        std::env::remove_var("AAD_MCP__HAMS__ADDRESS");
    }

    let _ = fs::remove_dir_all(test_dir);
}
