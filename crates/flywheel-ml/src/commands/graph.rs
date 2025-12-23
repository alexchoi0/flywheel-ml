use clap::{Args, Subcommand};
use std::path::PathBuf;

use super::Context;

#[derive(Args)]
pub struct GraphArgs {
    #[command(subcommand)]
    pub resource: GraphResource,

    #[arg(short, long)]
    pub output: Option<PathBuf>,

    #[arg(long, default_value = "ascii")]
    pub format: String,
}

#[derive(Subcommand)]
pub enum GraphResource {
    #[command(about = "Visualize pipeline DAG")]
    Pipeline { pipeline_id: String },
}

pub async fn run(ctx: &Context, args: GraphArgs) -> anyhow::Result<()> {
    let client = ctx.client().await?;

    match args.resource {
        GraphResource::Pipeline { pipeline_id } => {
            let response = client.get_pipeline(&pipeline_id).await?;

            if let Some(pipeline) = response.pipeline {
                println!("Pipeline DAG: {}", pipeline.name);
                println!("{}", "=".repeat(40));
                println!();

                if let Ok(manifest) = flywheel_ml_dsl::parser::parse_manifest(&pipeline.spec_yaml) {
                    print_source(&manifest.spec.source);

                    for (i, stage) in manifest.spec.stages.iter().enumerate() {
                        print_connector();
                        let stage_type_str = stage.stage_type.as_str();
                        print_stage(&stage.id, stage_type_str, i == manifest.spec.stages.len() - 1);
                    }

                    if !manifest.spec.sinks.is_empty() {
                        print_connector();
                        print_sinks(&manifest.spec.sinks);
                    }

                    if manifest.spec.feedback.is_some() || manifest.spec.training_export.is_some() {
                        println!();
                        println!("Additional Components:");
                        if manifest.spec.feedback.is_some() {
                            println!("  [Feedback Loop] enabled");
                        }
                        if manifest.spec.training_export.is_some() {
                            println!("  [Training Export] enabled");
                        }
                    }
                } else {
                    println!("  (Unable to parse pipeline spec)");
                    println!();
                    println!("  ┌─────────────┐");
                    println!("  │  Pipeline   │");
                    println!("  │   {}   │", truncate(&pipeline.name, 8));
                    println!("  └─────────────┘");
                }

                if let Some(output) = args.output {
                    std::fs::write(&output, format!("Pipeline: {}\nStatus: {}", pipeline.name, pipeline.status))?;
                    println!();
                    println!("Saved to: {}", output.display());
                }
            } else {
                println!("Pipeline not found: {}", pipeline_id);
            }
        }
    }

    Ok(())
}

fn print_source(source: &str) {
    println!("  ┌─────────────────┐");
    println!("  │     Source      │");
    println!("  │ {:^15} │", truncate(source, 15));
    println!("  └────────┬────────┘");
}

fn print_connector() {
    println!("           │");
    println!("           ▼");
}

fn print_stage(name: &str, stage_type: &str, _is_last: bool) {
    println!("  ┌─────────────────┐");
    println!("  │ {:^15} │", truncate(name, 15));
    println!("  │ ({:^13}) │", truncate(stage_type, 13));
    println!("  └────────┬────────┘");
}

fn print_sinks(sinks: &[flywheel_ml_dsl::SinkSpec]) {
    if sinks.len() == 1 {
        println!("  ┌─────────────────┐");
        println!("  │      Sink       │");
        println!("  │ {:^15} │", truncate(&sinks[0].name, 15));
        println!("  └─────────────────┘");
    } else {
        let width = sinks.len() * 12;
        println!("  {:^width$}", "┌".to_string() + &"─".repeat(width - 2) + "┐", width = width);
        for _ in sinks {
            print!("  ┌─────────┐");
        }
        println!();
        for sink in sinks {
            print!("  │ {:^7} │", truncate(&sink.name, 7));
        }
        println!();
        for _ in sinks {
            print!("  └─────────┘");
        }
        println!();
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
