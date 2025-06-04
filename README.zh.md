# npmclean

[English](README.md) | [简体中文](README.zh.md)

一款高性能命令行工具，用于安全高效地清理 JavaScript/TypeScript 项目中的 `node_modules` 目录和构建产物。

[![Crates.io](https://img.shields.io/crates/v/npmclean.svg)](https://crates.io/crates/npmclean)
[![Build Status](https://github.com/Xu-Mj/npmclean/workflows/CI/badge.svg)](https://github.com/Xu-Mj/npmclean/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 特性

- **快速：** 针对性能优化，特别是在删除 node_modules 通常较慢的 Windows 系统上
- **智能：** 自动检测项目类型及其构建目录
- **安全：** 删除前确认，支持干运行模式
- **高效：** 批量操作的并行处理
- **灵活：** 可自定义目标，递归模式，以及各种配置选项
- **跨平台：** 适用于 Windows、macOS 和 Linux

## 从源代码构建

要从源代码构建，您需要安装 Rust。然后：

```bash
# 克隆仓库
git clone https://github.com/yourusername/npmclean.git
cd npmclean

# 构建项目
cargo build --release

# 二进制文件将位于 target/release/npmclean
```

## 安装

### 通过 Cargo (推荐给 Rust 用户)

```bash
cargo install npmclean
```

### 预构建二进制文件

从 [Releases](https://github.com/yourusername/npmclean/releases) 页面下载适用于您平台的最新版本。

### 通过 npm

```bash
npm install -g npmclean-cli
```

### 通过 Homebrew (macOS)

```bash
brew install npmclean
```

## 快速开始

清理当前项目：

```bash
npmclean
```

递归清理目录中的所有项目：

```bash
npmclean -r /path/to/projects
```

显示将要删除的内容但不实际删除：

```bash
npmclean --dry-run
```

## 用法

```txt
用法:
    npmclean [选项] [路径]

参数:
    <路径>    项目或目录路径，默认为当前目录

选项:
    -r, --recursive       递归查找并清理子目录中的项目
    -f, --force           跳过确认提示
    -d, --dry-run         显示将被删除的内容而不实际删除
    -c, --config <文件>   使用指定配置文件
    -n, --node-modules    仅清理 node_modules 目录
    -b, --build           仅清理构建目录
    --include <目录>      额外清理的目录（逗号分隔）
    --exclude <目录>      排除的目录（逗号分隔）
    -s, --stats           显示节省空间的统计信息
    -v, --verbose         显示详细输出
    -h, --help            显示帮助信息
```

## 配置

npmclean 可以通过命令行选项或配置文件进行配置。

### 配置文件

在您的项目目录或主目录中创建 `.npmcleanrc.yml` 或 `npmclean.config.yml`：

```yaml
# 要清理的目标目录
targets:
  - node_modules
  - dist
  - build
  - .next
  - coverage

# 从清理中排除的目录
exclude:
  - some-special-module

# 一般选项
confirmDelete: true
stats: true
recursive: false
```

## 示例

### 仅清理构建目录

```bash
npmclean --build
```

### 仅清理 Node 模块

```bash
npmclean --node-modules
```

### 清理目录下的所有项目并显示统计信息

```bash
npmclean -r -s /path/to/projects
```

### 清理特定项目并包含自定义目录

```bash
npmclean --include=".cache,.yarn-cache" /path/to/project
```

### 排除特定目录

```bash
npmclean --exclude="node_modules/some-large-pkg" /path/to/project
```

## 框架检测

npmclean 自动检测以下框架类型及其构建目录：

| 框架     | 检测到的构建目录 |
|----------|-----------------|
| React    | build, dist     |
| Vue      | dist            |
| Angular  | dist            |
| Next.js  | .next, out      |
| Nuxt.js  | .nuxt, dist     |
| 默认     | dist, build, out |

## 性能提示

- 使用递归模式 (`-r`) 一次清理多个项目
- 对于非常大的目录，可以考虑增加线程数量：`npmclean --threads=8`
- 在 Windows 上，工具会自动使用优化的删除技术

## 贡献

欢迎贡献！请查阅我们的[贡献指南](docs/zh/CONTRIBUTING.md)了解详情。

## 文档

### 中文文档

- [设计文档](docs/zh/DESIGN.md)
- [贡献指南](docs/zh/CONTRIBUTING.md)
- [技术规范](docs/zh/TECHNICAL_SPEC.md)
- [架构文档](docs/zh/ARCHITECTURE.md)

### 英文文档

- [Design Document](docs/DESIGN.md)
- [Contributing Guide](docs/CONTRIBUTING.md)
- [Technical Specification](docs/TECHNICAL_SPEC.md)
- [Architecture Document](docs/ARCHITECTURE.md)

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 致谢

- 灵感来源于对更快、更安全地清理 node_modules 的需求
- 使用 Rust 🦀 构建，以获得性能和安全性 