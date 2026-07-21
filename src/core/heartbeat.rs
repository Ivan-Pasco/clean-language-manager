//! Heartbeat telemetry — reports the active toolchain for the current
//! project so the errors dashboard can advance bugs from `fix_released`
//! to `fix_installed`.
//!
//! Contract: `POST /api/v1/heartbeat` on the errors dashboard, defined in
//! `clean-errors/docs/api/bug-state-machine.md`. The registry is
//! authoritative — this module mirrors the shipped shape exactly.
//!
//! Privacy:
//! - No source code, file paths, or user identity in the body.
//! - `project_hash` is a one-way SHA-256 of `git_remote_origin_url + "|"
//!   + git rev-parse --show-toplevel`. Identical across compiler /
//!   framework / cleen so their reports and heartbeats match.
//! - Outside a git working tree, no heartbeat is sent — the contract
//!   forbids computing `project_hash` there.
//! - Set `CLEEN_HEARTBEAT=off` to disable entirely.

use crate::core::config::{read_active_version, Config};
use crate::plugin::list_installed_plugins;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const API_URL: &str = "https://errors.cleanlanguage.dev/api/v1/heartbeat";
const CURL_TIMEOUT_SECS: u64 = 5;
const WEEKLY_INTERVAL_SECS: u64 = 7 * 24 * 60 * 60;

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Install,
    Weekly,
    Manual,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Kind::Install => "install",
            Kind::Weekly => "weekly",
            Kind::Manual => "manual",
        }
    }
}

/// A single bug the server just transitioned to `fix_installed` because
/// of this heartbeat. Populated from the `advanced_bugs[]` field of the
/// 200 response.
#[derive(Debug, Clone, Deserialize)]
pub struct AdvancedBug {
    pub fingerprint: String,
    pub error_code: String,
    pub component: String,
    pub fixed_in_version: String,
    #[serde(default)]
    pub fix_description: String,
    #[serde(default)]
    pub stage_before: String,
    #[serde(default)]
    pub stage_after: String,
}

/// Fire an install-kind heartbeat describing the current active
/// toolchain. Called by `cleen install` and `cleen frame install` after
/// activation is complete. Reads the compiler + plugin state from the
/// on-disk config; the caller doesn't need to pass anything.
///
/// Prints one line per bug in `advanced_bugs` so the user sees which of
/// their reported bugs just moved to `fix_installed` on this machine.
/// Silent on transport failure — the next heartbeat retries.
pub fn send_install() {
    send(Kind::Install, /* print_advanced_bugs */ true);
}

/// If more than 7 days have elapsed since the last successful weekly
/// heartbeat for this project, fire one. Called at the top of every
/// `cleen` / `frame` invocation. Silent on failure. `accepted: false`
/// from the server (weekly dedup) is treated as success.
///
/// Advanced_bugs from a weekly firing are also printed — a weekly is
/// the fallback path for projects that install through some other
/// channel and never call `cleen install`.
pub fn maybe_send_weekly() {
    if disabled() {
        return;
    }
    let Some(hash) = project_hash() else {
        return;
    };
    let state = load_state(&hash);
    let now = now_epoch_secs();
    if let Some(last) = state.last_weekly_at {
        if now.saturating_sub(last) < WEEKLY_INTERVAL_SECS {
            return;
        }
    }
    send(Kind::Weekly, /* print_advanced_bugs */ true);
}

/// The bytes cleen POSTs and the response it expects, both flat structs
/// so they exist independently of any transient serde_json::Value.

#[derive(Debug, Serialize)]
struct RequestBody {
    project_hash: String,
    heartbeat_kind: &'static str,
    installed: Installed,
    #[serde(skip_serializing_if = "Option::is_none")]
    project: Option<ProjectMeta>,
    client: Client,
}

