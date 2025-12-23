use clap::{Args, Subcommand};
use std::path::PathBuf;

use super::Context;

#[derive(Args)]
pub struct PipelineArgs {
    #[command(subcommand)]
    pub command: PipelineCommand,
}

#[derive(Subcommand)]
pub enum PipelineCommand {
    #[command(about = "Apply a pipeline manifest")]
    Apply {
        #[arg(short, long)]
        file: PathBuf,
    },

    #[command(about = "List pipelines")]
    List,

    #[command(about = "Get pipeline details")]
    Get { pipeline_id: String },

    #[command(about = "Enable a pipeline (start execution)")]
    Enable { pipeline_id: String },

    #[command(about = "Disable a pipeline (stop execution)")]
    Disable { pipeline_id: String },

    #[command(about = "Delete a pipeline")]
    Delete { pipeline_id: String },
}

pub async fn run(ctx: &Context, args: PipelineArgs) -> anyhow::Result<()> {
    let client = ctx.client().await?;

    match args.command {
        PipelineCommand::Apply { file } => {
            let content = std::fs::read_to_string(&file)?;

            let manifest = flywheel_ml_dsl::parser::parse_manifest(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse manifest: {}", e))?;

            flywheel_ml_dsl::validation::validate_manifest(&manifest)
                .map_err(|e| anyhow::anyhow!("Validation failed: {}", e))?;

            let namespace = manifest
                .metadata
                .namespace
                .clone()
                .unwrap_or_else(|| ctx.namespace.clone());

            let response = client
                .create_pipeline(&manifest.metadata.name, &namespace, &content)
                .await?;

            println!("Pipeline created successfully");
            println!("  ID:        {}", response.pipeline_id);
            println!("  Name:      {}", response.name);
            println!("  Status:    {}", response.status);
            println!();
            println!("To start the pipeline:");
            println!("  flywheel-ml pipeline enable {}", response.pipeline_id);
        }

        PipelineCommand::List => {
            let response = client
                .list_pipelines(Some(ctx.namespace.clone()), 100, None)
                .await?;

            println!("Pipelines in namespace: {}", ctx.namespace);
            println!();
            println!(
                "{:<36}  {:<20}  {:<10}",
                "PIPELINE_ID", "NAME", "STATUS"
            );
            println!("{}", "-".repeat(70));

            for pipeline in &response.pipelines {
                println!(
                    "{:<36}  {:<20}  {:<10}",
                    pipeline.pipeline_id,
                    truncate(&pipeline.name, 20),
                    pipeline.status
                );
            }

            if response.pipelines.is_empty() {
                println!("No pipelines found.");
            }
        }

        PipelineCommand::Get { pipeline_id } => {
            let response = client.get_pipeline(&pipeline_id).await?;

            if let Some(pipeline) = response.pipeline {
                println!("Pipeline: {}", pipeline.name);
                println!("{}", "=".repeat(50));
                println!();
                println!("ID:         {}", pipeline.pipeline_id);
                println!("Namespace:  {}", pipeline.namespace);
                println!("Status:     {}", pipeline.status);

                if let Some(created) = pipeline.created_at {
                    let dt = chrono::DateTime::from_timestamp(created.seconds, created.nanos as u32)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    println!("Created:    {}", dt);
                }

                if let Some(stats) = pipeline.stats {
                    println!();
                    println!("Statistics:");
                    println!("  Records Processed: {}", stats.records_processed);
                    println!("  Predictions Made:  {}", stats.predictions_made);
                    println!("  Feedback Received: {}", stats.feedback_received);
                    println!("  Current Accuracy:  {:.2}", stats.current_accuracy);
                }
            } else {
                println!("Pipeline not found: {}", pipeline_id);
            }
        }

        PipelineCommand::Enable { pipeline_id } => {
            let response = client.enable_pipeline(&pipeline_id).await?;

            if response.success {
                println!("Pipeline enabled successfully");
                println!("  Status: {}", response.status);
                println!();
                println!("The execution engine will start processing shortly.");
            } else {
                println!("Failed to enable pipeline");
            }
        }

        PipelineCommand::Disable { pipeline_id } => {
            let response = client.disable_pipeline(&pipeline_id).await?;

            if response.success {
                println!("Pipeline disabled successfully");
                println!("  Status: {}", response.status);
            } else {
                println!("Failed to disable pipeline");
            }
        }

        PipelineCommand::Delete { pipeline_id } => {
            let response = client.delete_pipeline(&pipeline_id).await?;

            if response.success {
                println!("Pipeline deleted successfully");
            } else {
                println!("Failed to delete pipeline (may not exist)");
            }
        }
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
