# npm-clean Architecture

This document describes the architecture of npm-clean, focusing on the modular design and extension points.

## Architectural Overview

npm-clean follows a modular, layered architecture with clear separation of concerns:

```txt
┌─────────────────────────────────────────────────────────┐
│                     CLI Interface                       │
└───────────────────────────┬─────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────┐
│                    Configuration Layer                  │
└───────────────────────────┬─────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────┐
│                       Core Engine                       │
└──┬───────────────────┬───────────────────┬──────────────┘
   │                   │                   │
┌──▼────────┐     ┌───▼───────┐      ┌────▼────┐
│  Scanner  │     │  Cleaner  │      │ Reporter │
└──┬────────┘     └───┬───────┘      └────┬─────┘
   │                  │                   │
┌──▼────────┐     ┌───▼───────┐      ┌────▼────┐
│ Detectors │     │ Strategies │      │ Formatters │
└───────────┘     └───────────┘      └───────────┘
```

### Key Components

1. **CLI Interface**
   - Parses command-line arguments
   - Provides help information
   - Initial entry point

2. **Configuration Layer**
   - Merges config from multiple sources
   - Handles defaults and validation
   - Provides configuration to other components

3. **Core Engine**
   - Orchestrates the entire process
   - Manages component lifecycle
   - Error handling and reporting

4. **Scanning Subsystem**
   - Directory traversal
   - Project detection
   - Size calculation

5. **Cleaning Subsystem**
   - Directory removal strategies
   - Safety checks
   - Performance optimizations

6. **Reporting Subsystem**
   - Progress reporting
   - Statistics collection
   - Output formatting

## Modular Design

npm-clean uses a plugin-based architecture to enable extending functionality without modifying core code.

### Extension Points

1. **Project Detectors**
   - Detect specific project types
   - Each detector can identify framework-specific directories
   - Registered via trait implementations

2. **Cleaning Strategies**
   - Provide different ways to clean directories
   - Platform-specific optimizations
   - Selected based on capability and priority

3. **Output Formatters**
   - Format results for different outputs (console, JSON, etc.)
   - Control verbosity and styling
   - Support for machine-readable output

4. **Configuration Providers**
   - Load configuration from different sources
   - Custom configuration formats
   - Precedence handling

## Component Interactions

### Core Workflow

```txt
┌─────────┐     ┌───────────┐     ┌─────────────┐     ┌───────┐     ┌─────────┐
│  Parse  │────▶│  Load     │────▶│  Scan for   │────▶│ Clean │────▶│ Report  │
│  Args   │     │  Config   │     │  Projects   │     │       │     │ Results │
└─────────┘     └───────────┘     └─────────────┘     └───────┘     └─────────┘
                                        │                 ▲
                                        │                 │
                                        ▼                 │
                                  ┌──────────┐     ┌─────────────┐
                                  │ Analyze  │────▶│  Plan       │
                                  │ Projects │     │  Cleaning   │
                                  └──────────┘     └─────────────┘
```

### Scanner to Cleaner Communication

The Scanner identifies potential cleaning targets and passes them to the Cleaner:

```txt
┌───────────┐                           ┌───────────┐
│  Scanner  │                           │  Cleaner  │
└─────┬─────┘                           └─────┬─────┘
      │                                       │
      │ find_projects()                       │
      ├───────────────────┐                   │
      │                   │                   │
      │                   │                   │
      │                   │                   │
      │ return Project[]  │                   │
      │◀──────────────────┘                   │
      │                                       │
      │ clean_projects(Project[])             │
      ├───────────────────────────────────────▶
      │                                       │
      │                                       │ for each project:
      │                                       │  clean_targets()
      │                                       │
      │                                       │
      │                                       │
      │         return CleanResult            │
      │◀──────────────────────────────────────┘
      │                                       │
```

## Package Structure

The codebase is organized into modules that reflect the architectural components:

```txt
src/
  ├── main.rs           # Entry point and dependency wiring
  ├── cli.rs            # Command-line interface
  ├── config/           # Configuration handling
  │   ├── mod.rs
  │   ├── loader.rs
  │   └── schema.rs
  ├── engine.rs         # Core engine
  ├── project/          # Project model and detection
  │   ├── mod.rs
  │   ├── detector.rs
  │   └── detectors/    # Framework-specific detectors
  ├── cleaner/          # Directory removal
  │   ├── mod.rs
  │   ├── engine.rs
  │   └── strategies/   # Platform-specific strategies
  ├── scanner.rs        # Directory scanning
  ├── reporter/         # Output and progress reporting
  │   ├── mod.rs
  │   ├── progress.rs
  │   └── formatters/   # Output formatters
  ├── utils/            # Shared utilities
  │   ├── fs.rs
  │   ├── size.rs
  │   └── display.rs
  └── plugins/          # Plugin system
      ├── mod.rs
      └── registry.rs
```

## Dependency Graph

Core modules dependencies:

