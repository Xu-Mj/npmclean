# npmclean 技术规范

本文档提供了 npmclean 的详细技术规范，重点关注实现细节、数据结构、算法和架构决策。

## 核心数据结构

### 配置

```rust
pub struct Config {
    // 基本选项
    pub targets: Vec<String>,
    pub exclude: Vec<String>,
    pub recursive: bool,
    pub force: bool,
    pub dry_run: bool,
    pub stats: bool,
    pub verbose: bool,
    
    // 高级选项
    pub max_depth: Option<usize>,
    pub min_size: Option<u64>,
    pub threads: Option<usize>,
    pub timeout: Option<Duration>,
    
    // 平台特定选项
    pub platform_options: PlatformOptions,
}

pub struct PlatformOptions {
    pub windows: WindowsOptions,
    pub unix: UnixOptions,
}

pub struct WindowsOptions {
    pub use_native_api: bool,
    pub long_path_support: bool,
}

pub struct UnixOptions {
    pub follow_symlinks: bool,
}
```

### 项目模型

```rust
pub struct Project {
    pub path: PathBuf,
    pub project_type: ProjectType,
    pub package_info: Option<PackageInfo>,
    pub size_info: Option<SizeInfo>,
    pub detected_targets: Vec<CleanTarget>,
}

pub enum ProjectType {
    NodeJs,
    React,
    Vue,
    Angular,
    NextJs,
    NuxtJs,
    Unknown,
}

pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
}

pub struct CleanTarget {
    pub path: PathBuf,
    pub target_type: TargetType,
    pub size: Option<u64>,
}

pub enum TargetType {
    NodeModules,
    BuildDir,
    CacheDir,
    Coverage,
    Custom(String),
}

pub struct SizeInfo {
    pub total_size: u64,
    pub node_modules_size: u64,
    pub build_dirs_size: u64,
}
```

### 清理引擎

```rust
pub struct CleaningEngine {
    pub config: Config,
    pub strategies: Vec<Box<dyn CleaningStrategy>>,
    pub progress_reporter: Box<dyn ProgressReporter>,
}

pub trait CleaningStrategy: Send + Sync {
    fn can_handle(&self, path: &Path, platform: Platform) -> bool;
    fn clean(&self, path: &Path) -> Result<(), CleanError>;
}

pub trait ProgressReporter: Send + Sync {
    fn start(&mut self, total: usize);
    fn update(&mut self, current: usize, message: &str);
    fn finish(&mut self);
}
```

## 关键算法

### 项目检测

```rust
fn detect_projects(root_path: &Path, config: &Config) -> Vec<Project> {
    let mut projects = Vec::new();
    let mut visited_dirs = HashSet::new();
    
    // BFS 扫描，带深度限制
    let mut queue = VecDeque::new();
    queue.push_back((root_path.to_path_buf(), 0));
    
    while let Some((current_path, depth)) = queue.pop_front() {
        // 如果我们已经访问过这个目录或超过最大深度，则跳过
        if !visited_dirs.insert(current_path.clone()) 
           || config.max_depth.map_or(false, |max| depth > max) {
            continue;
        }
        
        // 检查这是否是项目目录
        if has_package_json(&current_path) {
            let project = analyze_project(&current_path, config);
            projects.push(project);
            
            // 如果非递归，则不探索项目的子目录
            if !config.recursive {
                continue;
            }
        }
        
        // 将子目录加入队列
        if let Ok(entries) = fs::read_dir(&current_path) {
            for entry in entries.filter_map(Result::ok) {
                if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                    queue.push_back((entry.path(), depth + 1));
                }
            }
        }
    }
    
    projects
}

fn analyze_project(path: &Path, config: &Config) -> Project {
    let package_info = parse_package_json(path);
    let project_type = determine_project_type(&package_info);
    let detected_targets = detect_clean_targets(path, &project_type, config);
    let size_info = if config.stats { calculate_size_info(&detected_targets) } else { None };
    
    Project {
        path: path.to_path_buf(),
        project_type,
        package_info: Some(package_info),
        size_info,
        detected_targets,
    }
}
```

### 目录大小计算

