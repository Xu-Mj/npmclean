// 导出模块
mod registry;
mod examples;

pub use registry::{Plugin, PluginRegistry};
pub use examples::ExamplePlugin;

/// 插件钩子类型
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum HookType {
    /// 在清理开始之前调用
    BeforeCleaning,
    /// 在清理结束之后调用
    AfterCleaning,
    /// 在每个项目清理之前调用
    BeforeCleanProject,
    /// 在每个项目清理之后调用
    AfterCleanProject,
    /// 在每个目标清理之前调用
    BeforeCleanTarget,
    /// 在每个目标清理之后调用
    AfterCleanTarget,
} 