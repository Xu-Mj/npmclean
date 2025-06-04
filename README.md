# npmclean

[English](README.md) | [简体中文](README.zh.md)

A high-performance CLI tool for safely and efficiently cleaning `node_modules` directories and build artifacts in JavaScript/TypeScript projects.

[![Crates.io](https://img.shields.io/crates/v/npmclean.svg)](https://crates.io/crates/npmclean)
[![Build Status](https://github.com/Xu-Mj/npmclean/workflows/CI/badge.svg)](https://github.com/Xu-Mj/npmclean/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Fast:** Optimized for performance, especially on Windows where deleting node_modules is notoriously slow
- **Smart:** Automatically detects project types and their build directories
- **Safe:** Confirms before deletion and supports dry-run mode
- **Efficient:** Parallel processing for batch operations
- **Flexible:** Customizable targets, recursive mode, and various configuration options
- **Cross-platform:** Works on Windows, macOS, and Linux

## Building from Source

To build from source, you'll need Rust installed. Then:

```bash
# Clone the repository
git clone https://github.com/yourusername/npmclean.git
cd npmclean

# Build the project
cargo build --release

# The binary will be available at target/release/npmclean
```

## Installation

### From Cargo (Recommended for Rust users)

```bash
cargo install npmclean
```

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/yourusername/npmclean/releases) page.

### From npm

```bash
npm install -g npmclean-cli
```

### Using Homebrew (macOS)

```bash
brew install npmclean
```

## Quick Start

Clean the current project:

```bash
npmclean
```

Recursively clean all projects in a directory:

```bash
npmclean -r /path/to/projects
```

Display what would be deleted without actually deleting:

```bash
npmclean --dry-run
```

## Usage

```txt
USAGE:
    npmclean [OPTIONS] [PATH]

ARGS:
    <PATH>    Project or directory path, defaults to current directory

OPTIONS:
    -r, --recursive       Recursively find and clean projects in subdirectories
    -f, --force           Skip confirmation prompts
    -d, --dry-run         Show what would be deleted without deleting
    -c, --config <FILE>   Use specific config file
    -n, --node-modules    Clean only node_modules directories
    -b, --build           Clean only build directories
    --include <DIRS>      Additional directories to clean (comma-separated)
    --exclude <DIRS>      Directories to exclude (comma-separated)
    -s, --stats           Show space-saving statistics
    -v, --verbose         Display detailed output
    -h, --help            Show help information
```

## Configuration

npmclean can be configured through command-line options or configuration files.

### Configuration File

Create a `.npmcleanrc.yml` or `npmclean.config.yml` in your project directory or home directory:

```yaml
# Target directories to clean
targets:
  - node_modules
  - dist
  - build
  - .next
  - coverage

# Directories to exclude from cleaning
exclude:
  - some-special-module

# General options
confirmDelete: true
stats: true
recursive: false
```

## Examples

### Clean Only Build Directories

```bash
npmclean --build
```

### Clean Only Node Modules

```bash
npmclean --node-modules
```

### Clean All Projects Under a Directory with Statistics

```bash
npmclean -r -s /path/to/projects
```

### Clean Specific Project with Custom Directories

```bash
npmclean --include=".cache,.yarn-cache" /path/to/project
```

### Exclude Specific Directories

```bash
npmclean --exclude="node_modules/some-large-pkg" /path/to/project
```

## Framework Detection

npmclean automatically detects these framework types and their build directories:

| Framework | Detected Build Directories |
|-----------|---------------------------|
| React     | build, dist               |
| Vue       | dist                      |
| Angular   | dist                      |
| Next.js   | .next, out                |
| Nuxt.js   | .nuxt, dist              |
| Default   | dist, build, out          |

## Performance Tips

- Use the recursive mode (`-r`) to clean multiple projects at once
- For very large directories, consider increasing thread count: `npmclean --threads=8`
- On Windows, the tool automatically uses optimized deletion techniques

## Contributing

Contributions are welcome! Please check out our [Contributing Guide](docs/CONTRIBUTING.md) for details.

## Documentation

- [Design Document](docs/DESIGN.md)
- [Contributing Guide](docs/CONTRIBUTING.md)
- [Technical Specification](docs/TECHNICAL_SPEC.md)
- [Architecture Document](docs/ARCHITECTURE.md)

### 中文文档

- [设计文档](docs/zh/DESIGN.md)
- [贡献指南](docs/zh/CONTRIBUTING.md)
- [技术规范](docs/zh/TECHNICAL_SPEC.md)
- [架构文档](docs/zh/ARCHITECTURE.md)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgements

- Inspired by the need for a faster, safer way to clean node_modules
- Built with Rust 🦀 for performance and safety
