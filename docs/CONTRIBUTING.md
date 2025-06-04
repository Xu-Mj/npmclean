# Contributing to npmclean

Thank you for your interest in contributing to npmclean! This document provides guidelines and information about how to contribute to the project.

## Project Philosophy

npmclean aims to be:

- **Fast**: Optimized for performance, especially on Windows
- **Safe**: Prevents accidental deletion of important files
- **Extensible**: Easy to add new features and integrations
- **User-friendly**: Simple CLI with sensible defaults

## Getting Started

### Prerequisites

- Rust toolchain (latest stable)
- Node.js/npm (for testing)

### Setup

1. Fork and clone the repository
2. Install dependencies: `cargo build`
3. Run tests: `cargo test`

## Project Structure

See the [DESIGN.md](./DESIGN.md) document for a detailed overview of the project structure and architecture.

## Extension Points

npmclean is designed to be extensible. Here are the main extension points:

### 1. Project Detectors

Project detectors identify and analyze JavaScript/TypeScript projects.

To add a new project detector:

1. Implement the `ProjectDetector` trait in `src/project/detectors/`
2. Register it in the detector registry in `src/project/mod.rs`

Example:

```rust
pub struct NextJsDetector;

impl ProjectDetector for NextJsDetector {
    fn detect(&self, path: &Path) -> bool {
        // Check if this is a Next.js project
        // e.g., check for next.config.js or next in dependencies
    }
    
    fn get_build_dirs(&self) -> Vec<String> {
        // Return Next.js specific directories
        vec![".next".to_string(), "out".to_string()]
    }
}

// Registration in detector_registry:
pub fn create_detector_registry() -> Vec<Box<dyn ProjectDetector>> {
    vec![
        Box::new(DefaultDetector::new()),
        Box::new(ReactDetector::new()),
        Box::new(NextJsDetector::new()),  // Add your detector here
    ]
}
```

### 2. Cleaning Strategies

Cleaning strategies define how directories should be removed.

To add a new cleaning strategy:

1. Implement the `CleaningStrategy` trait in `src/cleaner/strategies/`
2. Register it in the strategy registry in `src/cleaner/mod.rs`

Example:

```rust
pub struct FastWindowsStrategy;

impl CleaningStrategy for FastWindowsStrategy {
    fn can_handle(&self, path: &Path, platform: Platform) -> bool {
        platform == Platform::Windows && /* other conditions */
    }
    
    fn clean(&self, path: &Path) -> Result<(), CleanError> {
        // Implement Windows-optimized directory removal
    }
}

// Registration:
pub fn create_strategy_registry() -> Vec<Box<dyn CleaningStrategy>> {
    vec![
        Box::new(DefaultStrategy::new()),
        Box::new(FastWindowsStrategy::new()),  // Add your strategy here
    ]
}
```

### 3. Configuration Extensions

To add new configuration options:

1. Update the config schema in `src/config/schema.rs`
2. Add handling for the new options in `src/config/loader.rs`
3. Use the new options in your feature implementation

### 4. CLI Commands

To add a new command or flag:

1. Update the CLI definition in `src/cli.rs`
2. Add handling for the new command in `src/main.rs`

## Plugins (Future)

In the future, npmclean will support a formal plugin system. The architecture is being designed with this in mind.

## Code Style Guidelines

- Follow Rust standard naming conventions
- Use meaningful variable and function names
- Document public API with rustdoc comments
- Write unit tests for new features
- Use the `thiserror` crate for error handling
- Format code with `rustfmt`

## Pull Request Process

1. Create a new branch for your feature or bugfix
2. Implement your changes with tests
3. Update documentation as needed
4. Run `cargo fmt` and `cargo clippy` to ensure code quality
5. Open a PR with a clear description of changes
6. Address review comments

## Release Process

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Create a new release on GitHub
4. Publish to crates.io

## Communication

- For bugs and features, open an issue on GitHub
- For questions, use GitHub Discussions

Thank you for contributing to npmclean!
