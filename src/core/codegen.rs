//! Code generation module for Clean Framework
//!
//! Generates main.cln from discovered project structure:
//! - Handler functions for each route
//! - Route registration in start()
//! - Component imports and registry

use crate::core::discovery::{ApiRoute, Component, DiscoveredProject, Model, PageRoute};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Code generation options
#[derive(Debug, Default)]
pub struct CodegenOptions {
    /// Include debug comments in generated code
    pub debug_comments: bool,
    /// Generate component registry JSON
    pub generate_registry: bool,
}

/// Result of code generation
#[derive(Debug)]
pub struct GeneratedCode {
    /// Generated main.cln content
    pub main_cln: String,
    /// Component registry JSON (if requested)
    pub component_registry: Option<String>,
    /// List of files to compile (main.cln + dependencies)
    pub compile_order: Vec<String>,
}

/// Generate main.cln and related files from discovered project
pub fn generate_code(
    project: &DiscoveredProject,
    project_dir: &Path,
    options: &CodegenOptions,
) -> Result<GeneratedCode> {
    let mut main_cln = String::new();
    let mut handler_index: usize = 0;

    // Generate imports section
    main_cln.push_str(&generate_imports(project, project_dir)?);

    // Generate model classes
    if !project.models.is_empty() {
        main_cln.push_str("\n// Database models\n");
        for model in &project.models {
            main_cln.push_str(&generate_model_import(&model.source_file, project_dir)?);
        }
    }

    // Generate component classes (if any need server-side rendering)
    if !project.components.is_empty() && options.debug_comments {
        main_cln.push_str("\n// Components available: ");
        let tags: Vec<&str> = project.components.iter().map(|c| c.tag.as_str()).collect();
        main_cln.push_str(&tags.join(", "));
        main_cln.push('\n');
    }

    // Generate functions block with handlers
    main_cln.push_str("\nfunctions:\n");

    // Generate component render functions FIRST (so page handlers can call them)
    for component in &project.components {
        main_cln.push_str(&generate_component_render_function(component, options)?);
    }

    // Page handlers (with component expansion)
    for page in &project.pages {
        main_cln.push_str(&generate_page_handler(
            page,
            project_dir,
            handler_index,
            &project.components,
            options,
        )?);
        handler_index += 1;
    }

    // API handlers
    for api in &project.api_routes {
        main_cln.push_str(&generate_api_handler(
            api,
            project_dir,
            handler_index,
            options,
        )?);
        handler_index += 1;
    }

    // Generate start() function with route registration
    main_cln.push_str(&generate_start_function(project, options)?);

    // Generate component registry if requested
    let component_registry = if options.generate_registry && !project.components.is_empty() {
        Some(generate_component_registry(&project.components)?)
    } else {
        None
    };

    // Build compile order
    let compile_order = build_compile_order(project, project_dir)?;

    Ok(GeneratedCode {
        main_cln,
        component_registry,
        compile_order,
    })
}

/// Generate a component render function from its source file
fn generate_component_render_function(
    component: &Component,
    options: &CodegenOptions,
) -> Result<String> {
    let mut output = String::new();

    if options.debug_comments {
        output.push_str(&format!(
            "\t// Component: <{}> from {}\n",
            component.tag,
            component
                .source_file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));
    }

    // Read component source
    let content = fs::read_to_string(&component.source_file).with_context(|| {
        format!(
            "Failed to read component: {}",
            component.source_file.display()
        )
    })?;

    // Extract render function body
    let render_body = extract_component_render_body(&content)?;

    // Generate function with unique name based on class_name
    output.push_str(&format!(
        "\tstring __component_{}_render()\n",
        component.class_name
    ));
    output.push_str(&indent_code(&render_body, 2));
    output.push_str("\n\n");

    Ok(output)
}

/// Extract the render function body from a component source file
fn extract_component_render_body(content: &str) -> Result<String> {
    let mut in_render = false;
    let mut render_body = String::new();
    let mut base_indent = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Look for render() function start
        if trimmed.starts_with("string render()") || trimmed.contains("string render()") {
            in_render = true;
            // Calculate base indentation
            base_indent = line.len() - line.trim_start().len();
            continue;
        }

        if in_render {
            // Check if we've exited the render function (new function or block at same/lower indent)
            let current_indent = line.len() - line.trim_start().len();

            // Empty lines are okay
            if trimmed.is_empty() {
                continue;
            }

            // If we hit a line at the same or lower indentation that's not part of render, stop
            if current_indent <= base_indent && !trimmed.is_empty() {
                break;
            }

            // Add the line (removing the extra indentation)
            render_body.push_str(trimmed);
            render_body.push('\n');
        }
    }

    if render_body.is_empty() {
        // If no render body found, return a placeholder
        Ok("return \"\"".to_string())
    } else {
        Ok(render_body.trim_end().to_string())
    }
}

