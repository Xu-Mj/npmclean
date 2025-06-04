use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// 应用程序的主要配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // 基本选项
    #[serde(default)]
    pub targets: Vec<String>,

    #[serde(default)]
    pub exclude: Vec<String>,

    #[serde(default)]
    pub recursive: bool,

    #[serde(default)]
    pub force: bool,

    #[serde(default)]
    pub dry_run: bool,

    #[serde(default)]
    pub stats: bool,

    #[serde(default)]
    pub verbose: bool,

    // 清理选项
    #[serde(default = "default_true")]
    pub clean_node_modules: bool,

    #[serde(default = "default_true")]
    pub clean_build_dirs: bool,

    #[serde(default = "default_true")]
    pub clean_cache_dirs: bool,

    #[serde(default = "default_true")]
    pub clean_coverage_dirs: bool,

    #[serde(default)]
    pub custom_targets: Vec<String>,

    // 高级选项
    #[serde(default)]
    pub max_depth: Option<usize>,

    #[serde(default)]
    pub min_size: Option<u64>,

    #[serde(default)]
    pub threads: Option<usize>,

    #[serde(default)]
    pub timeout: Option<Duration>,

    // 内部使用，不从配置文件加载
    #[serde(skip)]
    #[allow(dead_code)]
    pub project_path: Option<PathBuf>,
}

// 实现默认值
impl Default for Config {
    fn default() -> Self {
        Self {
            targets: Vec::new(),
            exclude: Vec::new(),
            recursive: false,
            force: false,
            dry_run: false,
            stats: false,
            verbose: false,
            clean_node_modules: true,
            clean_build_dirs: true,
            clean_cache_dirs: true,
            clean_coverage_dirs: true,
            custom_targets: Vec::new(),
            max_depth: None,
            min_size: None,
            threads: None,
            timeout: None,
            project_path: None,
        }
    }
}

fn default_true() -> bool {
    true
}

/// 默认构建目录列表
#[allow(dead_code)]
pub fn default_build_dirs() -> Vec<&'static str> {
    vec![
        "dist", "build", "out", ".next", ".nuxt", ".cache", "coverage",
    ]
}

/// 默认缓存目录列表
#[allow(dead_code)]
pub fn default_cache_dirs() -> Vec<&'static str> {
    vec![".cache", ".angular", ".parcel-cache", ".nuxt"]
}

/// 默认覆盖率目录列表
#[allow(dead_code)]
pub fn default_coverage_dirs() -> Vec<&'static str> {
    vec!["coverage", ".nyc_output"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();

        // 清理选项默认值应该都是true
        assert!(
            config.clean_node_modules,
            "clean_node_modules should default to true"
        );
        assert!(
            config.clean_build_dirs,
            "clean_build_dirs should default to true"
        );
        assert!(
            config.clean_cache_dirs,
            "clean_cache_dirs should default to true"
        );
        assert!(
            config.clean_coverage_dirs,
            "clean_coverage_dirs should default to true"
        );

        // 基本选项默认值
        assert!(!config.recursive, "recursive should default to false");
        assert!(!config.force, "force should default to false");
        assert!(!config.dry_run, "dry_run should default to false");
        assert!(!config.stats, "stats should default to false");
        assert!(!config.verbose, "verbose should default to false");

        // 集合类型默认应为空
        assert!(config.targets.is_empty(), "targets should default to empty");
        assert!(config.exclude.is_empty(), "exclude should default to empty");
        assert!(
            config.custom_targets.is_empty(),
            "custom_targets should default to empty"
        );
    }
}
