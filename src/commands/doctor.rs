use crate::core::{
    compatibility, config::Config, frame, shim::ShimManager, version::VersionManager,
};
use crate::error::{CleenError, Result};
use std::env;
use std::process::Command;

pub fn check_environment(check_frame: bool) -> Result<()> {
    println!("🔍 Clean Language Manager - Environment Check");
    println!();

    let config = Config::load()?;
    let version_manager = VersionManager::new(config.clone());
    let _shim_manager = ShimManager::new(config.clone());

    let mut issues_found = 0;

    // Check cleen directories
    println!("📁 Directory Structure:");
    let cleen_dir = &config.cleen_dir;
    println!("  cleen directory: {cleen_dir:?}");

    if cleen_dir.exists() {
        println!("    ✅ exists");
    } else {
        println!("    ❌ missing");
        issues_found += 1;
    }

    let versions_dir = config.get_versions_dir();
    println!("  versions directory: {versions_dir:?}");
    if versions_dir.exists() {
        println!("    ✅ exists");
    } else {
        println!("    ❌ missing");
        issues_found += 1;
    }

    let bin_dir = config.get_bin_dir();
    println!("  bin directory: {bin_dir:?}");
    if bin_dir.exists() {
        println!("    ✅ exists");
    } else {
        println!("    ❌ missing");
        issues_found += 1;
    }

    println!();

    // Check installed versions
    println!("📦 Installed Versions:");
    let versions = version_manager.list_installed_versions()?;
    if versions.is_empty() {
        println!("  ⚠️  No versions installed");
    } else {
        for version_info in &versions {
            println!(
                "  {} {}",
                version_info.version,
                if version_info.is_valid { "✅" } else { "❌" }
            );

            if !version_info.is_valid {
                issues_found += 1;
            }
        }
    }

    println!();

    // Check version resolution (project-specific vs global)
    println!("🔗 Version Resolution:");

    // Show current directory
    if let Ok(current_dir) = env::current_dir() {
        println!("  Current directory: {current_dir:?}");

        // Check for project version
        if let Some(project_version) = config.get_project_version() {
            println!("  📁 Project version (.cleanlanguage/.cleanversion): {project_version}");

            // Verify project version is installed
            if version_manager.is_version_installed(&project_version) {
                println!("    ✅ Project version is installed");
            } else {
                println!(
                    "    ❌ Project version not installed - run 'cleen install {project_version}'"
                );
                issues_found += 1;
            }
        } else {
            println!("  📁 Project version: none (.cleanlanguage/.cleanversion file not found)");
        }
    }

    // Show global active version
    if let Some(ref global_version) = config.active_version {
        println!("  🌐 Global version: {global_version}");
    } else {
        println!("  🌐 Global version: none");
    }

    // Show effective version
    if let Some(effective_version) = config.get_effective_version() {
        println!("  ⚙️  Effective version (what 'cln' will use): {effective_version}");

        let binary_path = config.get_version_binary(&effective_version);
        if binary_path.exists() {
            println!("    ✅ Binary exists: {binary_path:?}");
        } else {
            println!("    ❌ Binary missing: {binary_path:?}");
            issues_found += 1;
        }
    } else {
        println!("  ⚙️  Effective version: none - no version set");
        println!("    ❌ No version available");
        issues_found += 1;
    }

    println!();

    // Check shim
    println!("🔗 Shim Status:");
    let shim_path = config.get_shim_path();
    println!("  Shim path: {shim_path:?}");

    if shim_path.exists() {
        println!("    ✅ Shim exists");
    } else {
        println!("    ❌ Shim missing");
        issues_found += 1;
    }

    // Check PATH
    println!("  PATH check:");
    let bin_dir_binding = config.get_bin_dir();
    let bin_dir_str = bin_dir_binding.to_string_lossy();
    if let Ok(path) = std::env::var("PATH") {
        if path.contains(&*bin_dir_str) {
            println!("    ✅ cleen bin directory is in PATH");
        } else {
            println!("    ❌ cleen bin directory not in PATH");
            println!("      Run 'cleen init' to fix this");
            issues_found += 1;
        }
    } else {
        println!("    ❌ PATH environment variable not found");
        issues_found += 1;
    }

    println!();

    // Test cln command
    println!("🧪 Command Test:");
    match Command::new("cln").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                let version_output = String::from_utf8_lossy(&output.stdout);
                println!("  ✅ 'cln --version' works: {}", version_output.trim());

                // Test runtime functionality
                println!("  🧪 Testing runtime execution...");
                match test_runtime_execution() {
                    Ok(_) => {
                        println!("    ✅ Runtime test passed");
                    }
                    Err(e) => {
                        println!("    ❌ Runtime test failed: {e}");
                        println!("      This indicates WebAssembly runtime issues");
                        issues_found += 1;
                    }
                }
            } else {
                println!("  ❌ 'cln --version' failed");
                issues_found += 1;
            }
        }
        Err(_) => {
            println!("  ❌ 'cln' command not found");
            issues_found += 1;
        }
    }

    println!();

    // Check Frame CLI if requested or if installed
    let frame_versions = frame::list_frame_versions(&config).unwrap_or_default();
    if check_frame || !frame_versions.is_empty() {
        println!("🚀 Frame CLI:");

        if frame_versions.is_empty() {
            println!("  ⚠️  No Frame CLI versions installed");
            if check_frame {
                println!("    To install: cleen frame install");
            }
        } else {
            println!("  Installed versions:");
            for version in &frame_versions {
                let is_active = config.frame_version.as_deref() == Some(version);
                let marker = if is_active { "✅" } else { "  " };
                println!("    {marker} {version}");
            }

            if let Some(active_frame) = &config.frame_version {
                println!();
                println!("  Active Frame version: {active_frame}");

                // Check Frame binary
                let frame_binary = config.get_frame_version_binary(active_frame);
                if frame_binary.exists() {
                    println!("    ✅ Binary exists: {frame_binary:?}");

                    // Test Frame command
                    match Command::new(&frame_binary).arg("--version").output() {
                        Ok(output) => {
                            if output.status.success() {
                                let version_output = String::from_utf8_lossy(&output.stdout);
                                println!(
                                    "    ✅ 'frame --version' works: {}",
                                    version_output.trim()
                                );
                            } else {
                                println!("    ❌ 'frame --version' failed");
                                issues_found += 1;
                            }
                        }
                        Err(_) => {
                            println!("    ❌ Failed to execute Frame binary");
                            issues_found += 1;
                        }
                    }
                } else {
                    println!("    ❌ Binary missing: {frame_binary:?}");
                    issues_found += 1;
                }

                // Check compatibility with compiler
                if let Some(compiler_version) = &config.active_version {
                    println!();
                    println!("  Compatibility check:");
                    match compatibility::check_frame_compatibility(compiler_version, active_frame) {
                        Ok(_) => {
                            println!("    ✅ Frame CLI {active_frame} is compatible with compiler {compiler_version}");
                        }
                        Err(e) => {
                            println!("    ⚠️  {e}");
                            println!("      Frame CLI may not work correctly");
                        }
                    }
                } else {
                    println!();
                    println!("  ⚠️  No compiler active - Frame CLI requires a compiler");
                }

                // Check Frame shim
                let frame_shim = config.get_frame_shim_path();
                if frame_shim.exists() {
                    println!();
                    println!("  Shim: {frame_shim:?}");
                    println!("    ✅ Frame shim exists");
                } else {
                    println!();
                    println!("  Shim: {frame_shim:?}");
                    println!("    ❌ Frame shim missing");
                    println!("      Run: cleen frame use {active_frame}");
                    issues_found += 1;
                }
            } else {
                println!();
                println!("  ⚠️  No active Frame version");
                println!("    To activate: cleen frame use <version>");
            }
        }

        println!();
    }

    // Summary
    if issues_found == 0 {
        println!("🎉 Environment looks good! No issues found.");

        // Show usage tips
        if config.get_project_version().is_some() {
            println!();
            println!("💡 Project Setup Tips:");
            println!("  - This project has a .cleanlanguage/.cleanversion file");
            println!("  - 'cln' commands will automatically use the project version");
            println!("  - Add .cleanlanguage/ to version control to share with your team");
        } else if config.active_version.is_some() {
            println!();
            println!("💡 Project Setup Tips:");
            println!("  - You're using a global Clean Language version");
            println!("  - Run 'cleen local <version>' to set a project-specific version");
            println!("  - This creates a .cleanlanguage/.cleanversion file for the project");
        }
    } else {
        println!("⚠️  Found {issues_found} issue(s) that need attention.");
        println!();
        println!("💡 To fix issues:");
        println!("  - Run 'cleen init' to set up shell configuration");
        println!("  - Run 'cleen install <version>' to install a version");
        println!("  - Run 'cleen use <version>' to set global version");
        println!("  - Run 'cleen local <version>' to set project version");
    }

    Ok(())
}

