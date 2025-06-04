use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::schema::Config;

/// 加载指定路径的配置文件
pub fn load_config_file(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)?;

    // 基于文件扩展名选择解析器
    match path.extension().and_then(|e| e.to_str()) {
        Some("yml") | Some("yaml") => serde_yaml::from_str(&content)
            .context(format!("Failed to parse YAML file: {}", path.display())),
        _ => {
            // 默认尝试作为 YAML 解析
            serde_yaml::from_str(&content)
                .context(format!("Failed to parse config file: {}", path.display()))
        }
    }
}

/// 加载用户主目录中的配置文件（如果存在）
pub fn load_user_config() -> Result<Option<Config>> {
    if let Some(home_dir) = dirs::home_dir() {
        let user_config_path = home_dir.join(".npmcleanrc.yml");
        if user_config_path.exists() {
            return Ok(Some(load_config_file(&user_config_path)?));
        }
    }

    Ok(None)
}

/// 合并两个配置，以第二个配置为优先
pub fn merge_configs(base: Config, override_config: Config) -> Config {
    // 创建一个新配置，从基础配置开始
    let mut result = base;

    // 合并简单字段（优先使用覆盖配置的值）
    result.recursive = override_config.recursive || result.recursive;
    result.force = override_config.force || result.force;
    result.dry_run = override_config.dry_run || result.dry_run;
    result.stats = override_config.stats || result.stats;
    result.verbose = override_config.verbose || result.verbose;
    result.clean_node_modules = override_config.clean_node_modules || result.clean_node_modules;
    result.clean_build_dirs = override_config.clean_build_dirs || result.clean_build_dirs;

    // 合并可选字段（如果覆盖配置中有值，则使用该值）
    if override_config.max_depth.is_some() {
        result.max_depth = override_config.max_depth;
    }

    if override_config.min_size.is_some() {
        result.min_size = override_config.min_size;
    }

    if override_config.threads.is_some() {
        result.threads = override_config.threads;
    }

    if override_config.timeout.is_some() {
        result.timeout = override_config.timeout;
    }

    // 合并列表（添加不重复的项）
    // 对于 targets 和 custom_targets，合并并去重
    for target in override_config.targets {
        if !result.targets.contains(&target) {
            result.targets.push(target);
        }
    }

    for custom_target in override_config.custom_targets {
        if !result.custom_targets.contains(&custom_target) {
            result.custom_targets.push(custom_target);
        }
    }

    // 对于 exclude，直接添加所有项（允许重复，简化处理）
    result.exclude.extend(override_config.exclude);

    result
}
