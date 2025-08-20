# Clean Language Manager - API Reference

## Command Line Interface

### Global Options

```bash
cleen [OPTIONS] <SUBCOMMAND>
```

**Options**:
- `-h, --help` - Print help information
- `-V, --version` - Print version information

## Commands

### install

Install a specific version of Clean Language compiler.

```bash
cleen install <VERSION>
```

**Arguments**:
- `<VERSION>` - Version to install (e.g., `v0.4.1`, `latest`)

**Examples**:
```bash
cleen install v0.4.1    # Install specific version
cleen install latest    # Install latest available version
```

**Behavior**:
- Downloads platform-appropriate binary from GitHub releases
- Installs to `~/.cleen/versions/<version>/`
- Validates binary and sets executable permissions
- Updates installation metadata

**Exit Codes**:
- `0` - Success
- `1` - Version not found or download failed
- `2` - Installation failed (permissions, disk space, etc.)

---

### list

List all installed Clean Language versions.

```bash
cleen list
```

**Output Format**:
```
Installed Clean Language versions:

  latest 
  v0.2.2-ls 
  v0.1.3 
  v0.3.0 
  v0.2.2 
  v0.4.0 
  v0.1.11 
  v0.4.1 ✅ (active)
  v0.1.9 
  v0.1.14 
  v0.1.12 
  v0.2.0 

Active version: v0.4.1
```

**Indicators**:
- `✅ (active)` - Currently active global version
- No indicator - Installed but not active

---

### available

List available versions from GitHub releases.

```bash
cleen available
```

**Output Format**:
```
Clean Language Compiler Versions
=================================

📋 Available versions:

  • v0.4.1 (latest)
    Assets: cln-linux-x86_64, cln-macos-aarch64, cln-macos-x86_64, cln-windows-x86_64.exe
  • v0.4.0
    Assets: cln-linux-aarch64, cln-linux-x86_64, cln-macos-aarch64, cln-macos-x86_64, cln-windows-x86_64.exe
```

**Information Provided**:
- Version numbers with latest indicator
- Available platform binaries
- Release metadata when available

---

### use

Switch to a specific version globally.

```bash
cleen use <VERSION>
```

**Arguments**:
- `<VERSION>` - Version to activate globally

**Examples**:
```bash
cleen use v0.4.1     # Switch to specific version
cleen use latest     # Switch to latest installed version
```

**Behavior**:
- Updates global shim symlink
- Modifies `~/.cleen/config.json`
- Affects all `cln` commands system-wide (unless overridden by project version)

**Exit Codes**:
- `0` - Success
- `1` - Version not installed
- `2` - Shim update failed

---

### local

Set project-specific version.

```bash
cleen local <VERSION>
```

**Arguments**:
- `<VERSION>` - Version to use in current project

**Examples**:
```bash
cleen local v0.3.0   # Set project to use v0.3.0
```

**Behavior**:
- Creates `.cleanlanguage/.cleanversion` file in current directory
- Version takes precedence over global setting when in project directory
- File should be committed to version control for team consistency

**File Contents**:
```
v0.3.0
```

---

### sync

Install the version specified in project configuration.

```bash
cleen sync
```

**Behavior**:
- Reads `.cleanlanguage/.cleanversion` from current directory
- Installs the specified version if not already present
- Automatically switches to that version for the project
- Useful for setting up projects and CI/CD environments

**Exit Codes**:
- `0` - Success
- `1` - No `.cleanversion` file found
- `2` - Installation failed

---

### uninstall

Remove a specific version from the system.

```bash
cleen uninstall <VERSION>
```

**Arguments**:
- `<VERSION>` - Version to remove

**Examples**:
```bash
cleen uninstall v0.3.0    # Remove specific version
```

**Behavior**:
- Removes version directory and all contents
- Updates configuration to remove version references
- Prevents removal of currently active version
- Prompts for confirmation before deletion

**Exit Codes**:
- `0` - Success
- `1` - Version not found or cannot remove active version
- `2` - Removal failed (permissions, etc.)

---

### init

Initialize shell configuration for Clean Language Manager.

```bash
cleen init
```

**Behavior**:
- Detects current shell (bash, zsh, fish, etc.)
- Adds `~/.cleen/bin` to PATH
- Creates or modifies shell configuration files
- Provides interactive prompts for user consent
- Shows manual setup instructions if automatic setup fails

**Shell Files Modified**:
- **Bash**: `~/.bashrc`, `~/.bash_profile`
- **Zsh**: `~/.zshrc`
- **Fish**: `~/.config/fish/config.fish`

**Exit Codes**:
- `0` - Success
- `1` - Shell detection failed
- `2` - Configuration update failed

---

### doctor

Check and repair environment setup.

```bash
cleen doctor
```

**Diagnostic Areas**:

1. **Directory Structure**
   - Verifies `~/.cleen/` exists
   - Checks `versions/` and `bin/` subdirectories
   - Validates permissions

2. **Installed Versions**
   - Lists all installed versions
   - Verifies binary existence and permissions
   - Checks version metadata consistency

3. **Version Resolution**
   - Shows current directory and project version detection
   - Displays global version setting
   - Explains effective version resolution

4. **Shim Status**
   - Verifies shim existence and integrity
   - Checks PATH configuration
   - Validates symlink targets

