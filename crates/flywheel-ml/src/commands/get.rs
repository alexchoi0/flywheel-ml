use clap::{Args, Subcommand};

use super::Context;

#[derive(Args)]
pub struct GetArgs {
    #[command(subcommand)]
    pub resource: GetResource,
}

#[derive(Subcommand)]
pub enum GetResource {
    #[command(about = "List pipelines")]
    Pipelines,
    #[command(about = "List models")]
    Models,
}

pub async fn run(ctx: &Context, args: GetArgs) -> anyhow::Result<()> {
    match args.resource {
        GetResource::Pipelines => {
            println!("Listing pipelines in namespace: {}", ctx.namespace);
            println!("NAME\t\tSTATUS\t\tCREATED");
            println!("example-pipeline\trunning\t\t2024-01-15");
        }
        GetResource::Models => {
            println!("Listing models");
            println!("MODEL_ID\t\tVERSION\t\tSTATUS\t\tACCURACY");
            println!("isolation-forest\tv3\t\tactive\t\t0.92");
        }
    }

    Ok(())
}
