# Clean Language Manager

A Rust-based version manager for the Clean Language compiler (`cln`). Allows developers to easily install, switch, and manage multiple versions of Clean Language across macOS, Linux, and Windows systems.

## Status

✅ **Ready for Use** - Full version management functionality implemented with automated releases.

## Features

- **Version Management**: Install and manage multiple Clean Language compiler versions
- **Easy Switching**: Switch between versions with simple commands
- **Cross-Platform**: Native support for macOS (Intel & Apple Silicon), Linux, and Windows
- **Shell Integration**: Automatic PATH configuration and environment setup
- **Environment Diagnostics**: Built-in health checking and troubleshooting

## Installation

### Quick Install (Recommended)

#### Unix/Linux/macOS
```bash
curl -sSL https://github.com/Ivan-Pasco/clean-language-manager/releases/latest/download/install.sh | bash
```

#### Windows (PowerShell)
```powershell
iwr https://github.com/Ivan-Pasco/clean-language-manager/releases/latest/download/install.ps1 | iex
```

### Manual Installation

1. **Download** the appropriate binary for your platform from [Releases](https://github.com/Ivan-Pasco/clean-language-manager/releases/latest):
   - **Linux (x86_64)**: `cleanmanager-linux-x86_64.tar.gz`
   - **macOS (Intel)**: `cleanmanager-macos-x86_64.tar.gz`
   - **macOS (Apple Silicon)**: `cleanmanager-macos-aarch64.tar.gz`
   - **Windows (x86_64)**: `cleanmanager-windows-x86_64.zip`

2. **Extract** the archive:
   ```bash
   # Unix/Linux/macOS
   tar -xzf cleanmanager-*.tar.gz
   
   # Windows
   # Extract using Windows Explorer or 7-Zip
   ```

3. **Move** the binary to a directory in your PATH:
   ```bash
   # Unix/Linux/macOS
   sudo mv cleanmanager /usr/local/bin/
   
   # Or to user directory
   mkdir -p ~/.local/bin
   mv cleanmanager ~/.local/bin/
   ```

4. **Initialize** the environment:
   ```bash
   cleanmanager init
   ```

### From Source

```bash
git clone https://github.com/Ivan-Pasco/clean-language-manager.git
cd clean-language-manager
cargo build --release
cp target/release/cleanmanager ~/.local/bin/  # or your preferred location
```

## Usage

### Initial Setup
```bash
# Set up your environment (adds cleanmanager to PATH)
cleanmanager init

# Verify everything is working
cleanmanager doctor
```

### Version Management
```bash
# Install a specific version
cleanmanager install 1.2.3

# List installed versions
cleanmanager list

# Switch to a version (makes it active)
cleanmanager use 1.2.3

# List available versions from GitHub
cleanmanager available

# Uninstall a version
cleanmanager uninstall 1.2.3
```

### Getting Help
```bash
# Show help for all commands
cleanmanager --help

# Show help for a specific command
cleanmanager install --help
```

## Architecture

The manager organizes versions in isolated directories:

```
~/.cleanmanager/
├── bin/cln                    # Shim to active version
├── versions/
│   ├── 1.2.3/cln             # Version-specific binaries
│   └── 1.3.0/cln
└── config.json               # Manager configuration
```

## How It Works

Clean Language Manager stores different compiler versions in isolated directories and uses a shim system to route the `cln` command to the currently active version:

1. **Download**: Fetches compiler binaries from GitHub releases
2. **Install**: Extracts and stores each version in `~/.cleanmanager/versions/<version>/`
3. **Activate**: Creates a symlink/shim in `~/.cleanmanager/bin/cln` pointing to the active version
4. **Route**: When you run `cln`, it automatically uses the active version

## Troubleshooting

### Command Not Found
If you get `command not found: cleanmanager`:
1. Make sure the binary is in your PATH: `echo $PATH`
2. Run `cleanmanager init` to get setup instructions
3. Restart your terminal after updating your shell configuration

### Clean Language Not Working
If `cln` doesn't work after installation:
1. Run `cleanmanager doctor` to diagnose issues
2. Make sure you've activated a version: `cleanmanager use <version>`
3. Check that `~/.cleanmanager/bin` is in your PATH

### Permission Issues
If you get permission errors:
- On Unix systems, make sure the binary is executable: `chmod +x cleanmanager`
- On Windows, you may need to run PowerShell as Administrator

## Development

See [TASKS.md](TASKS.md) for current implementation progress and [CLAUDE.md](CLAUDE.md) for development guidance.

### Building from Source

```bash
# Check compilation
cargo check

# Run with help
cargo run -- --help

# Test a command
cargo run -- doctor

# Build release binary
cargo build --release
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## Related Projects

- [Clean Language Compiler](https://github.com/Ivan-Pasco/clean-language-compiler) - The main compiler this tool manages

## License

MIT License