//! Agent-As-Data Backend (`aad-be-container`) binary CLI entrypoint.

use std::path::PathBuf;
use clap::{Parser, Subcommand};

use aad_be_container::config::AppConfig;
use aad_be_container::tokio_tools::run_in_tokio;
use aad_be_container::{run_migrations, service_main, VERSION};

#[derive(Parser, Debug)]
#[command(name = "aad-be", about = "Agent-As-Data Backend Microservice", version)]
pub struct Cli {
    #[arg(short, long, default_value = "config/default.yaml")]
    pub config_path: PathBuf,

    #[arg(short, long, default_value = "config")]
    pub secrets_dir: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

use aad_be_container::cli::ctl::{run_ctl, CtlCommands};

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the main application server
    Serve,
    /// Execute database migrations
    Migrate,
    /// Display application version
    Version,
    /// Manage resources declaratively via API
    Ctl {
        #[command(subcommand)]
        command: CtlCommands,
    },
}

fn init_logging(log_level: &str) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve => {
            let mut delay = None;

            let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                let config = AppConfig::load(&cli.config_path, &cli.secrets_dir).map_err(|e| {
                    init_logging("info");
                    tracing::error!("Failed to load config: {}", e);
                    format!("Fail-Fast Error: Failed to load config: {}", e)
                })?;

                init_logging(&config.debugging.log_level);
                delay = Some(config.debugging.fail_debug_delay);

                run_in_tokio(&config.runtime, async {
                    service_main(&cli.config_path, &cli.secrets_dir)
                        .await
                        .map_err(|e| format!("Service Error: {}", e))
                })?;

                Ok(())
            })();

            if let Err(e) = result {
                if let Some(d) = delay {
                    if !d.is_zero() {
                        tracing::error!(
                            "Serve failed: {}. Sleeping for {:?} before exiting...",
                            e,
                            d
                        );
                        std::thread::sleep(d);
                    }
                }
                return Err(e);
            }
        }
        Commands::Migrate => {
            let mut delay = None;

            let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                let config = AppConfig::load(&cli.config_path, &cli.secrets_dir).map_err(|e| {
                    init_logging("info");
                    tracing::error!("Failed to load config: {}", e);
                    format!("Fail-Fast Error: Failed to load config: {}", e)
                })?;

                init_logging(&config.debugging.log_level);
                delay = Some(config.debugging.fail_debug_delay);

                run_in_tokio(&config.runtime, async {
                    run_migrations(&cli.config_path, &cli.secrets_dir)
                        .await
                        .map_err(|e| format!("Migration error: {}", e))
                })?;

                Ok(())
            })();

            if let Err(e) = result {
                if let Some(d) = delay {
                    if !d.is_zero() {
                        tracing::error!(
                            "Migrate failed: {}. Sleeping for {:?} before exiting...",
                            e,
                            d
                        );
                        std::thread::sleep(d);
                    }
                }
                return Err(e);
            }
        }
        Commands::Version => {
            println!("aad-be {}", VERSION);
        }
        Commands::Ctl { command } => {
            let result = (|| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let config = AppConfig::load(&cli.config_path, &cli.secrets_dir)?;

                // Construct the base URL for the API
                let address = config.webservice.address.replace("0.0.0.0", "127.0.0.1");
                let base_url = format!("http://{}/{}", address, config.webservice.api_prefix);

                run_in_tokio(&config.runtime, async {
                    run_ctl(command, &base_url)
                        .await
                        .map_err(|e| format!("CTL Error: {}", e))
                })?;

                Ok(())
            })();

            if let Err(e) = result {
                tracing::error!("ctl command failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
