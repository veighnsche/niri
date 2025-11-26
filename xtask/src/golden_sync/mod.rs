//! Golden snapshot synchronization.
//!
//! This module manages golden snapshots for regression testing.
//! Golden snapshots capture the MAIN BRANCH behavior — refactor branches
//! compare their output against these to detect regressions.
//!
//! ## Commands
//!
//! - `generate` - Generate fresh snapshots from golden branch
//! - `pull` - Pull existing snapshots from golden branch
//! - `status` - Show sync status
//! - `clean` - Remove .snap.new files (regression evidence)
//!
//! ## ⚠️ IMPORTANT: Never Accept .snap.new Files!
//!
//! When tests fail, insta creates `.snap.new` files. These contain your
//! (potentially broken) code's output — NOT the golden baseline.
//!
//! **NEVER run `cargo insta accept` on golden tests!**
//!
//! If tests fail, fix your code until they pass.

use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use clap::Subcommand;

pub const GOLDEN_BRANCH: &str = "golden-snapshots";
pub const SNAPSHOTS_PATH: &str = "src/layout/tests/snapshots";

#[derive(Subcommand)]
pub enum Action {
    /// Generate fresh golden snapshots from golden-snapshots branch
    ///
    /// This will:
    /// 1. Stash your current changes
    /// 2. Checkout golden-snapshots branch
    /// 3. Run tests to generate snapshots
    /// 4. Copy snapshots to temp directory
    /// 5. Return to your branch and pop stash
    /// 6. Copy snapshots to your working directory
    Generate {
        /// Don't actually do anything, just show what would be done
        #[arg(long)]
        dry_run: bool,
    },
    /// Pull existing golden snapshots from golden-snapshots branch (no regeneration)
    Pull {
        /// Don't actually copy files, just show what would be done
        #[arg(long)]
        dry_run: bool,
    },
    /// Show status of golden snapshot sync
    Status,
    /// Clean up .snap.new files (failed test outputs)
    ///
    /// When tests fail, insta creates .snap.new files with your code's output.
    /// These are NOT golden snapshots — they're evidence of regressions.
    /// This command removes them.
    Clean,
}

pub fn run(action: Action) -> Result<()> {
    match action {
        Action::Generate { dry_run } => generate(dry_run),
        Action::Pull { dry_run } => pull(dry_run),
        Action::Status => status(),
        Action::Clean => clean(),
    }
}

// =============================================================================
// Git helpers
// =============================================================================

fn run_git(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context(format!("Failed to run: git {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_git_ok(args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run_cargo(args: &[&str]) -> Result<()> {
    let status = Command::new("cargo")
        .args(args)
        .status()
        .context(format!("Failed to run: cargo {}", args.join(" ")))?;

    if !status.success() {
        bail!("cargo {} failed", args.join(" "));
    }

    Ok(())
}

// =============================================================================
// File helpers
// =============================================================================

fn count_new_files() -> usize {
    std::fs::read_dir(SNAPSHOTS_PATH)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().to_string_lossy().ends_with(".snap.new"))
                .count()
        })
        .unwrap_or(0)
}

fn clean_new_files() -> usize {
    let mut removed = 0;
    if let Ok(entries) = std::fs::read_dir(SNAPSHOTS_PATH) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.to_string_lossy().ends_with(".snap.new") {
                if std::fs::remove_file(&path).is_ok() {
                    removed += 1;
                }
            }
        }
    }
    removed
}

fn copy_snapshots(from: &str, to: &str) -> Result<usize> {
    let mut count = 0;

    for entry in std::fs::read_dir(from).context(format!("Failed to read {from}"))? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "snap") {
            let filename = path.file_name().unwrap();
            let dest = Path::new(to).join(filename);
            std::fs::copy(&path, &dest).context(format!("Failed to copy {:?}", path))?;
            count += 1;
        }
    }

    Ok(count)
}

// =============================================================================
// Commands
// =============================================================================