5. **Command Testing**
   - Tests `cln --version` execution
   - Validates runtime compatibility
   - Checks for WebAssembly runtime issues

**Output Example**:
```
🔍 Clean Language Manager - Environment Check

📁 Directory Structure:
  cleen directory: "/Users/user/.cleen"
    ✅ exists
  versions directory: "/Users/user/.cleen/versions"
    ✅ exists

📦 Installed Versions:
  v0.4.1 ✅
  v0.4.0 ✅

🔗 Version Resolution:
  📁 Project version: none
  🌐 Global version: v0.4.1
  ⚙️  Effective version: v0.4.1

🧪 Command Test:
  ✅ 'cln --version' works
```

---

### update

Check for Clean Language compiler updates.

```bash
cleen update
```

**Behavior**:
- Fetches latest releases from GitHub
- Compares with currently active version
- Shows available updates and installation instructions
- Updates last check timestamp to avoid frequent API calls

**Output Example**:
```
🔄 Checking for Clean Language compiler updates...
🎉 New version available: v0.4.2 (current: v0.4.1)

To update:
  cleen install latest
  cleen use latest
```

**Exit Codes**:
- `0` - Success (whether updates available or not)
- `1` - Network error or GitHub API failure

---

### self-update

Update cleen itself to the latest version.

```bash
cleen self-update
```

**Behavior**:
- Checks GitHub releases for cleen updates
- Compares with current binary version
- Provides installation instructions if update available
- Updates last self-check timestamp

**Output Example**:
```
🔄 Checking for cleen updates...
🎉 New version available: v0.1.8 (current: v0.1.7)

To update cleen:
  1. Visit: https://github.com/Ivan-Pasco/clean-language-manager/releases/latest
  2. Or use the install script:
     curl -sSL https://github.com/Ivan-Pasco/clean-language-manager/releases/latest/download/install.sh | bash
  3. Or build from source:
     git pull && cargo install --path .
```

**Exit Codes**:
- `0` - Success
- `1` - Network error or GitHub API failure

## Environment Variables

### Supported Variables

- `CLEANMANAGER_HOME` - Override default installation directory (default: `~/.cleen`)
- `CLEANMANAGER_GITHUB_TOKEN` - GitHub personal access token for rate limit increases
- `NO_COLOR` - Disable colored output
- `CLEANMANAGER_VERBOSE` - Enable verbose logging

### Usage Examples

```bash
# Custom installation directory
export CLEANMANAGER_HOME="/opt/cleen"
cleen install latest

# GitHub token for higher rate limits
export CLEANMANAGER_GITHUB_TOKEN="ghp_xxxxxxxxxxxx"
cleen available

# Disable colors for scripting
export NO_COLOR=1
cleen list
```

## Exit Codes

All commands follow standard Unix exit code conventions:

- **0**: Success
- **1**: General error (invalid arguments, version not found, etc.)
- **2**: System error (permissions, network, file system)
- **3**: Configuration error (invalid config files, corrupted state)

## Configuration File Format

### Global Config (`~/.cleen/config.json`)

```json
{
  "active_version": "v0.4.1",
  "installed_versions": {
    "v0.4.1": {
      "installed_at": "2024-01-15T10:30:00Z",
      "binary_path": "/Users/user/.cleen/versions/v0.4.1/cln",
      "platform": "macos-aarch64",
      "github_release": {
        "tag_name": "v0.4.1",
        "published_at": "2024-01-10T12:00:00Z",
        "asset_name": "cln-macos-aarch64"
      }
    }
  },
  "settings": {
    "auto_install_on_use": false,
    "check_updates": true,
    "github_token": null,
    "last_update_check": "1724140800",
    "last_self_update_check": "1724140800"
  }
}
```

### Project Config (`.cleanlanguage/.cleanversion`)

Simple text file containing version specification:
```
v0.3.0
```

## Integration APIs

### Shell Integration

The manager modifies shell configuration files to add the bin directory to PATH:

```bash
# Added by Clean Language Manager
export PATH="$HOME/.cleen/bin:$PATH"
```

### Shim Mechanism

The shim at `~/.cleen/bin/cln` performs version resolution:

1. Check for `.cleanlanguage/.cleanversion` in current directory tree
2. If found, use specified version
3. Otherwise, use global active version
4. Execute appropriate binary with original arguments

## Error Messages

### Common Error Scenarios

**Version Not Found**:
```
Error: Version 'v0.5.0' not found in available releases
Available versions: v0.4.1, v0.4.0, v0.3.0...
```

**Version Not Installed**:
```
Error: Version 'v0.4.0' is not installed
Run 'cleen install v0.4.0' to install it
```

**Network Error**:
```
Error: Failed to fetch releases from GitHub
Check your internet connection and try again
```

**Permission Error**:
```
Error: Permission denied writing to ~/.cleen
Check directory permissions or run with appropriate privileges
```

## Logging and Debugging

### Verbose Mode
Set `CLEANMANAGER_VERBOSE=1` for detailed operation logging:

```bash
export CLEANMANAGER_VERBOSE=1
cleen install v0.4.1
```

**Verbose Output Includes**:
- Network request details
- File system operations
- Configuration changes
- Binary validation steps

### Debug Information
Use `cleen doctor` for comprehensive environment debugging and issue diagnosis.