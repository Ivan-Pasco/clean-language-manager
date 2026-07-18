# Clean Language Manager - Functional Specification

## Purpose

The Clean Language Manager (`cleen`) is a version management tool that allows developers to install, switch between, and manage multiple versions of the Clean Language compiler (`cln`) on their development machines.

## Core Functionality

### Version Installation

**Function**: Install specific versions of the Clean Language compiler from GitHub releases.

**Behavior**:
- Downloads platform-appropriate binaries from the official GitHub repository
- Installs to isolated directories (`~/.cleen/versions/<version>/`)
- Validates binary integrity and compatibility
- Sets up executable permissions
- Updates global version tracking

**Commands**:
- `cleen install <version>` - Install specific version (e.g., `v0.4.1`)
- `cleen install latest` - Install the most recent release

**Examples**:
```bash
cleen install v0.4.1
cleen install latest
cleen install v0.3.0
```

### Version Listing

**Function**: Display installed and available versions.

**Installed Versions**:
- Lists all locally installed versions
- Shows active version with visual indicator
- Displays version metadata and installation status

**Available Versions**:
- Fetches latest releases from GitHub API
- Shows available assets for each platform
- Indicates download sizes and release notes

**Commands**:
- `cleen list` - Show installed versions
- `cleen available` - Show available versions from GitHub

### Version Switching

**Function**: Change the active Clean Language compiler version.

**Global Switching**:
- Updates system-wide default version
- Modifies shim symlinks to point to selected version
- Persists selection in configuration

**Project-Specific Versions**:
- Sets version for current project only
- Creates `.cleanlanguage/.cleanversion` file
- Overrides global version when in project directory

**Commands**:
- `cleen use <version>` - Set global active version
- `cleen local <version>` - Set project-specific version

**Examples**:
```bash
cleen use v0.4.1              # Global switch
cleen local v0.3.0            # Project-specific version
```

### Version Synchronization

**Function**: Automatically install and use versions specified in project configuration.

**Behavior**:
- Reads `.cleanlanguage/.cleanversion` file in current directory
- Installs the specified version if not already present
- Activates the version for the current project
- Useful for team environments and CI/CD

**Command**:
- `cleen sync` - Install and use project-specified version

### Version Uninstallation

**Function**: Remove installed versions and clean up associated files.

**Behavior**:
- Removes version directory and all contained files
- Updates configuration to remove version references
- Prevents removal of currently active version
- Provides confirmation prompts for safety

**Command**:
- `cleen uninstall <version>` - Remove specific version

### Environment Setup

**Function**: Configure shell environment for Clean Language Manager.

**Shell Configuration**:
- Automatically detects current shell (bash, zsh, fish, etc.)
- Adds `~/.cleen/bin` to PATH
- Creates or updates shell configuration files
- Provides manual setup instructions as fallback

**Interactive Setup**:
- Prompts user for permission before modifying files
- Explains changes being made
- Offers to restart shell or source configuration

**Command**:
- `cleen init` - Set up shell environment

### Environment Diagnostics

**Function**: Verify installation and diagnose common issues.

**Diagnostic Checks**:
- Directory structure validation
- Installed version verification
- Shim integrity checking
- PATH configuration validation
- Binary execution testing
- Runtime compatibility verification

**Issue Resolution**:
- Provides specific fix recommendations
- Suggests commands to resolve problems
- Validates fixes after resolution

**Command**:
- `cleen doctor` - Run environment diagnostics

## Version Resolution Priority

The manager uses a hierarchical approach to determine which version to use:

1. **Project Version**: `.cleanlanguage/.cleanversion` in current directory or parent directories
2. **Global Version**: System-wide default version set via `cleen use`
3. **Fallback**: Latest installed version if no explicit choice

## File System Layout

### User Directory Structure
```
~/.cleen/
├── bin/
│   └── cln                   # Shim that routes to active version
├── versions/
│   ├── v0.4.1/
│   │   └── cln              # Clean Language compiler v0.4.1
│   ├── v0.4.0/
│   │   └── cln              # Clean Language compiler v0.4.0
│   └── latest/              # Symlink to latest installed version
│       └── cln
└── config.json             # Manager configuration and state
```

### Project Directory Structure
```
project-root/
├── .cleanlanguage/
│   └── .cleanversion        # Project-specific version specification
├── src/
│   └── main.cln            # Clean Language source files
└── other-files...
```

## Configuration Management

### Global Configuration (`~/.cleen/config.json`)
```json
{
  "active_version": "v0.4.1",
  "installed_versions": {
    "v0.4.1": {
      "installed_at": "2024-01-15T10:30:00Z",
      "binary_path": "/Users/user/.cleen/versions/v0.4.1/cln",
      "platform": "macos-aarch64"
    }
  },
  "settings": {
    "auto_install_on_use": false,
    "check_updates": true
  }
}
```

### Project Configuration (`.cleanlanguage/.cleanversion`)
```
v0.3.0
```

## Platform-Specific Behavior

