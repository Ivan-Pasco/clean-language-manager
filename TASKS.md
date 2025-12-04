# Clean Language Manager - Task Tracker

## ✅ Completed Features

### Core Infrastructure
- [x] Initialize Rust project structure with Cargo.toml
- [x] Set up CLI interface with clap (derive)
- [x] Implement command routing and module structure
- [x] Add error handling framework with thiserror
- [x] Create JSON-based configuration system
- [x] Implement version storage structure (~/.cleen/versions/)
- [x] Create symlink/shim management system
- [x] Add PATH management and shell integration
- [x] GitHub API integration for releases
- [x] Download and extract compiler binaries

### Compiler Version Management
- [x] `cleen install <version>` - Download and install specific version
- [x] `cleen install latest` - Install most recent release
- [x] `cleen list` - Show installed versions with active indicator
- [x] `cleen available` - List available versions from GitHub
- [x] `cleen use <version>` - Switch to specific version globally
- [x] `cleen local <version>` - Set project-specific version
- [x] `cleen sync` - Install version from .cleanlanguage/.cleanversion
- [x] `cleen uninstall <version>` - Remove installed version

### Environment Setup
- [x] `cleen init` - Initialize shell configuration
- [x] `cleen doctor` - Check and repair environment setup

### Maintenance
- [x] `cleen cleanup` - Remove old compiler versions (dry-run by default)
- [x] `cleen cleanup --confirm` - Actually remove old versions
- [x] `cleen cleanup --keep N` - Keep N most recent versions (default: 3)
- [x] `cleen cleanup --plugins` - Clean up old plugin versions

### Update System
- [x] `cleen update` - Check for compiler updates
- [x] `cleen self-update` - Update cleen itself

### Frame CLI Management
- [x] `cleen frame install [version]` - Install Frame CLI
- [x] `cleen frame list` - List installed Frame versions
- [x] `cleen frame use <version>` - Switch Frame version
- [x] `cleen frame uninstall <version>` - Remove Frame version
- [x] Compiler/Frame version compatibility checking

### Plugin Management
- [x] `cleen plugin install <name>[@version]` - Install plugin from registry
- [x] `cleen plugin list` - List installed plugins
- [x] `cleen plugin create <name>` - Scaffold new plugin project
- [x] `cleen plugin build` - Build plugin to WASM
- [x] `cleen plugin publish` - Publish to registry (placeholder)
- [x] `cleen plugin remove <name>` - Remove installed plugin
- [x] `cleen plugin use <name> <version>` - Switch plugin version
- [x] Plugin manifest parsing (plugin.toml)
- [x] Plugin project scaffolding with templates

### Cross-Platform Support
- [x] Linux x86_64 binary downloads
- [x] macOS x86_64 and ARM64 binary downloads
- [x] Windows x86_64 binary downloads
- [x] Platform-specific shell configuration

## 🔄 In Progress

*No tasks currently in progress*

## 📋 Planned Features

### Nice-to-Have Enhancements
- [ ] **Shell completions** - Bash/zsh/fish completion scripts
- [x] **Automatic cleanup** - Remove old/unused versions with `cleen cleanup`
- [ ] **Verbose logging mode** - Debug output with `--verbose` flag
- [ ] **Offline mode** - Work with cached version info when offline
- [ ] **Progress indicators** - Better download progress display
- [ ] **Version aliasing** - Support `stable` alias in addition to `latest`

### Plugin System Enhancements
- [ ] **Plugin registry API** - Full registry implementation at plugins.cleanlang.org
- [ ] **Plugin dependencies** - Support for plugin-to-plugin dependencies
- [ ] **Plugin search** - `cleen plugin search <query>`
- [ ] **Plugin info** - `cleen plugin info <name>` for detailed metadata

## 📁 Project Structure

```
src/
├── main.rs                 # CLI entry point with clap
├── error.rs                # Custom error types
├── commands/
│   ├── mod.rs
│   ├── available.rs        # Available command
│   ├── cleanup.rs          # Cleanup command
│   ├── doctor.rs           # Doctor command
│   ├── init.rs             # Init command
│   ├── install.rs          # Install command
│   ├── list.rs             # List command
│   ├── local.rs            # Local version command
│   ├── plugin.rs           # Plugin subcommands
│   ├── sync.rs             # Sync command
│   ├── uninstall.rs        # Uninstall command
│   ├── update.rs           # Update commands
│   └── use_version.rs      # Use command
├── core/
│   ├── mod.rs
│   ├── config.rs           # Configuration management
│   ├── version.rs          # Version handling
│   ├── github.rs           # GitHub API integration
│   ├── download.rs         # Download functionality
│   ├── shim.rs             # Symlink management
│   ├── frame.rs            # Frame CLI management
│   └── compatibility.rs    # Version compatibility
├── plugin/
│   ├── mod.rs              # Plugin core functions
│   ├── manifest.rs         # plugin.toml parsing
│   ├── scaffold.rs         # Project scaffolding
│   └── registry.rs         # Registry client
└── utils/
    ├── mod.rs
    ├── fs.rs               # File system utilities
    └── shell.rs            # Shell integration
```

## 🔧 Configuration

### Global Config (~/.cleen/config.json)
```json
{
  "active_version": "0.14.0",
  "frame_version": "1.0.0",
  "cleen_dir": "/Users/user/.cleen",
  "auto_cleanup": false,
  "check_updates": true,
  "auto_offer_frame": true,
  "active_plugins": {
    "frame.web": "1.0.0"
  }
}
```

### Directory Structure
```
~/.cleen/
├── bin/                    # Shim directory (in PATH)
│   ├── cln                 # Compiler shim
│   └── frame               # Frame CLI shim
├── versions/               # Installed compiler versions
│   ├── 0.14.0/
│   │   └── cln
│   └── frame/
│       └── 1.0.0/
│           └── frame
├── plugins/                # Installed plugins
│   └── frame.web/
│       └── 1.0.0/
│           ├── plugin.toml
│           └── plugin.wasm
└── config.json             # Manager configuration
```

## 📝 Notes

- GitHub repo: `https://github.com/Ivan-Pasco/clean-language-compiler`
- Frame CLI repo: `https://github.com/Ivan-Pasco/frame`
- Compiler binary: `cln`
- Manager binary: `cleen`
- Plugin registry: `https://plugins.cleanlang.org` (planned)
