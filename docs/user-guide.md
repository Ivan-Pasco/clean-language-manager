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
   cleanmanager init
   ```

4. **Install the latest Clean Language compiler**:
   ```bash
   cleanmanager install latest
   ```

5. **Verify installation**:
   ```bash
   cln --version
   ```

### First Time Setup

After installing cleanmanager, run the initialization command:

```bash
cleanmanager init
```

This will:
- Add `~/.cleanmanager/bin` to your PATH
- Configure your shell (bash, zsh, fish, etc.)
- Create necessary directories
- Set up the shim system

**Note**: You may need to restart your terminal or run `source ~/.bashrc` (or similar) for PATH changes to take effect.

## Basic Usage

### Installing Versions

**Install the latest version**:
```bash
cleanmanager install latest
```

**Install a specific version**:
```bash
cleanmanager install v0.4.1
cleanmanager install v0.3.0
```

**See what's available**:
```bash
cleanmanager available
```

### Switching Versions

**Switch globally** (affects all projects):
```bash
cleanmanager use v0.4.1
```

**Set project-specific version**:
```bash
cd my-clean-project
cleanmanager local v0.3.0
```

This creates a `.cleanlanguage/.cleanversion` file that should be committed to your repository.

**Check what's installed**:
```bash
cleanmanager list
```

### Managing Versions

**Remove an unused version**:
```bash
cleanmanager uninstall v0.2.0
```

**Install version from project config**:
```bash
cleanmanager sync
```

This reads `.cleanlanguage/.cleanversion` and installs/activates that version.

### Staying Updated

**Check for Updates Regularly**:

1. **Monitor GitHub releases manually**:
   ```bash
   cleanmanager available
   ```
   This shows all available versions from the GitHub repository.

2. **Visit the GitHub repository**:
   - Go to https://github.com/Ivan-Pasco/clean-language-compiler/releases
   - Subscribe to release notifications
   - Check for new compiler versions

3. **Check for updates**:
   ```bash
   cleanmanager update          # Check for compiler updates
   cleanmanager self-update     # Check for cleanmanager updates
   ```

4. **Update to latest version**:
   ```bash
   cleanmanager install latest
   cleanmanager use latest
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
   cleanmanager local v0.4.1
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
   cleanmanager sync
   ```

### Team Collaboration

#### Ensuring Version Consistency

1. **Use project-specific versions**:
   ```bash
   cleanmanager local v0.4.1
   ```

2. **Commit the version file**:
   ```bash
   git add .cleanlanguage/.cleanversion
   git commit -m "Lock Clean Language version"
   ```

3. **Team members sync with**:
   ```bash
   cleanmanager sync
   ```

#### CI/CD Integration

Add to your CI pipeline:
```yaml
- name: Install Clean Language
  run: |
    curl -sSL https://install-script-url | bash
    cleanmanager sync
```

### Troubleshooting

#### Check Environment

```bash
cleanmanager doctor
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
# Check if cleanmanager bin is in PATH
echo $PATH | grep cleanmanager

# If not, re-run init
cleanmanager init

# Or manually add to PATH
export PATH="$HOME/.cleanmanager/bin:$PATH"
```

**Permission denied**
```bash
# Check directory permissions
ls -la ~/.cleanmanager

# Fix permissions if needed
chmod -R u+w ~/.cleanmanager
```

**Download failures**
```bash
# Check network connectivity
curl -I https://api.github.com

# Try with verbose logging
CLEANMANAGER_VERBOSE=1 cleanmanager install v0.4.1
```

**Binary compatibility issues**
```bash
# Check platform architecture
uname -m

# Verify binary
file ~/.cleanmanager/versions/v0.4.1/cln

# Reinstall if corrupted
cleanmanager uninstall v0.4.1
cleanmanager install v0.4.1
```

## Directory Layout

Understanding the file structure helps with troubleshooting:

```
~/.cleanmanager/
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

Edit `~/.cleanmanager/config.json` to customize behavior:

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
- `last_self_update_check`: Timestamp of last cleanmanager self-update check (Unix timestamp)

**Update Checking Behavior**:
- Compiler updates: Checked daily (24 hours) if `check_updates` is true
- Self-updates: Checked weekly (7 days) if `check_updates` is true
- Automatic checks occur during `cleanmanager list` and other common commands
- Manual checks available via `cleanmanager update` and `cleanmanager self-update`