### macOS
- Downloads `cln-macos-aarch64` for Apple Silicon or `cln-macos-x86_64` for Intel
- Uses symbolic links for shims
- Modifies `.zshrc` by default (or `.bash_profile` for bash)
- Handles Gatekeeper security prompts

### Linux
- Downloads appropriate architecture binary (`cln-linux-x86_64` or `cln-linux-aarch64`)
- Uses symbolic links for shims
- Modifies `.bashrc` or `.zshrc` based on shell detection
- Sets executable permissions via chmod

### Windows
- Downloads `cln-windows-x86_64.exe`
- Uses hardlinks or junction points for shims
- Updates system PATH via environment variables
- Handles Windows security and execution policies

## Network Operations

### GitHub API Integration
- **Endpoint**: `https://api.github.com/repos/Ivan-Pasco/clean-language-compiler/releases`
- **Rate Limiting**: Respects GitHub API rate limits
- **Caching**: Caches API responses to reduce network requests
- **Authentication**: Uses public API (no authentication required)

### Download Management
- **Parallel Downloads**: Supports concurrent version installations
- **Progress Tracking**: Shows download progress for large binaries
- **Resume Support**: Can resume interrupted downloads
- **Verification**: Validates downloaded files against checksums when available

## Error Handling Strategy

### Error Categories
1. **User Errors**: Invalid commands, missing versions, permission issues
2. **Network Errors**: GitHub API failures, download interruptions
3. **System Errors**: File system problems, permission denied
4. **Runtime Errors**: Binary compatibility, execution failures

### Error Recovery
- **Automatic Retry**: Network operations with exponential backoff
- **Graceful Degradation**: Fallback to cached data when possible
- **User Guidance**: Clear error messages with resolution steps
- **State Consistency**: Ensures clean state even after failures

## Performance Characteristics

### Startup Time
- Cold start: ~50ms for simple commands (list, version)
- Network operations: 1-5 seconds depending on connection
- Installation: 30 seconds to 2 minutes per version

### Memory Usage
- Base memory: ~5-10MB for CLI operations
- Download operations: ~20-50MB depending on binary size
- Concurrent installations: Scales linearly with number of versions

### Disk Usage
- Per version: ~10-50MB depending on compiler binary size
- Metadata: ~1KB per installed version
- Total overhead: ~1-2MB for manager itself

## Integration Points

### Development Workflow
- Integrates with existing Clean Language projects
- Supports CI/CD environments
- Compatible with Docker and containerized builds
- Works with IDE integrations and build tools

### Team Collaboration
- Project-specific version files ensure consistency
- Supports shared configuration across team members
- Compatible with version control systems
- Enables reproducible builds

## Security Model

### Principle of Least Privilege
- Only modifies user-specific directories
- Never requires administrator/root privileges
- Isolated version installations prevent conflicts

### Binary Verification
- Validates downloaded binaries before installation
- Checks file signatures when available
- Ensures platform compatibility before execution

### Network Security
- All communications over HTTPS
- Certificate validation enabled
- No sensitive data transmission

---

## Plugin Management

### Plugin Installation

**Function**: Install plugins from the registry or local sources.

**Behavior**:
- Parses plugin name and optional version (e.g., `frame.web@1.0.0`)
- Queries plugin registry for metadata and download URL
- Downloads plugin archive containing `plugin.toml` and `plugin.wasm`
- Extracts to `~/.cleen/plugins/<name>/<version>/`
- Validates manifest and WASM binary integrity
- Updates local plugin index

**Commands**:
- `cleen plugin install <name>` - Install latest version
- `cleen plugin install <name>@<version>` - Install specific version

**Examples**:
```bash
cleen plugin install frame.web
# Output:
# Downloading frame.web@1.0.0...
# Extracting to ~/.cleen/plugins/frame.web/1.0.0/
# Verifying plugin.toml...
# Verifying plugin.wasm...
# Plugin frame.web@1.0.0 installed successfully

cleen plugin install frame.data@0.5.0
# Output:
# Downloading frame.data@0.5.0...
# Plugin frame.data@0.5.0 installed successfully
```

### Plugin Listing

**Function**: Display installed plugins and their versions.

**Behavior**:
- Scans `~/.cleen/plugins/` directory
- Reads plugin.toml manifests
- Shows active version indicator for multi-version plugins
- Displays compatibility information

**Command**:
- `cleen plugin list` - Show all installed plugins

**Examples**:
```bash
cleen plugin list
# Output:
# Installed plugins:
#   frame.web
#     * 1.0.0 (active)
#       0.9.0
#   frame.data
#     * 0.5.0 (active)

cleen plugin list
# Output (when no plugins installed):
# No plugins installed
#
# To install a plugin:
#   cleen plugin install <name>
```

### Plugin Creation

**Function**: Scaffold a new plugin project with template files.

**Behavior**:
- Creates plugin directory structure
- Generates plugin.toml with metadata placeholders
- Creates src/main.cln with plugin entry point template
- Creates tests directory with example test
- Generates README.md with usage instructions

**Command**:
- `cleen plugin create <name>` - Create new plugin project