fn generate(dry_run: bool) -> Result<()> {
    println!("🔄 Generating golden snapshots from '{GOLDEN_BRANCH}' branch\n");

    // Get current branch
    let current_branch = run_git(&["rev-parse", "--abbrev-ref", "HEAD"])?;
    println!("📍 Current branch: {current_branch}");

    if current_branch == GOLDEN_BRANCH {
        bail!("Already on '{GOLDEN_BRANCH}' branch. Run from your working branch.");
    }

    // Check if golden branch exists
    if !run_git_ok(&["rev-parse", "--verify", GOLDEN_BRANCH]) {
        bail!(
            "Branch '{GOLDEN_BRANCH}' not found.\n\
             \n\
             To create it:\n\
             git checkout -b {GOLDEN_BRANCH} main\n\
             # Add snapshot infrastructure\n\
             git push -u origin {GOLDEN_BRANCH}"
        );
    }

    if dry_run {
        println!("\n[DRY RUN] Would perform these steps:");
        println!("  1. git stash");
        println!("  2. git checkout {GOLDEN_BRANCH}");
        println!("  3. cargo insta test --accept -- golden");
        println!("  4. Copy snapshots to /tmp/golden-snapshots/");
        println!("  5. git checkout {current_branch}");
        println!("  6. git stash pop");
        println!("  7. Copy snapshots to {SNAPSHOTS_PATH}/");
        return Ok(());
    }

    // Check for uncommitted changes
    let has_changes = !run_git(&["status", "--porcelain"])?.is_empty();

    // Stash if needed
    if has_changes {
        println!("\n📦 Stashing uncommitted changes...");
        run_git(&["stash", "push", "-m", "xtask golden-sync: temporary stash"])?;
    }

    // Create temp directory for snapshots
    let temp_dir = "/tmp/golden-snapshots";
    if Path::new(temp_dir).exists() {
        std::fs::remove_dir_all(temp_dir).context("Failed to clean temp directory")?;
    }
    std::fs::create_dir_all(temp_dir).context("Failed to create temp directory")?;

    // Checkout golden branch
    println!("\n🔀 Switching to '{GOLDEN_BRANCH}' branch...");
    let checkout_result = run_git(&["checkout", GOLDEN_BRANCH]);

    if let Err(e) = checkout_result {
        // Restore stash if checkout failed
        if has_changes {
            let _ = run_git(&["stash", "pop"]);
        }
        return Err(e);
    }

    // Generate snapshots
    println!("\n🧪 Generating golden snapshots...");
    let test_result = run_cargo(&["insta", "test", "--accept", "--", "golden"]);

    // Copy snapshots to temp before switching back
    let copy_result = if test_result.is_ok() {
        println!("\n📋 Copying snapshots to temp directory...");
        copy_snapshots(SNAPSHOTS_PATH, temp_dir)
    } else {
        Err(anyhow::anyhow!("Test generation failed"))
    };

    // Always switch back to original branch
    println!("\n🔀 Switching back to '{current_branch}'...");
    let _ = run_git(&["checkout", &current_branch]);

    // Pop stash if we stashed
    if has_changes {
        println!("\n📦 Restoring stashed changes...");
        let _ = run_git(&["stash", "pop"]);
    }

    // Check if we succeeded
    copy_result?;
    test_result?;

    // Copy from temp to working directory
    println!("\n📋 Copying golden snapshots to working directory...");
    std::fs::create_dir_all(SNAPSHOTS_PATH).context("Failed to create snapshots directory")?;
    let copied = copy_snapshots(temp_dir, SNAPSHOTS_PATH)?;

    println!("\n✅ Successfully generated {copied} golden snapshots!");
    println!("\nNext steps:");
    println!("  1. Run: cargo test --lib golden");
    println!("  2. If tests fail → your refactor changed behavior!");
    println!("  3. Fix regressions or document intentional changes");

    Ok(())
}

