use clap::Parser;
use std::path::PathBuf;

use crate::cleaner::CleanResults;
use crate::config::Config;
use crate::project::Project;

#[derive(Parser, Debug)]
#[command(
    name = "npm-clean",
    about = "Fast and safe cleaner for node_modules and build directories",
    version
)]
pub struct CliArgs {
    /// Path to project or directory, defaults to current directory
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Recursively find and clean projects in subdirectories
    #[arg(short, long)]
    pub recursive: bool,

    /// Skip confirmation prompts
    #[arg(short, long)]
    pub force: bool,

    /// Show what would be deleted without deleting
    #[arg(short = 'd', long = "dry-run")]
    pub dry_run: bool,

    /// Use specific config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Clean only node_modules directories
    #[arg(short = 'n', long = "node-modules")]
    pub node_modules_only: bool,

    /// Clean only build directories
    #[arg(short, long)]
    pub build: bool,

    /// Additional directories to clean (comma-separated)
    #[arg(long, value_name = "DIRS")]
    pub include: Option<String>,

    /// Directories to exclude (comma-separated)
    #[arg(long, value_name = "DIRS")]
    pub exclude: Option<String>,

    /// Show space-saving statistics
    #[arg(short, long)]
    pub stats: bool,

    /// Display detailed output
    #[arg(short, long)]
    pub verbose: bool,
}

pub fn parse_args() -> CliArgs {
    CliArgs::parse()
}

pub fn display_scan_results(projects: &[Project], config: &Config) {
    if projects.is_empty() {
        println!("No projects found.");
        return;
    }

    println!("Found {} projects:", projects.len());

    for (i, project) in projects.iter().enumerate() {
        println!("{}. {}", i + 1, project.path.display());
        println!("   Type: {:?}", project.project_type);

        if let Some(size_info) = &project.size_info {
            let total_mb = size_info.total_size / (1024 * 1024);
            println!("   Total Size: {} MB", total_mb);
        }

        if config.verbose {
            println!("   Targets to clean:");
            for target in &project.detected_targets {
                let size_str = if let Some(size) = target.size {
                    format!(" ({} MB)", size / (1024 * 1024))
                } else {
                    String::new()
                };

                println!(
                    "     - {} [{}]{}",
                    target.path.display(),
                    target.target_type,
                    size_str
                );
            }
        }
        println!();
    }
}

pub fn display_clean_results(results: &CleanResults, config: &Config) {
    if config.dry_run {
        println!("\n[DRY RUN] - No files were actually deleted");
    } else {
        println!("\n[CLEANING COMPLETED]");
    }

    // 显示统计数据
    let freed_mb = results.total_bytes_removed / (1024 * 1024);

    if config.dry_run {
        println!("Space that would be freed: {} MB", freed_mb);
    } else {
        println!("Space freed: {} MB", freed_mb);
    }

    // 仅在详细模式下显示更多统计信息
    if config.stats {
        println!("Projects processed: {}/{}", results.cleaned_projects, results.total_projects);
        println!("Targets cleaned: {}/{}", results.cleaned_targets, results.total_targets);
        
        if results.failed_projects > 0 || results.failed_targets > 0 {
            println!("\n[WARNING] Issues encountered during cleaning:");
            if results.failed_projects > 0 {
                println!("  - Failed projects: {}", results.failed_projects);
            }
            if results.failed_targets > 0 {
                println!("  - Failed targets: {}", results.failed_targets);
            }
        }
    }
}
