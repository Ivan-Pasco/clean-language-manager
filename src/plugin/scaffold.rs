use crate::error::{CleenError, Result};
use crate::plugin::manifest::PluginManifest;
use std::fs;
use std::path::Path;

/// Create a new plugin project with all scaffold files
pub fn create_plugin_project(name: &str, target_dir: Option<&Path>) -> Result<()> {
    // Determine the project directory
    let project_dir = match target_dir {
        Some(dir) => dir.join(name),
        None => std::env::current_dir()?.join(name),
    };

    // Check if directory already exists
    if project_dir.exists() {
        return Err(CleenError::PluginManifestError {
            message: format!("Directory '{}' already exists", project_dir.display()),
        });
    }

    println!("Creating plugin project '{}'...", name);

    // Create directory structure
    fs::create_dir_all(&project_dir)?;
    fs::create_dir_all(project_dir.join("src"))?;
    fs::create_dir_all(project_dir.join("tests"))?;

    println!("  Created {}/", name);

    // Create plugin.toml
    let manifest = PluginManifest::new(name);
    let manifest_path = project_dir.join("plugin.toml");
    manifest.save(&manifest_path)?;
    println!("  Created {}/plugin.toml", name);

    // Create src/main.cln
    let main_content = generate_main_cln(name);
    let main_path = project_dir.join("src").join("main.cln");
    fs::write(&main_path, main_content)?;
    println!("  Created {}/src/main.cln", name);

    // Create tests/test_expand.cln
    let test_content = generate_test_cln(name);
    let test_path = project_dir.join("tests").join("test_expand.cln");
    fs::write(&test_path, test_content)?;
    println!("  Created {}/tests/test_expand.cln", name);

    // Create README.md
    let readme_content = generate_readme(name);
    let readme_path = project_dir.join("README.md");
    fs::write(&readme_path, readme_content)?;
    println!("  Created {}/README.md", name);

    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  # Edit src/main.cln to implement your plugin");
    println!("  cleen plugin build");

    Ok(())
}

/// Generate the main.cln source file template
fn generate_main_cln(name: &str) -> String {
    format!(
        r#"// Plugin: {}
// Expand framework blocks into Clean Language code

// The expand_block function is called by the compiler when it encounters
// a block that matches this plugin's namespace.
//
// Parameters:
//   block_name: The name of the block being expanded (e.g., "route", "model")
//   attributes: The block attributes as a JSON string
//   body: The body content of the block
//
// Returns:
//   The expanded Clean Language code as a string

expand_block(block_name: string, attributes: string, body: string) -> string
    // Example: Simply return the body unchanged
    // In a real plugin, you would parse the attributes and body,
    // then generate Clean Language code based on them.

    // For now, wrap the body in a comment showing it was processed
    result: string = "// Expanded by {} plugin\n" + body
    return result


// The validate_block function is called to validate block syntax before expansion.
//
// Parameters:
//   block_name: The name of the block being validated
//   attributes: The block attributes as a JSON string
//   body: The body content of the block
//
// Returns:
//   true if the block is valid, false otherwise

validate_block(block_name: string, attributes: string, body: string) -> boolean
    // Basic validation - check that we have a valid block name
    if block_name == ""
        return false

    return true
"#,
        name, name
    )
}

/// Generate the test file template
fn generate_test_cln(name: &str) -> String {
    format!(
        r#"// Test file for {} plugin

// Test that the expand_block function works correctly
test_expand_basic()
    block_name: string = "example"
    attributes: string = "{{}}"
    body: string = "x: integer = 42"

    result: string = expand_block(block_name, attributes, body)

    // Verify the result contains the original body
    assert(contains(result, "x: integer = 42"))


// Test that validation works correctly
test_validate_empty_name()
    result: boolean = validate_block("", "{{}}", "body")
    assert(result == false)


test_validate_valid_block()
    result: boolean = validate_block("route", "{{}}", "get /users")
    assert(result == true)
"#,
        name
    )
}

/// Generate the README.md template
fn generate_readme(name: &str) -> String {
    format!(
        r#"# {}

A Clean Language plugin.

## Description

This plugin extends Clean Language with additional functionality.

## Installation

```bash
cleen plugin install {}
```

## Usage

In your Clean Language source files, use the plugin's blocks:

```clean
import: {}

// Use plugin-specific blocks here
```

## Development

### Building

```bash
cleen plugin build
```

### Testing

```bash
cln test tests/test_expand.cln
```

### Publishing

```bash
cleen plugin publish
```

## License

MIT
"#,
        name, name, name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_main_cln() {
        let content = generate_main_cln("test-plugin");
        assert!(content.contains("Plugin: test-plugin"));
        assert!(content.contains("expand_block"));
        assert!(content.contains("validate_block"));
    }

    #[test]
    fn test_generate_test_cln() {
        let content = generate_test_cln("test-plugin");
        assert!(content.contains("Test file for test-plugin"));
        assert!(content.contains("test_expand_basic"));
    }

    #[test]
    fn test_generate_readme() {
        let content = generate_readme("test-plugin");
        assert!(content.contains("# test-plugin"));
        assert!(content.contains("cleen plugin install test-plugin"));
    }
}
