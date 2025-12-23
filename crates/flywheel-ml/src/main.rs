use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;
mod output;

#[derive(Parser)]
#[command(name = "flywheel-ml")]
#[command(about = "Flywheel-ML CLI - AIOps ML pipeline operations")]
#[command(long_about = "Flywheel-ML CLI for ML pipeline operations.\n\nFor pipeline CRUD operations, use kubectl:\n  kubectl apply -f pipeline.yaml\n  kubectl get flywheelpipelines\n  kubectl delete flywheelpipeline <name>")]
#[command(version)]
struct Cli {
    #[arg(short, long, default_value = "http://localhost:50051")]
    server: String,

    #[arg(short, long, default_value = "default")]
    namespace: String,

    #[arg(short, long, default_value = "false")]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Pipeline management (apply, enable, disable, delete)")]
    Pipeline(commands::pipeline::PipelineArgs),

    #[command(about = "Check health status")]
    Health(commands::health::HealthArgs),

    #[command(about = "View pipeline logs")]
    Logs(commands::logs::LogsArgs),

    #[command(about = "Visualize pipeline DAG")]
    Graph(commands::graph::GraphArgs),

    #[command(about = "Export training data")]
    Export(commands::export::ExportArgs),

    #[command(about = "Drift detection status and history")]
    Drift(commands::drift::DriftArgs),

    #[command(about = "Model management")]
    Model(commands::model::ModelArgs),

    #[command(about = "Pipeline metrics and statistics")]
    Stats(commands::stats::StatsArgs),

    #[command(about = "Validate a pipeline manifest")]
    Validate(commands::validate::ValidateArgs),
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

    let ctx = commands::Context {
        server: cli.server,
        namespace: cli.namespace,
        verbose: cli.verbose,
    };

    match cli.command {
        Commands::Pipeline(args) => commands::pipeline::run(&ctx, args).await?,
        Commands::Health(args) => commands::health::run(&ctx, args).await?,
        Commands::Logs(args) => commands::logs::run(&ctx, args).await?,
        Commands::Graph(args) => commands::graph::run(&ctx, args).await?,
        Commands::Export(args) => commands::export::run(&ctx, args).await?,
        Commands::Drift(args) => commands::drift::run(&ctx, args).await?,
        Commands::Model(args) => commands::model::run(&ctx, args).await?,
        Commands::Stats(args) => commands::stats::run(&ctx, args).await?,
        Commands::Validate(args) => commands::validate::run(&ctx, args).await?,
    }

    Ok(())
}
