mod generator;
mod templates;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "docgen", about = "Generate Magic Merlin documentation site")]
struct Args {
    /// Output directory for generated docs
    #[arg(long, default_value = "docs")]
    out: PathBuf,

    /// Path to the OpenClaw docs index JSON
    #[arg(long, default_value = "parity/openclaw_docs_index.json")]
    index: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let stats = generator::generate_all(&args.index, &args.out)?;
    println!(
        "Generated {} pages to {}",
        stats.total_pages,
        args.out.display()
    );
    for (section, count) in &stats.sections {
        println!("  {}: {} pages", section, count);
    }
    Ok(())
}
