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

## Architecture Boundary Rules

**CRITICAL: Read `../system-documents/ARCHITECTURE_BOUNDARIES.md` before implementing ANY new functionality.**

The manager is a **version manager and orchestrator**. It delegates to other binaries — it does NOT reimplement their logic.

### The Manager MUST NOT:

- Parse or understand Clean Language `.cln` syntax
- Generate Clean Language source code
- Know about framework folder conventions (`pages/`, `api/`, `data/`, `components/`)
- Discover/scan project files for routes, components, or models
- Transform HTML templates or expand component tags
- Implement any part of the build pipeline (codegen, discovery, compilation)
- Implement host bridge functions

### The Manager MUST delegate framework operations:

```rust
// CORRECT — delegate to frame-cli binary
Command::new("frame-cli").args(["build"]).status()?;
Command::new("frame-cli").args(["new", name]).status()?;
Command::new("frame-cli").args(["scan"]).status()?;

// WRONG — implement framework logic directly
codegen::generate_code(&project);  // DO NOT DO THIS
discovery::discover_project(&path); // DO NOT DO THIS
```

### Boundary Violation Check

Before writing ANY new function, ask:
1. Does this function read/parse/generate `.cln` code? → **STOP — belongs in compiler or frame-cli**
2. Does this function know about `pages/`, `api/`, `data/` folders? → **STOP — belongs in frame-cli**
3. Does this function transform HTML or expand templates? → **STOP — belongs in frame-cli**
4. Does this function implement a `_*` host function? → **STOP — belongs in clean-server**

### Known Violations (Pending Extraction)

The following files contain framework code that must be extracted to `clean-framework/frame-cli/`:
- `src/core/codegen.rs` (1,491 lines) — entire file is misplaced
- `src/core/discovery.rs` (745 lines) — entire file is misplaced
- `src/core/frame.rs` (~663 lines) — `create_project()`, `build_project()`, `scan_project()` and template functions

See: `../system-documents/cross-component-prompts/manager-extract-framework-code.md`

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

## Cross-Component Work Policy

**CRITICAL: AI Instance Separation of Concerns**

When working in this component and discovering errors, bugs, or required changes in **another component** (different folder in the Clean Language project), you must **NOT** directly fix or modify code in that other component.

Instead:

1. **Document the issue** by creating a prompt/task description
2. **Save the prompt** in a file that can be executed by the AI instance working in the correct folder
3. **Location for cross-component prompts**: Save prompts in `../system-documents/cross-component-prompts/` at the project root

### Prompt Format for Cross-Component Issues

```
Component: [target component name, e.g., clean-language-compiler]
Issue Type: [bug/feature/enhancement/compatibility]
Priority: [critical/high/medium/low]
Description: [Detailed description of the issue discovered]
Context: [Why this was discovered while working in the current component]
Suggested Fix: [If known, describe the potential solution]
Files Affected: [List of files in the target component that need changes]
```

### Why This Rule Exists

- Each component has its own context, dependencies, and testing requirements
- AI instances are optimized for their specific component's codebase
- Cross-component changes without proper context can introduce bugs
- This maintains clear boundaries and accountability
- Ensures changes are properly tested in the target component's environment

### What You CAN Do

- Read files from other components to understand interfaces
- Document compatibility issues found
- Create detailed prompts for the correct AI instance
- Update your component to work with existing interfaces

### What You MUST NOT Do

- Directly edit code in other components
- Make changes to other components' configuration files
- Modify shared resources without coordination
- Skip the prompt creation step for cross-component issues