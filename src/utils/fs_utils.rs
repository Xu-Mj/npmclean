use anyhow::{Context, Result};
use log::debug;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// 递归计算目录大小
pub fn calculate_directory_size(path: &Path) -> Result<u64> {
    if !path.exists() {
        debug!("Path does not exist: {}", path.display());
        return Ok(0);
    }

    let mut total_size = 0;
    let walker = WalkDir::new(path).min_depth(1).into_iter();

    // 使用walkdir，更可靠地处理深层次目录结构
    for entry in walker.filter_map(|e| e.ok()) {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                let file_size = metadata.len();
                total_size += file_size;
                debug!("File: {} Size: {} bytes", entry.path().display(), file_size);
            }
        }
    }

    debug!(
        "Directory {} total size: {} bytes",
        path.display(),
        total_size
    );
    Ok(total_size)
}

/// 递归删除目录，具有更好的错误处理和性能优化
pub fn remove_directory(path: &Path) -> Result<()> {
    // 尝试使用 remove_dir_all 库（一个更可靠的跨平台实现）
    remove_dir_all::remove_dir_all(path)
        .context(format!("Failed to remove directory: {}", path.display()))
}

/// 递归删除目录，但用深度优先策略，适用于包含大量小文件的深层目录结构
#[allow(dead_code)]
pub fn remove_directory_deep_first(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    // 如果是文件，直接删除
    if path.is_file() {
        return fs::remove_file(path).context(format!("Failed to remove file: {}", path.display()));
    }

    // 如果是目录，先删除所有子项，然后删除自身
    let mut stack = Vec::new();
    let mut dirs_to_delete = Vec::new();
    stack.push(path.to_path_buf());

    // 深度优先遍历，收集所有需要删除的目录
    while let Some(dir) = stack.pop() {
        if !dir.exists() {
            continue;
        }

        // 记录需要删除的目录
        dirs_to_delete.push(dir.clone());

        // 处理子项
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.filter_map(Result::ok) {
                let entry_path = entry.path();

                if entry_path.is_file() {
                    // 直接删除文件
                    if let Err(e) = fs::remove_file(&entry_path) {
                        eprintln!(
                            "Warning: Failed to remove file {}: {}",
                            entry_path.display(),
                            e
                        );
                    }
                } else if entry_path.is_dir() {
                    // 将目录添加到栈中
                    stack.push(entry_path);
                }
            }
        }
    }

    // 从深到浅删除目录（从叶子节点向上）
    dirs_to_delete.reverse();
    for dir in dirs_to_delete {
        if let Err(e) = fs::remove_dir(&dir) {
            // 如果直接删除失败，尝试使用 remove_dir_all
            if let Err(fallback_error) = remove_dir_all::remove_dir_all(&dir) {
                let error_msg = format!(
                    "Failed to remove directory {}: {} (and fallback also failed: {})",
                    dir.display(),
                    e,
                    fallback_error
                );
                return Err(anyhow::anyhow!(error_msg));
            }
        }
    }

    Ok(())
}

/// 检查路径是否为空目录
#[allow(dead_code)]
pub fn is_empty_dir(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }

    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => false, // 如果无法读取目录，则视为非空
    }
}
