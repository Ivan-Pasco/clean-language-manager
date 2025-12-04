# 🧼 Clean Language Manager – Functional Definition

**`cleen`** is the official version manager for the Clean Language compiler (`cln`). It allows developers to easily install, switch, and manage multiple versions of Clean Language across macOS, Linux, and Windows systems. It is written in **Rust** and distributed as a native executable via GitHub Releases.

---

## 📦 Core Features

### a. Download and Install Binaries

* Downloads pre-built compiler binaries (or optionally source code) from [GitHub Releases](https://github.com/Ivan-Pasco/clean-language-compiler/releases).
* Each version is stored in its own isolated directory:

```
~/.cleen/versions/1.2.3/cln
```

* This ensures clean separation of versions and simplifies upgrades/downgrades.

---

### b. Registering the Command

* `cleen` installs a **shim** or **symbolic link** in the bin directory:

```
~/.cleen/bin/cln
```

* This path is added to the user's environment `PATH`.
* When the user runs `cln`, it routes to the **currently active version**.

---

### c. Switching Versions

* To activate a specific version, the user runs:

```bash
cleen use 1.2.3
```

* The symlink is updated to point to the selected version's binary.
* **Optional:** The manager can support per-project version overrides via a `.cleen` file in the project directory.

---

### d. Uninstalling and Upgrading

* Versions can be uninstalled with:

```bash
cleen uninstall 1.2.3
```

* The latest version can be upgraded or installed with:

```bash
cleen install latest
```

* Old versions can be listed and cleaned automatically to free up space.

---

## ⚙️ What Happens Under the Hood?

| Step           | Description                                                                |
| -------------- | -------------------------------------------------------------------------- |
| **Download**   | Fetches the requested version's binary or source archive from GitHub       |
| **Extract**    | Unpacks the file to `~/.cleen/versions/<version>/`                  |
| **Shim Link**  | Creates/updates `~/.cleen/bin/cln` to point to the chosen binary |
| **PATH Check** | Ensures `~/.cleen/bin/` is included in `.bashrc`, `.zshrc`, etc.    |
| **Execution**  | Running `cln` invokes the shim, which redirects to the active version   |

---

## 🧪 Example Workflow

```bash
cleen install 1.2.3
# → Downloads and installs Clean Language v1.2.3

cleen use 1.2.3
# → Activates version 1.2.3 as the default

cln --version
# → Runs ~/.cleen/versions/1.2.3/cln --version
```

---

## 🧰 Additional Planned Commands

| Command                        | Description                                 |
| ------------------------------ | ------------------------------------------- |
| `cleen list`            | Lists all installed versions                |
| `cleen available`       | Lists available versions from GitHub        |
| `cleen uninstall x.y.z` | Removes a specific installed version        |
| `cleen doctor`          | Checks and repairs environment config       |
| `cleen init`            | Adds the bin path to your shell config file |

---

## 🛠 Deployment & Distribution

* Built in Rust for safety and performance
* Distributed via GitHub Actions:

  * Linux (x86\_64)
  * macOS (x86\_64 / ARM64)
  * Windows (x86\_64)
* Each release includes a downloadable binary with platform-specific naming
* Installable via curl script, e.g.:

```bash
curl -sSL https://github.com/Ivan-Pasco/clean-language-compiler/releases/latest/download/install.sh | bash
```

---

## 📦 Plugin Management Features

Plugins extend Clean Language by providing framework-specific functionality. Plugins are written in Clean Language itself, compiled to WebAssembly, and loaded by the compiler during compilation.

### Plugin Commands

| Command | Description |
|---------|-------------|
| `cleen plugin install <name>` | Install plugin from registry |
| `cleen plugin install <name>@<ver>` | Install specific version |
| `cleen plugin create <name>` | Scaffold new plugin project |
| `cleen plugin build` | Build current plugin to WASM |
| `cleen plugin publish` | Publish to registry |
| `cleen plugin list` | List installed plugins |
| `cleen plugin remove <name>` | Remove installed plugin |

---

### Plugin Directory Structure

Plugins are installed to `~/.cleen/plugins/`:

```
~/.cleen/
├── versions/           # (existing) Compiler versions
│   └── 0.15.0/
│       └── cln
├── plugins/            # (new) Installed plugins
│   ├── frame.web/
│   │   └── 1.0.0/
│   │       ├── plugin.toml
│   │       └── plugin.wasm
│   └── frame.data/
│       └── 1.0.0/
│           ├── plugin.toml
│           └── plugin.wasm
├── bin/                # (existing) Shim directory
│   └── cln
└── config.toml         # (existing) Configuration
```

---

### Plugin Manifest Format (plugin.toml)

Each plugin includes a `plugin.toml` manifest that describes the plugin metadata and capabilities:

```toml
[plugin]
name = "frame.web"
version = "1.0.0"
description = "Web framework plugin for Clean Language"
author = "Clean Language Team"
license = "MIT"

[compatibility]
min_compiler_version = "0.15.0"

[exports]
expand = "expand_block"
validate = "validate_block"
```

**Fields:**
- `name`: Unique plugin identifier (e.g., `frame.web`, `frame.data`)
- `version`: Semantic version of the plugin
- `description`: Human-readable description
- `author`: Plugin author or organization
- `license`: SPDX license identifier
- `min_compiler_version`: Minimum required compiler version
- `expand`: Entry point function for block expansion
- `validate`: Entry point function for block validation

---

### Plugin Scaffold Template

Running `cleen plugin create <name>` generates a new plugin project:

```
<name>/
├── plugin.toml
├── src/
│   └── main.cln      # Plugin source in Clean Language
├── tests/
│   └── test_expand.cln
└── README.md
```

**Generated `src/main.cln` template:**

```clean
// Plugin: <name>
// Expand framework blocks into Clean Language code

expand_block(block_name: string, attributes: string, body: string) -> string
    // TODO: Implement block expansion
    return body

validate_block(block_name: string, attributes: string, body: string) -> boolean
    // TODO: Implement block validation
    return true
```

---

### Plugin Workflow

**Installing a Plugin:**
```bash
cleen plugin install frame.web
# → Downloads and installs frame.web plugin

cleen plugin install frame.web@1.0.0
# → Installs specific version 1.0.0
```

**Creating a New Plugin:**
```bash
cleen plugin create my-plugin
# → Creates my-plugin/ directory with scaffold

cd my-plugin
cleen plugin build
# → Compiles src/main.cln to plugin.wasm

cleen plugin publish
# → Publishes to the plugin registry
```

**Managing Installed Plugins:**
```bash
cleen plugin list
# → Shows all installed plugins

cleen plugin remove frame.web
# → Removes the frame.web plugin
```

---

### Plugin Registry

Plugins are distributed via a central registry at `https://plugins.cleanlang.org` (planned). The registry provides:

- **Discovery**: Search and browse available plugins
- **Version Management**: Semantic versioning with compatibility checks
- **Integrity**: Checksum verification for downloaded plugins
- **Metadata**: Description, documentation, and dependency information

---

### How Plugins Work

1. **Source Code**: Plugins are written in Clean Language using the plugin API
2. **Compilation**: `cleen plugin build` compiles `.cln` files to `plugin.wasm`
3. **Installation**: Plugins are installed to `~/.cleen/plugins/<name>/<version>/`
4. **Loading**: The compiler loads plugins based on `import:` blocks in source files
5. **Execution**: Plugin WASM is executed at compile time to transform code 