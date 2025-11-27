//! Run all tests with proper cleanup.
//!
//! This module provides a comprehensive test runner that:
//! - Cleans up stale `.snap.new` files before testing
//! - Runs all tests (or specific test filters)
//! - Reports results clearly
//! - Optionally cleans up after failed tests
//! - Verifies golden snapshots for regression testing
//!
//! ## Commands
//!
//! - `run` - Run all tests with cleanup
//! - `golden` - Verify golden snapshots (MANDATORY before touching layout code)
//! - `clean` - Clean up all test artifacts
//! - `status` - Show test status and artifact counts
//!
//! ## ⚠️ AI AGENTS: READ THIS ⚠️
//!
//! Golden snapshots come from the `golden-snapshots` branch, NOT this branch.
//! NEVER run `cargo insta accept` on golden tests!
//!
//! If tests fail:
//!   1. FIX YOUR CODE — your refactor changed behavior
//!   2. Run `cargo xtask test-all clean` to remove .snap.new files
//!   3. DO NOT accept .snap.new files — they contain your broken output

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use anyhow::{bail, Context, Result};
use clap::Subcommand;

/// Paths that may contain snapshot files
const SNAPSHOT_PATHS: &[&str] = &[
    "src/layout/tests/snapshots",
];

/// Primary snapshot directory
const SNAPSHOT_DIR: &str = "src/layout/tests/snapshots";

#[derive(Subcommand)]
pub enum Action {
    /// Run all tests with cleanup
    Run {
        /// Only run tests matching this filter
        #[arg(long, short)]
        filter: Option<String>,

        /// Don't clean up .snap.new files before running
        #[arg(long)]
        no_pre_clean: bool,

        /// Clean up .snap.new files after test failures
        #[arg(long)]
        post_clean: bool,

        /// Run tests in release mode
        #[arg(long)]
        release: bool,

        /// Show verbose output
        #[arg(long, short)]
        verbose: bool,
    },
    /// Verify golden snapshots - MANDATORY before touching layout code!
    ///
    /// This runs only the golden snapshot tests to verify your changes
    /// haven't introduced behavioral regressions.
    ///
    /// ⚠️ NEVER run 'cargo insta accept' on golden tests!
    Golden,
    /// Clean up all test artifacts (.snap.new files)
    Clean {
        /// Show what would be cleaned without actually cleaning
        #[arg(long)]
        dry_run: bool,
    },
    /// Show test status and artifact counts
    Status,
}

pub fn run(action: Action) -> Result<()> {
    match action {
        Action::Run {
            filter,
            no_pre_clean,
            post_clean,
            release,
            verbose,
        } => run_tests(filter, no_pre_clean, post_clean, release, verbose),
        Action::Golden => verify_golden(),
        Action::Clean { dry_run } => clean_artifacts(dry_run),
        Action::Status => show_status(),
    }
}

// =============================================================================
// Golden snapshot verification
// =============================================================================

fn verify_golden() -> Result<()> {
    println!("=== Golden Snapshot Verification ===\n");

    // Check: Snapshot directory exists
    if !Path::new(SNAPSHOT_DIR).exists() {
        bail!(
            "Snapshot directory not found: {SNAPSHOT_DIR}\n\n\
             Run: cargo xtask golden-sync pull"
        );
    }
    println!("✓ Snapshot directory exists");

    // Check for .snap.new files (regression evidence)
    let new_count = count_snap_new_files();
    if new_count > 0 {
        println!("⚠ Found {new_count} .snap.new files (regression evidence)");
        println!("  Run: cargo xtask test-all clean");
    }

    // Run golden tests
    println!("\nRunning golden tests...");
    
    let output = Command::new("cargo")
        .args(["test", "--lib", "--", "golden"])
        .output()
        .context("Failed to run cargo test")?;

    // Show last few lines of output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Print test output
    for line in stdout.lines().rev().take(10).collect::<Vec<_>>().into_iter().rev() {
        println!("{line}");
    }

    if output.status.success() {
        println!("\n✓ Golden tests pass");
        println!("\n=== All verifications passed ===");
        Ok(())
    } else {
        // Print any stderr
        if !stderr.is_empty() {
            eprintln!("{stderr}");
        }

        println!("\n═══════════════════════════════════════════════════════════");
        println!("  DO NOT run 'cargo insta accept' — this corrupts the baseline!");
        println!("═══════════════════════════════════════════════════════════");
        println!();
        println!("Your refactor changed behavior. To fix:");
        println!("  1. Run 'cargo insta review' to see what changed");
        println!("  2. Fix your code until tests pass");
        println!("  3. Run 'cargo xtask test-all clean' to remove .snap.new files");
        println!();
        println!("Golden snapshots come from: golden-snapshots branch");
        println!("Sync command: cargo xtask golden-sync pull");
        
        std::process::exit(1);
    }
}

