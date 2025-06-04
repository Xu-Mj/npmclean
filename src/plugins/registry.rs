use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::plugins::HookType;
use crate::project::{Project, ProjectDetector};

/// 插件特性
#[allow(dead_code)]
pub trait Plugin: Send + Sync {
    /// 获取插件名称
    fn name(&self) -> &str;

    /// 获取插件版本
    fn version(&self) -> &str;

    /// 获取插件描述
    fn description(&self) -> &str;

    /// 初始化插件
    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// 获取项目检测器
    fn get_project_detectors(&self) -> Vec<Box<dyn ProjectDetector>> {
        Vec::new()
    }

    /// 执行钩子
    fn execute_hook(
        &self,
        _hook_type: HookType,
        _context: &HashMap<String, Box<dyn Any>>,
    ) -> Result<()> {
        Ok(())
    }
}

/// 插件注册表
pub struct PluginRegistry {
    plugins: Vec<Arc<Box<dyn Plugin>>>,
}

impl PluginRegistry {
    /// 创建新的插件注册表
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// 注册插件
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        self.plugins.push(Arc::new(plugin));
        Ok(())
    }

    /// 获取所有插件
    pub fn get_plugins(&self) -> &[Arc<Box<dyn Plugin>>] {
        &self.plugins
    }

    /// 获取所有项目检测器
    pub fn get_project_detectors(&self) -> Vec<Box<dyn ProjectDetector>> {
        let mut detectors = Vec::new();

        for plugin in &self.plugins {
            detectors.extend(plugin.get_project_detectors());
        }

        detectors
    }

    /// 执行钩子
    pub fn execute_hook(
        &self,
        hook_type: HookType,
        context: &HashMap<String, Box<dyn Any>>,
    ) -> Result<()> {
        for plugin in &self.plugins {
            plugin.execute_hook(hook_type, context)?;
        }

        Ok(())
    }

    /// 通过项目类型筛选插件
    #[allow(dead_code)]
    pub fn filter_plugins_for_project(&self, project: &Project) -> Vec<Arc<Box<dyn Plugin>>> {
        let mut result = Vec::new();

        for plugin in &self.plugins {
            // 如果该插件提供的任何检测器能够检测该项目，则添加该插件
            let detectors = plugin.get_project_detectors();
            for detector in &detectors {
                if let Ok(true) = detector.detect(&mut project.clone()) {
                    result.push(plugin.clone());
                    break;
                }
            }
        }

        result
    }
}
