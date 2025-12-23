use clap::{Args, Subcommand};

use super::Context;

#[derive(Args)]
pub struct HealthArgs {
    #[command(subcommand)]
    pub resource: Option<HealthResource>,
}

#[derive(Subcommand)]
pub enum HealthResource {
    #[command(about = "Check pipeline health")]
    Pipeline { name: String },
}

pub async fn run(ctx: &Context, args: HealthArgs) -> anyhow::Result<()> {
    let client = ctx.client().await?;

    match args.resource {
        None => {
            let health = client.get_health().await?;

            println!("Flywheel-ML Server Health");
            println!("======================");
            println!("Server: {}", ctx.server);
            println!("Status: {}", health.status);
            println!("Version: {}", health.version);
            println!("Active Pipelines: {}", health.active_pipelines);
            println!("Registered Models: {}", health.registered_models);

            if let Some(db) = health.database {
                println!("\nDatabase:");
                println!("  Connected: {}", db.connected);
                println!("  Latency: {}ms", db.latency_ms);
                println!("  Active Connections: {}", db.active_connections);
            }
        }
        Some(HealthResource::Pipeline { name }) => {
            let health = client.get_pipeline_health(&name).await?;

            println!("Pipeline Health: {}", name);
            println!("================");
            println!("Status: {}", health.status);

            if let Some(metrics) = health.metrics {
                println!("\nMetrics:");
                println!("  Records/sec: {:.0}", metrics.records_per_second);
                println!("  Predictions/sec: {:.0}", metrics.predictions_per_second);
                println!("  Error Rate: {:.2}%", metrics.error_rate * 100.0);
                println!("  Avg Latency: {}ms", metrics.avg_latency_ms);
                println!("  P99 Latency: {}ms", metrics.p99_latency_ms);
                println!("  Accuracy: {:.2}", metrics.current_accuracy);
            }

            if let Some(drift) = health.drift {
                println!("\nDrift Status:");
                println!("  Drifted: {}", drift.is_drifted);
                println!("  Severity: {}", drift.severity);
                println!("  PSI Score: {:.2}", drift.psi_score);
            }
        }
    }

    Ok(())
}
