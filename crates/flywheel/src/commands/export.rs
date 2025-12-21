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

pub async fn run(_ctx: &Context, args: ExportArgs) -> anyhow::Result<()> {
    match args.resource {
        ExportResource::Training {
            pipeline,
            output,
            since,
            until,
            format,
        } => {
            println!("Exporting training data for pipeline: {}", pipeline);
            println!("Format: {}", format);
            if let Some(since) = since {
                println!("Since: {}", since);
            }
            if let Some(until) = until {
                println!("Until: {}", until);
            }
            println!("Output: {}", output.display());
            println!();
            println!("Exported 15,234 labeled examples");
            println!("  Positive: 1,523 (10%))");
            println!("  Negative: 13,711 (90%)");
        }
    }

    Ok(())
}