```rust
fn calculate_directory_size(path: &Path) -> Result<u64, io::Error> {
    let mut total_size = 0;
    let mut stack = Vec::new();
    stack.push(path.to_path_buf());
    
    while let Some(current_path) = stack.pop() {
        let metadata = fs::metadata(&current_path)?;
        
        if metadata.is_file() {
            total_size += metadata.len();
        } else if metadata.is_dir() {
            match fs::read_dir(&current_path) {
                Ok(entries) => {
                    for entry in entries.filter_map(Result::ok) {
                        stack.push(entry.path());
                    }
                }
                Err(_) => continue,
            }
        }
    }
    
    Ok(total_size)
}
```

### 并行目录删除

```rust
fn parallel_clean(targets: Vec<CleanTarget>, config: &Config) -> Result<CleanStats, CleanError> {
    let thread_count = config.threads.unwrap_or_else(|| 
        std::cmp::min(num_cpus::get(), targets.len()));
    
    let pool = ThreadPool::new(thread_count);
    let (tx, rx) = mpsc::channel();
    let targets_count = targets.len();
    let shared_config = Arc::new(config.clone());
    
    // 分配工作
    for target in targets {
        let tx = tx.clone();
        let config = shared_config.clone();
        
        pool.execute(move || {
            let result = clean_target(&target, &config);
            tx.send((target, result)).expect("通道发送失败");
        });
    }
    
    // 收集结果
    let mut stats = CleanStats::default();
    let mut errors = Vec::new();
    
    for _ in 0..targets_count {
        match rx.recv() {
            Ok((target, Ok(target_stats))) => {
                stats.merge(target_stats);
            }
            Ok((target, Err(e))) => {
                errors.push((target.path, e));
            }
            Err(_) => break,
        }
    }
    
    if errors.is_empty() {
        Ok(stats)
    } else {
        Err(CleanError::MultipleErrors(errors))
    }
}
```

## 平台特定优化

### Windows 优化

```rust
#[cfg(target_os = "windows")]
fn fast_windows_remove(path: &Path) -> Result<(), io::Error> {
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
    use winapi::um::winnt::{FILE_FLAG_DELETE_ON_CLOSE, DELETE};
    
    // 将路径转换为 Windows API 的宽字符串
    let mut wide_path: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide_path.push(0);
    
    // 使用 DELETE_ON_CLOSE 标志打开
    let handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            DELETE,
            0,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_DELETE_ON_CLOSE,
            std::ptr::null_mut(),
        )
    };
    
    if handle == winapi::um::handleapi::INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    
    unsafe {
        winapi::um::handleapi::CloseHandle(handle);
    }
    
    Ok(())
}
```

### Unix 优化

```rust
#[cfg(target_family = "unix")]
fn fast_unix_remove(path: &Path) -> Result<(), io::Error> {
    use std::process::Command;
    
    // 使用系统 rm 命令，通常已优化
    let output = Command::new("rm")
        .args(&["-rf", path.to_str().unwrap()])
        .output()?;
    
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "rm 命令失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }
    
    Ok(())
}
```

## 错误处理

```rust
#[derive(Debug, Error)]
pub enum CleanError {
    #[error("I/O 错误: {0}")]
    IoError(#[from] io::Error),
    
    #[error("权限被拒绝: {0}")]
    PermissionDenied(PathBuf),
    
    #[error("路径未找到: {0}")]
    NotFound(PathBuf),
    
    #[error("操作在 {0:?} 后超时")]
    Timeout(Duration),
    
    #[error("发生多个错误")]
    MultipleErrors(Vec<(PathBuf, CleanError)>),
    
    #[error("操作已取消")]
    Cancelled,
    
    #[error("未知错误: {0}")]
    Unknown(String),
}

// 带上下文的错误处理帮助函数
pub fn with_context<T, E, C>(result: Result<T, E>, context: C) -> Result<T, CleanError>
where
    E: Into<CleanError>,
    C: FnOnce() -> String,
{
    result.map_err(|e| {
        let err = e.into();
        match err {
            CleanError::Unknown(_) => CleanError::Unknown(context()),
            _ => err,
        }
    })
}
```

## 配置加载

