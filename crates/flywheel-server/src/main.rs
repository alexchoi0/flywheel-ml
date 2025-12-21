use clap::Parser;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod grpc;
mod health;
mod registry;

#[derive(Parser)]
#[command(name = "flywheel-server")]
#[command(about = "Flywheel server - AIOps ML pipeline control plane")]
#[command(version)]
struct Cli {
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,

    #[arg(long, env = "FLYWHEEL_BIND_ADDRESS", default_value = "0.0.0.0:50051")]
    bind_address: SocketAddr,

    #[arg(long, env = "FLYWHEEL_METRICS_ADDRESS", default_value = "0.0.0.0:9090")]
    metrics_address: SocketAddr,

    #[arg(long, env = "DATABASE_URL")]
    db_url: Option<String>,

    #[arg(long, env = "CONVEYOR_ROUTER_ENDPOINT")]
    conveyor_endpoint: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    tracing::info!("Starting Flywheel server...");
    tracing::info!("Bind address: {}", cli.bind_address);
    tracing::info!("Metrics address: {}", cli.metrics_address);

    // Load configuration
    let config = if let Some(config_path) = &cli.config {
        config::Config::from_file(config_path)?
    } else {
        config::Config::default()
    };

    // Override with CLI args
    let db_url = cli.db_url.unwrap_or(config.database.url);

    // Connect to database
    tracing::info!("Connecting to database...");
    let db = flywheel_db::Database::connect(&db_url).await?;
    tracing::info!("Database connected");

    // Run migrations
    tracing::info!("Running migrations...");
    use sea_orm_migration::MigratorTrait;
    flywheel_db::migration::Migrator::up(db.conn(), None).await?;
    tracing::info!("Migrations complete");

    // Start gRPC server
    tracing::info!("Starting gRPC server on {}", cli.bind_address);

    let control_service = grpc::ControlServiceImpl::new(db.clone());
    let health_service = grpc::HealthServiceImpl::new(db.clone());

    tonic::transport::Server::builder()
        .add_service(flywheel_proto::control_service_server::ControlServiceServer::new(
            control_service,
        ))
        .add_service(flywheel_proto::health_service_server::HealthServiceServer::new(
            health_service,
        ))
        .serve(cli.bind_address)
        .await?;

    Ok(())
}
