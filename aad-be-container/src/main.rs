mod api;
mod config;
mod error;
mod models;

use axum::{
    routing::get,
    Router,
};
use clap::{Parser, Subcommand};
use config::AppConfig;
use error::AppError;
use url::Url;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "aad-be")]
#[command(about = "Agent-As-Data Backend Service")]
struct Cli {
    #[arg(short, long, default_value = "config/default.yaml", env = "CONFIG_PATH")]
    config_path: PathBuf,

    #[arg(short, long, default_value = "config", env = "SECRETS_DIR")]
    secrets_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP server
    Serve,
    /// Run database migrations
    Migrate,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aad_be_container=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let config = AppConfig::load(&cli.config_path, &cli.secrets_dir)?;

    let database_url: String = Url::from(config.database.url).to_string();

    match cli.command {
        Commands::Serve => {
            tracing::info!("Connecting to database...");
            let pool = PgPoolOptions::new()
                .max_connections(config.database.max_connections)
                .connect(&database_url)
                .await?;

            tracing::info!("Running database migrations...");
            sqlx::migrate!("./migrations").run(&pool).await
                .map_err(|e| AppError::Message(format!("Migration failed: {}", e)))?;

            let app = Router::new()
                .route("/health", get(|| async { "OK" }))
                .route(
                    "/api/v1/agents",
                    get(api::agents::list_agents).post(api::agents::create_agent),
                )
                .route(
                    "/api/v1/agents/{id}",
                    get(api::agents::get_agent)
                        .put(api::agents::update_agent)
                        .delete(api::agents::delete_agent),
                )
                .layer(tower_http::trace::TraceLayer::new_for_http())
                .with_state(pool);

            let addr: SocketAddr = config.webservice.address.parse()
                .map_err(|e| AppError::Message(format!("Invalid socket address: {}", e)))?;

            tracing::info!("Agent-As-Data Backend listening on {}", addr);
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
        }
        Commands::Migrate => {
            tracing::info!("Connecting to database for migrations...");
            let pool = PgPoolOptions::new().connect(&database_url).await?;
            sqlx::migrate!("./migrations").run(&pool).await
                .map_err(|e| AppError::Message(format!("Migration failed: {}", e)))?;
            tracing::info!("Migrations complete.");
        }
    }

    Ok(())
}