```txt
cli.rs ──────┐
             │
             ▼
config/ ─────────▶ engine.rs ◀───── plugins/
             ▲         │
             │         │
             │         ▼
scanner.rs ──┴──▶ project/ ◀─┬─── cleaner/
                             │
                      reporter/ ─┘
```

## Extensibility Mechanisms

### 1. Trait-Based Plugins

npm-clean uses traits as the primary extension mechanism:

```rust
pub trait ProjectDetector: Send + Sync {
    fn detect(&self, path: &Path) -> bool;
    fn get_build_dirs(&self) -> Vec<String>;
    fn get_priority(&self) -> u8 { 100 }
}

pub trait CleaningStrategy: Send + Sync {
    fn can_handle(&self, path: &Path, platform: Platform) -> bool;
    fn clean(&self, path: &Path) -> Result<(), CleanError>;
    fn get_priority(&self) -> u8 { 100 }
}

pub trait OutputFormatter: Send + Sync {
    fn format_results(&self, results: &CleanResults) -> String;
    fn supports_format(&self, format: OutputFormat) -> bool;
}
```

### 2. Registry Pattern

Components are registered in registries that allow dynamic loading and prioritization:

```rust
pub struct Registry<T> {
    items: Vec<Box<T>>,
}

impl<T: 'static> Registry<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    
    pub fn register(&mut self, item: Box<T>) {
        self.items.push(item);
    }
    
    pub fn get_all(&self) -> &[Box<T>] {
        &self.items
    }
}
```

### 3. Strategy Pattern

For behaviors that vary by platform or context:

```rust
pub fn clean_directory(path: &Path, config: &Config) -> Result<(), CleanError> {
    let platform = detect_platform();
    
    // Get all registered strategies
    let strategies = get_cleaning_strategies();
    
    // Find the first strategy that can handle this path on this platform
    for strategy in strategies {
        if strategy.can_handle(path, platform) {
            return strategy.clean(path);
        }
    }
    
    // Fall back to default strategy
    default_clean_strategy(path)
}
```

### 4. Configuration-Driven Behavior

Many behaviors are configurable without code changes:

```rust
pub fn get_target_directories(config: &Config, project: &Project) -> Vec<PathBuf> {
    let mut targets = Vec::new();
    
    // Add node_modules if enabled
    if config.clean_node_modules {
        targets.push(project.path.join("node_modules"));
    }
    
    // Add build dirs if enabled
    if config.clean_build_dirs {
        for build_dir in get_build_dirs_for_project(project) {
            targets.push(project.path.join(build_dir));
        }
    }
    
    // Add custom targets
    for custom_target in &config.custom_targets {
        targets.push(project.path.join(custom_target));
    }
    
    // Filter excluded targets
    targets.into_iter()
        .filter(|path| !is_excluded(path, &config.exclude))
        .collect()
}
```

## Cross-Cutting Concerns

### Error Handling

Error handling is centralized using the `thiserror` crate, with specific error types for each component:

```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),
    
    #[error("Scanning error: {0}")]
    ScanError(#[from] ScanError),
    
    #[error("Cleaning error: {0}")]
    CleanError(#[from] CleanError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### Logging

Structured logging is implemented throughout the application:

```rust
log::info!("Starting scan in directory: {}", path.display());
log::debug!("Found project: {} of type {:?}", project.name, project.project_type);
log::warn!("Skipping unreadable directory: {}", path.display());
```

### Concurrency

Thread management is handled through a thread pool to limit resource usage:

```rust
pub struct CleaningWorker {
    thread_pool: ThreadPool,
    max_threads: usize,
    // ...
}

impl CleaningWorker {
    pub fn new(config: &Config) -> Self {
        let max_threads = config.threads.unwrap_or_else(|| num_cpus::get());
        Self {
            thread_pool: ThreadPool::new(max_threads),
            max_threads,
        }
    }
    
    pub fn clean_targets(&self, targets: Vec<CleanTarget>) -> Results {
        // Distribute targets to worker threads
    }
}
```

## Future-Proofing

The architecture is designed with these future enhancements in mind:

1. **Formal Plugin System**: Adding external plugins without modifying core code
2. **Remote Cleaning**: Supporting cleaning on remote systems
3. **GUI Interface**: Adding graphical interfaces on top of the core engine
4. **IDE Integration**: Exposing APIs for integration with editors and IDEs
5. **Telemetry**: Optional usage statistics for improving the tool

## Performance Considerations

The architecture is optimized for:

1. **Fast Startup**: Minimizing initialization time
2. **Parallel Processing**: Leveraging multi-core systems
3. **Memory Efficiency**: Streaming large directory structures
4. **Cancelable Operations**: Responsive to user interruption

## Security Architecture

Security is built into the design:

1. **Path Validation**: Preventing accidental deletion of system directories
2. **Permissions Checking**: Ensuring proper access rights before operations
3. **Safe Defaults**: Conservative default settings
4. **Sandboxed Operations**: Limiting operations to the target directory tree
