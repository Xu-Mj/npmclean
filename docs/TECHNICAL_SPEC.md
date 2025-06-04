# npmclean Technical Specification

This document provides detailed technical specifications for npmclean, focusing on implementation details, data structures, algorithms, and architectural decisions.

## Core Data Structures

### Configuration

```rust
pub struct Config {
    // Basic options
    pub targets: Vec<String>,
    pub exclude: Vec<String>,
    pub recursive: bool,
    pub force: bool,
    pub dry_run: bool,
    pub stats: bool,
    pub verbose: bool,
    
    // Advanced options
    pub max_depth: Option<usize>,
    pub min_size: Option<u64>,
    pub threads: Option<usize>,
    pub timeout: Option<Duration>,
    
    // Platform-specific options
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

### Project Model

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

### Cleaning Engine

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

## Key Algorithms

### Project Detection

```rust
fn detect_projects(root_path: &Path, config: &Config) -> Vec<Project> {
    let mut projects = Vec::new();
    let mut visited_dirs = HashSet::new();
    
    // BFS scan with depth limit
    let mut queue = VecDeque::new();
    queue.push_back((root_path.to_path_buf(), 0));
    
    while let Some((current_path, depth)) = queue.pop_front() {
        // Skip if we've visited this directory or exceeded max depth
        if !visited_dirs.insert(current_path.clone()) 
           || config.max_depth.map_or(false, |max| depth > max) {
            continue;
        }
        
        // Check if this is a project directory
        if has_package_json(&current_path) {
            let project = analyze_project(&current_path, config);
            projects.push(project);
            
            // If not recursive, don't explore subdirectories of a project
            if !config.recursive {
                continue;
            }
        }
        
        // Enqueue subdirectories
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

### Directory Size Calculation

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

### Parallel Directory Removal

```rust
fn parallel_clean(targets: Vec<CleanTarget>, config: &Config) -> Result<CleanStats, CleanError> {
    let thread_count = config.threads.unwrap_or_else(|| 
        std::cmp::min(num_cpus::get(), targets.len()));
    
    let pool = ThreadPool::new(thread_count);
    let (tx, rx) = mpsc::channel();
    let targets_count = targets.len();
    let shared_config = Arc::new(config.clone());
    
    // Distribute work
    for target in targets {
        let tx = tx.clone();
        let config = shared_config.clone();
        
        pool.execute(move || {
            let result = clean_target(&target, &config);
            tx.send((target, result)).expect("Channel send failed");
        });
    }
    
    // Collect results
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

## Platform-Specific Optimizations

### Windows Optimizations

```rust
#[cfg(target_os = "windows")]
fn fast_windows_remove(path: &Path) -> Result<(), io::Error> {
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
    use winapi::um::winnt::{FILE_FLAG_DELETE_ON_CLOSE, DELETE};
    
    // Convert path to wide string for Windows API
    let mut wide_path: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide_path.push(0);
    
    // Open with DELETE_ON_CLOSE flag
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

### Unix Optimizations

```rust
#[cfg(target_family = "unix")]
fn fast_unix_remove(path: &Path) -> Result<(), io::Error> {
    use std::process::Command;
    
    // Use system rm command which is often optimized
    let output = Command::new("rm")
        .args(&["-rf", path.to_str().unwrap()])
        .output()?;
    
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "rm command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }
    
    Ok(())
}
```

## Error Handling

```rust
#[derive(Debug, Error)]
pub enum CleanError {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),
    
    #[error("Path not found: {0}")]
    NotFound(PathBuf),
    
    #[error("Operation timed out after {0:?}")]
    Timeout(Duration),
    
    #[error("Multiple errors occurred")]
    MultipleErrors(Vec<(PathBuf, CleanError)>),
    
    #[error("Operation cancelled")]
    Cancelled,
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

// Helper for handling errors with context
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

## Configuration Loading

```rust
fn load_config() -> Result<Config, ConfigError> {
    // Load config in order of precedence:
    // 1. Command line args
    // 2. Project config file
    // 3. User config file
    // 4. Default config
    
    let mut config = Config::default();
    
    // Load user config from home directory
    if let Some(home_dir) = dirs::home_dir() {
        let user_config_path = home_dir.join(".npmcleanrc.yml");
        if user_config_path.exists() {
            let user_config = load_config_file(&user_config_path)?;
            config = merge_configs(config, user_config);
        }
    }
    
    // Load project config
    let project_config_path = std::env::current_dir()?.join(".npmcleanrc.yml");
    if project_config_path.exists() {
        let project_config = load_config_file(&project_config_path)?;
        config = merge_configs(config, project_config);
    }
    
    // Parse command line args (highest priority)
    let cli_config = parse_cli_args()?;
    config = merge_configs(config, cli_config);
    
    validate_config(&config)?;
    
    Ok(config)
}
```

## Plugin System Design

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&mut self, config: &Config) -> Result<(), PluginError>;
    
    // Lifecycle hooks
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
    
    // Extension points
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
    
    // ... other methods to call plugin hooks
}
```

## Progress Reporting

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
            pb.finish_with_message("Done!");
        }
    }
}
```

## Testing Strategy

The project will use a combination of unit tests, integration tests, and property-based tests:

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_project_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("test-project");
        fs::create_dir(&project_dir).unwrap();
        
        // Create mock package.json
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
        
        // Create mock node_modules
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

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::process::Command;
    
    #[test]
    fn test_end_to_end_cleaning() {
        // Create a test project
        let temp_dir = tempfile::tempdir().unwrap();
        let output = Command::new("npm")
            .args(&["init", "-y"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();
            
        assert!(output.status.success());
        
        // Install some packages
        let output = Command::new("npm")
            .args(&["install", "lodash"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();
            
        assert!(output.status.success());
        
        // Verify node_modules exists
        let node_modules_path = temp_dir.path().join("node_modules");
        assert!(node_modules_path.exists());
        
        // Run our cleaner
        let config = Config {
            force: true,
            ..Config::default()
        };
        
        let engine = CleaningEngine::new(config);
        let result = engine.clean(temp_dir.path());
        
        assert!(result.is_ok());
        
        // Verify node_modules is gone
        assert!(!node_modules_path.exists());
    }
}
```