fn pull(dry_run: bool) -> Result<()> {
    println!("🔄 Pulling golden snapshots from '{GOLDEN_BRANCH}' branch...\n");

    // Check if golden-snapshots branch exists
    if !run_git_ok(&["rev-parse", "--verify", GOLDEN_BRANCH]) {
        bail!(
            "Branch '{GOLDEN_BRANCH}' not found.\n\
             \n\
             To set up golden snapshots:\n\
             1. Fetch from remote: git fetch origin {GOLDEN_BRANCH}\n\
             2. Or run: cargo xtask golden-sync generate"
        );
    }

    // Get list of snapshot files from golden branch
    let files_output = run_git(&["ls-tree", "-r", "--name-only", GOLDEN_BRANCH, SNAPSHOTS_PATH])?;

    let files: Vec<&str> = files_output
        .lines()
        .filter(|l| l.ends_with(".snap"))
        .collect();

    if files.is_empty() {
        bail!(
            "No snapshot files found in '{GOLDEN_BRANCH}:{SNAPSHOTS_PATH}'\n\
             \n\
             Run 'cargo xtask golden-sync generate' to create them."
        );
    }

    println!("Found {} golden snapshot files", files.len());

    if dry_run {
        println!("\n[DRY RUN] Would copy these files:");
        for file in &files {
            println!("  {file}");
        }
        return Ok(());
    }

    // Create snapshots directory if it doesn't exist
    std::fs::create_dir_all(SNAPSHOTS_PATH).context("Failed to create snapshots directory")?;

    // Copy each file from golden branch
    let mut copied = 0;
    for file in &files {
        let content = run_git(&["show", &format!("{GOLDEN_BRANCH}:{file}")])?;
        std::fs::write(file, content).context(format!("Failed to write {file}"))?;
        copied += 1;
    }

    println!("\n✅ Copied {copied} golden snapshot files");

    // Clean up any .new files
    let new_count = count_new_files();
    if new_count > 0 {
        println!("\n🧹 Cleaning up {new_count} .snap.new files (old regression evidence)...");
        let _ = clean_new_files();
    }

    println!("\nNext steps:");
    println!("  1. Run: cargo test --lib golden");
    println!("  2. If tests fail → your refactor changed behavior!");
    println!("  3. Fix regressions or document intentional changes");

    Ok(())
}

fn status() -> Result<()> {
    println!("📊 Golden Snapshot Status\n");

    // Get current branch
    let current_branch = run_git(&["rev-parse", "--abbrev-ref", "HEAD"])?;
    println!("📍 Current branch: {current_branch}");

    // Check if golden branch exists
    if !run_git_ok(&["rev-parse", "--verify", GOLDEN_BRANCH]) {
        println!("\n❌ Branch '{GOLDEN_BRANCH}' not found");
        println!("   Create with: cargo xtask golden-sync generate");
        return Ok(());
    }

    println!("✅ Branch '{GOLDEN_BRANCH}' exists");

    // Get golden branch commit info
    let info = run_git(&["log", "-1", "--format=%h %s", GOLDEN_BRANCH])?;
    println!("   Latest commit: {info}");

    // Count local snapshots
    let local_count = std::fs::read_dir(SNAPSHOTS_PATH)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "snap"))
                .count()
        })
        .unwrap_or(0);

    println!("\n📁 Local snapshots: {local_count} files");

    // Get golden branch snapshot count
    let files_output =
        run_git(&["ls-tree", "-r", "--name-only", GOLDEN_BRANCH, SNAPSHOTS_PATH]).unwrap_or_default();

    let golden_count = files_output
        .lines()
        .filter(|l| l.ends_with(".snap"))
        .count();

    println!("🏆 Golden snapshots: {golden_count} files");

    if local_count == 0 && golden_count > 0 {
        println!("\n⚠️  No local snapshots! Run:");
        println!("   cargo xtask golden-sync pull");
    } else if local_count != golden_count {
        println!("\n⚠️  Snapshot count mismatch!");
        println!("   Run 'cargo xtask golden-sync pull' to sync");
    } else {
        println!("\n✅ Snapshot counts match");
    }

    // Check for .new files (failed test outputs)
    let new_count = count_new_files();
    if new_count > 0 {
        println!("\n🔴 Found {new_count} .snap.new files (regression evidence)");
        println!("   These are NOT golden snapshots — they're your broken code's output.");
        println!("   Run 'cargo xtask golden-sync clean' to remove them.");
    }

    Ok(())
}

fn clean() -> Result<()> {
    println!("🧹 Cleaning up .snap.new files...\n");

    let new_count = count_new_files();
    if new_count == 0 {
        println!("✅ No .snap.new files found");
        return Ok(());
    }

    println!("Found {new_count} .snap.new files to remove:\n");

    // Show what we're removing
    if let Ok(entries) = std::fs::read_dir(SNAPSHOTS_PATH) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.to_string_lossy().ends_with(".snap.new") {
                println!(
                    "  🗑️  {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
            }
        }
    }

    let removed = clean_new_files();

    println!("\n✅ Removed {removed} .snap.new files");
    println!("\n📝 About .snap.new files:");
    println!("   • Created when golden tests FAIL");
    println!("   • Contain YOUR CODE's output (potentially broken)");
    println!("   • Are NOT golden snapshots");
    println!("   • Should NEVER be accepted with 'cargo insta accept'");
    println!("   • Fix your code instead!");

    Ok(())
}
