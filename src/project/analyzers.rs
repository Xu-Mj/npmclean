use anyhow::Result;

use crate::project::{Project, ProjectDetector, ProjectType};

/// React 项目检测器
pub struct ReactDetector;

impl ReactDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDetector for ReactDetector {
    fn detect(&self, project: &mut Project) -> Result<bool> {
        let package_info = match &project.package_info {
            Some(info) => info,
            None => return Ok(false),
        };

        // 检查是否是 React 项目
        let is_react = package_info.dependencies.contains_key("react")
            || package_info.dev_dependencies.contains_key("react");

        if is_react {
            project.project_type = ProjectType::React;
            return Ok(true);
        }

        Ok(false)
    }

    fn get_build_dirs(&self, _project: &Project) -> Vec<String> {
        vec!["build".to_string(), "dist".to_string()]
    }

    fn get_priority(&self) -> u8 {
        100
    }
}

/// Vue.js 项目检测器
pub struct VueDetector;

impl VueDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDetector for VueDetector {
    fn detect(&self, project: &mut Project) -> Result<bool> {
        let package_info = match &project.package_info {
            Some(info) => info,
            None => return Ok(false),
        };

        // 添加更全面的Vue3检测
        let is_vue = package_info.dependencies.contains_key("vue") ||
            package_info.dev_dependencies.contains_key("vue") ||
            package_info.dependencies.contains_key("@vue/cli-service") ||
            package_info.dev_dependencies.contains_key("@vue/cli-service") ||
            project.path.join("vue.config.js").exists() || 
            project.path.join("vite.config.js").exists();

        if is_vue {
            project.project_type = ProjectType::Vue;
            return Ok(true);
        }

        Ok(false)
    }

    fn get_build_dirs(&self, _project: &Project) -> Vec<String> {
        vec!["dist".to_string()]
    }

    fn get_priority(&self) -> u8 {
        100
    }
}

/// Next.js 项目检测器
pub struct NextJsDetector;

impl NextJsDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDetector for NextJsDetector {
    fn detect(&self, project: &mut Project) -> Result<bool> {
        let package_info = match &project.package_info {
            Some(info) => info,
            None => return Ok(false),
        };

        // 检查是否是 Next.js 项目
        let is_nextjs = package_info.dependencies.contains_key("next")
            || package_info.dev_dependencies.contains_key("next");

        // 或者检查是否有 next.config.js 文件
        let has_next_config = project.path.join("next.config.js").exists();

        if is_nextjs || has_next_config {
            project.project_type = ProjectType::NextJs;
            return Ok(true);
        }

        Ok(false)
    }

    fn get_build_dirs(&self, _project: &Project) -> Vec<String> {
        vec![".next".to_string(), "out".to_string()]
    }

    fn get_priority(&self) -> u8 {
        80 // Next.js 通常也使用 React，所以优先级应该高一些
    }
}

/// Angular 项目检测器
pub struct AngularDetector;

impl AngularDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDetector for AngularDetector {
    fn detect(&self, project: &mut Project) -> Result<bool> {
        let package_info = match &project.package_info {
            Some(info) => info,
            None => return Ok(false),
        };

        // 检查是否是 Angular 项目
        let is_angular = package_info.dependencies.contains_key("@angular/core")
            || package_info.dev_dependencies.contains_key("@angular/core");

        // 或者检查是否有 angular.json 文件
        let has_angular_config = project.path.join("angular.json").exists();

        if is_angular || has_angular_config {
            project.project_type = ProjectType::Angular;
            return Ok(true);
        }

        Ok(false)
    }

    fn get_build_dirs(&self, _project: &Project) -> Vec<String> {
        vec!["dist".to_string()]
    }

    fn get_cache_dirs(&self, _project: &Project) -> Vec<String> {
        vec![".angular".to_string()]
    }

    fn get_priority(&self) -> u8 {
        90
    }
}

/// Nuxt.js 项目检测器
pub struct NuxtJsDetector;

impl NuxtJsDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDetector for NuxtJsDetector {
    fn detect(&self, project: &mut Project) -> Result<bool> {
        let package_info = match &project.package_info {
            Some(info) => info,
            None => return Ok(false),
        };

        // 检查是否是 Nuxt.js 项目
        let is_nuxtjs = package_info.dependencies.contains_key("nuxt")
            || package_info.dev_dependencies.contains_key("nuxt");

        // 或者检查是否有 nuxt.config.js 文件
        let has_nuxt_config = project.path.join("nuxt.config.js").exists()
            || project.path.join("nuxt.config.ts").exists();

        if is_nuxtjs || has_nuxt_config {
            project.project_type = ProjectType::NuxtJs;
            return Ok(true);
        }

        Ok(false)
    }

    fn get_build_dirs(&self, _project: &Project) -> Vec<String> {
        vec![".nuxt".to_string(), "dist".to_string()]
    }

    fn get_cache_dirs(&self, _project: &Project) -> Vec<String> {
        vec![".cache".to_string()]
    }

    fn get_priority(&self) -> u8 {
        85 // Nuxt.js 通常也使用 Vue，所以优先级应该高一些
    }
}

/// 获取所有项目检测器
pub fn get_all_detectors() -> Vec<Box<dyn ProjectDetector>> {
    vec![
        Box::new(NextJsDetector::new()),
        Box::new(NuxtJsDetector::new()),
        Box::new(AngularDetector::new()),
        Box::new(VueDetector::new()),
        Box::new(ReactDetector::new()),
        Box::new(crate::project::detector::DefaultDetector::new()),
    ]
}
