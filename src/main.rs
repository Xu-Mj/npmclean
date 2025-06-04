mod cleaner;
mod cli;
mod config;
mod plugins;
mod project;
mod scanner;
mod utils;

use anyhow::Result;
use log::{info, LevelFilter};
use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::plugins::{ExamplePlugin, HookType, PluginRegistry};

fn main() -> Result<()> {
    // 初始化日志系统 - 日志输出到文件
    let log_dir = setup_logging()?;
    info!("Starting npm-clean");

    // 初始化插件系统
    let plugin_registry = initialize_plugins()?;

    // 解析命令行参数
    let args = cli::parse_args();

    // 加载配置
    let config = config::load_config(&args)?;

    // 创建上下文
    let mut context: HashMap<String, Box<dyn Any>> = HashMap::new();
    context.insert("config".to_string(), Box::new(config.clone()));

    // 执行清理前钩子
    if let Err(e) = plugin_registry.execute_hook(HookType::BeforeCleaning, &context) {
        eprintln!("Warning: Plugin execution failed: {}. See log file for details.", e);
    }

    // 创建扫描器并扫描项目
    let scanner = scanner::Scanner::new(&config);
    let projects = match scanner.scan(&args.path) {
        Ok(projects) => projects,
        Err(e) => {
            eprintln!("Error: Failed to scan projects: {}", e);
            eprintln!("Detailed logs can be found at {}", log_dir.display());
            return Err(e);
        }
    };

    // 显示扫描结果
    if config.verbose {
        cli::display_scan_results(&projects, &config);
    }

    // 创建清理器并执行清理
    let mut cleaner = cleaner::Cleaner::new(&config);

    // 将插件检测器添加到清理器
    let plugin_detectors = plugin_registry.get_project_detectors();
    if !plugin_detectors.is_empty() {
        info!("Loaded {} project detectors from plugins", plugin_detectors.len());
        // 这里需要更新cleaner，将插件检测器集成到清理器中
        cleaner.add_detectors(plugin_detectors);
    }

    let results = match cleaner.clean(projects) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("Error: Failed during cleaning process: {}", e);
            eprintln!("Detailed logs can be found at {}", log_dir.display());
            return Err(e);
        }
    };

    // 显示清理结果
    cli::display_clean_results(&results, &config);

    // 执行清理后钩子
    context.insert("results".to_string(), Box::new(results));
    if let Err(e) = plugin_registry.execute_hook(HookType::AfterCleaning, &context) {
        eprintln!("Warning: Plugin execution failed: {}. See log file for details.", e);
    }

    info!("npm-clean completed successfully");
    Ok(())
}

/// 初始化插件系统
fn initialize_plugins() -> Result<PluginRegistry> {
    let mut registry = PluginRegistry::new();

    // 注册内置插件
    registry.register(Box::new(ExamplePlugin::new()))?;

    // 这里可以添加从外部加载插件的逻辑
    // 例如从特定目录或环境变量指定的路径加载动态库

    info!(
        "Plugin system initialized with {} plugins",
        registry.get_plugins().len()
    );

    Ok(registry)
}

/// 设置日志系统，将日志输出到文件
fn setup_logging() -> Result<PathBuf> {
    // 创建日志目录
    let log_dir = get_log_directory()?;
    fs::create_dir_all(&log_dir)?;

    // 生成日志文件名
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let log_file = log_dir.join(format!("npm-clean_{}.log", timestamp));

    // 配置文件日志记录器
    let file_logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .chain(fern::log_file(log_file)?);

    // 应用日志配置
    file_logger.apply()?;

    Ok(log_dir)
}

/// 获取日志目录路径
fn get_log_directory() -> Result<PathBuf> {
    let mut log_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine local data directory"))?
        .join("npm-clean")
        .join("logs");

    // 如果不存在则使用临时目录
    if !log_dir.exists() && fs::create_dir_all(&log_dir).is_err() {
        log_dir = std::env::temp_dir().join("npm-clean").join("logs");
    }

    Ok(log_dir)
}
