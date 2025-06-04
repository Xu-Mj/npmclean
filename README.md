# npm-clean

A high-performance CLI tool for safely and efficiently cleaning `node_modules` directories and build artifacts in JavaScript/TypeScript projects.

[![Crates.io](https://img.shields.io/crates/v/npm-clean.svg)](https://crates.io/crates/npm-clean)
[![Build Status](https://github.com/yourusername/npm-clean/workflows/CI/badge.svg)](https://github.com/yourusername/npm-clean/actions)
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
git clone https://github.com/yourusername/npm-clean.git
cd npm-clean

# Build the project
cargo build --release

# The binary will be available at target/release/npm-clean
```

## Installation

### From Cargo (Recommended for Rust users)

```bash
cargo install npm-clean
```

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/yourusername/npm-clean/releases) page.

### From npm

```bash
npm install -g npm-clean-cli
```

### Using Homebrew (macOS)

```bash
brew install npm-clean
```

## Quick Start

Clean the current project:

```bash
npm-clean
```

Recursively clean all projects in a directory:

```bash
npm-clean -r /path/to/projects
```

Display what would be deleted without actually deleting:

```bash
npm-clean --dry-run
```

## Usage

```txt
USAGE:
    npm-clean [OPTIONS] [PATH]

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

npm-clean can be configured through command-line options or configuration files.

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
npm-clean --build
```

### Clean Only Node Modules

```bash
npm-clean --node-modules
```

### Clean All Projects Under a Directory with Statistics

```bash
npm-clean -r -s /path/to/projects
```

### Clean Specific Project with Custom Directories

```bash
npm-clean --include=".cache,.yarn-cache" /path/to/project
```

### Exclude Specific Directories

```bash
npm-clean --exclude="node_modules/some-large-pkg" /path/to/project
```

## Framework Detection

npm-clean automatically detects these framework types and their build directories:

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
- For very large directories, consider increasing thread count: `npm-clean --threads=8`
- On Windows, the tool automatically uses optimized deletion techniques

## Contributing

Contributions are welcome! Please check out our [Contributing Guide](docs/CONTRIBUTING.md) for details.

## Documentation

- [Design Document](docs/DESIGN.md)
- [Contributing Guide](docs/CONTRIBUTING.md)
- [Technical Specification](docs/TECHNICAL_SPEC.md)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgements

- Inspired by the need for a faster, safer way to clean node_modules
- Built with Rust 🦀 for performance and safety