/// Generate import statements
fn generate_imports(project: &DiscoveredProject, _project_dir: &Path) -> Result<String> {
    let mut imports = String::new();

    // Check what plugins are needed
    let needs_web = !project.pages.is_empty() || !project.api_routes.is_empty();
    let needs_data = !project.models.is_empty();

    if needs_web {
        imports.push_str("import frame.web\n");
    }
    if needs_data {
        imports.push_str("import frame.data\n");
    }

    // Add lib module imports
    for lib in &project.lib_modules {
        imports.push_str(&format!("// lib: {}\n", lib.name));
    }

    if !imports.is_empty() {
        imports.push('\n');
    }

    Ok(imports)
}

/// Generate a page handler function
fn generate_page_handler(
    page: &PageRoute,
    project_dir: &Path,
    handler_index: usize,
    components: &[Component],
    options: &CodegenOptions,
) -> Result<String> {
    let mut handler = String::new();

    if options.debug_comments {
        handler.push_str(&format!(
            "\t// Handler for {} (from {})\n",
            page.path,
            page.source_file
                .strip_prefix(project_dir)
                .unwrap_or(&page.source_file)
                .display()
        ));
    }

    handler.push_str(&format!("\tstring __route_handler_{}()\n", handler_index));

    // Extract route parameters
    let params = extract_route_params(&page.path);
    for param in &params {
        handler.push_str(&format!(
            "\t\tstring {} = _req_param(\"{}\")\n",
            param, param
        ));
    }

    // Read and inline the page source (with component expansion)
    let source = read_page_source(&page.source_file, components)?;
    handler.push_str(&indent_code(&source, 2));
    handler.push('\n');

    Ok(handler)
}

/// Generate an API handler function
fn generate_api_handler(
    api: &ApiRoute,
    project_dir: &Path,
    handler_index: usize,
    options: &CodegenOptions,
) -> Result<String> {
    let mut handler = String::new();

    if options.debug_comments {
        handler.push_str(&format!(
            "\t// API handler for {} {} (from {})\n",
            api.method,
            api.path,
            api.source_file
                .strip_prefix(project_dir)
                .unwrap_or(&api.source_file)
                .display()
        ));
    }

    handler.push_str(&format!("\tstring __route_handler_{}()\n", handler_index));

    // Extract route parameters
    let params = extract_route_params(&api.path);
    for param in &params {
        handler.push_str(&format!(
            "\t\tstring {} = _req_param(\"{}\")\n",
            param, param
        ));
    }

    // Read and inline the API source
    let source = read_api_source(&api.source_file)?;
    handler.push_str(&indent_code(&source, 2));
    handler.push('\n');

    Ok(handler)
}

/// Generate the start() function with route registration
fn generate_start_function(
    project: &DiscoveredProject,
    options: &CodegenOptions,
) -> Result<String> {
    let mut start = String::new();

    start.push_str("\nstart()\n");
    start.push_str("\tinteger status = 0\n");

    if options.debug_comments && !project.pages.is_empty() {
        start.push_str("\n\t// Page routes\n");
    }

    let mut handler_index: usize = 0;

    // Register page routes
    for page in &project.pages {
        start.push_str(&format!(
            "\tstatus = _http_route(\"{}\", \"{}\", {})\n",
            page.method, page.path, handler_index
        ));
        handler_index += 1;
    }

    if options.debug_comments && !project.api_routes.is_empty() {
        start.push_str("\n\t// API routes\n");
    }

    // Register API routes
    for api in &project.api_routes {
        start.push_str(&format!(
            "\tstatus = _http_route(\"{}\", \"{}\", {})\n",
            api.method, api.path, handler_index
        ));
        handler_index += 1;
    }

    start.push('\n');

    Ok(start)
}

/// Generate model import/include
fn generate_model_import(source_file: &Path, project_dir: &Path) -> Result<String> {
    let relative = source_file.strip_prefix(project_dir).unwrap_or(source_file);
    Ok(format!("// include: {}\n", relative.display()))
}

/// Generate component registry JSON
fn generate_component_registry(components: &[Component]) -> Result<String> {
    let mut registry = String::from("{\n  \"components\": [\n");

    for (i, component) in components.iter().enumerate() {
        registry.push_str(&format!(
            "    {{\n      \"tag\": \"{}\",\n      \"class\": \"{}\",\n      \"hydration\": \"{}\"\n    }}",
            component.tag, component.class_name, component.hydration
        ));
        if i < components.len() - 1 {
            registry.push(',');
        }
        registry.push('\n');
    }

    registry.push_str("  ]\n}");
    Ok(registry)
}

