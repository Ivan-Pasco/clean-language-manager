# Clean Language Manager Architecture

## Overview

The Clean Language Manager (`cleen`) is a Rust-based version manager that provides installation, switching, and management of multiple Clean Language compiler versions across different platforms.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Clean Language Manager                   │
├─────────────────────────────────────────────────────────────┤
│  CLI Interface (clap)                                      │
│  ├─ Commands: install, list, use, available, etc.         │
│  └─ Argument parsing and validation                       │
├─────────────────────────────────────────────────────────────┤
│  Core Components                                           │
│  ├─ Version Manager: Installation and switching logic     │
│  ├─ GitHub Client: API integration for releases           │
│  ├─ Shim Manager: Binary routing and PATH management      │
│  └─ Config Manager: Settings and state persistence        │
├─────────────────────────────────────────────────────────────┤
│  Utilities                                                 │
│  ├─ File System: Cross-platform file operations          │
│  └─ Shell Integration: Environment setup and detection    │
├─────────────────────────────────────────────────────────────┤
│  Storage Layer                                             │
│  └─ ~/.cleen/ directory structure                  │
└─────────────────────────────────────────────────────────────┘
```

## Directory Structure

The Clean Language Manager uses a well-defined directory structure:

```
~/.cleen/
├── bin/
│   └── cln                    # Shim executable (symlink to active version)
├── versions/
│   ├── v0.4.1/
│   │   └── cln               # Clean Language compiler binary
│   ├── v0.4.0/
│   │   └── cln
│   └── latest/               # Symlink to latest installed version
│       └── cln
└── config.json              # Manager configuration and state
```

## Core Components

### 1. CLI Interface (`src/main.rs`)

Entry point that handles command-line argument parsing using `clap`. Routes commands to appropriate handlers and manages error reporting.

**Responsibilities:**
- Parse CLI arguments and subcommands
- Route commands to appropriate modules
- Handle top-level error reporting
- Provide version information

### 2. Command Modules (`src/commands/`)

Each command is implemented as a separate module:

- **`install.rs`**: Download and install compiler versions from GitHub releases
- **`list.rs`**: Display installed versions with active version indication
- **`available.rs`**: Fetch and display available versions from GitHub API
- **`use_version.rs`**: Switch global active version via shim management
- **`local.rs`**: Set project-specific versions via `.cleanlanguage/.cleanversion`
- **`uninstall.rs`**: Remove installed versions and clean up directories
- **`init.rs`**: Set up shell environment and PATH configuration
- **`doctor.rs`**: Diagnose environment issues and verify setup
- **`sync.rs`**: Install version specified in project configuration

### 3. Core Modules (`src/core/`)

#### Version Manager (`version.rs`)
Manages the lifecycle of compiler versions:
- Installation directory management
- Version resolution (global vs project-specific)
- Binary validation and verification
- Cleanup operations

#### GitHub Client (`github.rs`)
Interfaces with GitHub Releases API:
- Fetches available releases from `Ivan-Pasco/clean-language-compiler`
- Downloads platform-specific binaries
- Handles release metadata parsing
- Manages download progress and validation

#### Shim Manager (`shim.rs`)
Handles binary routing and PATH management:
- Creates and manages symlinks to active versions
- Updates shell configurations
- Resolves effective versions (project vs global)
- Validates shim integrity

#### Configuration (`config.rs`)
Manages persistent state and settings:
- Stores active version information
- Manages installation metadata
- Handles configuration file I/O
- Provides default settings

### 4. Utility Modules (`src/utils/`)

#### File System (`fs.rs`)
Cross-platform file operations:
- Recursive directory operations
- File copying with permissions
- Executable bit management
- Platform-specific path handling

#### Shell Integration (`shell.rs`)
Environment setup and detection:
- Shell type detection (bash, zsh, fish, etc.)
- PATH modification
- Configuration file updates
- Interactive setup prompts

## Data Flow

### Installation Flow
```
User Command
    ↓
Command Parser
    ↓
Install Command
    ↓
GitHub Client (fetch releases)
    ↓
Download Manager (get binary)
    ↓
Version Manager (install to ~/.cleen/versions/)
    ↓
Shim Manager (update symlinks if needed)
```

### Version Switching Flow
```
User Command (use/local)
    ↓
Command Parser
    ↓
Use/Local Command
    ↓
Version Manager (validate version exists)
    ↓
Shim Manager (update symlinks)
    ↓
Config Manager (persist state)
```

### Version Resolution Flow
```
cln command executed
    ↓
Shim (~/.cleen/bin/cln)
    ↓
Check for project version (.cleanlanguage/.cleanversion)
    ↓
If not found, use global version
    ↓
Execute appropriate binary
```

## Platform Support

### Supported Platforms
- **Linux**: x86_64, aarch64
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x86_64

### Platform-Specific Considerations

#### Binary Handling
- **Linux/macOS**: ELF binaries, executable permissions required
- **Windows**: PE executables with `.exe` extension

#### Symlink Management
- **Linux/macOS**: Uses symbolic links for shims
- **Windows**: Uses hardlinks or junction points

#### Shell Integration
- **Linux/macOS**: Modifies `.bashrc`, `.zshrc`, `.profile`
- **Windows**: Updates environment variables via registry

## Error Handling

The manager uses a hierarchical error handling system:

```rust
CleenError
├── ConfigError          // Configuration file issues
├── VersionError         // Version management problems
├── DownloadError        // Network and download failures
├── ShimError           // Symlink and PATH issues
├── NoActiveVersion     // No version currently active
├── GitHubError         // GitHub API problems
└── EnvironmentError    // Shell and system issues
```

Each error type provides:
- Descriptive error messages
- Suggested recovery actions
- Context information for debugging

## Security Considerations

### Binary Validation
- Verifies downloaded binaries against expected checksums
- Validates binary format and architecture compatibility
- Ensures executable permissions are set correctly

### PATH Security
- Only modifies user-specific shell configuration files
- Validates PATH entries before modification
- Provides rollback mechanisms for configuration changes

### Network Security
- Uses HTTPS for all GitHub API communications
- Validates SSL certificates
- Implements timeout and retry mechanisms

## Performance Optimizations

### Caching Strategy
- Caches GitHub API responses to reduce network requests
- Maintains local version metadata for fast lookups
- Uses incremental updates for version lists

### Parallel Operations
- Concurrent downloads for multiple versions
- Parallel validation of installed binaries
- Asynchronous network operations

### Memory Management
- Streaming downloads to avoid memory bloat
- Efficient JSON parsing for large API responses
- Resource cleanup after operations

## Extensibility Points

### Plugin Architecture
The manager is designed to support future extensions:
- Custom version sources (beyond GitHub)
- Additional shell integrations
- Platform-specific optimizations
- Enhanced validation mechanisms

### Configuration Extensions
- Per-project configuration overrides
- Global settings and preferences
- Custom installation directories
- Network proxy configuration