```rust
fn load_config() -> Result<Config, ConfigError> {
    // 按优先级顺序加载配置:
    // 1. 命令行参数
    // 2. 项目配置文件
    // 3. 用户配置文件
    // 4. 默认配置
    
    let mut config = Config::default();
    
    // 从主目录加载用户配置
    if let Some(home_dir) = dirs::home_dir() {
        let user_config_path = home_dir.join(".npmcleanrc.yml");
        if user_config_path.exists() {
            let user_config = load_config_file(&user_config_path)?;
            config = merge_configs(config, user_config);
        }
    }
    
    // 加载项目配置
    let project_config_path = std::env::current_dir()?.join(".npmcleanrc.yml");
    if project_config_path.exists() {
        let project_config = load_config_file(&project_config_path)?;
        config = merge_configs(config, project_config);
    }
    
    // 解析命令行参数（最高优先级）
    let cli_config = parse_cli_args()?;
    config = merge_configs(config, cli_config);
    
    validate_config(&config)?;
    
    Ok(config)
}
```

## 插件系统设计

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&mut self, config: &Config) -> Result<(), PluginError>;
    
    // 生命周期钩子
    fn pre_scan(&self, context: &mut ScanContext) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn post_scan(&self, context: &mut ScanContext) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn pre_clean(&self, context: &mut CleanContext) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn post_clean(&self, context: &mut CleanContext) -> Result<(), PluginError> {
        Ok(())
    }
    
    // 扩展点
    fn provide_detectors(&self) -> Vec<Box<dyn ProjectDetector>> {
        Vec::new()
    }
    
    fn provide_strategies(&self) -> Vec<Box<dyn CleaningStrategy>> {
        Vec::new()
    }
}

pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }
    
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }
    
    pub fn init_all(&mut self, config: &Config) -> Result<(), PluginError> {
        for plugin in &mut self.plugins {
            plugin.init(config)?;
        }
        Ok(())
    }
    
    pub fn get_all_detectors(&self) -> Vec<Box<dyn ProjectDetector>> {
        self.plugins
            .iter()
            .flat_map(|p| p.provide_detectors())
            .collect()
    }
    
    pub fn get_all_strategies(&self) -> Vec<Box<dyn CleaningStrategy>> {
        self.plugins
            .iter()
            .flat_map(|p| p.provide_strategies())
            .collect()
    }
    
    // ... 调用插件钩子的其他方法
}
```

## 进度报告

```rust
pub struct ConsoleProgressReporter {
    pb: Option<ProgressBar>,
    total: usize,
}

impl ConsoleProgressReporter {
    pub fn new(quiet: bool) -> Self {
        Self { pb: None, total: 0 }
    }
}

impl ProgressReporter for ConsoleProgressReporter {
    fn start(&mut self, total: usize) {
        self.total = total;
        if total > 0 {
            let pb = ProgressBar::new(total as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("##-"));
            self.pb = Some(pb);
        }
    }
    
    fn update(&mut self, current: usize, message: &str) {
        if let Some(pb) = &self.pb {
            pb.set_position(current as u64);
            pb.set_message(message.to_string());
        }
    }
    
    fn finish(&mut self) {
        if let Some(pb) = &self.pb {
            pb.finish_with_message("完成!");
        }
    }
}
```

## 测试策略

项目将使用单元测试、集成测试和基于属性的测试的组合：

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_project_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("test-project");
        fs::create_dir(&project_dir).unwrap();
        
        // 创建模拟 package.json
        let package_json = r#"
        {
            "name": "test-project",
            "version": "1.0.0",
            "dependencies": {
                "react": "^17.0.0"
            }
        }
        "#;
        
        fs::write(project_dir.join("package.json"), package_json).unwrap();
        
        // 创建模拟 node_modules
        fs::create_dir(project_dir.join("node_modules")).unwrap();
        
        let config = Config::default();
        let projects = detect_projects(temp_dir.path(), &config);
        
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].project_type, ProjectType::React);
        assert_eq!(projects[0].detected_targets.len(), 1);
        assert_eq!(
            projects[0].detected_targets[0].target_type, 
            TargetType::NodeModules
        );
    }
}
```