/// Build list of files to compile in order
fn build_compile_order(project: &DiscoveredProject, project_dir: &Path) -> Result<Vec<String>> {
    let mut order = Vec::new();

    // Models first (they define data structures)
    for model in &project.models {
        order.push(
            model
                .source_file
                .strip_prefix(project_dir)
                .unwrap_or(&model.source_file)
                .to_string_lossy()
                .to_string(),
        );
    }

    // Lib modules
    for lib in &project.lib_modules {
        order.push(
            lib.source_file
                .strip_prefix(project_dir)
                .unwrap_or(&lib.source_file)
                .to_string_lossy()
                .to_string(),
        );
    }

    // Components
    for component in &project.components {
        order.push(
            component
                .source_file
                .strip_prefix(project_dir)
                .unwrap_or(&component.source_file)
                .to_string_lossy()
                .to_string(),
        );
    }

    // Generated main.cln is last
    order.push("dist/.generated/main.cln".to_string());

    Ok(order)
}

/// Extract route parameters from path (e.g., /blog/:slug -> ["slug"])
fn extract_route_params(path: &str) -> Vec<String> {
    let mut params = Vec::new();

    for segment in path.split('/') {
        if segment.starts_with(':') {
            params.push(segment[1..].to_string());
        }
    }

    params
}

/// Read page source file and convert HTML to Clean Language
fn read_page_source(source_file: &Path, components: &[Component]) -> Result<String> {
    let content = fs::read_to_string(source_file)
        .with_context(|| format!("Failed to read page file: {}", source_file.display()))?;

    // Convert HTML to Clean Language string concatenation (with component expansion)
    convert_html_to_clean(&content, components)
}

/// Convert HTML content to Clean Language string concatenation code
fn convert_html_to_clean(html: &str, components: &[Component]) -> Result<String> {
    let mut output = String::new();
    let mut lines: Vec<&str> = html.lines().collect();

    // Check for <script type="text/clean"> block - extract data loading
    let mut data_block = String::new();
    let mut in_script = false;
    let mut script_start = 0;
    let mut script_end = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains("<script type=\"text/clean\">")
            || trimmed.contains("<script type='text/clean'>")
        {
            in_script = true;
            script_start = i;
        } else if in_script && trimmed.contains("</script>") {
            script_end = i;
            in_script = false;
        } else if in_script {
            // Collect data block lines (skip the data: keyword itself if present)
            if !trimmed.starts_with("data:") && !trimmed.is_empty() {
                data_block.push_str(trimmed);
                data_block.push('\n');
            }
        }
    }

    // Remove script block from lines if found
    if script_end > script_start {
        lines = lines
            .iter()
            .enumerate()
            .filter(|(i, _)| *i < script_start || *i > script_end)
            .map(|(_, l)| *l)
            .collect();
    }

    // Add data loading code first (commented for now - needs proper variable handling)
    if !data_block.is_empty() {
        output.push_str("// Data loading:\n");
        for line in data_block.lines() {
            output.push_str(&format!("// {}\n", line));
        }
    }

    // Build HTML as string concatenation
    output.push_str("string html = \"");

    let mut first_line = true;
    for line in &lines {
        let trimmed = line.trim();

        // Skip empty lines at start
        if first_line && trimmed.is_empty() {
            continue;
        }
        first_line = false;

        // Skip HTML comments
        if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
            continue;
        }

        // Skip <page> directive tags
        if trimmed.starts_with("<page ") && trimmed.ends_with(">") {
            continue;
        }

        // Close current string and start new concatenation for each line
        if !output.ends_with("\"") {
            output.push_str("\"\n");
            output.push_str("html = html + \"");
        }

        // Check for component tags and expand them
        let expanded = expand_component_tags(trimmed, components);
        output.push_str(&expanded);
    }

    output.push_str("\"\n");
    output.push_str("return html");

    Ok(output)
}

/// Expand component tags in HTML line to function calls
fn expand_component_tags(line: &str, components: &[Component]) -> String {
    let mut result = line.to_string();

    for component in components {
        // Match self-closing tags: <app-header></app-header> or <app-header />
        let self_closing = format!("<{}></{}>", component.tag, component.tag);
        let self_closing_short = format!("<{} />", component.tag);
        let self_closing_nospace = format!("<{}/>", component.tag);

        // Also match just opening/closing if on same line
        if result.contains(&self_closing) {
            // Replace with function call
            let replacement = format!("\" + __component_{}_render() + \"", component.class_name);
            result = result.replace(&self_closing, &replacement);
        } else if result.contains(&self_closing_short) {
            let replacement = format!("\" + __component_{}_render() + \"", component.class_name);
            result = result.replace(&self_closing_short, &replacement);
        } else if result.contains(&self_closing_nospace) {
            let replacement = format!("\" + __component_{}_render() + \"", component.class_name);
            result = result.replace(&self_closing_nospace, &replacement);
        }
    }

    // Now escape remaining HTML (but preserve our function call insertions)
    escape_html_for_clean_with_calls(&result)
}

