# 为 npmclean 做贡献

感谢您有兴趣为 npmclean 做贡献！本文档提供了有关如何为项目做贡献的指南和信息。

## 项目哲学

npmclean 旨在做到：

- **快速**：优化性能，特别是在 Windows 上
- **安全**：防止意外删除重要文件
- **可扩展**：易于添加新功能和集成
- **用户友好**：带有合理默认值的简单命令行界面

## 入门

### 先决条件

- Rust 工具链（最新稳定版）
- Node.js/npm（用于测试）

### 设置

1. Fork 并克隆仓库
2. 安装依赖：`cargo build`
3. 运行测试：`cargo test`

## 项目结构

请查看 [DESIGN.md](./DESIGN.md) 文档，了解项目结构和架构的详细概述。

## 扩展点

npmclean 的设计是可扩展的。以下是主要扩展点：

### 1. 项目检测器

项目检测器用于识别和分析 JavaScript/TypeScript 项目。

要添加新的项目检测器：

1. 在 `src/project/detectors/` 中实现 `ProjectDetector` trait
2. 在 `src/project/mod.rs` 中的检测器注册表中注册它

示例：

```rust
pub struct NextJsDetector;

impl ProjectDetector for NextJsDetector {
    fn detect(&self, path: &Path) -> bool {
        // 检查这是否是 Next.js 项目
        // 例如，检查 next.config.js 或依赖中的 next
    }
    
    fn get_build_dirs(&self) -> Vec<String> {
        // 返回 Next.js 特定目录
        vec![".next".to_string(), "out".to_string()]
    }
}

// 在 detector_registry 中注册：
pub fn create_detector_registry() -> Vec<Box<dyn ProjectDetector>> {
    vec![
        Box::new(DefaultDetector::new()),
        Box::new(ReactDetector::new()),
        Box::new(NextJsDetector::new()),  // 在这里添加你的检测器
    ]
}
```

### 2. 清理策略

清理策略定义了应该如何删除目录。

要添加新的清理策略：

1. 在 `src/cleaner/strategies/` 中实现 `CleaningStrategy` trait
2. 在 `src/cleaner/mod.rs` 中的策略注册表中注册它

示例：

```rust
pub struct FastWindowsStrategy;

impl CleaningStrategy for FastWindowsStrategy {
    fn can_handle(&self, path: &Path, platform: Platform) -> bool {
        platform == Platform::Windows && /* 其他条件 */
    }
    
    fn clean(&self, path: &Path) -> Result<(), CleanError> {
        // 实现 Windows 优化的目录删除
    }
}

// 注册：
pub fn create_strategy_registry() -> Vec<Box<dyn CleaningStrategy>> {
    vec![
        Box::new(DefaultStrategy::new()),
        Box::new(FastWindowsStrategy::new()),  // 在这里添加你的策略
    ]
}
```

### 3. 配置扩展

要添加新的配置选项：

1. 更新 `src/config/schema.rs` 中的配置模式
2. 在 `src/config/loader.rs` 中添加对新选项的处理
3. 在你的功能实现中使用新选项

### 4. 命令行命令

要添加新的命令或标志：

1. 更新 `src/cli.rs` 中的 CLI 定义
2. 在 `src/main.rs` 中添加对新命令的处理

## 插件（未来）

未来，npmclean 将支持一个正式的插件系统。架构正在考虑这一点设计。

## 代码风格指南

- 遵循 Rust 标准命名约定
- 使用有意义的变量和函数名
- 使用 rustdoc 注释文档化公共 API
- 为新功能编写单元测试
- 使用 `thiserror` 库进行错误处理
- 使用 `rustfmt` 格式化代码

## Pull Request 流程

1. 为你的功能或错误修复创建一个新分支
2. 实现你的更改，包括测试
3. 根据需要更新文档
4. 运行 `cargo fmt` 和 `cargo clippy` 确保代码质量
5. 提交一个清晰描述更改的 PR
6. 处理审查评论

## 发布流程

1. 在 `Cargo.toml` 中更新版本
2. 更新 CHANGELOG.md
3. 在 GitHub 上创建一个新发布
4. 发布到 crates.io

## 沟通

- 对于 bug 和功能，在 GitHub 上开 issue
- 对于问题，使用 GitHub Discussions

感谢您为 npmclean 做贡献！ 