# Clean Language Manager - User Guide

## Quick Start

### Installation

1. **Clone and build the manager**:
   ```bash
   git clone https://github.com/Ivan-Pasco/clean-language-manager
   cd clean-language-manager
   cargo build --release
   ```

2. **Install the binary**:
   ```bash
   cargo install --path .
   ```

3. **Initialize your shell**:
   ```bash
   cleen init
   ```

4. **Install the latest Clean Language compiler**:
   ```bash
   cleen install latest
   ```

5. **Verify installation**:
   ```bash
   cln --version
   ```

### First Time Setup

After installing cleen, run the initialization command:

```bash
cleen init
```

This will:
- Add `~/.cleen/bin` to your PATH
- Configure your shell (bash, zsh, fish, etc.)
- Create necessary directories
- Set up the shim system

**Note**: You may need to restart your terminal or run `source ~/.bashrc` (or similar) for PATH changes to take effect.

## Basic Usage

### Installing Versions

**Install the latest version**:
```bash
cleen install latest
```

**Install a specific version**:
```bash
cleen install v0.4.1
cleen install v0.3.0
```

**See what's available**:
```bash
cleen available
```

### Switching Versions

**Switch globally** (affects all projects):
```bash
cleen use v0.4.1
```

**Set project-specific version**:
```bash
cd my-clean-project
cleen local v0.3.0
```

This creates a `.cleanlanguage/.cleanversion` file that should be committed to your repository.

**Check what's installed**:
```bash
cleen list
```

### Managing Versions

**Remove an unused version**:
```bash
cleen uninstall v0.2.0
```

**Install version from project config**:
```bash
cleen sync
```

This reads `.cleanlanguage/.cleanversion` and installs/activates that version.

### Staying Updated

**Check for Updates Regularly**:

1. **Monitor GitHub releases manually**:
   ```bash
   cleen available
   ```
   This shows all available versions from the GitHub repository.

2. **Visit the GitHub repository**:
   - Go to https://github.com/Ivan-Pasco/clean-language-compiler/releases
   - Subscribe to release notifications
   - Check for new compiler versions

3. **Check for updates**:
   ```bash
   cleen update          # Check for compiler updates
   cleen self-update     # Check for cleen updates
   ```

4. **Update to latest version**:
   ```bash
   cleen install latest
   cleen use latest
   ```

**Best Practices for Updates**:
- Check for updates weekly or before starting new projects
- Test new versions in non-critical projects first
- Keep at least one stable version installed as backup
- Update project versions gradually after testing

## Advanced Usage

### Project Workflows

#### Setting Up a New Project

1. **Navigate to your project directory**:
   ```bash
   cd my-clean-project
   ```

2. **Set the project version**:
   ```bash
   cleen local v0.4.1
   ```

3. **Commit the version file**:
   ```bash
   git add .cleanlanguage/.cleanversion
   git commit -m "Set Clean Language version to v0.4.1"
   ```

#### Working with Existing Projects

1. **Clone the project**:
   ```bash
   git clone <project-url>
   cd project
   ```

2. **Install and use the project's specified version**:
   ```bash
   cleen sync
   ```

### Team Collaboration

#### Ensuring Version Consistency

1. **Use project-specific versions**:
   ```bash
   cleen local v0.4.1
   ```

2. **Commit the version file**:
   ```bash
   git add .cleanlanguage/.cleanversion
   git commit -m "Lock Clean Language version"
   ```

3. **Team members sync with**:
   ```bash
   cleen sync
   ```

#### CI/CD Integration

Add to your CI pipeline:
```yaml
- name: Install Clean Language
  run: |
    curl -sSL https://install-script-url | bash
    cleen sync
```

### Troubleshooting

#### Check Environment

```bash
cleen doctor
```

This comprehensive diagnostic will:
- Verify directory structure
- Check installed versions
- Test shim functionality
- Validate PATH configuration
- Test command execution

#### Common Issues

**Command not found: cln**
```bash
# Check if cleen bin is in PATH
echo $PATH | grep cleen

# If not, re-run init
cleen init

# Or manually add to PATH
export PATH="$HOME/.cleen/bin:$PATH"
```

**Permission denied**
```bash
# Check directory permissions
ls -la ~/.cleen

# Fix permissions if needed
chmod -R u+w ~/.cleen
```

**Download failures**
```bash
# Check network connectivity
curl -I https://api.github.com

# Try with verbose logging
CLEEN_VERBOSE=1 cleen install v0.4.1
```

**Binary compatibility issues**
```bash
# Check platform architecture
uname -m

# Verify binary
file ~/.cleen/versions/v0.4.1/cln

# Reinstall if corrupted
cleen uninstall v0.4.1
cleen install v0.4.1
```

## Directory Layout

Understanding the file structure helps with troubleshooting:

