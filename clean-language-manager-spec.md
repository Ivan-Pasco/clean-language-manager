# 🧼 Clean Language Manager – Functional Definition

**`cleanmanager`** is the official version manager for the Clean Language compiler (`cln`). It allows developers to easily install, switch, and manage multiple versions of Clean Language across macOS, Linux, and Windows systems. It is written in **Rust** and distributed as a native executable via GitHub Releases.

---

## 📦 Core Features

### a. Download and Install Binaries

* Downloads pre-built compiler binaries (or optionally source code) from [GitHub Releases](https://github.com/Ivan-Pasco/clean-language-compiler/releases).
* Each version is stored in its own isolated directory:

```
~/.cleanmanager/versions/1.2.3/cln
```

* This ensures clean separation of versions and simplifies upgrades/downgrades.

---

### b. Registering the Command

* `cleanmanager` installs a **shim** or **symbolic link** in the bin directory:

```
~/.cleanmanager/bin/cln
```

* This path is added to the user's environment `PATH`.
* When the user runs `cln`, it routes to the **currently active version**.

---

### c. Switching Versions

* To activate a specific version, the user runs:

```bash
cleanmanager use 1.2.3
```

* The symlink is updated to point to the selected version's binary.
* **Optional:** The manager can support per-project version overrides via a `.cleanmanager` file in the project directory.

---

### d. Uninstalling and Upgrading

* Versions can be uninstalled with:

```bash
cleanmanager uninstall 1.2.3
```

* The latest version can be upgraded or installed with:

```bash
cleanmanager install latest
```

* Old versions can be listed and cleaned automatically to free up space.

---

## ⚙️ What Happens Under the Hood?

| Step           | Description                                                                |
| -------------- | -------------------------------------------------------------------------- |
| **Download**   | Fetches the requested version's binary or source archive from GitHub       |
| **Extract**    | Unpacks the file to `~/.cleanmanager/versions/<version>/`                  |
| **Shim Link**  | Creates/updates `~/.cleanmanager/bin/cln` to point to the chosen binary |
| **PATH Check** | Ensures `~/.cleanmanager/bin/` is included in `.bashrc`, `.zshrc`, etc.    |
| **Execution**  | Running `cln` invokes the shim, which redirects to the active version   |

---

## 🧪 Example Workflow

```bash
cleanmanager install 1.2.3
# → Downloads and installs Clean Language v1.2.3

cleanmanager use 1.2.3
# → Activates version 1.2.3 as the default

cln --version
# → Runs ~/.cleanmanager/versions/1.2.3/cln --version
```

---

## 🧰 Additional Planned Commands

| Command                        | Description                                 |
| ------------------------------ | ------------------------------------------- |
| `cleanmanager list`            | Lists all installed versions                |
| `cleanmanager available`       | Lists available versions from GitHub        |
| `cleanmanager uninstall x.y.z` | Removes a specific installed version        |
| `cleanmanager doctor`          | Checks and repairs environment config       |
| `cleanmanager init`            | Adds the bin path to your shell config file |

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