use anyhow::Result;
use log::{debug, info};
use rayon::prelude::*;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::project::analyzers::get_all_detectors;
use crate::project::{CleanTarget, Project, SizeInfo, TargetType};
use crate::utils::fs_utils::calculate_directory_size;

pub struct Scanner<'a> {
    config: &'a Config,
}

impl<'a> Scanner<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// 扫描指定路径下的项目
    pub fn scan(&self, root_path: &Path) -> Result<Vec<Project>> {
        info!("Scanning directory: {}", root_path.display());

        let project_paths = self.find_project_paths(root_path)?;
        info!("Found {} potential projects", project_paths.len());

        let projects = self.analyze_projects(project_paths)?;
        info!("Successfully analyzed {} projects", projects.len());

        Ok(projects)
    }

    /// 查找包含 package.json 的目录
    fn find_project_paths(&self, root_path: &Path) -> Result<Vec<PathBuf>> {
        let mut project_paths = Vec::new();
        let mut visited_dirs = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((root_path.to_path_buf(), 0));

        while let Some((path, depth)) = queue.pop_front() {
            // 跳过已访问的目录
            if !visited_dirs.insert(path.clone()) {
                continue;
            }

            // 检查深度限制
            if let Some(max_depth) = self.config.max_depth {
                if depth > max_depth {
                    continue;
                }
            }

            // 检查是否是项目目录
            if Project::has_package_json(&path) {
                debug!("Found project at {}", path.display());
                project_paths.push(path.clone());

                // 如果不是递归模式，则不继续扫描此目录下的子目录
                if !self.config.recursive {
                    continue;
                }
            }

            // 扫描子目录
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.filter_map(Result::ok) {
                    if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                        let path = entry.path();
                        // 跳过 node_modules 目录以提高性能
                        if path
                            .file_name()
                            .map_or(false, |name| name == "node_modules")
                        {
                            continue;
                        }

                        queue.push_back((path, depth + 1));
                    }
                }
            }
        }

        Ok(project_paths)
    }

    /// 分析项目，检测项目类型并确定清理目标
    fn analyze_projects(&self, project_paths: Vec<PathBuf>) -> Result<Vec<Project>> {
        // 使用 rayon 进行并行处理
        let projects: Vec<Project> = project_paths
            .into_par_iter()
            .filter_map(|path| match self.analyze_project(&path) {
                Ok(project) => Some(project),
                Err(e) => {
                    debug!("Failed to analyze project at {}: {}", path.display(), e);
                    None
                }
            })
            .collect();

        Ok(projects)
    }

    /// 分析单个项目
    fn analyze_project(&self, project_path: &Path) -> Result<Project> {
        debug!("Analyzing project at {}", project_path.display());

        // 创建项目实例
        let mut project = Project::new(project_path.to_path_buf());

        // 获取所有项目检测器
        let detectors = get_all_detectors();

        // 按优先级顺序尝试每个检测器
        for detector in &detectors {
            match detector.detect(&mut project) {
                Ok(true) => {
                    debug!(
                        "Project at {} detected as {:?}",
                        project_path.display(),
                        project.project_type
                    );
                    break;
                }
                Ok(false) => continue,
                Err(e) => {
                    debug!("Detector error for {}: {}", project_path.display(), e);
                    continue;
                }
            }
        }

        // 确定清理目标
        self.determine_clean_targets(&mut project, &detectors)?;

        // 如果需要统计，计算大小信息
        if self.config.stats {
            self.calculate_size_info(&mut project)?;
        }

        Ok(project)
    }

    /// 确定项目的清理目标
    fn determine_clean_targets(
        &self,
        project: &mut Project,
        detectors: &[Box<dyn crate::project::ProjectDetector>],
    ) -> Result<()> {
        let mut targets = Vec::new();

        // 找到适合当前项目的检测器
        let project_detector = detectors
            .iter()
            .find(|d| {
                if let Ok(true) = d.detect(&mut project.clone()) {
                    return true;
                }
                false
            })
            .unwrap_or(&detectors[detectors.len() - 1]); // 使用默认检测器

        // 添加 node_modules - 总是检查node_modules，即使配置未启用
        // 这样我们至少能显示它，用户可以决定是否清理
        let node_modules_path = project.path.join("node_modules");
        if node_modules_path.exists() {
            debug!("Found node_modules directory: {}", node_modules_path.display());
            
            let size = if self.config.stats {
                let size = calculate_directory_size(&node_modules_path)?;
                debug!("node_modules size: {} bytes", size);
                Some(size)
            } else {
                None
            };
            
            targets.push(CleanTarget {
                path: node_modules_path,
                target_type: TargetType::NodeModules,
                size: size,
            });
        }

        // 添加构建目录
        if self.config.clean_build_dirs {
            // 获取适合项目类型的构建目录
            let build_dirs = project_detector.get_build_dirs(project);

            for dir_name in build_dirs {
                let dir_path = project.path.join(&dir_name);
                if dir_path.exists() && dir_path.is_dir() {
                    debug!("Found build directory: {}", dir_path.display());
                    
                    let size = if self.config.stats {
                        Some(calculate_directory_size(&dir_path)?)
                    } else {
                        None
                    };
                    
                    targets.push(CleanTarget {
                        path: dir_path,
                        target_type: TargetType::BuildDir,
                        size: size,
                    });
                }
            }
        }

        // 添加缓存目录
        if self.config.clean_cache_dirs {
            let cache_dirs = project_detector.get_cache_dirs(project);

            for dir_name in cache_dirs {
                let dir_path = project.path.join(&dir_name);
                if dir_path.exists() && dir_path.is_dir() {
                    debug!("Found cache directory: {}", dir_path.display());
                    
                    let size = if self.config.stats {
                        Some(calculate_directory_size(&dir_path)?)
                    } else {
                        None
                    };
                    
                    targets.push(CleanTarget {
                        path: dir_path,
                        target_type: TargetType::CacheDir,
                        size: size,
                    });
                }
            }
        }

        // 添加覆盖率目录
        if self.config.clean_coverage_dirs {
            let coverage_dirs = project_detector.get_coverage_dirs(project);

            for dir_name in coverage_dirs {
                let dir_path = project.path.join(&dir_name);
                if dir_path.exists() && dir_path.is_dir() {
                    debug!("Found coverage directory: {}", dir_path.display());
                    
                    let size = if self.config.stats {
                        Some(calculate_directory_size(&dir_path)?)
                    } else {
                        None
                    };
                    
                    targets.push(CleanTarget {
                        path: dir_path,
                        target_type: TargetType::Coverage,
                        size: size,
                    });
                }
            }
        }

        // 处理用户指定的自定义目标
        for target_name in &self.config.custom_targets {
            let target_path = project.path.join(target_name);
            if target_path.exists() {
                debug!("Found custom target: {}", target_path.display());
                
                let size = if self.config.stats {
                    Some(calculate_directory_size(&target_path)?)
                } else {
                    None
                };
                
                targets.push(CleanTarget {
                    path: target_path,
                    target_type: TargetType::Custom(target_name.clone()),
                    size: size,
                });
            }
        }

        // 应用过滤规则
        targets = targets
            .into_iter()
            .filter(|target| !self.is_excluded(&target.path))
            .collect();

        project.detected_targets = targets;
        Ok(())
    }

    /// 检查路径是否在排除列表中
    fn is_excluded(&self, path: &Path) -> bool {
        for pattern in &self.config.exclude {
            if let Ok(glob) = globset::Glob::new(pattern) {
                if glob.compile_matcher().is_match(path) {
                    return true;
                }
            }
        }
        false
    }

    /// 计算项目大小信息
    fn calculate_size_info(&self, project: &mut Project) -> Result<()> {
        let mut total_size = 0;
        let mut node_modules_size = 0;
        let mut build_dirs_size = 0;
        let mut cache_dirs_size = 0;
        let mut coverage_dirs_size = 0;

        for target in &project.detected_targets {
            if let Some(size) = target.size {
                total_size += size;

                match target.target_type {
                    TargetType::NodeModules => node_modules_size += size,
                    TargetType::BuildDir => build_dirs_size += size,
                    TargetType::CacheDir => cache_dirs_size += size,
                    TargetType::Coverage => coverage_dirs_size += size,
                    TargetType::Custom(_) => {}
                }
            }
        }

        project.size_info = Some(SizeInfo {
            total_size,
            node_modules_size,
            build_dirs_size,
            cache_dirs_size,
            coverage_dirs_size,
        });

        Ok(())
    }
}
