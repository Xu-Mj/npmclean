# Release Process

This document outlines the process for releasing new versions of npmclean.

## Automated Release with GitHub Actions

The project includes a GitHub Actions workflow that automatically builds and publishes releases for multiple platforms when code is pushed to the `release` branch or when a tag starting with `v` is created.

### Supported Platforms

The automated build process creates binaries for:

- **Linux**
  - x86_64 (AMD64)
  - ARM64 (AArch64)
- **Windows**
  - x86_64 (AMD64)
- **macOS**
  - x86_64 (Intel)
  - ARM64 (Apple Silicon)

### Release Process

1. **Merge Code to `release` Branch**
   - Ensure all tests pass and the code is ready for release
   - Merge changes from `main` into the `release` branch

2. **Create a Version Tag**
   - Create a tag using semantic versioning: `v1.0.0`, `v1.1.0`, etc.

   ```bash
   git tag -a v1.0.0 -m "Version 1.0.0"
   git push origin v1.0.0
   ```

3. **Monitor the Build Process**
   - Go to the "Actions" tab in GitHub
   - Watch the "Build Release Packages" workflow

4. **Verify the Release**
   - Once the workflow completes, check the "Releases" section in GitHub
   - Verify that all artifacts were uploaded correctly
   - The release notes should be automatically generated

## Manual Release Process

If needed, you can also build releases manually:

```bash
# Build for the current platform
cargo build --release

# For cross-compilation, install rust targets first
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-apple-darwin

# Then build for specific targets
cargo build --release --target=aarch64-unknown-linux-gnu
cargo build --release --target=x86_64-pc-windows-msvc
cargo build --release --target=aarch64-apple-darwin
```

## Troubleshooting

If the automated build fails:

1. Check the workflow logs for errors
2. Verify that the binary name in `Cargo.toml` matches the artifact names in the workflow file
3. Ensure all required dependencies are specified in the workflow file
4. Try building manually for the platform that's failing