### Property-Based Testing

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
            
            // Second config should override first
            prop_assert_eq!(merged.recursive, recursive2);
            
            // For collections, they should be combined
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

## Performance Benchmarks

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn benchmark_directory_size(c: &mut Criterion) {
        let temp_dir = tempfile::tempdir().unwrap();
        // Create test directory structure with many files
        create_test_directory_structure(&temp_dir.path());
        
        c.bench_function("calculate_directory_size", |b| {
            b.iter(|| {
                calculate_directory_size(black_box(temp_dir.path())).unwrap()
            })
        });
    }
    
    fn benchmark_project_detection(c: &mut Criterion) {
        let temp_dir = tempfile::tempdir().unwrap();
        // Create multiple test projects
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

## Build and Packaging

### Cargo.toml

```toml
[package]
name = "npmclean"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "A fast and safe cleaner for node_modules and other frontend build artifacts"
license = "MIT"
repository = "https://github.com/yourusername/npmclean"
readme = "README.md"
keywords = ["npm", "node", "cleaner", "node_modules", "tool"]
categories = ["command-line-utilities", "development-tools"]

[dependencies]
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"  # Added for YAML configuration support
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

### Binary Size Optimization

```toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
opt-level = "z"
```

## Deployment and Distribution

The project will be distributed through multiple channels:

1. **Cargo (crates.io)**: Primary distribution for Rust users

   ```bash
   cargo install npmclean
   ```

2. **Pre-built Binaries**: For non-Rust users, binaries will be provided for:
   - Windows (x86_64, ARM64)
   - macOS (x86_64, ARM64)
   - Linux (x86_64, ARM64)

3. **npm Package**: For convenience for JavaScript developers:

   ```bash
   npm install -g npmclean-cli
   ```

   (This will download the appropriate binary for the user's platform)

4. **Homebrew**: For macOS users

   ```bash
   brew install npmclean
   ```

5. **Scoop/Chocolatey**: For Windows users

## Security Considerations

1. **Permissions**: The tool will check for appropriate permissions before attempting to delete files.

2. **Path Traversal Prevention**: Strict checking to prevent cleaning directories outside the intended scope.

3. **Safe Default Configuration**: Default settings will be conservative.

4. **Confirmation Prompts**: By default, the tool will require confirmation before bulk deletion.

5. **Audit Trail**: Detailed logs of what was deleted.
