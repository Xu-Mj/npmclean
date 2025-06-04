mod loader;
mod schema;

use crate::cli::CliArgs;
use anyhow::{Context, Result};

pub use schema::Config;

/// 加载配置，按优先级从高到低：命令行参数 > 项目配置 > 用户配置 > 默认配置
pub fn load_config(args: &CliArgs) -> Result<Config> {
    // 加载默认配置
    let mut config = Config::default();

    // 尝试加载用户配置（~/.npmcleanrc.yml）
    if let Some(user_config) = loader::load_user_config().context("Failed to load user config")? {
        config = loader::merge_configs(config, user_config);
    }

    // 尝试加载项目配置
    let project_config_path = if let Some(config_path) = &args.config {
        config_path.clone()
    } else {
        // 检查当前目录中是否有配置文件
        let current_dir = std::env::current_dir()?;
        current_dir.join(".npmcleanrc.yml")
    };

    if project_config_path.exists() {
        let project_config = loader::load_config_file(&project_config_path).context(format!(
            "Failed to load config from {}",
            project_config_path.display()
        ))?;
        config = loader::merge_configs(config, project_config);
    }

    // 应用命令行参数覆盖配置
    config = apply_cli_args(config, args);

    Ok(config)
}

/// 将命令行参数应用到配置中
fn apply_cli_args(mut config: Config, args: &CliArgs) -> Config {
    // 基本选项
    config.recursive = args.recursive || config.recursive;
    config.force = args.force || config.force;
    config.dry_run = args.dry_run;
    config.stats = args.stats || config.stats;
    config.verbose = args.verbose || config.verbose;

    // 清理模式 - 修改逻辑，使默认清理所有目标类型
    // 只有当用户明确指定了某一类型时，才限制为仅清理该类型
    if args.node_modules_only {
        // 只清理 node_modules
        config.clean_node_modules = true;
        config.clean_build_dirs = false;
        config.clean_cache_dirs = false;
        config.clean_coverage_dirs = false;
    } else if args.build {
        // 只清理构建目录
        config.clean_node_modules = false;
        config.clean_build_dirs = true;
        config.clean_cache_dirs = false;
        config.clean_coverage_dirs = false;
    } else {
        // 默认情况：清理所有类型的目标
        config.clean_node_modules = true;
        config.clean_build_dirs = true;
        config.clean_cache_dirs = true;
        config.clean_coverage_dirs = true;
    }

    // 自定义包含/排除目录
    if let Some(include_str) = &args.include {
        let includes: Vec<String> = include_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        config.custom_targets.extend(includes);
    }

    if let Some(exclude_str) = &args.exclude {
        let excludes: Vec<String> = exclude_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        config.exclude.extend(excludes);
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.clean_node_modules);
        assert!(config.clean_build_dirs);
        assert!(!config.dry_run);
        assert!(!config.force);
    }
}
