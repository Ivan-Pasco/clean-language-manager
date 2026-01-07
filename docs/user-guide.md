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

## Frame CLI Management

Frame is the official full-stack web framework for Clean Language. The `cleen frame` commands help you manage Frame CLI installations and run Frame applications.

### Installing Frame CLI

**Install Frame CLI (auto-detects compatible version)**:
```bash
cleen frame install
```

**Install a specific version**:
```bash
cleen frame install 1.0.0
```

**See what's installed**:
```bash
cleen frame list
```

### Creating Frame Projects

**Create a new Frame project**:
```bash
cleen frame new myapp
```

This creates a new Frame project using the default API template. Available templates:
- `api` - API-only backend server (default)
- `web` - Full-stack web application (frontend + backend)
- `minimal` - Bare minimum single-file project

**Create with a specific template**:
```bash
cleen frame new myapp --template web
cleen frame new myapp --template api
cleen frame new myapp --template minimal
```

**Customize the port**:
```bash
cleen frame new myapp --port 8080
```

The generated project structure for the `api` template:
```
myapp/
├── app/
│   └── api/
│       └── main.cln          # API endpoints
├── config/
│   └── app.cln              # Configuration
├── dist/                    # Build output
├── config.cln               # Project configuration
└── .gitignore
```

### Building Frame Projects

**Build for production**:
```bash
cd myapp
cleen frame build
```

This will:
1. Find the entry point (from `config.cln` or default locations)
2. Compile with plugin support
3. Optimize the WASM output
4. Output to `dist/` directory

**Build with custom paths**:
```bash
cleen frame build --input ./app/api/main.cln --output ./build
```

**Set optimization level**:
```bash
cleen frame build --optimize 3    # Maximum optimization
cleen frame build --optimize 0    # No optimization (faster builds)
```

Optimization levels:
- `0` - No optimization (fastest compilation)
- `1` - Basic optimization
- `2` - Default optimization (balance speed/size)
- `3` - Maximum optimization (smallest size)

### Running Frame Applications

**Start a development server**:
```bash
cleen frame serve main.cln
```

This will:
1. Compile your `.cln` source file with plugins enabled
2. Start the frame-runtime HTTP server
3. Listen on http://127.0.0.1:3000 by default

**Customize port and host**:
```bash
cleen frame serve main.cln --port 8080 --host 0.0.0.0
```

**Enable debug output**:
```bash
cleen frame serve main.cln --debug
```

**Stop a running server**:
```bash
cleen frame stop
```

### Frame Serve Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--port` | `-p` | Port to listen on | 3000 |
| `--host` | | Host to bind to | 127.0.0.1 |
| `--debug` | `-d` | Enable debug output | false |

### Example: Running an API Server

Create a file `api.cln` with endpoints:

```clean
import:
	frame.web

endpoints:
	GET /api/hello:
		return "Hello, World!"

	GET /api/users/:id:
		string id = req.params.id
		return "User ID: " + id
```

Run it:
```bash
cleen frame serve api.cln
```

Test with curl:
```bash
curl http://localhost:3000/api/hello
# Output: Hello, World!

curl http://localhost:3000/api/users/123
# Output: User ID: 123
```

### Troubleshooting Frame

**Compilation errors**:
```bash
# The serve command will show compilation errors
cleen frame serve main.cln

# For detailed output, use debug mode
cleen frame serve main.cln --debug
```

**Port already in use**:
```bash
# Check if a server is already running
cleen frame stop

# Or use a different port
cleen frame serve main.cln --port 8080
```

**frame-runtime not found**:
```bash
# Ensure Frame CLI is installed
cleen frame install

# Check Frame installation
cleen frame list
```

## Clean Server Management

Clean Server is the runtime execution environment for compiled Clean Language WASM applications. It provides HTTP server capabilities, database connectivity, and other system integrations.

### Installing Clean Server

**Install Clean Server (gets latest version)**:
```bash
cleen server install
```

**Install a specific version**:
```bash
cleen server install 1.0.0
```

**Note**: When you run `cleen frame install`, Clean Server is automatically installed if not already present.

### Managing Server Versions

**See what's installed**:
```bash
cleen server list
```

**Switch to a specific version**:
```bash
cleen server use 1.0.0
```

**Remove an unused version**:
```bash
cleen server uninstall 1.0.0
```