// =============================================================================
// Test runner
// =============================================================================

fn run_tests(
    filter: Option<String>,
    no_pre_clean: bool,
    post_clean: bool,
    release: bool,
    verbose: bool,
) -> Result<()> {
    println!("🧪 Running all tests\n");

    // Pre-clean unless disabled
    if !no_pre_clean {
        let cleaned = clean_snap_new_files()?;
        if cleaned > 0 {
            println!("🧹 Cleaned {cleaned} stale .snap.new files\n");
        }
    }

    // Build test command
    let mut args = vec!["test", "--lib"];
    
    if release {
        args.push("--release");
    }

    // Add filter if provided
    let filter_str;
    if let Some(ref f) = filter {
        filter_str = f.clone();
        args.push("--");
        args.push(&filter_str);
    }

    println!("📦 Running: cargo {}\n", args.join(" "));

    let start = Instant::now();

    // Run tests
    let mut cmd = Command::new("cargo");
    cmd.args(&args);

    if !verbose {
        // Capture output for summary
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
    }

    let status = cmd.status().context("Failed to run cargo test")?;
    let duration = start.elapsed();

    println!();

    if status.success() {
        println!("✅ All tests passed in {:.2}s", duration.as_secs_f64());
    } else {
        println!("❌ Some tests failed (took {:.2}s)", duration.as_secs_f64());

        // Count new snapshot files
        let new_count = count_snap_new_files();
        if new_count > 0 {
            println!("\n📝 Found {new_count} .snap.new files (test output diffs)");
            println!("   These show what your code produced vs expected.");
            
            if post_clean {
                println!("\n🧹 Cleaning up .snap.new files (--post-clean)...");
                let cleaned = clean_snap_new_files()?;
                println!("   Removed {cleaned} files");
            } else {
                println!("\n   To clean up: cargo xtask test-all clean");
                println!("   To view diffs: cargo insta review");
            }
        }

        // Return error to indicate failure
        std::process::exit(1);
    }

    Ok(())
}

// =============================================================================
// Cleanup
// =============================================================================

fn clean_artifacts(dry_run: bool) -> Result<()> {
    println!("🧹 Cleaning test artifacts\n");

    let new_files = find_snap_new_files();

    if new_files.is_empty() {
        println!("✅ No .snap.new files found");
        return Ok(());
    }

    println!("Found {} .snap.new files:\n", new_files.len());

    for path in &new_files {
        if dry_run {
            println!("  [dry-run] Would remove: {path}");
        } else {
            println!("  🗑️  {path}");
            std::fs::remove_file(path).context(format!("Failed to remove {path}"))?;
        }
    }

    if dry_run {
        println!("\n[DRY RUN] Would have removed {} files", new_files.len());
    } else {
        println!("\n✅ Removed {} .snap.new files", new_files.len());
    }

    Ok(())
}

fn show_status() -> Result<()> {
    println!("📊 Test Artifact Status\n");

    // Count snapshot files
    let mut total_snaps = 0;
    let mut total_new = 0;

    for path in SNAPSHOT_PATHS {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                
                if name_str.ends_with(".snap.new") {
                    total_new += 1;
                } else if name_str.ends_with(".snap") {
                    total_snaps += 1;
                }
            }
        }
    }

    println!("📁 Golden snapshots: {total_snaps} files");
    println!("📝 Pending .snap.new: {total_new} files");

    if total_new > 0 {
        println!("\n⚠️  You have {total_new} .snap.new files!");
        println!("   These are test outputs that differ from golden snapshots.");
        println!();
        println!("   Options:");
        println!("   • Fix your code to match expected output");
        println!("   • View diffs: cargo insta review");
        println!("   • Clean up: cargo xtask test-all clean");
        println!();
        println!("   ⚠️  NEVER run 'cargo insta accept' on golden tests!");
    } else {
        println!("\n✅ No pending test artifacts");
    }

    Ok(())
}

// =============================================================================
// Helpers
// =============================================================================

fn find_snap_new_files() -> Vec<String> {
    let mut files = Vec::new();

    for path in SNAPSHOT_PATHS {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if entry_path.to_string_lossy().ends_with(".snap.new") {
                    files.push(entry_path.to_string_lossy().to_string());
                }
            }
        }
    }

    files.sort();
    files
}

fn count_snap_new_files() -> usize {
    find_snap_new_files().len()
}

fn clean_snap_new_files() -> Result<usize> {
    let files = find_snap_new_files();
    let count = files.len();

    for path in files {
        std::fs::remove_file(&path).context(format!("Failed to remove {path}"))?;
    }

    Ok(count)
}
