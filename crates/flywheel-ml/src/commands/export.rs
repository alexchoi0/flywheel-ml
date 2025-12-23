use clap::{Args, Subcommand};
use std::path::PathBuf;

use super::Context;

#[derive(Args)]
pub struct ExportArgs {
    #[command(subcommand)]
    pub resource: ExportResource,
}

#[derive(Subcommand)]
pub enum ExportResource {
    #[command(about = "Export training data")]
    Training {
        #[arg(short, long)]
        pipeline: String,

        #[arg(short, long)]
        output: PathBuf,

        #[arg(long)]
        since: Option<String>,

        #[arg(long)]
        until: Option<String>,

        #[arg(long, default_value = "parquet")]
        format: String,
    },
}

pub async fn run(ctx: &Context, args: ExportArgs) -> anyhow::Result<()> {
    let client = ctx.client().await?;

    match args.resource {
        ExportResource::Training {
            pipeline,
            output,
            since,
            until,
            format,
        } => {
            let response = client.get_pipeline(&pipeline).await?;

            if let Some(pipeline_info) = response.pipeline {
                println!("Export Training Data");
                println!("{}", "=".repeat(50));
                println!();
                println!("Pipeline: {} ({})", pipeline_info.name, pipeline_info.status);
                println!("Format: {}", format);
                println!("Output: {}", output.display());

                if let Some(ref since) = since {
                    println!("Since: {}", since);
                }
                if let Some(ref until) = until {
                    println!("Until: {}", until);
                }
                println!();

                if !output.exists() {
                    std::fs::create_dir_all(&output)?;
                    println!("Created output directory: {}", output.display());
                }

                println!("Note: Training data export runs on the server side.");
                println!("      Labeled examples from the feedback loop are");
                println!("      exported to the configured training storage.");
                println!();
                println!("To configure automatic exports, add 'training_export'");
                println!("to your pipeline manifest.");
            } else {
                println!("Pipeline not found: {}", pipeline);
            }
        }
    }

    Ok(())
}