```
~/.cleen/
├── bin/
│   └── cln                    # Shim that routes to active version
├── versions/
│   ├── v0.4.1/
│   │   └── cln               # Clean Language compiler v0.4.1
│   ├── v0.4.0/
│   │   └── cln               # Clean Language compiler v0.4.0
│   └── latest/               # Symlink to latest installed version
│       └── cln
└── config.json              # Manager state and configuration
```

## Configuration

### Global Settings

Edit `~/.cleen/config.json` to customize behavior:

```json
{
  "active_version": "v0.4.1",
  "settings": {
    "auto_install_on_use": false,
    "check_updates": true
  }
}
```

**Settings**:
- `auto_install_on_use`: Automatically install versions when switching to them
- `check_updates`: Check for newer versions periodically (default: true)
- `last_update_check`: Timestamp of last compiler update check (Unix timestamp)
- `last_self_update_check`: Timestamp of last cleen self-update check (Unix timestamp)

**Update Checking Behavior**:
- Compiler updates: Checked daily (24 hours) if `check_updates` is true
- Self-updates: Checked weekly (7 days) if `check_updates` is true
- Automatic checks occur during `cleen list` and other common commands
- Manual checks available via `cleen update` and `cleen self-update`

### Environment Variables

**`CLEEN_HOME`**: Override installation directory
```bash
export CLEEN_HOME="/opt/cleen"
cleen install latest
```

**`CLEEN_GITHUB_TOKEN`**: GitHub token for higher rate limits
```bash
export CLEEN_GITHUB_TOKEN="ghp_xxxxxxxxxxxx"
cleen available
```

**`NO_COLOR`**: Disable colored output
```bash
export NO_COLOR=1
cleen list
```

**`CLEEN_VERBOSE`**: Enable verbose logging
```bash
export CLEEN_VERBOSE=1
cleen install v0.4.1
```

## Best Practices

### Version Management

1. **Use project-specific versions for shared projects**:
   ```bash
   cleen local v0.4.1
   git add .cleanlanguage/.cleanversion
   ```

2. **Keep global version reasonably current**:
   ```bash
   cleen use latest
   ```

3. **Regularly clean up old versions**:
   ```bash
   cleen list
   cleen uninstall v0.1.0  # Remove old versions
   ```

### Development Workflow

1. **Check project requirements before starting**:
   ```bash
   cat .cleanlanguage/.cleanversion  # Check project version
   cleen sync                 # Install if needed
   ```

2. **Verify environment before important work**:
   ```bash
   cleen doctor
   cln --version
   ```

3. **Test with multiple versions when needed**:
   ```bash
   cleen use v0.3.0
   cln build
   cleen use v0.4.1
   cln build
   ```

## Tips and Tricks

### Bash/Zsh Aliases

Add these to your shell config for convenience:

```bash
alias clm="cleen"
alias cln-switch="cleen use"
alias cln-local="cleen local"
alias cln-sync="cleen sync"
```

### Quick Commands

```bash
# Install and immediately use latest
cleen install latest && cleen use latest

# Check status quickly
cleen list | grep "✅"

# Install project version if .cleanversion exists
[ -f .cleanlanguage/.cleanversion ] && cleen sync
```

### Scripting

cleen is script-friendly:

```bash
#!/bin/bash
set -e

# Ensure specific version is available
if ! cleen list | grep -q "v0.4.1"; then
    cleen install v0.4.1
fi

# Use for this script
cleen use v0.4.1

# Your Clean Language build commands here
cln build
cln test
```

## Migration Guide

### From Manual Installation

If you were previously managing Clean Language versions manually:

1. **Install cleen**
2. **Run doctor to see what versions are detected**:
   ```bash
   cleen doctor
   ```
3. **Install missing versions through cleen**:
   ```bash
   cleen install v0.4.1
   ```
4. **Remove old manual installations from PATH**

### From Other Version Managers

If you were using a different version manager:

1. **List your current versions** in the old manager
2. **Uninstall the old manager** (following its documentation)
3. **Install cleen and set up environment**:
   ```bash
   cleen init
   ```
4. **Install needed versions**:
   ```bash
   cleen install v0.4.1
   cleen install v0.3.0
   ```
5. **Set appropriate global/project versions**

## Getting Help

### Built-in Help

```bash
cleen --help              # General help
cleen install --help      # Command-specific help
```

### Diagnostics

```bash
cleen doctor              # Comprehensive environment check
CLEEN_VERBOSE=1 cleen <command>  # Verbose output
```

### Common Debugging Steps

1. **Check environment**:
   ```bash
   cleen doctor
   ```

2. **Verify PATH**:
   ```bash
   echo $PATH | tr ':' '\n' | grep cleen
   ```

3. **Check shim**:
   ```bash
   ls -la ~/.cleen/bin/cln
   ```

4. **Test direct binary**:
   ```bash
   ~/.cleen/versions/v0.4.1/cln --version
   ```

5. **Verify config**:
   ```bash
   cat ~/.cleen/config.json
   ```