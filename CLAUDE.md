# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is the **Clean Language Manager** (`cleanmanager`), a Rust-based version manager for the Clean Language compiler (`cln`). It allows developers to install, switch, and manage multiple versions of Clean Language across macOS, Linux, and Windows systems.

## Project Status

This is a **new project** that needs to be implemented from scratch. The only existing file is `clean-language-manager-spec.md` which contains the functional specification.

## Architecture

Based on the specification, the manager should implement:

**Core Components:**
- **Version Management**: Download and install compiler binaries from GitHub releases
- **Shim System**: Create symbolic links to route `cln` commands to active versions
- **PATH Management**: Automatically configure shell environments
- **Storage Structure**: Organize versions in isolated directories (`~/.cleanmanager/versions/<version>/`)

**Directory Structure:**
```
~/.cleanmanager/
├── bin/cln                    # Shim/symlink to active version
├── versions/
│   ├── 1.2.3/cln             # Isolated version binaries
│   └── 1.3.0/cln
└── config.json               # Manager configuration
```

## Common Commands (Future Implementation)

### Building and Testing
```bash
# Build the manager
cargo build --release

# Run tests
cargo test

# Install locally for testing
cargo install --path .
```

### Core Manager Commands
```bash
# Install a specific version
cleanmanager install 1.2.3

# List available versions
cleanmanager available

# List installed versions
cleanmanager list

# Switch to a version
cleanmanager use 1.2.3

# Uninstall a version
cleanmanager uninstall 1.2.3

# Check environment setup
cleanmanager doctor

# Initialize shell configuration
cleanmanager init
```

## Implementation Strategy

**Phase 1: Core Structure**
1. Set up Rust project with `cargo init`
2. Define CLI interface using `clap`
3. Implement basic version storage structure
4. Add GitHub API integration for release fetching

**Phase 2: Version Management**
1. Download and extract compiler binaries
2. Implement version switching via symlinks
3. Add PATH management for shell integration
4. Create shim system for command routing

**Phase 3: Advanced Features**
1. Per-project version overrides (`.cleanmanager` files)
2. Automatic cleanup of old versions
3. Self-update functionality
4. Enhanced error handling and recovery

## Key Dependencies

**Essential Crates:**
- `clap`: CLI argument parsing
- `tokio`: Async runtime for downloads
- `reqwest`: HTTP client for GitHub API
- `serde`: JSON serialization for API responses
- `tar`/`zip`: Archive extraction
- `dirs`: Platform-specific directory paths

**Platform Support:**
- Linux (x86_64)
- macOS (x86_64, ARM64)
- Windows (x86_64)

## Development Guidelines

**File Organization:**
- `src/main.rs`: CLI entry point and command routing
- `src/commands/`: Individual command implementations
- `src/version/`: Version management logic
- `src/github/`: GitHub API integration
- `src/config/`: Configuration management
- `src/shim/`: Symlink and PATH management

**Error Handling:**
- Use `anyhow` for error chaining
- Provide clear user-facing error messages
- Implement recovery suggestions for common issues

**Cross-Platform Considerations:**
- Handle different executable extensions (Windows `.exe`)
- Use appropriate symlink vs hardlink strategies
- Account for different shell configuration files

## Integration Points

**GitHub Releases API:**
- Fetch available versions from `https://api.github.com/repos/Ivan-Pasco/clean-language-compiler/releases`
- Download platform-specific binaries
- Verify checksums when available

**Shell Integration:**
- Modify `.bashrc`, `.zshrc`, `.profile` as needed
- Detect current shell environment
- Provide manual setup instructions as fallback

**Clean Language Compiler:**
- Target repository: `https://github.com/Ivan-Pasco/clean-language-compiler`
- Expected binary name: `cln`
- Version detection via `cln --version`