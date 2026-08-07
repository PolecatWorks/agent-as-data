use std::path::PathBuf;
use clap::{Parser, Subcommand};
use tracing::info;

use aad_be_container::config::AppConfig;
use aad_be_container::db::{init_db_pool, verify_pgvector_extension};
use aad_be_container::tokio_tools::run_in_tokio;
use aad_be_container::{NAME, VERSION};
use ::hams::hams::Hams;

#[derive(Parser, Debug)]
#[command(name = "aad-be", about = "Agent-As-Data Backend Microservice", version)]
pub struct Cli {
    #[arg(short, long, default_value = "config/default.yaml")]
    pub config_path: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Serve,
    Migrate,
    Version,
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
            let config = AppConfig::load(&cli.config_path).unwrap_or_else(|e| {
                init_logging("info");
                panic!("Fail-Fast Error: Failed to load config: {}", e);
            });

            init_logging(&config.debugging.log_level);
            info!("Starting Agent-As-Data backend v{}", VERSION);

            // 1. Fail-Fast Config Validation
            if let Err(e) = config.validate() {
                panic!("Fail-Fast Configuration Error: {}", e);
            }

            // 2. Start HaMS Health Monitoring Sidecar
            let mut hams_config = config.hams.clone();
            hams_config.name = NAME.to_owned();
            hams_config.version = VERSION.to_owned();

            let mut hams = Hams::new(hams_config);
            hams.start().map_err(|e| format!("Failed to start HaMS: {}", e))?;
            info!("HaMS health monitoring sidecar started on port 8079.");

            // 3. Launch Async Application inside Configurable Tokio Runtime
            let runtime_config = config.runtime.clone();

            run_in_tokio(&runtime_config, async move {
                // Connect DB Pool & Verify pgvector (Fail-Fast)
                let pool = match init_db_pool(&config.database.url, config.database.max_connections).await {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("Fail-Fast Error: Database connection failed: {}", e);
                        panic!("Fail-Fast Error: Database connection failed: {}", e);
                    }
                };

                if let Err(e) = verify_pgvector_extension(&pool).await {
                    tracing::error!("{}", e);
                    panic!("{}", e);
                }

                // Run Automatic Schema Migrations
                info!("Executing database schema migrations...");
                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .map_err(|e| format!("Migration failed: {}", e))?;
                info!("Database migrations applied successfully.");

                // Start Axum Main REST Service
                let app = axum::Router::new()
                    .route("/health", axum::routing::get(|| async { "OK" }));

                let listener = tokio::net::TcpListener::bind(&config.webservice.address).await
                    .map_err(|e| format!("Listener bind error: {}", e))?;

                info!("Axum REST Service listening on {}", config.webservice.address);
                axum::serve(listener, app).await.map_err(|e| format!("Axum error: {}", e))?;

                Ok(())
            })?;
        }
        Commands::Migrate => {
            let config = AppConfig::load(&cli.config_path)?;
            init_logging(&config.debugging.log_level);

            run_in_tokio(&config.runtime, async move {
                let pool = init_db_pool(&config.database.url, config.database.max_connections).await
                    .map_err(|e| format!("DB connection error: {}", e))?;
                sqlx::migrate!("./migrations").run(&pool).await
                    .map_err(|e| format!("Migration error: {}", e))?;
                println!("Migrations completed successfully.");
                Ok(())
            })?;
        }
        Commands::Version => {
            println!("aad-be {}", VERSION);
        }
    }

    Ok(())
}