### 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::process::Command;
    
    #[test]
    fn test_end_to_end_cleaning() {
        // 创建测试项目
        let temp_dir = tempfile::tempdir().unwrap();
        let output = Command::new("npm")
            .args(&["init", "-y"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();
            
        assert!(output.status.success());
        
        // 安装一些包
        let output = Command::new("npm")
            .args(&["install", "lodash"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();
            
        assert!(output.status.success());
        
        // 验证 node_modules 存在
        let node_modules_path = temp_dir.path().join("node_modules");
        assert!(node_modules_path.exists());
        
        // 运行我们的清理器
        let config = Config {
            force: true,
            ..Config::default()
        };
        
        let engine = CleaningEngine::new(config);
        let result = engine.clean(temp_dir.path());
        
        assert!(result.is_ok());
        
        // 验证 node_modules 已被删除
        assert!(!node_modules_path.exists());
    }
}
```

### 基于属性的测试

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_config_merge_properties(
            targets1 in prop::collection::vec(prop::string::string_regex("[a-z0-9_]+").unwrap(), 0..5),
            targets2 in prop::collection::vec(prop::string::string_regex("[a-z0-9_]+").unwrap(), 0..5),
            recursive1 in prop::bool::ANY,
            recursive2 in prop::bool::ANY,
        ) {
            let config1 = Config {
                targets: targets1.clone(),
                recursive: recursive1,
                ..Config::default()
            };
            
            let config2 = Config {
                targets: targets2.clone(),
                recursive: recursive2,
                ..Config::default()
            };
            
            let merged = merge_configs(config1, config2);
            
            // 第二个配置应该覆盖第一个
            prop_assert_eq!(merged.recursive, recursive2);
            
            // 对于集合，它们应该合并
            let expected_targets: Vec<_> = targets1.into_iter()
                .chain(targets2.into_iter())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
                
            prop_assert_eq!(merged.targets.len(), expected_targets.len());
            
            for target in &expected_targets {
                prop_assert!(merged.targets.contains(target));
            }
        }
    }
}
```

## 性能基准测试

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn benchmark_directory_size(c: &mut Criterion) {
        let temp_dir = tempfile::tempdir().unwrap();
        // 创建具有多个文件的测试目录结构
        create_test_directory_structure(&temp_dir.path());
        
        c.bench_function("calculate_directory_size", |b| {
            b.iter(|| {
                calculate_directory_size(black_box(temp_dir.path())).unwrap()
            })
        });
    }
    
    fn benchmark_project_detection(c: &mut Criterion) {
        let temp_dir = tempfile::tempdir().unwrap();
        // 创建多个测试项目
        create_test_projects(&temp_dir.path());
        
        let config = Config::default();
        
        c.bench_function("detect_projects", |b| {
            b.iter(|| {
                detect_projects(black_box(temp_dir.path()), 
                               black_box(&config))
            })
        });
    }
    
    criterion_group!(benches, benchmark_directory_size, benchmark_project_detection);
    criterion_main!(benches);
}
```

## 构建和打包

### Cargo.toml

```toml
[package]
name = "npmclean"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "一个快速且安全的 node_modules 和其他前端构建产物清理工具"
license = "MIT"
repository = "https://github.com/yourusername/npmclean"
readme = "README.md"
keywords = ["npm", "node", "cleaner", "node_modules", "tool"]
categories = ["command-line-utilities", "development-tools"]

[dependencies]
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"  # 添加用于 YAML 配置支持
walkdir = "2.3"
thiserror = "1.0"
anyhow = "1.0"
indicatif = "0.17"
console = "0.15"
remove_dir_all = "0.8"
num_cpus = "1.15"
dirs = "5.0"
globset = "0.4"

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["fileapi", "handleapi", "winnt"] }

[dev-dependencies]
tempfile = "3.5"
criterion = "0.4"
proptest = "1.1"

[[bench]]
name = "performance_benchmarks"
harness = false
```

### 二进制大小优化

```toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
opt-level = "z"
```

## 部署和分发

项目将通过多个渠道分发：

1. **Cargo (crates.io)**：Rust 用户的主要分发渠道

   ```bash
   cargo install npmclean
   ```

2. **预构建的二进制文件**：为非 Rust 用户提供：
   - Windows (x86_64, ARM64)
   - macOS (x86_64, ARM64)
   - Linux (x86_64, ARM64)

3. **npm 包**：为 JavaScript 开发者提供便利：

   ```bash
   npm install -g npmclean-cli
   ```

   （这将下载适合用户平台的二进制文件）

4. **Homebrew**：为 macOS 用户提供

   ```bash
   brew install npmclean
   ```

5. **Scoop/Chocolatey**：为 Windows 用户提供

## 安全考虑

1. **权限**：工具将在尝试删除文件之前检查适当的权限。

2. **路径遍历防止**：严格检查以防止清理意图范围之外的目录。

3. **安全的默认配置**：默认设置将是保守的。

4. **确认提示**：默认情况下，工具将在大量删除前要求确认。

5. **审计跟踪**：详细记录删除内容的日志。 