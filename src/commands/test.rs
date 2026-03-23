use crate::core::config::Config;
use crate::error::{CleenError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// Result of a single test execution
struct TestFileResult {
    file: String,
    tests: Vec<SingleTestResult>,
    compile_error: Option<String>,
}

struct SingleTestResult {
    name: String,
    passed: bool,
    message: Option<String>,
    duration_ms: u64,
}

/// Run tests in the project
pub fn run_tests(
    file: Option<&str>,
    filter: Option<&str>,
    verbose: bool,
    timing: bool,
) -> Result<()> {
    let config = Config::load()?;

    // Find the compiler binary
    let compiler_path = get_compiler_path(&config)?;

    // Discover test files
    let test_files = discover_test_files(file, filter)?;

    if test_files.is_empty() {
        println!("No test files found.");
        if file.is_some() {
            println!("   Check that the specified file exists and contains tests: blocks");
        } else {
            println!("   Test files should be in app/ or tests/ directories with .cln extension");
        }
        return Ok(());
    }

    println!("Running tests...\n");

    let overall_start = Instant::now();
    let mut total_passed = 0u32;
    let mut total_failed = 0u32;
    let mut total_errors = 0u32;
    let mut results: Vec<TestFileResult> = Vec::new();

    // Create temp directory for compiled test WASM files
    let temp_dir = std::env::temp_dir().join("cleen-tests");
    if !temp_dir.exists() {
        std::fs::create_dir_all(&temp_dir)?;
    }

    for test_file in &test_files {
        let result = run_test_file(&compiler_path, test_file, &temp_dir, filter, verbose)?;

        match &result.compile_error {
            Some(_) => total_errors += 1,
            None => {
                for test in &result.tests {
                    if test.passed {
                        total_passed += 1;
                    } else {
                        total_failed += 1;
                    }
                }
            }
        }

        results.push(result);
    }

    let overall_duration = overall_start.elapsed();

    // Print results
    println!();
    for result in &results {
        println!("  {}", result.file);

        if let Some(err) = &result.compile_error {
            println!("    \x1b[31m✗ compilation failed\x1b[0m");
            if verbose {
                for line in err.lines() {
                    println!("      {line}");
                }
            } else {
                // Show first line of error
                if let Some(first_line) = err.lines().next() {
                    println!("      {first_line}");
                }
            }
            println!();
            continue;
        }

        for test in &result.tests {
            if test.passed {
                if timing {
                    println!(
                        "    \x1b[32m✓\x1b[0m {} ({}ms)",
                        test.name, test.duration_ms
                    );
                } else {
                    println!("    \x1b[32m✓\x1b[0m {}", test.name);
                }
            } else {
                println!("    \x1b[31m✗\x1b[0m {}", test.name);
                if let Some(msg) = &test.message {
                    for line in msg.lines() {
                        println!("      {line}");
                    }
                }
            }
        }
        println!();
    }

    // Summary
    let total = total_passed + total_failed;
    let status_color = if total_failed > 0 || total_errors > 0 {
        "\x1b[31m"
    } else {
        "\x1b[32m"
    };

    print!("{status_color}Results: {total_passed} passed");
    if total_failed > 0 {
        print!(", {total_failed} failed");
    }
    if total_errors > 0 {
        print!(", {total_errors} compile errors");
    }
    println!(", {total} total\x1b[0m");

    if timing {
        println!("Time: {:.2}s", overall_duration.as_secs_f64());
    }

    // Clean up temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    // Exit with appropriate code
    if total_failed > 0 {
        std::process::exit(1);
    } else if total_errors > 0 {
        std::process::exit(2);
    }

    Ok(())
}

/// Get the path to the active compiler binary
fn get_compiler_path(config: &Config) -> Result<PathBuf> {
    let version = config
        .get_effective_version()
        .ok_or_else(|| CleenError::TestError {
            message: "No compiler version is active. Run 'cleen install <version>' first"
                .to_string(),
        })?;

    let binary_path = config.get_version_binary(&version);

    if !binary_path.exists() {
        return Err(CleenError::TestError {
            message: format!(
                "Compiler binary not found for version {version}. Run 'cleen install {version}'"
            ),
        });
    }

    Ok(binary_path)
}

/// Discover test files in the project
fn discover_test_files(file: Option<&str>, filter: Option<&str>) -> Result<Vec<PathBuf>> {
    // If a specific file is given, just use that
    if let Some(path) = file {
        let p = PathBuf::from(path);
        if !p.exists() {
            return Err(CleenError::FileNotFound {
                path: path.to_string(),
            });
        }
        return Ok(vec![p]);
    }

    let mut test_files = Vec::new();
    let cwd = std::env::current_dir()?;

    // Scan standard directories for .cln files
    let scan_dirs = ["tests", "app", "src"];

    for dir_name in &scan_dirs {
        let dir = cwd.join(dir_name);
        if dir.exists() && dir.is_dir() {
            scan_directory_for_tests(&dir, &mut test_files)?;
        }
    }

    // Also check for .cln files in the project root
    if let Ok(entries) = std::fs::read_dir(&cwd) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path.extension().is_some_and(|e| e == "cln")
                && file_contains_tests(&path)
            {
                test_files.push(path);
            }
        }
    }

    // Apply filter if provided
    if let Some(pattern) = filter {
        let pattern_lower = pattern.to_lowercase();
        test_files.retain(|f| f.to_string_lossy().to_lowercase().contains(&pattern_lower));
    }

    // Sort for consistent output
    test_files.sort();

    Ok(test_files)
}

