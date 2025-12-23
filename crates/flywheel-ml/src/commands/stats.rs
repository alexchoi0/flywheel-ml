use clap::{Args, Subcommand};

use super::Context;

#[derive(Args)]
pub struct StatsArgs {
    #[command(subcommand)]
    pub command: Option<StatsCommand>,

    #[arg(short, long)]
    pub pipeline: Option<String>,
}

#[derive(Subcommand)]
pub enum StatsCommand {
    #[command(about = "Show prediction statistics")]
    Predictions {
        #[arg(short, long)]
        pipeline: String,
    },

    #[command(about = "Show feedback statistics")]
    Feedback {
        #[arg(short, long)]
        pipeline: String,
    },

    #[command(about = "Show training data statistics")]
    Training {
        #[arg(short, long)]
        pipeline: String,
    },
}

pub async fn run(ctx: &Context, args: StatsArgs) -> anyhow::Result<()> {
    let client = ctx.client().await?;

    match args.command {
        None => {
            let pipelines = client.list_pipelines(Some(ctx.namespace.clone()), 100, None).await?;

            println!("Pipeline Statistics");
            println!("{}", "=".repeat(70));
            println!();
            println!(
                "{:<26}  {:<10}  {:<10}  {:<12}",
                "PIPELINE", "RECORDS/s", "PREDS/s", "ERROR_RATE"
            );
            println!("{}", "-".repeat(70));

            for pipeline in &pipelines.pipelines {
                let health = client.get_pipeline_health(&pipeline.pipeline_id).await;

                if let Ok(health_resp) = health {
                    if let Some(metrics) = health_resp.metrics {
                        println!(
                            "{:<26}  {:<10.0}  {:<10.0}  {:<.2}%",
                            truncate(&pipeline.name, 26),
                            metrics.records_per_second,
                            metrics.predictions_per_second,
                            metrics.error_rate * 100.0
                        );
                    } else {
                        println!(
                            "{:<26}  {:<10}  {:<10}  {:<12}",
                            truncate(&pipeline.name, 26),
                            "-", "-", "-"
                        );
                    }
                }
            }

            if pipelines.pipelines.is_empty() {
                println!("No pipelines found.");
            }
        }
        Some(StatsCommand::Predictions { pipeline }) => {
            let health = client.get_pipeline_health(&pipeline).await?;

            println!("Prediction Statistics: {}", pipeline);
            println!("{}", "=".repeat(50));
            println!();

            if let Some(metrics) = health.metrics {
                println!("Current Metrics:");
                println!("  Predictions/sec:  {:.0}", metrics.predictions_per_second);
                println!("  Avg Latency:      {}ms", metrics.avg_latency_ms);
                println!("  P99 Latency:      {}ms", metrics.p99_latency_ms);
                println!("  Current Accuracy: {:.2}", metrics.current_accuracy);
            } else {
                println!("No prediction metrics available.");
            }
        }
        Some(StatsCommand::Feedback { pipeline }) => {
            let health = client.get_pipeline_health(&pipeline).await?;

            println!("Feedback Statistics: {}", pipeline);
            println!("{}", "=".repeat(50));
            println!();

            if let Some(metrics) = health.metrics {
                println!("Current Metrics:");
                println!("  Records/sec:      {:.0}", metrics.records_per_second);
                println!("  Error Rate:       {:.2}%", metrics.error_rate * 100.0);
                println!();
                println!("Note: Detailed feedback statistics require");
                println!("      the feedback collection to be enabled.");
            } else {
                println!("No feedback metrics available.");
            }
        }
        Some(StatsCommand::Training { pipeline }) => {
            println!("Training Data Statistics: {}", pipeline);
            println!("{}", "=".repeat(50));
            println!();
            println!("Note: Training data statistics are available after");
            println!("      running an export. Use 'flywheel-ml export training'");
            println!("      to export labeled examples.");
        }
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}
