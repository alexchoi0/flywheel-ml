use clap::Args;
use std::path::PathBuf;

use super::Context;

#[derive(Args)]
pub struct ApplyArgs {
    #[arg(short, long)]
    pub file: PathBuf,

    #[arg(short, long, default_value = "false")]
    pub recursive: bool,

    #[arg(long, default_value = "false")]
    pub dry_run: bool,
}

pub async fn run(ctx: &Context, args: ApplyArgs) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(&args.file)?;

    if args.dry_run {
        println!("Dry run - would apply:");
        println!("{}", content);
        return Ok(());
    }

    println!(
        "Applying {} to namespace {} on server {}",
        args.file.display(),
        ctx.namespace,
        ctx.server
    );

    // TODO: Connect to server and apply manifest
    println!("Pipeline applied successfully");

    Ok(())
}
