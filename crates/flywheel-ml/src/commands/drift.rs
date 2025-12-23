use clap::{Args, Subcommand};

use super::Context;

#[derive(Args)]
pub struct DriftArgs {
    #[command(subcommand)]
    pub command: DriftCommand,
}

#[derive(Subcommand)]
pub enum DriftCommand {
    #[command(about = "Show current drift status")]
    Status {
        #[arg(short, long)]
        pipeline: Option<String>,

        #[arg(short, long)]
        model: Option<String>,
    },
    #[command(about = "Show drift event history")]
    History {
        #[arg(short, long)]
        pipeline: String,

        #[arg(short, long)]
        model: Option<String>,

        #[arg(long, default_value = "10")]
        limit: i32,
    },
}

pub async fn run(ctx: &Context, args: DriftArgs) -> anyhow::Result<()> {
    let client = ctx.client().await?;

    match args.command {
        DriftCommand::Status { pipeline, model } => {
            let pipeline_id = pipeline.unwrap_or_default();
            let model_id = model.unwrap_or_default();

            let response = client.get_drift_status(&pipeline_id, &model_id).await?;

            println!("Drift Status");
            if !response.pipeline_id.is_empty() {
                println!("Pipeline: {}", response.pipeline_id);
            }
            if !response.model_id.is_empty() {
                println!("Model: {}", response.model_id);
            }
            println!("{}", "=".repeat(40));
            println!();
            println!("Overall: {}", if response.is_drifted { "DRIFT DETECTED" } else { "No drift detected" });
            println!("Type: {}", response.drift_type);
            println!("Severity: {}", response.severity);
            println!();

            if let Some(stats) = response.statistical {
                println!("Statistical Drift:");
                println!("  PSI Score: {:.4}", stats.psi_score);
                println!("  KL Divergence: {:.4}", stats.kl_divergence);
                println!();
            }

            if let Some(perf) = response.performance {
                println!("Performance Drift:");
                println!("  Accuracy: {:.2} (baseline: {:.2})", perf.accuracy, perf.accuracy_baseline);
                println!("  Accuracy Delta: {:.2}%", perf.accuracy_delta * 100.0);
                println!("  Latency P99: {}ms", perf.latency_p99_ms);
                println!();
            }
        }
        DriftCommand::History { pipeline, model, limit } => {
            let response = client.list_drift_events(&pipeline, model, limit).await?;

            println!("Drift History for pipeline: {} (last {})", pipeline, limit);
            println!("{}", "=".repeat(60));
            println!();
            println!(
                "{:<20}  {:<12}  {:<10}  {:<8}",
                "TIME", "TYPE", "SEVERITY", "RESOLVED"
            );
            println!("{}", "-".repeat(60));

            for event in &response.events {
                let timestamp = event
                    .detected_at
                    .as_ref()
                    .map(|ts| {
                        chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    })
                    .unwrap_or_else(|| "unknown".to_string());

                let resolved = event.resolved_at.is_some();

                println!(
                    "{:<20}  {:<12}  {:<10}  {:<8}",
                    timestamp,
                    event.drift_type,
                    event.severity,
                    if resolved { "yes" } else { "no" }
                );
            }

            if response.events.is_empty() {
                println!("No drift events recorded.");
            }
        }
    }

    Ok(())
}
