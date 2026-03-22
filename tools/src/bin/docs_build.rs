//! Generates all MagicMerlin documentation pages from the docs index.
//!
//! Usage:
//!     cargo run -p magicmerlin-tools --bin docs_build [-- /path/to/project]

use std::path::PathBuf;

fn main() {
    let project_root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // Walk up from the binary to find the project root (contains Cargo.toml workspace)
            let manifest = std::env::var("CARGO_MANIFEST_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| std::env::current_dir().expect("cannot get cwd"));
            // If we're in tools/, go up one level
            if manifest.ends_with("tools") {
                manifest.parent().unwrap().to_path_buf()
            } else {
                manifest
            }
        });

    eprintln!("Project root: {}", project_root.display());

    match magicmerlin_tools::docs_builder::generate_docs(&project_root) {
        Ok(report) => {
            eprintln!("{report}");
            if !report.errors.is_empty() {
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Fatal: {e:#}");
            std::process::exit(2);
        }
    }
}
