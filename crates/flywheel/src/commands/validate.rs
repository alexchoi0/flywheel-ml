use clap::Args;
use std::path::PathBuf;

use super::Context;

#[derive(Args)]
pub struct ValidateArgs {
    #[arg(short, long)]
    pub file: PathBuf,

    #[arg(long, default_value = "false")]
    pub strict: bool,
}

pub async fn run(_ctx: &Context, args: ValidateArgs) -> anyhow::Result<()> {
    println!("Validating: {}", args.file.display());
    println!();

    let content = std::fs::read_to_string(&args.file)?;

    // Parse YAML
    match flywheel_dsl::parser::parse_manifest(&content) {
        Ok(manifest) => {
            // Validate
            match flywheel_dsl::validation::validate_manifest(&manifest) {
                Ok(()) => {
                    println!("✓ Valid FlywheelPipeline manifest");
                    println!();
                    println!("Pipeline: {}", manifest.metadata.name);
                    println!("Namespace: {}", manifest.metadata.namespace.unwrap_or_default());
                    println!("Stages: {}", manifest.spec.stages.len());
                    println!("Sinks: {}", manifest.spec.sinks.len());

                    if manifest.spec.feedback.is_some() {
                        println!("Feedback: configured");
                    }
                    if manifest.spec.training_export.is_some() {
                        println!("Training Export: configured");
                    }

                    println!();
                    println!("Ready for: kubectl apply -f {}", args.file.display());
                }
                Err(e) => {
                    println!("✗ Validation error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            println!("✗ Parse error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