**Check server status**:
```bash
cleen server status
```

### Running WASM Applications

**Run a compiled WASM file**:
```bash
cleen server run app.wasm
```

**Customize port and host**:
```bash
cleen server run app.wasm --port 8080 --host 0.0.0.0
```

### Server Run Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--port` | `-p` | Port to listen on | 3000 |
| `--host` | | Host to bind to | 127.0.0.1 |

### Directory Layout

```
~/.cleen/
├── server/
│   ├── 1.0.0/
│   │   └── clean-server       # Clean Server v1.0.0 binary
│   └── 1.1.0/
│       └── clean-server       # Clean Server v1.1.0 binary
└── config.json               # Tracks active server version
```

### Troubleshooting Server

**Server binary not found**:
```bash
# Check server is installed
cleen server list

# Install if missing
cleen server install
```

**WASM file not found**:
```bash
# Ensure the WASM file exists
ls app.wasm

# Build your project first
cleen frame build
```

**Port already in use**:
```bash
# Use a different port
cleen server run app.wasm --port 8080

# Or find and stop the process using the port
lsof -i :3000
```

## Plugin Management

Plugins extend Clean Language with framework-specific functionality. Plugins are written in Clean Language and compiled to WebAssembly.

### Installing Plugins

**Install a plugin from the registry**:
```bash
cleen plugin install frame.web
```

**Install a specific version**:
```bash
cleen plugin install frame.web@1.0.0
```

**See what's installed**:
```bash
cleen plugin list
```

### Creating a Plugin

**Create a new plugin project**:
```bash
cleen plugin create my-plugin
cd my-plugin
```

This creates:
```
my-plugin/
├── plugin.toml       # Plugin manifest
├── src/
│   └── main.cln     # Plugin source
├── tests/
│   └── test_expand.cln
└── README.md
```

**Build your plugin**:
```bash
cleen plugin build
```

This compiles `src/main.cln` to `plugin.wasm`.

**Publish to the registry**:
```bash
cleen plugin publish
```

### Removing Plugins

**Remove an installed plugin**:
```bash
cleen plugin remove frame.web
```

### Plugin Project Structure

#### plugin.toml

The manifest file defines your plugin:

```toml
[plugin]
name = "my-plugin"
version = "1.0.0"
description = "My custom Clean Language plugin"
author = "Your Name"
license = "MIT"

[compatibility]
min_compiler_version = "0.15.0"

[exports]
expand = "expand_block"
validate = "validate_block"
```

#### src/main.cln

The entry point for your plugin:

```clean
// Plugin: my-plugin
// Expand framework blocks into Clean Language code

expand_block(block_name: string, attributes: string, body: string) -> string
    // Implement block expansion logic here
    return body

validate_block(block_name: string, attributes: string, body: string) -> boolean
    // Implement validation logic here
    return true
```

### Plugin Development Workflow

1. **Create the plugin project**:
   ```bash
   cleen plugin create my-framework
   cd my-framework
   ```

2. **Edit the source code**:
   ```bash
   # Edit src/main.cln with your plugin logic
   ```

3. **Build and test**:
   ```bash
   cleen plugin build
   # Run tests
   cln test tests/test_expand.cln
   ```

4. **Publish when ready**:
   ```bash
   cleen plugin publish
   ```

### Troubleshooting Plugins

**Plugin build fails**:
```bash
# Check compiler is installed
cln --version

# Ensure you're in the plugin directory
ls plugin.toml

# Check for syntax errors in source
cln check src/main.cln
```

**Plugin not loading**:
```bash
# Verify plugin is installed
cleen plugin list

# Check plugin location
ls ~/.cleen/plugins/

# Verify plugin.wasm exists
ls ~/.cleen/plugins/<name>/<version>/plugin.wasm
```

**Compatibility errors**:
```bash
# Check your compiler version
cln --version

# Check plugin requirements
cat ~/.cleen/plugins/<name>/<version>/plugin.toml
```

### Plugin Directory Layout

```
~/.cleen/
├── plugins/
│   ├── frame.web/
│   │   └── 1.0.0/
│   │       ├── plugin.toml
│   │       └── plugin.wasm
│   └── frame.data/
│       └── 0.5.0/
│           ├── plugin.toml
│           └── plugin.wasm
└── config.json        # Tracks active plugins
```