#[derive(Debug, Serialize)]
struct Installed {
    compiler: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    plugins: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct ProjectMeta {
    name: String,
}

#[derive(Debug, Serialize)]
struct Client {
    cleen_version: String,
    os: String,
    arch: String,
}

#[derive(Debug, Deserialize)]
struct ResponseBody {
    // Documented by the contract: `false` when the server dedup'd this
    // weekly heartbeat. Not acted on — a false is not an error, not a
    // retry signal. Kept in the struct so a rename would show up as a
    // deserialization test failure rather than a silent drop.
    #[allow(dead_code)]
    #[serde(default)]
    accepted: bool,
    #[serde(default)]
    advanced_bugs: Vec<AdvancedBug>,
}

fn send(kind: Kind, print_advanced_bugs: bool) {
    if disabled() {
        return;
    }
    let Some(hash) = project_hash() else {
        return;
    };
    let Ok(config) = Config::load() else {
        return;
    };
    let Some(compiler) = config.active_version.clone() else {
        // No compiler active in this project — nothing meaningful to
        // report. The contract requires installed.compiler.
        return;
    };

    let plugins = collect_plugin_pins(&config);

    let body = RequestBody {
        project_hash: hash.clone(),
        heartbeat_kind: kind.as_str(),
        installed: Installed { compiler, plugins },
        project: None,
        client: Client {
            cleen_version: env!("CARGO_PKG_VERSION").to_string(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        },
    };

    let Ok(payload) = serde_json::to_string(&body) else {
        return;
    };

    let Some((http_ok, body_text)) = post(&payload) else {
        return;
    };
    if !http_ok {
        return;
    }
    let parsed: ResponseBody = serde_json::from_str(&body_text).unwrap_or(ResponseBody {
        accepted: false,
        advanced_bugs: Vec::new(),
    });
    // `accepted: false` is the weekly-dedup path — not an error, not a
    // retry signal. Either way, advanced_bugs is still populated by
    // the server and worth surfacing.
    if print_advanced_bugs {
        print_advanced_bugs_lines(&parsed.advanced_bugs);
    }
    if matches!(kind, Kind::Weekly) {
        let _ = record_weekly_sent(&hash);
    }
}

fn collect_plugin_pins(config: &Config) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Ok(installed) = list_installed_plugins(config) else {
        return out;
    };
    for plugin in installed {
        if let Some(active) = read_active_version(config, &plugin.name) {
            if active == plugin.version {
                out.insert(plugin.name, active);
            }
        }
    }
    out
}

fn print_advanced_bugs_lines(bugs: &[AdvancedBug]) {
    for bug in bugs {
        println!(
            "  🔧 Bug #{fp} ({code}) fixed in {ver} — now installed.",
            fp = short_fp(&bug.fingerprint),
            code = bug.error_code,
            ver = bug.fixed_in_version,
        );
    }
}

fn short_fp(fp: &str) -> String {
    fp.chars().take(12).collect()
}

fn disabled() -> bool {
    matches!(
        std::env::var("CLEEN_HEARTBEAT")
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
        "0" | "off" | "false" | "no"
    )
}

/// POST the payload and return `Some((http_2xx, body_text))` on success,
/// `None` on any transport error. Uses curl for consistency with the
/// rest of the manager (see `core::download`); the write-out format
/// prints the status code on the last line so we can parse it after the
/// body.
fn post(payload: &str) -> Option<(bool, String)> {
    // `-w '\n%{http_code}'` appends the status on its own trailing line;
    // that keeps the body clean even when the server returns no
    // trailing newline of its own.
    let out = Command::new("curl")
        .arg("-sS")
        .arg("--max-time")
        .arg(CURL_TIMEOUT_SECS.to_string())
        .arg("-X")
        .arg("POST")
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-H")
        .arg(format!("User-Agent: cleen/{}", env!("CARGO_PKG_VERSION")))
        .arg("-d")
        .arg(payload)
        .arg("-w")
        .arg("\n%{http_code}")
        .arg(API_URL)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    let combined = String::from_utf8_lossy(&out.stdout);
    let (body, code) = split_body_and_code(&combined);
    let http_ok = matches!(code.as_str(), "200" | "201" | "202" | "204");
    Some((http_ok, body.to_string()))
}

fn split_body_and_code(combined: &str) -> (&str, String) {
    match combined.rsplit_once('\n') {
        Some((body, code)) => (body, code.trim().to_string()),
        None => ("", combined.trim().to_string()),
    }
}

/// Canonical `project_hash` per bug-state-machine.md §project_hash:
/// `SHA256(trim(git_remote_origin_url) + '|' + absolute_repo_root_path)`.
/// Returns `None` when the current directory is not inside a git
/// working tree — the contract forbids computing this outside git.
pub fn project_hash() -> Option<String> {
    let repo_root = git_repo_root()?;
    let remote = git_remote_url();
    let mut hasher = Sha256::new();
    hasher.update(remote.as_bytes());
    hasher.update(b"|");
    hasher.update(repo_root.as_bytes());
    Some(hex_encode(&hasher.finalize()))
}

fn git_repo_root() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn git_remote_url() -> String {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .stderr(Stdio::null())
        .output();
    match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => String::new(),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ------- State file -------
//
// Stored at `~/.cleen/heartbeat/<project_hash>.json`. Small,
// per-project. Records the last successful weekly send so cleen doesn't
// re-fire more than once per 7 days. Install/manual heartbeats do not
// touch this file — they're always sent, and the server's own dedup
// handles duplicates.

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    #[serde(default)]
    last_weekly_at: Option<u64>,
}

fn state_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    Some(home.join(".cleen").join("heartbeat"))
}

fn state_path(project_hash: &str) -> Option<PathBuf> {
    Some(state_dir()?.join(format!("{project_hash}.json")))
}

