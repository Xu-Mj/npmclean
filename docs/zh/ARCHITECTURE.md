# npmclean 架构

本文档描述了 npmclean 的架构，重点关注模块化设计和扩展点。

## 架构概述

npmclean 遵循模块化、分层架构，具有清晰的职责分离：

```txt
┌─────────────────────────────────────────────────────────┐
│                     命令行界面                          │
└───────────────────────────┬─────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────┐
│                    配置层                               │
└───────────────────────────┬─────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────┐
│                       核心引擎                          │
└──┬───────────────────┬───────────────────┬──────────────┘
   │                   │                   │
┌──▼────────┐     ┌───▼───────┐      ┌────▼────┐
│  扫描器   │     │  清理器   │      │ 报告器  │
└──┬────────┘     └───┬───────┘      └────┬─────┘
   │                  │                   │
┌──▼────────┐     ┌───▼───────┐      ┌────▼────┐
│ 检测器    │     │ 策略      │      │ 格式化器 │
└───────────┘     └───────────┘      └───────────┘
```

### 核心组件

1. **命令行界面**
   - 解析命令行参数
   - 提供帮助信息
   - 初始入口点

2. **配置层**
   - 合并多个来源的配置
   - 处理默认值和验证
   - 向其他组件提供配置

3. **核心引擎**
   - 协调整个流程
   - 管理组件生命周期
   - 错误处理和报告

4. **扫描子系统**
   - 目录遍历
   - 项目检测
   - 大小计算

5. **清理子系统**
   - 目录删除策略
   - 安全检查
   - 性能优化

6. **报告子系统**
   - 进度报告
   - 统计信息收集
   - 输出格式化

## 模块化设计

npmclean 使用基于插件的架构，使得在不修改核心代码的情况下扩展功能成为可能。

### 扩展点

1. **项目检测器**
   - 检测特定项目类型
   - 每个检测器可以识别特定框架的目录
   - 通过 trait 实现注册

2. **清理策略**
   - 提供不同的目录清理方式
   - 平台特定优化
   - 基于能力和优先级选择

3. **输出格式化器**
   - 为不同输出格式化结果（控制台、JSON等）
   - 控制详细程度和样式
   - 支持机器可读输出

4. **配置提供者**
   - 从不同来源加载配置
   - 自定义配置格式
   - 优先级处理

## 组件交互

### 核心工作流程

```txt
┌─────────┐     ┌───────────┐     ┌─────────────┐     ┌───────┐     ┌─────────┐
│  解析   │────▶│  加载     │────▶│  扫描       │────▶│ 清理  │────▶│ 报告    │
│  参数   │     │  配置     │     │  项目       │     │       │     │ 结果    │
└─────────┘     └───────────┘     └─────────────┘     └───────┘     └─────────┘
                                        │                 ▲
                                        │                 │
                                        ▼                 │
                                  ┌──────────┐     ┌─────────────┐
                                  │ 分析     │────▶│  规划       │
                                  │ 项目     │     │  清理       │
                                  └──────────┘     └─────────────┘
```

### 扫描器与清理器通信

扫描器识别潜在的清理目标并将其传递给清理器：

```txt
┌───────────┐                           ┌───────────┐
│  扫描器   │                           │  清理器   │
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

## 包结构

代码库按照反映架构组件的模块组织：

```txt
src/
  ├── main.rs           # 入口点和依赖连接
  ├── cli.rs            # 命令行界面
  ├── config/           # 配置处理
  │   ├── mod.rs
  │   ├── loader.rs
  │   └── schema.rs
  ├── engine.rs         # 核心引擎
  ├── project/          # 项目模型和检测
  │   ├── mod.rs
  │   ├── detector.rs
  │   └── detectors/    # 框架特定检测器
  ├── cleaner/          # 目录删除
  │   ├── mod.rs
  │   ├── engine.rs
  │   └── strategies/   # 平台特定策略
  ├── scanner.rs        # 目录扫描
  ├── reporter/         # 输出和进度报告
  │   ├── mod.rs
  │   ├── progress.rs
  │   └── formatters/   # 输出格式化器
  ├── utils/            # 共享工具
  │   ├── fs.rs
  │   ├── size.rs
  │   └── display.rs
  └── plugins/          # 插件系统
      ├── mod.rs
      └── registry.rs
```

## 依赖关系图

核心模块依赖：

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

## 可扩展性机制

### 1. 基于特性的插件

npmclean 使用 traits 作为主要扩展机制：

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

### 2. 注册表模式

组件在注册表中注册，允许动态加载和优先级排序：

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

### 3. 策略模式

用于根据平台或上下文变化的行为：

```rust
pub fn clean_directory(path: &Path, config: &Config) -> Result<(), CleanError> {
    let platform = detect_platform();
    
    // 获取所有注册的策略
    let strategies = get_cleaning_strategies();
    
    // 找到第一个可以在此平台上处理此路径的策略
    for strategy in strategies {
        if strategy.can_handle(path, platform) {
            return strategy.clean(path);
        }
    }
    
    // 回退到默认策略
    default_clean_strategy(path)
}
```

### 4. 配置驱动行为

许多行为可以在不更改代码的情况下配置：

```rust
pub fn get_target_directories(config: &Config, project: &Project) -> Vec<PathBuf> {
    let mut targets = Vec::new();
    
    // 如果启用，添加 node_modules
    if config.clean_node_modules {
        targets.push(project.path.join("node_modules"));
    }
    
    // 如果启用，添加构建目录
    if config.clean_build_dirs {
        for build_dir in get_build_dirs_for_project(project) {
            targets.push(project.path.join(build_dir));
        }
    }
    
    // 添加自定义目标
    for custom_target in &config.custom_targets {
        targets.push(project.path.join(custom_target));
    }
    
    // 过滤排除的目标
    targets.into_iter()
        .filter(|path| !is_excluded(path, &config.exclude))
        .collect()
}
```

## 横切关注点

### 错误处理

使用 `thiserror` 库集中处理错误，为每个组件提供特定的错误类型：

```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("配置错误: {0}")]
    ConfigError(#[from] ConfigError),
    
    #[error("扫描错误: {0}")]
    ScanError(#[from] ScanError),
    
    #[error("清理错误: {0}")]
    CleanError(#[from] CleanError),
    
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
}
```

### 日志

在整个应用程序中实现结构化日志：

```rust
log::info!("开始在目录中扫描: {}", path.display());
log::debug!("找到项目: {} 类型 {:?}", project.name, project.project_type);
log::warn!("跳过不可读目录: {}", path.display());
```

### 并发

通过线程池管理线程以限制资源使用：

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
        // 将目标分配给工作线程
    }
}
```

## 未来规划

架构设计考虑了以下未来增强功能：

1. **正式插件系统**：在不修改核心代码的情况下添加外部插件
2. **远程清理**：支持在远程系统上清理
3. **GUI 界面**：在核心引擎之上添加图形界面
4. **IDE 集成**：为编辑器和 IDE 集成暴露 API
5. **遥测**：用于改进工具的可选使用统计

## 性能考虑

架构针对以下方面进行了优化：

1. **快速启动**：最小化初始化时间
2. **并行处理**：利用多核系统
3. **内存效率**：流式处理大型目录结构
4. **可取消操作**：响应用户中断

## 安全架构

安全性是设计的内置部分：

1. **路径验证**：防止意外删除系统目录
2. **权限检查**：确保在操作前有适当的访问权限
3. **安全默认值**：保守的默认设置
4. **沙盒操作**：将操作限制在目标目录树内 