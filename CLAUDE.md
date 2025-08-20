# CLAUDE.md

This file provides guidance when working with code in this repository.

## Important Constraints

- **NEVER** write any reference to Claude Code in any documents, git commits, or any part of the code
- **NEVER** mention Claude Code in git commit messages or any part of the codebase

## Overview

This is the **Clean Language Manager** (`cleen`), a Rust-based version manager for the Clean Language compiler (`cln`). It allows developers to install, switch, and manage multiple versions of Clean Language across macOS, Linux, and Windows systems.

## Project Status

This is a **fully implemented and functional project**. The codebase is complete with all core functionality working.

## Documentation

**Comprehensive documentation is available in the `docs/` folder:**

- **[docs/README.md](docs/README.md)** - Documentation overview and navigation guide
- **[docs/architecture.md](docs/architecture.md)** - Technical architecture and system design
- **[docs/functional-specification.md](docs/functional-specification.md)** - Complete functional requirements and behavior
- **[docs/api-reference.md](docs/api-reference.md)** - Command-line interface documentation
- **[docs/user-guide.md](docs/user-guide.md)** - Practical usage guide and troubleshooting

**When working with this codebase, always reference the docs/ folder for:**
- Understanding the architecture before making changes
- Following established patterns and conventions
- Understanding command behavior and expected outputs
- Troubleshooting and debugging guidance

## Current Implementation

The Clean Language Manager is **fully implemented** with the following architecture:

**Implemented Core Components:**
- ✅ **Version Management**: Downloads and installs compiler binaries from GitHub releases
- ✅ **Shim System**: Creates symbolic links to route `cln` commands to active versions  
- ✅ **PATH Management**: Automatically configures shell environments
- ✅ **Storage Structure**: Organizes versions in isolated directories (`~/.cleen/versions/<version>/`)

**Working Commands:**
```bash
# Build and test (implemented)
cargo build --release
cargo test

# Core functionality (all working)
cleen install <version>     # Install any available version
cleen available            # List GitHub releases
cleen list                # Show installed versions
cleen use <version>       # Switch global version
cleen local <version>     # Set project version
cleen uninstall <version> # Remove version
cleen doctor              # Environment diagnostics
cleen init                # Shell setup
cleen sync                # Install from .cleanversion
```

## Codebase Structure

**Implemented Modules:**
- `src/main.rs` - CLI entry point with clap integration
- `src/commands/` - Complete command implementations (9 commands)
- `src/core/` - Core functionality (version, github, shim, config management)
- `src/utils/` - Cross-platform utilities (fs, shell integration)
- `src/error.rs` - Comprehensive error handling

**Integration Status:**
- ✅ GitHub API integration working (`Ivan-Pasco/clean-language-compiler`)
- ✅ Cross-platform binary downloads (Linux, macOS, Windows)
- ✅ Shell environment setup (bash, zsh, fish)
- ✅ Project-specific version files (`.cleanlanguage/.cleanversion`)

## Building and Testing

```bash
# Build (working)
cargo build --release

# Run any command
cargo run -- <command>

# Examples that work right now:
cargo run -- available    # Shows GitHub releases
cargo run -- list        # Shows installed versions  
cargo run -- doctor      # Environment check
```