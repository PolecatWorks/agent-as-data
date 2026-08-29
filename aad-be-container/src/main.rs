use std::path::PathBuf;
use clap::{Parser, Subcommand};
use tracing::info;

use aad_be_container::config::AppConfig;
use aad_be_container::db::{init_db_pool, verify_pgvector_extension};
use aad_be_container::hams_tools::HamsHarness;
use aad_be_container::tokio_tools::run_in_tokio;
use aad_be_container::{NAME, VERSION};
use ::hams::hams::Hams;

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
            let config = AppConfig::load(&cli.config_path, &cli.secrets_dir).unwrap_or_else(|e| {
                init_logging("info");
                panic!("Fail-Fast Error: Failed to load config: {}", e);
            });

            init_logging(&config.debugging.log_level);
            info!("Starting Agent-As-Data backend v{}", VERSION);

            // 1. Fail-Fast Config Validation
            if let Err(e) = config.validate() {
                panic!("Fail-Fast Configuration Error: {}", e);
            }

            // 2. Launch Application in Configurable Tokio Runtime
            let runtime_config = config.runtime.clone();

            run_in_tokio(&runtime_config, async move {
                // Initialize HaMS Health Monitoring Sidecar & ProbeManual readiness signal
                let mut hams_config = config.hams.clone();
                hams_config.name = NAME.to_owned();
                hams_config.version = VERSION.to_owned();

                let hams = Hams::new(hams_config);
                let _hams_harness = HamsHarness::init(hams).await
                    .map_err(|e| format!("HaMS init error: {}", e))?;
                info!("HaMS health sidecar started on port 8079 with readiness probe.");

                // Connect DB Pool & Verify pgvector (Fail-Fast)
                let db_url: url::Url = config.database.url.clone().into();
                let pool = match init_db_pool(db_url.as_str(), config.database.max_connections).await {
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

                let app_state = aad_be_container::AppState {
                    pool,
                    config: config.clone(),
                };

                // API Routes Group
                let api_routes = axum::Router::new()
                    .route("/v1/knowledge", axum::routing::post(aad_be_container::knowledge::ingest_knowledge))
                    .route("/v1/knowledge/search", axum::routing::post(aad_be_container::knowledge::search_knowledge))
                    .route("/v1/knowledge/graph/traverse", axum::routing::post(aad_be_container::knowledge::traverse_graph))
                    .route("/v1/agents", axum::routing::post(aad_be_container::agents::create_agent))
                    .route("/v1/agents/{id}", axum::routing::get(aad_be_container::agents::get_agent))
                    .route("/v1/agents/{id}", axum::routing::put(aad_be_container::agents::update_agent))
                    .route("/v1/agents/{id}", axum::routing::delete(aad_be_container::agents::delete_agent))
                    .route("/v1/agents/{id}/test", axum::routing::post(aad_be_container::agents::test_agent))
                    .route("/v1/agents/search", axum::routing::post(aad_be_container::agents::search_agents))
                    .route("/v1/agents/verify-contract", axum::routing::post(aad_be_container::agents::verify_contract))
                    .route("/v1/agents/refactor/analyze", axum::routing::post(aad_be_container::agents::analyze_refactor))
                    .route("/v1/agents/compile", axum::routing::post(aad_be_container::agents::compile_agent))
                    .route("/v1/skills", axum::routing::post(aad_be_container::agents::create_skill))
                    .route("/v1/skills", axum::routing::get(aad_be_container::agents::list_skills))
                    .route("/v1/skills/{id}", axum::routing::get(aad_be_container::agents::get_skill))
                    .route("/v1/skills/{id}", axum::routing::put(aad_be_container::agents::update_skill))
                    .route("/v1/skills/{id}", axum::routing::delete(aad_be_container::agents::delete_skill))
                    .route("/v1/skills/{id}/promote", axum::routing::post(aad_be_container::agents::promote_skill))
                    .route("/v1/agents/{id}/demote", axum::routing::post(aad_be_container::agents::demote_skill))
                    .route("/v1/agents/tools/register", axum::routing::post(aad_be_container::tools::register_tool))
                    .route("/v1/agents/tools", axum::routing::get(aad_be_container::tools::list_tools))
                    .route("/v1/agents/tools/{id}", axum::routing::delete(aad_be_container::tools::delete_tool))
                    .route("/v1/traits", axum::routing::get(aad_be_container::traits::list_traits))
                    .route("/v1/traits", axum::routing::post(aad_be_container::traits::create_trait))
                    .route("/v1/traits/{id}", axum::routing::get(aad_be_container::traits::get_trait))
                    .route("/v1/traits/{id}", axum::routing::put(aad_be_container::traits::update_trait))
                    .route("/v1/traits/{id}", axum::routing::delete(aad_be_container::traits::delete_trait))
                    .route("/v1/agents/{id}/execute", axum::routing::post(aad_be_container::execution::execute_agent))
                    .route("/v1/agents/search-and-execute", axum::routing::post(aad_be_container::execution::search_and_execute))
                    .route("/v1/executions/{id}", axum::routing::get(aad_be_container::execution::get_execution))
                    .route("/v1/threads", axum::routing::post(aad_be_container::threads::list_threads))
                    .route("/v1/threads/create", axum::routing::post(aad_be_container::threads::create_thread))
                    .route("/v1/threads/{id}", axum::routing::get(aad_be_container::threads::get_thread))
                    .route("/v1/threads/{id}", axum::routing::put(aad_be_container::threads::update_thread))
                    .route("/v1/threads/{id}/messages", axum::routing::get(aad_be_container::threads::list_messages))
                    .route("/v1/threads/{id}/messages", axum::routing::post(aad_be_container::threads::create_message))
                    .route("/v1/threads/{id}/fs/write", axum::routing::post(aad_be_container::fs_tools::write_file))
                    .route("/v1/threads/{id}/fs/read/{*filepath}", axum::routing::get(aad_be_container::fs_tools::read_file))
                    .route("/v1/threads/{id}/fs/list", axum::routing::post(aad_be_container::fs_tools::list_files))
                    .route("/v1/threads/{id}/fs/delete", axum::routing::post(aad_be_container::fs_tools::delete_file))
                    .with_state(app_state);

                // Start Axum Main REST Service
                let app = axum::Router::new()
                    .route("/health", axum::routing::get(|| async { "OK" }))
                    .nest(&config.webservice.api_prefix, api_routes);


                let listener = tokio::net::TcpListener::bind(&config.webservice.address).await
                    .map_err(|e| format!("Listener bind error: {}", e))?;

                info!("Axum REST Service listening on {}", config.webservice.address);
                axum::serve(listener, app).await.map_err(|e| format!("Axum error: {}", e))?;

                Ok(())
            })?;
        }
        Commands::Migrate => {
            let config = AppConfig::load(&cli.config_path, &cli.secrets_dir)?;
            init_logging(&config.debugging.log_level);

            run_in_tokio(&config.runtime, async move {
                let db_url: url::Url = config.database.url.into();
                let pool = init_db_pool(db_url.as_str(), config.database.max_connections).await
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