### Environment Variables

**`CLEANMANAGER_HOME`**: Override installation directory
```bash
export CLEANMANAGER_HOME="/opt/cleanmanager"
cleanmanager install latest
```

**`CLEANMANAGER_GITHUB_TOKEN`**: GitHub token for higher rate limits
```bash
export CLEANMANAGER_GITHUB_TOKEN="ghp_xxxxxxxxxxxx"
cleanmanager available
```

**`NO_COLOR`**: Disable colored output
```bash
export NO_COLOR=1
cleanmanager list
```

**`CLEANMANAGER_VERBOSE`**: Enable verbose logging
```bash
export CLEANMANAGER_VERBOSE=1
cleanmanager install v0.4.1
```

## Best Practices

### Version Management

1. **Use project-specific versions for shared projects**:
   ```bash
   cleanmanager local v0.4.1
   git add .cleanlanguage/.cleanversion
   ```

2. **Keep global version reasonably current**:
   ```bash
   cleanmanager use latest
   ```

3. **Regularly clean up old versions**:
   ```bash
   cleanmanager list
   cleanmanager uninstall v0.1.0  # Remove old versions
   ```

### Development Workflow

1. **Check project requirements before starting**:
   ```bash
   cat .cleanlanguage/.cleanversion  # Check project version
   cleanmanager sync                 # Install if needed
   ```

2. **Verify environment before important work**:
   ```bash
   cleanmanager doctor
   cln --version
   ```

3. **Test with multiple versions when needed**:
   ```bash
   cleanmanager use v0.3.0
   cln build
   cleanmanager use v0.4.1
   cln build
   ```

## Tips and Tricks

### Bash/Zsh Aliases

Add these to your shell config for convenience:

```bash
alias clm="cleanmanager"
alias cln-switch="cleanmanager use"
alias cln-local="cleanmanager local"
alias cln-sync="cleanmanager sync"
```

### Quick Commands

```bash
# Install and immediately use latest
cleanmanager install latest && cleanmanager use latest

# Check status quickly
cleanmanager list | grep "✅"

# Install project version if .cleanversion exists
[ -f .cleanlanguage/.cleanversion ] && cleanmanager sync
```

### Scripting

cleanmanager is script-friendly:

```bash
#!/bin/bash
set -e

# Ensure specific version is available
if ! cleanmanager list | grep -q "v0.4.1"; then
    cleanmanager install v0.4.1
fi

# Use for this script
cleanmanager use v0.4.1

# Your Clean Language build commands here
cln build
cln test
```

## Migration Guide

### From Manual Installation

If you were previously managing Clean Language versions manually:

1. **Install cleanmanager**
2. **Run doctor to see what versions are detected**:
   ```bash
   cleanmanager doctor
   ```
3. **Install missing versions through cleanmanager**:
   ```bash
   cleanmanager install v0.4.1
   ```
4. **Remove old manual installations from PATH**

### From Other Version Managers

If you were using a different version manager:

1. **List your current versions** in the old manager
2. **Uninstall the old manager** (following its documentation)
3. **Install cleanmanager and set up environment**:
   ```bash
   cleanmanager init
   ```
4. **Install needed versions**:
   ```bash
   cleanmanager install v0.4.1
   cleanmanager install v0.3.0
   ```
5. **Set appropriate global/project versions**

## Getting Help

### Built-in Help

```bash
cleanmanager --help              # General help
cleanmanager install --help      # Command-specific help
```

### Diagnostics

```bash
cleanmanager doctor              # Comprehensive environment check
CLEANMANAGER_VERBOSE=1 cleanmanager <command>  # Verbose output
```

### Common Debugging Steps

1. **Check environment**:
   ```bash
   cleanmanager doctor
   ```

2. **Verify PATH**:
   ```bash
   echo $PATH | tr ':' '\n' | grep cleanmanager
   ```

3. **Check shim**:
   ```bash
   ls -la ~/.cleanmanager/bin/cln
   ```

4. **Test direct binary**:
   ```bash
   ~/.cleanmanager/versions/v0.4.1/cln --version
   ```

5. **Verify config**:
   ```bash
   cat ~/.cleanmanager/config.json
   ```