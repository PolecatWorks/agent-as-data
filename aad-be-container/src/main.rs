//! Agent-As-Data Backend (`aad-be-container`) binary CLI entrypoint.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use aad_be_container::config::AppConfig;
use aad_be_container::tokio_tools::run_in_tokio;
use aad_be_container::{VERSION, run_migrations, service_main};

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

use aad_be_container::cli::client::{base_url_from_webservice_config, normalize_base_url};
use aad_be_container::cli::ctl::{CtlArgs, CtlCommands, run_ctl, run_validate_offline};

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the main application server
    Serve,
    /// Execute database migrations
    Migrate,
    /// Display application version
    Version,
    /// Hydrate, validate, inspect, export, and manage AAD resources via REST API
    Ctl(CtlArgs),
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
        Commands::Ctl(ctl_args) => {
            if let CtlCommands::Validate {
                ref file,
                recursive,
                self_contained,
            } = ctl_args.command
            {
                if let Err(e) = run_validate_offline(
                    file,
                    recursive,
                    self_contained,
                    ctl_args.vars_file.as_deref(),
                    &ctl_args.vars,
                ) {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                return Ok(());
            }

            let result = (|| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let (base_url, runtime_cfg) = if let Some(ref raw_url) = ctl_args.url {
                    (
                        normalize_base_url(raw_url)?,
                        aad_be_container::tokio_tools::ThreadRuntime::default(),
                    )
                } else {
                    let config = AppConfig::load(&cli.config_path, &cli.secrets_dir)?;
                    let url = base_url_from_webservice_config(&config.webservice)?;
                    (url, config.runtime)
                };

                run_in_tokio(&runtime_cfg, async {
                    run_ctl(ctl_args, &base_url)
                        .await
                        .map_err(|e| format!("{}", e))
                })?;

                Ok(())
            })();

            if let Err(e) = result {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
