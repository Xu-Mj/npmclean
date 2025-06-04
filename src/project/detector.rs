use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::project::{PackageInfo, Project, ProjectType};

/// 项目检测器特性
pub trait ProjectDetector: Send + Sync {
    /// 检测项目类型
    fn detect(&self, project: &mut Project) -> Result<bool>;

    /// 获取项目对应的构建目录
    fn get_build_dirs(&self, project: &Project) -> Vec<String>;

    /// 获取项目对应的缓存目录
    fn get_cache_dirs(&self, _project: &Project) -> Vec<String> {
        Vec::new() // 默认实现，返回空列表
    }

    /// 获取项目对应的代码覆盖率目录
    fn get_coverage_dirs(&self, _project: &Project) -> Vec<String> {
        vec!["coverage".to_string()] // 默认实现，返回标准覆盖率目录
    }

    /// 获取检测器优先级，数字越小优先级越高
    #[allow(dead_code)]
    fn get_priority(&self) -> u8 {
        100
    }
}

/// 默认项目检测器
pub struct DefaultDetector;

impl DefaultDetector {
    pub fn new() -> Self {
        Self
    }

    /// 从项目路径中解析 package.json 文件
    pub fn parse_package_json(project_path: &Path) -> Result<PackageInfo> {
        let package_json_path = project_path.join("package.json");
        let content =
            fs::read_to_string(package_json_path).context("Failed to read package.json")?;

        let json: Value = serde_json::from_str(&content).context("Failed to parse package.json")?;

        // 提取基本信息
        let name = json
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();

        let version = json
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("0.0.0")
            .to_string();

        // 提取依赖信息
        let dependencies = extract_dependencies(&json, "dependencies");
        let dev_dependencies = extract_dependencies(&json, "devDependencies");

        Ok(PackageInfo {
            name,
            version,
            dependencies,
            dev_dependencies,
        })
    }
}

impl ProjectDetector for DefaultDetector {
    fn detect(&self, project: &mut Project) -> Result<bool> {
        if !Project::has_package_json(&project.path) {
            return Ok(false);
        }

        // 解析 package.json
        let package_info = Self::parse_package_json(&project.path)?;
        project.package_info = Some(package_info);
        project.project_type = ProjectType::NodeJs;

        Ok(true)
    }

    fn get_build_dirs(&self, _project: &Project) -> Vec<String> {
        vec!["dist".to_string(), "build".to_string(), "out".to_string()]
    }

    fn get_cache_dirs(&self, _project: &Project) -> Vec<String> {
        vec![".cache".to_string()]
    }

    fn get_priority(&self) -> u8 {
        200 // 最低优先级，其他检测器都应该比这个优先级高
    }
}

/// 从 JSON 对象中提取依赖信息
fn extract_dependencies(json: &Value, field_name: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    if let Some(deps) = json.get(field_name) {
        if let Some(deps_obj) = deps.as_object() {
            for (key, value) in deps_obj {
                if let Some(version) = value.as_str() {
                    result.insert(key.clone(), version.to_string());
                }
            }
        }
    }

    result
}
