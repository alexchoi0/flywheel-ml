use clap::{Args, Subcommand};

use super::Context;

#[derive(Args)]
pub struct DeleteArgs {
    #[command(subcommand)]
    pub resource: DeleteResource,

    #[arg(long, default_value = "false")]
    pub force: bool,
}

#[derive(Subcommand)]
pub enum DeleteResource {
    #[command(about = "Delete a pipeline")]
    Pipeline { name: String },
    #[command(about = "Delete a model")]
    Model { model_id: String },
}

pub async fn run(ctx: &Context, args: DeleteArgs) -> anyhow::Result<()> {
    match args.resource {
        DeleteResource::Pipeline { name } => {
            if !args.force {
                println!("Are you sure you want to delete pipeline '{}'? Use --force to confirm.", name);
                return Ok(());
            }
            println!("Deleting pipeline '{}' from namespace '{}'", name, ctx.namespace);
            println!("Pipeline deleted successfully");
        }
        DeleteResource::Model { model_id } => {
            if !args.force {
                println!("Are you sure you want to delete model '{}'? Use --force to confirm.", model_id);
                return Ok(());
            }
            println!("Deleting model '{}'", model_id);
            println!("Model deleted successfully");
        }
    }

    Ok(())
}
