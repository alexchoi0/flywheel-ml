use clap::{Args, Subcommand};

use super::Context;

#[derive(Args)]
pub struct ModelArgs {
    #[command(subcommand)]
    pub command: ModelCommand,
}

#[derive(Subcommand)]
pub enum ModelCommand {
    #[command(about = "List registered models")]
    List,

    #[command(about = "Show model details and metrics")]
    Show { model_id: String },

    #[command(about = "Show model performance history")]
    History {
        model_id: String,

        #[arg(long, default_value = "7d")]
        since: String,
    },

    #[command(about = "Compare two model versions")]
    Compare {
        model_a: String,
        model_b: String,
    },
}

pub async fn run(ctx: &Context, args: ModelArgs) -> anyhow::Result<()> {
    let client = ctx.client().await?;

    match args.command {
        ModelCommand::List => {
            let response = client.list_models(100, None).await?;

            println!("Models in namespace: {}", ctx.namespace);
            println!();
            println!(
                "{:<26}  {:<8}  {:<8}  {:<8}  {:<11}",
                "MODEL_ID", "VERSION", "STATUS", "ACCURACY", "LATENCY_P99"
            );
            println!("{}", "-".repeat(70));

            for model in &response.models {
                println!(
                    "{:<26}  {:<8}  {:<8}  {:<8.2}  {}ms",
                    truncate(&model.model_id, 26),
                    truncate(&model.version, 8),
                    truncate(&model.status, 8),
                    model.accuracy,
                    model.latency_p99_ms
                );
            }

            if response.models.is_empty() {
                println!("No models registered.");
            }
        }
        ModelCommand::Show { model_id } => {
            let response = client.get_model(&model_id).await?;

            if let Some(model) = response.model {
                println!("Model: {}", model.model_id);
                println!("{}", "=".repeat(40));
                println!();
                println!("Version:     {}", model.version);
                println!("Status:      {}", model.status);
                println!("Endpoint:    {}", model.endpoint);
                println!("Type:        {}", model.model_type);
                println!();
                println!("Performance:");
                println!("  Accuracy:     {:.2}", model.accuracy);
                println!("  Latency P99:  {}ms", model.latency_p99_ms);
            } else {
                println!("Model not found: {}", model_id);
            }
        }
        ModelCommand::History { model_id, since } => {
            println!("Performance history for {} (since {})", model_id, since);
            println!();
            println!("Note: Historical data not yet implemented in database.");
            println!("This feature will be available in a future release.");
        }
        ModelCommand::Compare { model_a, model_b } => {
            let response_a = client.get_model(&model_a).await;
            let response_b = client.get_model(&model_b).await;

            match (response_a, response_b) {
                (Ok(a), Ok(b)) => {
                    let ma = a.model;
                    let mb = b.model;

                    if ma.is_none() || mb.is_none() {
                        println!("One or both models not found.");
                        return Ok(());
                    }

                    let ma = ma.unwrap();
                    let mb = mb.unwrap();

                    println!("Model Comparison: {} vs {}", model_a, model_b);
                    println!();
                    println!("{:<14}  {:<12}  {:<12}", "METRIC", model_a, model_b);
                    println!("{}", "-".repeat(42));
                    println!("{:<14}  {:<12.2}  {:<12.2}", "Accuracy", ma.accuracy, mb.accuracy);
                    println!(
                        "{:<14}  {:<12}  {:<12}",
                        "Latency P99",
                        format!("{}ms", ma.latency_p99_ms),
                        format!("{}ms", mb.latency_p99_ms)
                    );
                    println!("{:<14}  {:<12}  {:<12}", "Status", ma.status, mb.status);
                    println!();

                    if ma.accuracy > mb.accuracy {
                        println!("Recommendation: {} performs better on accuracy", model_a);
                    } else if mb.accuracy > ma.accuracy {
                        println!("Recommendation: {} performs better on accuracy", model_b);
                    } else {
                        println!("Both models have similar performance.");
                    }
                }
                _ => {
                    println!("Failed to fetch model data for comparison.");
                }
            }
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
