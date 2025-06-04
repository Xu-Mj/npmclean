use anyhow::Result;
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{debug, error, info};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

use crate::config::Config;
use crate::project::{CleanTarget, Project, ProjectDetector, TargetType};
use crate::utils::fs_utils::remove_directory;

/// 清理结果数据
#[derive(Debug, Clone)]
pub struct CleanResults {
    pub total_projects: usize,
    pub cleaned_projects: usize,
    pub failed_projects: usize,
    pub total_targets: usize,
    pub cleaned_targets: usize,
    pub failed_targets: usize,
    pub total_bytes_removed: u64,
}

/// 清理器，用于执行清理操作
pub struct Cleaner<'a> {
    config: &'a Config,
    multi_progress: MultiProgress,
    additional_detectors: Vec<Box<dyn ProjectDetector>>,
}

impl<'a> Cleaner<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            multi_progress: MultiProgress::new(),
            additional_detectors: Vec::new(),
        }
    }

    /// 添加额外的项目检测器（来自插件）
    pub fn add_detectors(&mut self, detectors: Vec<Box<dyn ProjectDetector>>) {
        self.additional_detectors.extend(detectors);
    }

    /// 清理项目列表
    pub fn clean(&self, projects: Vec<Project>) -> Result<CleanResults> {
        let results = CleanResults {
            total_projects: projects.len(),
            cleaned_projects: 0,
            failed_projects: 0,
            total_targets: 0,
            cleaned_targets: 0,
            failed_targets: 0,
            total_bytes_removed: 0,
        };

        let results = Arc::new(Mutex::new(results));

        // 如果没有找到项目
        if projects.is_empty() {
            info!("No projects found to clean");
            println!("No projects found to clean");
            return Ok(Arc::try_unwrap(results).unwrap().into_inner().unwrap());
        }

        // 显示清理前统计
        self.display_cleaning_preview(&projects)?;

        // 如果需要确认且不是强制模式
        if !self.config.force && !self.config.dry_run && !self.confirm_cleaning()? {
            info!("Cleaning cancelled by user");
            println!("Cleaning cancelled by user");
            return Ok(Arc::try_unwrap(results).unwrap().into_inner().unwrap());
        }

        // 开始清理
        info!(
            "Starting {} of {} projects{}",
            if self.config.dry_run {
                "dry run"
            } else {
                "cleaning"
            },
            projects.len(),
            if self.config.dry_run {
                " (no files will be deleted)"
            } else {
                ""
            }
        );

        // 创建进度条
        let progress = self.create_progress_bar(
            projects.len(),
            if self.config.dry_run {
                "Simulating cleaning"
            } else {
                "Cleaning projects"
            },
        );

        // 并行处理每个项目
        let _cleaned_results: Vec<_> = projects
            .into_par_iter()
            .map(|project| {
                let project_result = self.clean_project(&project, &results);
                progress.inc(1);
                project_result
            })
            .collect();

        progress.finish_with_message("Cleaning completed");

        let final_results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();

        if self.config.dry_run {
            info!(
                "Dry run completed. Would have freed {} MB",
                final_results.total_bytes_removed / (1024 * 1024)
            );
        } else {
            info!(
                "Cleaning completed. Freed {} MB",
                final_results.total_bytes_removed / (1024 * 1024)
            );
        }

        Ok(final_results)
    }

    /// 清理单个项目
    fn clean_project(&self, project: &Project, results: &Arc<Mutex<CleanResults>>) -> Result<()> {
        // 更新统计
        {
            let mut r = results.lock().unwrap();
            r.total_targets += project.detected_targets.len();
        }

        debug!("Cleaning project: {}", project.path.display());

        // 处理项目中的每个目标
        for target in &project.detected_targets {
            // 检查是否应该清理此目标
            let should_clean = match target.target_type {
                TargetType::NodeModules => self.config.clean_node_modules,
                TargetType::BuildDir => self.config.clean_build_dirs,
                TargetType::CacheDir => self.config.clean_cache_dirs,
                TargetType::Coverage => self.config.clean_coverage_dirs,
                TargetType::Custom(_) => true, // Custom targets are always cleaned
            };

            // 如果不应该清理，跳过
            if !should_clean {
                debug!(
                    "Skipping target {} (not configured to clean this type)",
                    target.path.display()
                );
                continue;
            }

            if let Err(e) = self.clean_target(project, target, results) {
                error!(
                    "Failed to clean {} in {}: {}",
                    target.path.display(),
                    project.path.display(),
                    e
                );
                continue;
            }
        }

        // 更新统计
        {
            let mut r = results.lock().unwrap();
            r.cleaned_projects += 1;
        }

        Ok(())
    }

    /// 清理单个目标
    fn clean_target(
        &self,
        project: &Project,
        target: &CleanTarget,
        results: &Arc<Mutex<CleanResults>>,
    ) -> Result<()> {
        let target_path = &target.path;
        let target_type_str = format!("{}", target.target_type);
        let target_size = target.size.unwrap_or(0) / (1024 * 1024); // MB

        debug!(
            "Cleaning {} ({} MB) in {}",
            target_type_str,
            target_size,
            project.path.display()
        );

        // 清理目录
        if self.config.dry_run {
            // 模拟清理
            {
                let mut r = results.lock().unwrap();
                r.cleaned_targets += 1;
                r.total_bytes_removed += target.size.unwrap_or(0);
            }
        } else {
            // 实际清理
            match remove_directory(target_path) {
                Ok(_) => {
                    let mut r = results.lock().unwrap();
                    r.cleaned_targets += 1;
                    r.total_bytes_removed += target.size.unwrap_or(0);

                    debug!(
                        "Successfully cleaned {} ({} MB)",
                        target_path.display(),
                        target_size
                    );
                }
                Err(e) => {
                    error!("Failed to clean {}: {}", target_path.display(), e);
                    let mut r = results.lock().unwrap();
                    r.failed_targets += 1;
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// 显示清理预览
    fn display_cleaning_preview(&self, projects: &[Project]) -> Result<()> {
        println!("\n{}", style("Projects to clean:").bold().underlined());

        let mut total_size = 0;
        let mut found_targets = false;

        for project in projects {
            if project.detected_targets.is_empty() {
                continue;
            }

            found_targets = true;
            println!(
                "\n• Project: {} [{}]",
                style(project.path.display().to_string()).green().bold(),
                style(format!("{:?}", project.project_type)).yellow()
            );

            for target in &project.detected_targets {
                // 检查是否应该清理此目标
                let should_clean = match target.target_type {
                    TargetType::NodeModules => self.config.clean_node_modules,
                    TargetType::BuildDir => self.config.clean_build_dirs,
                    TargetType::CacheDir => self.config.clean_cache_dirs,
                    TargetType::Coverage => self.config.clean_coverage_dirs,
                    TargetType::Custom(_) => true, // Custom targets are always cleaned
                };

                let size_str = if let Some(size) = target.size {
                    let size_mb = size / (1024 * 1024);
                    if should_clean {
                        total_size += size;
                    }
                    format!(" ({} MB)", size_mb)
                } else {
                    " (size unknown)".to_string()
                };

                let clean_status = if should_clean {
                    if self.config.dry_run {
                        style("[Simulating]").yellow()
                    } else {
                        style("[Cleaning]").green()
                    }
                } else {
                    style("[Skipped]").dim()
                };

                println!(
                    "  - {} {} {}{}",
                    clean_status,
                    target.path.display(),
                    style(format!("[{}]", target.target_type)).yellow(),
                    style(size_str).cyan()
                );
            }
        }

        if !found_targets {
            println!("{}", style("No cleanable targets found!").yellow());
        }

        println!(
            "\nTotal estimated space to free: {} MB\n",
            style(format!("{}", total_size / (1024 * 1024)))
                .green()
                .bold()
        );

        Ok(())
    }

    /// 请求用户确认清理
    fn confirm_cleaning(&self) -> Result<bool> {
        println!(
            "{}",
            style("Do you want to proceed with cleaning? [y/N]:").bold()
        );

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        Ok(input.trim().to_lowercase() == "y")
    }

    /// 创建进度条
    fn create_progress_bar(&self, total: usize, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new(total as u64));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("=>-"),
        );
        pb.set_message(message.to_string());
        pb
    }
}