/// Recursively scan a directory for .cln files containing tests
fn scan_directory_for_tests(dir: &Path, results: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir)?;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            scan_directory_for_tests(&path, results)?;
        } else if path.is_file()
            && path.extension().is_some_and(|e| e == "cln")
            && file_contains_tests(&path)
        {
            results.push(path);
        }
    }

    Ok(())
}

/// Check if a .cln file contains a tests: block
fn file_contains_tests(path: &Path) -> bool {
    match std::fs::read_to_string(path) {
        Ok(content) => content.contains("tests:"),
        Err(_) => false,
    }
}

/// Compile and run tests for a single file
fn run_test_file(
    compiler_path: &Path,
    test_file: &Path,
    temp_dir: &Path,
    filter: Option<&str>,
    verbose: bool,
) -> Result<TestFileResult> {
    let file_display = test_file
        .strip_prefix(std::env::current_dir().unwrap_or_default())
        .unwrap_or(test_file)
        .to_string_lossy()
        .to_string();

    // Generate output WASM path in temp directory
    let stem = test_file
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let wasm_path = temp_dir.join(format!("{stem}_test.wasm"));

    // Step 1: Compile with --test-mode
    if verbose {
        println!("  Compiling {file_display}...");
    }

    let compile_output = Command::new(compiler_path)
        .args([
            "compile",
            "-i",
            &test_file.to_string_lossy(),
            "-o",
            &wasm_path.to_string_lossy(),
            "--test-mode",
        ])
        .output()
        .map_err(|e| CleenError::TestError {
            message: format!("Failed to run compiler: {e}"),
        })?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&compile_output.stdout).to_string();
        let error_msg = if stderr.is_empty() { stdout } else { stderr };

        return Ok(TestFileResult {
            file: file_display,
            tests: Vec::new(),
            compile_error: Some(error_msg),
        });
    }

    // Step 2: Run tests via compiler's run-test subcommand
    let mut run_args = vec![
        "run-test".to_string(),
        wasm_path.to_string_lossy().to_string(),
        "--json".to_string(),
    ];

    if let Some(f) = filter {
        run_args.push("--filter".to_string());
        run_args.push(f.to_string());
    }

    let start = Instant::now();

    let run_output = Command::new(compiler_path)
        .args(&run_args)
        .output()
        .map_err(|e| CleenError::TestError {
            message: format!("Failed to run tests: {e}"),
        })?;

    let elapsed = start.elapsed();

    // Parse test results from JSON output
    let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
    let tests = parse_test_results(&stdout, elapsed.as_millis() as u64);

    Ok(TestFileResult {
        file: file_display,
        tests,
        compile_error: None,
    })
}

/// Parse test results from JSON output
///
/// Expected JSON format:
/// ```json
/// {
///   "tests": [
///     {"name": "test name", "passed": true, "duration_ms": 2},
///     {"name": "failing test", "passed": false, "message": "Expected: ...\nGot: ...", "duration_ms": 1}
///   ]
/// }
/// ```
fn parse_test_results(output: &str, fallback_duration: u64) -> Vec<SingleTestResult> {
    // Try to parse as JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(tests) = json.get("tests").and_then(|t| t.as_array()) {
            return tests
                .iter()
                .map(|t| SingleTestResult {
                    name: t
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    passed: t.get("passed").and_then(|p| p.as_bool()).unwrap_or(false),
                    message: t
                        .get("message")
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string()),
                    duration_ms: t.get("duration_ms").and_then(|d| d.as_u64()).unwrap_or(0),
                })
                .collect();
        }
    }

    // If JSON parsing fails, try line-based output parsing
    // Format: PASS: test name (2ms) or FAIL: test name: message
    let mut results = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("PASS:") {
            let name = line
                .trim_start_matches("PASS:")
                .trim()
                .split('(')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            results.push(SingleTestResult {
                name,
                passed: true,
                message: None,
                duration_ms: 0,
            });
        } else if line.starts_with("FAIL:") {
            let rest = line.trim_start_matches("FAIL:").trim();
            let (name, message) = if let Some((n, m)) = rest.split_once(':') {
                (n.trim().to_string(), Some(m.trim().to_string()))
            } else {
                (rest.to_string(), None)
            };
            results.push(SingleTestResult {
                name,
                passed: false,
                message,
                duration_ms: 0,
            });
        }
    }

    // If nothing parsed, create a single result based on exit status
    if results.is_empty() && !output.trim().is_empty() {
        results.push(SingleTestResult {
            name: "test suite".to_string(),
            passed: false,
            message: Some(output.trim().to_string()),
            duration_ms: fallback_duration,
        });
    }

    results
}
