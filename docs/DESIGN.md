# npm-clean Design Document

## Overview

npm-clean is a high-performance CLI tool designed to efficiently clean node_modules directories and build artifacts in JavaScript/TypeScript projects. Built with Rust, it focuses on speed, safety, and user experience.

## Core Functionality

1. **Basic Cleaning**
   - Remove node_modules directories
   - Remove build directories (dist, build, out, etc.)
   - Support for custom cleaning targets

2. **Project Detection**
   - Identify projects via package.json presence
   - Auto-detect project type (React, Vue, Next.js, etc.)
   - Framework-specific cleaning strategies

3. **Batch Processing**
   - Recursively scan directories for projects
   - Sort projects by size for prioritized cleaning
   - Parallel processing for multiple projects

4. **Safety Mechanisms**
   - Pre-deletion size calculation and display
   - Confirmation prompts to prevent accidents
   - Dry-run mode for previewing operations

## CLI Interface

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

## Configuration System

Configuration can be defined through:

- Command line arguments (highest priority)
- Project-level config (.npmcleanrc.yml or npmclean.config.yml)
- User-level config (~/.npmcleanrc.yml)
- Default built-in configuration (lowest priority)

Example config file:

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

## Extension Points

The tool is designed with several extension points to facilitate future enhancements:

1. **Plugin System**
   - Support for custom project detectors
   - Support for custom cleaning strategies
   - Hook system for pre/post cleaning operations

2. **Project Type Detection**
   - Extensible detector interface
   - Registry of framework-specific detectors
   - Custom detection rules

3. **Directory Filters**
   - Custom filter rules via configuration
   - Programmable filters via plugins

4. **Output Formatters**
   - Extensible output formatting system
   - Support for different output formats (text, JSON, etc.)

## Technical Implementation

### Project Structure

```txt
src/
  ├── main.rs           # Entry point
  ├── cli.rs            # CLI argument processing
  ├── config/           # Configuration handling
  │   ├── mod.rs        # Config module exports
  │   ├── loader.rs     # Config loading
  │   └── schema.rs     # Config schema validation
  ├── project/          # Project detection & analysis
  │   ├── mod.rs        # Project module exports
  │   ├── detector.rs   # Project detection logic
  │   └── analyzers/    # Framework-specific analyzers
  ├── cleaner/          # Cleaning operations
  │   ├── mod.rs        # Cleaner module exports
  │   ├── engine.rs     # Cleaning orchestration
  │   └── strategies/   # Cleaning strategies
  ├── scanner.rs        # Directory scanning logic
  ├── plugins/          # Plugin system
  │   ├── mod.rs        # Plugin module exports
  │   └── registry.rs   # Plugin registration
  ├── utils/
  │   ├── fs.rs         # File system operations
  │   ├── size.rs       # Directory size calculations
  │   └── display.rs    # Output formatting
  └── tests/            # Tests
```

### Key Components

1. **Config System**
   - Merges configs from multiple sources
   - Validates against schema
   - Provides defaults for missing values

2. **Project Detection**
   - Abstract interface for project detection
   - Implementation for standard JS/TS projects
   - Framework-specific detectors
   - Plugin support for custom detectors

3. **Cleaner Engine**
   - Orchestrates the cleaning process
   - Applies safety checks
   - Manages statistics collection
   - Triggers appropriate cleaning strategies

4. **Scanner**
   - Fast directory traversal
   - Filtering mechanisms
   - Size calculation
   - Project detector integration

5. **Plugin System**
   - Registration mechanism
   - Lifecycle hooks
   - Configuration integration

### Performance Considerations

1. **Parallel Processing**
   - Multi-threaded project scanning
   - Parallel project cleaning
   - Workload balancing based on directory sizes

2. **OS-Specific Optimizations**
   - Windows-specific optimizations for directory removal
   - Unix-specific fast path operations
   - Platform detection and strategy selection

3. **Memory Management**
   - Stream processing for large directories
   - Bounded memory usage
   - Cancelable operations

## Future Extensions

1. **Package Manager Integration**
   - Custom hooks for npm, yarn, pnpm
   - Script integration templates

2. **Advanced Project Type Detection**
   - Machine learning-based project classification
   - Build system analysis

3. **Global Node Modules Management**
   - Global installation cleaning
   - Version management

4. **User Interfaces**
   - Interactive TUI version
   - GUI application
   - IDE plugins

5. **Remote Operations**
   - Clean projects on remote servers
   - CI/CD integration

## Implementation Phases

1. **Phase 1: Core Functionality**
   - Basic CLI interface
   - Local project cleaning
   - Simple project detection
2. **Phase 2: Enhanced Features**
   - Configuration system
   - Advanced project detection
   - Performance optimizations
3. **Phase 3: Extension System**
   - Plugin architecture
   - Custom cleaning strategies
   - Hook system
4. **Phase 4: Integrations**
   - Package manager integration
   - CI/CD system integration
