use std::any::Any;
use std::collections::HashMap;
use anyhow::Result;
use log::info;

use crate::plugins::{Plugin, HookType};
use crate::project::{Project, ProjectDetector, ProjectType};

/// 示例插件
pub struct ExamplePlugin;

impl ExamplePlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        "example_plugin"
    }
    
    fn version(&self) -> &str {
        "0.1.0"
    }
    
    fn description(&self) -> &str {
        "An example plugin demonstrating the use of the plugin system"
    }
    
    fn initialize(&mut self) -> Result<()> {
        info!("Example plugin initialized");
        Ok(())
    }
    
    fn get_project_detectors(&self) -> Vec<Box<dyn ProjectDetector>> {
        vec![Box::new(ExampleDetector::new())]
    }
    
    fn execute_hook(&self, hook_type: HookType, context: &HashMap<String, Box<dyn Any>>) -> Result<()> {
        match hook_type {
            HookType::BeforeCleaning => {
                info!("Example plugin: Before cleaning");
            },
            HookType::AfterCleaning => {
                info!("Example plugin: After cleaning");
            },
            HookType::BeforeCleanProject => {
                if let Some(project) = context.get("project") {
                    if let Some(project) = project.downcast_ref::<Project>() {
                        info!("Example plugin: Preparing to clean project {}", project.path.display());
                    }
                }
            },
            HookType::AfterCleanProject => {
                info!("Example plugin: Project cleaning completed");
            },
            _ => {}
        }
        
        Ok(())
    }
}

/// 示例项目检测器
pub struct ExampleDetector;

impl ExampleDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDetector for ExampleDetector {
    fn detect(&self, project: &mut Project) -> Result<bool> {
        // 检查是否是一个特定类型的项目
        let is_example = project.path.join("example.config.js").exists();
        
        if is_example {
            info!("Detected example project: {}", project.path.display());
            project.project_type = ProjectType::Unknown; // 可以定义自己的项目类型
            return Ok(true);
        }
        
        Ok(false)
    }
    
    fn get_build_dirs(&self, _project: &Project) -> Vec<String> {
        vec!["example-build".to_string(), "example-dist".to_string()]
    }
    
    fn get_cache_dirs(&self, _project: &Project) -> Vec<String> {
        vec![".example-cache".to_string()]
    }
    
    fn get_coverage_dirs(&self, _project: &Project) -> Vec<String> {
        vec!["example-coverage".to_string()]
    }
    
    fn get_priority(&self) -> u8 {
        50 // 中等优先级
    }
} 