**Examples**:
```bash
cleen plugin create my-framework
# Output:
# Creating plugin project 'my-framework'...
# Created my-framework/
# Created my-framework/plugin.toml
# Created my-framework/src/main.cln
# Created my-framework/tests/test_expand.cln
# Created my-framework/README.md
#
# Next steps:
#   cd my-framework
#   # Edit src/main.cln to implement your plugin
#   cleen plugin build
```

### Plugin Building

**Function**: Compile plugin source to WebAssembly.

**Behavior**:
- Locates plugin.toml in current directory
- Parses manifest for entry point configuration
- Invokes Clean Language compiler: `cln compile src/main.cln -o plugin.wasm`
- Validates generated WASM output
- Reports build status and any errors

**Command**:
- `cleen plugin build` - Build plugin in current directory

**Examples**:
```bash
cleen plugin build
# Output:
# Building plugin 'my-framework'...
# Compiling src/main.cln...
# Generated plugin.wasm (24.5 KB)
# Build successful

cleen plugin build
# Output (on error):
# Building plugin 'my-framework'...
# Compiling src/main.cln...
# Error: Compilation failed
#   src/main.cln:15:10 - Type mismatch: expected string, got integer
```

### Plugin Publishing

**Function**: Publish plugin to the central registry.

**Behavior**:
- Validates plugin.toml completeness
- Ensures plugin.wasm exists and is valid
- Authenticates with registry (uses stored credentials)
- Uploads plugin archive
- Registers version in registry

**Command**:
- `cleen plugin publish` - Publish to registry

**Examples**:
```bash
cleen plugin publish
# Output:
# Publishing my-framework@1.0.0...
# Validating manifest...
# Validating plugin.wasm...
# Uploading to registry...
# Published successfully
#
# Your plugin is now available:
#   cleen plugin install my-framework
```

### Plugin Removal

**Function**: Remove installed plugins.

**Behavior**:
- Removes plugin directory from `~/.cleen/plugins/<name>/`
- Updates local plugin index
- Warns if plugin is referenced in project files

**Command**:
- `cleen plugin remove <name>` - Remove plugin

**Examples**:
```bash
cleen plugin remove frame.web
# Output:
# Removing frame.web...
# Removed ~/.cleen/plugins/frame.web/
# Plugin frame.web removed successfully

cleen plugin remove frame.web
# Output (when not installed):
# Error: Plugin 'frame.web' is not installed
```

---

## Plugin File System Layout

### Plugin Installation Directory
```
~/.cleen/
├── plugins/
│   ├── frame.web/
│   │   ├── 1.0.0/
│   │   │   ├── plugin.toml      # Plugin manifest
│   │   │   └── plugin.wasm      # Compiled plugin binary
│   │   └── 0.9.0/
│   │       ├── plugin.toml
│   │       └── plugin.wasm
│   └── frame.data/
│       └── 0.5.0/
│           ├── plugin.toml
│           └── plugin.wasm
└── config.json                   # Includes active_plugins section
```

### Plugin Project Structure
```
my-plugin/
├── plugin.toml                   # Plugin manifest
├── src/
│   └── main.cln                 # Plugin entry point
├── tests/
│   └── test_expand.cln          # Plugin tests
└── README.md                    # Plugin documentation
```

---

## Plugin Configuration

### Plugin Manifest (plugin.toml)
```toml
[plugin]
name = "frame.web"
version = "1.0.0"
description = "Web framework plugin for Clean Language"
author = "Clean Language Team"
license = "MIT"
repository = "https://github.com/example/frame.web"

[compatibility]
min_compiler_version = "0.15.0"
max_compiler_version = "1.0.0"

[exports]
expand = "expand_block"
validate = "validate_block"

[dependencies]
# Other plugins this plugin depends on (planned feature)
```

### Global Configuration Updates
The `~/.cleen/config.json` file includes plugin configuration:
```json
{
  "active_version": "0.15.0",
  "installed_versions": { ... },
  "active_plugins": {
    "frame.web": "1.0.0",
    "frame.data": "0.5.0"
  },
  "settings": { ... }
}
```

---

## Plugin Error Handling

### Error Categories
1. **Registry Errors**: Network failures, version not found, authentication issues
2. **Manifest Errors**: Invalid plugin.toml, missing required fields
3. **Build Errors**: Compilation failures, missing source files
4. **Compatibility Errors**: Compiler version mismatch

### Error Messages

**Plugin Not Found**:
```
Error: Plugin 'unknown-plugin' not found in registry
  Try 'cleen plugin list --available' to see available plugins
```

**Version Not Found**:
```
Error: Version '2.0.0' of plugin 'frame.web' not found
  Available versions: 1.0.0, 0.9.0, 0.8.0
```

**Compatibility Error**:
```
Error: Plugin 'frame.web@1.0.0' requires compiler >= 0.15.0
  Current compiler version: 0.14.0
  Install a compatible compiler: cleen install 0.15.0
```

**Build Error**:
```
Error: Cannot find plugin.toml in current directory
  Run this command from a plugin project directory
  Or create a new plugin: cleen plugin create <name>
```