fn test_runtime_execution() -> Result<()> {
    // Create a simple test program
    let test_program = r#"start()
	print("test")"#;

    // Create temporary files
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("cleen_runtime_test.cln");

    // Write test program
    std::fs::write(&test_file, test_program).map_err(|e| CleenError::ValidationError {
        message: format!("Failed to create test file: {e}"),
    })?;

    // Try to run the program
    let run_result = Command::new("cln")
        .args(["run", test_file.to_str().unwrap()])
        .output();

    // Clean up test file
    let _ = std::fs::remove_file(&test_file);

    match run_result {
        Ok(output) => {
            if output.status.success() {
                // Check if we got the expected output
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("test") {
                    Ok(())
                } else {
                    Err(CleenError::ValidationError {
                        message: "Runtime executed but output was unexpected".to_string(),
                    })
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("WebAssembly translation error") {
                    Err(CleenError::ValidationError {
                        message: "WebAssembly runtime configuration issue".to_string(),
                    })
                } else if stderr.contains("incompatible import type") {
                    Err(CleenError::ValidationError {
                        message: "Host function signature mismatch".to_string(),
                    })
                } else {
                    Err(CleenError::ValidationError {
                        message: format!("Runtime execution failed: {stderr}"),
                    })
                }
            }
        }
        Err(e) => Err(CleenError::ValidationError {
            message: format!("Failed to execute runtime test: {e}"),
        }),
    }
}
