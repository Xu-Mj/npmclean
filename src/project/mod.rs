pub mod analyzers;
mod detector;

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

// 重导出
pub use detector::ProjectDetector;

/// 项目类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectType {
    NodeJs,
    React,
    Vue,
    Angular,
    NextJs,
    NuxtJs,
    Unknown,
}

/// 清理目标类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetType {
    NodeModules,
    BuildDir,
    CacheDir,
    Coverage,
    Custom(String),
}

impl fmt::Display for TargetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetType::NodeModules => write!(f, "node_modules"),
            TargetType::BuildDir => write!(f, "build"),
            TargetType::CacheDir => write!(f, "cache"),
            TargetType::Coverage => write!(f, "coverage"),
            TargetType::Custom(name) => write!(f, "custom: {}", name),
        }
    }
}

/// 项目大小信息
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SizeInfo {
    pub total_size: u64,
    pub node_modules_size: u64,
    pub build_dirs_size: u64,
    pub cache_dirs_size: u64,
    pub coverage_dirs_size: u64,
}

/// 包信息，从 package.json 解析
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
}

/// 清理目标
#[derive(Debug, Clone)]
pub struct CleanTarget {
    pub path: PathBuf,
    pub target_type: TargetType,
    pub size: Option<u64>,
}

/// 项目模型
#[derive(Debug, Clone)]
pub struct Project {
    pub path: PathBuf,
    pub project_type: ProjectType,
    pub package_info: Option<PackageInfo>,
    pub size_info: Option<SizeInfo>,
    pub detected_targets: Vec<CleanTarget>,
}

impl Project {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            project_type: ProjectType::Unknown,
            package_info: None,
            size_info: None,
            detected_targets: Vec::new(),
        }
    }

    /// 检查路径是否包含 package.json 文件
    pub fn has_package_json(path: &Path) -> bool {
        path.join("package.json").exists()
    }
}
