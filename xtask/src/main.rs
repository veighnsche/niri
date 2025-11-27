//! xtask for niri development workflows.
//!
//! ## Available Tasks
//!
//! - `golden-sync` - Manage golden snapshots for regression testing
//! - `test-all` - Run all tests with cleanup
//!
//! ## Future Tasks (planned)
//!
//! - `golden-stepper` - TUI to guide users through golden test scenarios

use anyhow::Result;
use clap::{Parser, Subcommand};

mod golden_sync;
mod test_all;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development tasks for niri")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage golden snapshots for regression testing
    GoldenSync {
        #[command(subcommand)]
        action: golden_sync::Action,
    },
    /// Run all tests with cleanup
    TestAll {
        #[command(subcommand)]
        action: test_all::Action,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GoldenSync { action } => golden_sync::run(action),
        Commands::TestAll { action } => test_all::run(action),
    }
}
