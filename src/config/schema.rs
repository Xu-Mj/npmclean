use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// 应用程序的主要配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