/// Escape HTML content for Clean strings, but preserve function call insertions
fn escape_html_for_clean_with_calls(html: &str) -> String {
    let mut result = String::new();
    let mut chars = html.chars().peekable();
    let mut in_function_call = false;

    while let Some(c) = chars.next() {
        // Check for function call marker: " + __component_
        if c == '"' && !in_function_call {
            // Look ahead for function call pattern
            let remaining: String = chars.clone().take(20).collect();
            if remaining.starts_with(" + __component_") {
                // This is a function call insertion - pass through as-is
                result.push(c);
                in_function_call = true;
                continue;
            }
        }

        // Check for end of function call: () + "
        if in_function_call && c == '(' {
            result.push(c);
            if chars.peek() == Some(&')') {
                result.push(chars.next().unwrap()); // )
                                                    // Look for + "
                if chars.peek() == Some(&' ') {
                    result.push(chars.next().unwrap()); // space
                    if chars.peek() == Some(&'+') {
                        result.push(chars.next().unwrap()); // +
                        if chars.peek() == Some(&' ') {
                            result.push(chars.next().unwrap()); // space
                            if chars.peek() == Some(&'"') {
                                result.push(chars.next().unwrap()); // "
                                in_function_call = false;
                            }
                        }
                    }
                }
            }
            continue;
        }

        if in_function_call {
            // Inside function call - pass through as-is
            result.push(c);
            continue;
        }

        // Normal HTML escaping
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\t' => result.push_str("\\t"),
            '{' if chars.peek() == Some(&'{') => {
                // Handle {{variable}} interpolation
                chars.next(); // consume second {
                let mut var_name = String::new();
                while let Some(vc) = chars.next() {
                    if vc == '}' && chars.peek() == Some(&'}') {
                        chars.next(); // consume second }
                        break;
                    }
                    var_name.push(vc);
                }
                // Convert {{var}} to Clean string concatenation
                result.push_str("\" + ");
                result.push_str(&var_name);
                result.push_str(" + \"");
            }
            _ => result.push(c),
        }
    }

    result
}

/// Escape HTML content for embedding in Clean Language strings
fn escape_html_for_clean(html: &str) -> String {
    let mut result = String::new();
    let mut chars = html.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\t' => result.push_str("\\t"),
            '{' if chars.peek() == Some(&'{') => {
                // Handle {{variable}} interpolation
                chars.next(); // consume second {
                let mut var_name = String::new();
                while let Some(vc) = chars.next() {
                    if vc == '}' && chars.peek() == Some(&'}') {
                        chars.next(); // consume second }
                        break;
                    }
                    var_name.push(vc);
                }
                // Convert {{var}} to Clean string concatenation
                result.push_str("\" + ");
                result.push_str(&var_name);
                result.push_str(" + \"");
            }
            _ => result.push(c),
        }
    }

    result
}

/// Read API source file and extract the handler body
fn read_api_source(source_file: &Path) -> Result<String> {
    let content = fs::read_to_string(source_file)
        .with_context(|| format!("Failed to read API file: {}", source_file.display()))?;

    // Return content as-is
    Ok(content)
}

/// Indent code by the specified number of tabs
fn indent_code(code: &str, tabs: usize) -> String {
    let indent = "\t".repeat(tabs);
    code.lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{}{}", indent, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Write generated code to disk
pub fn write_generated_code(generated: &GeneratedCode, output_dir: &Path) -> Result<()> {
    let gen_dir = output_dir.join(".generated");
    fs::create_dir_all(&gen_dir).context("Failed to create .generated directory")?;

    // Write main.cln
    fs::write(gen_dir.join("main.cln"), &generated.main_cln)
        .context("Failed to write generated main.cln")?;

    // Write component registry if present
    if let Some(registry) = &generated.component_registry {
        fs::write(gen_dir.join("components.json"), registry)
            .context("Failed to write component registry")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_route_params() {
        assert_eq!(extract_route_params("/"), Vec::<String>::new());
        assert_eq!(extract_route_params("/about"), Vec::<String>::new());
        assert_eq!(extract_route_params("/blog/:slug"), vec!["slug"]);
        assert_eq!(
            extract_route_params("/users/:id/posts/:postId"),
            vec!["id", "postId"]
        );
    }

    #[test]
    fn test_indent_code() {
        let code = "line1\nline2\nline3";
        let indented = indent_code(code, 2);
        assert_eq!(indented, "\t\tline1\n\t\tline2\n\t\tline3");
    }
}