fn load_state(project_hash: &str) -> State {
    let Some(path) = state_path(project_hash) else {
        return State::default();
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return State::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn record_weekly_sent(project_hash: &str) -> std::io::Result<()> {
    let mut state = load_state(project_hash);
    state.last_weekly_at = Some(now_epoch_secs());
    let Some(path) = state_path(project_hash) else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(&state).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(&path, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_body_shape_matches_contract() {
        // Explicit spec test: the JSON we send must have exactly the
        // top-level keys the contract lists. If a key rename happens
        // here it must also happen in the docs/api/bug-state-machine.md
        // contract — do not diverge silently.
        let body = RequestBody {
            project_hash: "a".repeat(64),
            heartbeat_kind: "install",
            installed: Installed {
                compiler: "0.30.109".into(),
                plugins: {
                    let mut m = BTreeMap::new();
                    m.insert("frame.ui".to_string(), "1.2.3".to_string());
                    m
                },
            },
            project: None,
            client: Client {
                cleen_version: "9.9.9".into(),
                os: "macos".into(),
                arch: "aarch64".into(),
            },
        };
        let json = serde_json::to_value(&body).unwrap();
        assert!(json.get("project_hash").is_some(), "project_hash required");
        assert_eq!(json.get("heartbeat_kind").unwrap(), "install");
        let installed = json.get("installed").expect("installed required");
        assert_eq!(installed.get("compiler").unwrap(), "0.30.109");
        assert_eq!(
            installed
                .get("plugins")
                .and_then(|p| p.get("frame.ui"))
                .unwrap(),
            "1.2.3"
        );
        let client = json.get("client").expect("client required");
        assert!(client.get("cleen_version").is_some());
        assert!(client.get("os").is_some());
        assert!(client.get("arch").is_some());
    }

    #[test]
    fn kind_serializes_to_contract_values() {
        assert_eq!(Kind::Install.as_str(), "install");
        assert_eq!(Kind::Weekly.as_str(), "weekly");
        assert_eq!(Kind::Manual.as_str(), "manual");
    }

    #[test]
    fn response_body_parses_advanced_bugs() {
        // Sample lifted from the contract's 200 OK example. Guards
        // against renamed fields on the response side.
        let sample = r#"{
            "project_hash":  "abc",
            "recorded_at":   "2026-07-14T00:00:00Z",
            "accepted":      true,
            "advanced_bugs": [
              {
                "fingerprint":      "abc123def456",
                "error_code":       "string-heap-init-order",
                "component":        "compiler",
                "fixed_in_version": "0.30.53",
                "fix_description":  "some text",
                "stage_before":     "fix_released",
                "stage_after":      "fix_installed"
              }
            ]
        }"#;
        let parsed: ResponseBody = serde_json::from_str(sample).unwrap();
        assert!(parsed.accepted);
        assert_eq!(parsed.advanced_bugs.len(), 1);
        assert_eq!(parsed.advanced_bugs[0].error_code, "string-heap-init-order");
        assert_eq!(parsed.advanced_bugs[0].fixed_in_version, "0.30.53");
    }

    #[test]
    fn response_body_accepted_false_still_parses() {
        // Weekly-dedup path: accepted=false is a normal success, not a
        // failure. Must parse and advanced_bugs must remain accessible.
        let sample = r#"{
            "project_hash":  "abc",
            "recorded_at":   "2026-07-14T00:00:00Z",
            "accepted":      false,
            "advanced_bugs": []
        }"#;
        let parsed: ResponseBody = serde_json::from_str(sample).unwrap();
        assert!(!parsed.accepted);
        assert!(parsed.advanced_bugs.is_empty());
    }

    #[test]
    fn split_body_and_code_extracts_trailing_status() {
        let combined = "{\"accepted\":true}\n200";
        let (body, code) = split_body_and_code(combined);
        assert_eq!(body, "{\"accepted\":true}");
        assert_eq!(code, "200");
    }

    #[test]
    fn split_body_and_code_handles_empty_body() {
        let (body, code) = split_body_and_code("\n204");
        assert_eq!(body, "");
        assert_eq!(code, "204");
    }

    #[test]
    fn project_hash_is_64_hex_or_none() {
        // Runs both in a git repo (returns Some) and outside one
        // (returns None). Whichever we hit, the invariants below must
        // hold.
        match project_hash() {
            Some(h) => {
                assert_eq!(h.len(), 64);
                assert!(h.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
            }
            None => {}
        }
    }

    #[test]
    fn disabled_env_stops_send() {
        // We can't run a real POST in a unit test, but we can verify
        // the disabled() guard honors CLEEN_HEARTBEAT=off. Not thread-
        // safe across parallel tests — this test just proves parsing.
        for val in ["off", "0", "false", "no", "OFF", "False"] {
            // SAFETY: single-threaded within this test body.
            unsafe {
                std::env::set_var("CLEEN_HEARTBEAT", val);
            }
            assert!(
                disabled(),
                "CLEEN_HEARTBEAT={val} should disable heartbeats"
            );
        }
        unsafe {
            std::env::remove_var("CLEEN_HEARTBEAT");
        }
        assert!(!disabled());
    }

    #[test]
    fn state_roundtrip() {
        let hash = "test-hash-not-real-weekly";
        if let Some(p) = state_path(hash) {
            let _ = std::fs::remove_file(&p);
        }
        record_weekly_sent(hash).unwrap();
        let state = load_state(hash);
        assert!(state.last_weekly_at.is_some());
        if let Some(p) = state_path(hash) {
            let _ = std::fs::remove_file(&p);
        }
    }

    #[test]
    fn short_fp_truncates_to_twelve() {
        assert_eq!(short_fp("abcdef0123456789abcdef"), "abcdef012345");
        assert_eq!(short_fp("short"), "short");
    }
}
