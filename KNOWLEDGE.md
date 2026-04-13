# KNOWLEDGE.md — Clean Manager (cleen)

Known considerations. Read before modifying manager code.

---

## 1. Version Resolution

**What:** The manager (`cleen`) handles installing, switching, and managing Clean Language compiler versions. Version resolution must handle semver ranges, "latest" alias, and local development builds.

**Watch for:** Version string parsing edge cases, network failures during downloads, interrupted installations leaving partial state.

---

## 2. Plugin Registry

**What:** The manager handles plugin installation from the registry. Plugin checksum verification is planned but not yet implemented.

**Where:** `plugin/registry.rs`

**Watch for:** Installing plugins without checksum verification means tampered packages won't be detected. Track this as a security improvement.

---

## 3. Platform Differences

**What:** Windows process checking is not yet implemented in `core/frame.rs`. The manager currently assumes Unix-like behavior for process management.

**Watch for:** Any process management code needs platform-conditional logic for Windows support.
